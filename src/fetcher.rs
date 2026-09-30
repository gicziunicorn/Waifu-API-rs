use bytes::Bytes;
use log::{debug, info};
use tokio::sync::{mpsc, oneshot::{self, Sender}};
use std::{any::Any, thread};
use rand::{Rng, RngExt, rngs::ThreadRng};
use wreq::{Client, Response, header};
use thiserror::Error;

use crate::{site::{self, NekosResponse, Rule34Response, Sites}, types::{FetchError, FetchResult}};


#[derive(Clone)]
pub enum FetchRequestType {
    Image,
    Posts,
}

pub struct FetchJob {
    pub fetch_type: FetchRequestType,
    pub url: String,
    pub sender: Sender<FetchResult<FetchResponse>>,
}

pub enum FetchResponse {
    Image(Bytes),
    Posts(Response),
}

/// The object that stores and manages all fetching logic.
#[derive(Clone)]
pub struct Fetcher {
    pub job_sender: mpsc::UnboundedSender<FetchJob>,
}

impl Fetcher {
    /// Start the background thread with tokio for fetching.
    /// Will panic if the thread failed to start.
    pub fn new(client: Client) -> Self {
        let (job_sender, mut job_receiver) = mpsc::unbounded_channel::<FetchJob>();

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to start the background fetcher thread.");

            // Run an infinite loop waiting for incoming fetch requests
            // thanks to await, the loop won't use any CPU while waiting for requests :D
            rt.block_on(async {
                while let Some(job) = job_receiver.recv().await {
                    let client_ref = &client;
                    let result = async move {
                        // send the request and get the response
                        let response = client_ref.get(job.url.as_str()).send().await?;
                        // check the status
                        if !response.status().is_success() {
                            return Err(FetchError::HTTPError {
                                code: response.status().as_u16(),
                                message: "Site returned an error status.",
                                body: response.text().await.ok(),
                            });
                        }

                        // decide what to do with the response
                        match job.fetch_type {
                            FetchRequestType::Posts => {
                                Ok(FetchResponse::Posts(response))
                            }
                            FetchRequestType::Image => {
                                let bytes = response.bytes().await?;
                                Ok(FetchResponse::Image(bytes))
                            }
                        }
                    }.await;

                    let _ = job.sender.send(result);
                }
            });
        });

        Self { job_sender }
    }

    pub async fn fetch_image_link(&self, url: String) -> FetchResult<Bytes> {
        let (response_tx, response_rx) = oneshot::channel();

        let job = FetchJob {
            fetch_type: FetchRequestType::Image,
            url: url,
            sender: response_tx,
        };

        if self.job_sender.send(job).is_err() {
            Err(FetchError::ThreadError("Network thread worker crashed or closed!"))?
        }

        let res = response_rx.await?;

        match res {
            Ok(FetchResponse::Image(bytes)) => Ok(bytes),
            Ok(_) => Err(FetchError::UnexpectedResponseError),
            Err(e) => Err(e),
        }
    }

    /*async fn rule34_fetch(&self, data: FetchRequest) -> Result<Vec<PostResponse>, String> {
        info!("fetching rule34");
        let (response_tx, response_rx) = oneshot::channel();

        let url = format!("{}&tags={}&limit={}&api_key={}&user_id={}", get_api_url("rule34").unwrap(), data.tags, data.limit, data.api_key, data.user_id);

        let job = FetchJob {
            fetch_type: FetchRequestType::Posts(Sites::Rule34),
            url: url,
            sender: response_tx,
        };
        if self.job_sender.send(job).is_err() {
            return Err(String::from("Network thread worker crashed or closed"));
        }

        let res = response_rx.await
            .map_err(|e| format!("Channel error: {}", e))?;

        match res {
            Ok(FetchResponse::Posts(result)) => result,
            Ok(_) => Err(String::from("Unexpected response")),
            Err(e) => Err(e),
        }
    }

    pub async fn rule34_get_random(&self, fetch_data: FetchRequest) -> Result<(Vec<u8>, usize, usize), String> {
        let posts = self.rule34_fetch(fetch_data).await;
        info!("Got response");
        match posts {
            Ok(data) => {
                let sample_url = {
                    // Generate the random inside a block so it's dropped
                    // and won't cause problems with future
                    let mut rng = rand::rng();
                    let r = rng.random_range(0..data.iter().len());
                    data[r].image_url.clone()
                };
                info!("Sample URL: {}", sample_url);
                let bytes = self.fetch_image_link(sample_url).await?;
                let image_vec = bytes.to_vec();
                let size = imagesize::blob_size(&image_vec).err_to_string()?;
                Ok((image_vec, size.width, size.height))
            },
            Err(err) => Err(err)
        }
    }



    pub async fn nekos_get_random(&self, fetch_data: FetchRequest) -> Result<(Vec<u8>, usize, usize), String> {
        let url_random = "https://nekos.moe/api/v1/random/image".to_string();

        let (response_tx, response_rx) = oneshot::channel();
        let job = FetchJob {
            fetch_type: FetchRequestType::Posts(Sites::Nekos),
            url: url_random,
            sender: response_tx,
        };

        if self.job_sender.send(job).is_err() {
            return Err(String::from("Network thread worker crashed or closed!"));
        }
        let res = response_rx.await
            .map_err(|e| format!("Channel error: {}", e))?;
        let posts = match res {
            Ok(FetchResponse::Posts(result)) => result,
            Ok(_) => Err(String::from("Unexpected response")),
            Err(e) => Err(e),
        }?;

        info!("Fetched posts from Nekos: \n{:#?}", posts);

        // get the image
        let url = posts[0].image_url.clone();
        let bytes = self.fetch_image_link(url).await?;
        let image_vec = bytes.to_vec();
        let size = imagesize::blob_size(&image_vec).err_to_string()?;
        Ok((image_vec, size.width, size.height))
    }

    pub async fn fetch_site(&self, data: FetchRequest, site: Sites) -> Result<(Vec<u8>, usize, usize), String> {
        match site {
            Sites::Rule34 => self.rule34_get_random(data).await,
            Sites::Nekos => self.nekos_get_random(data).await,
        }
    }*/
}
