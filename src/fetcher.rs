use bytes::Bytes;
use wreq::{Client, Response};

use crate::types::{FetchError, FetchResult};


/// The struct that stores and handles fetching logic
#[derive(Clone)]
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_url(&self, url: &str) -> FetchResult<Response> {
        Ok(self.client.get(url).send().await?)
    }

    pub async fn fetch_image(&self, url: &str) -> FetchResult<Bytes> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(FetchError::HTTPError {
                code: response.status().as_u16(),
                message: "Site returned an error status.",
                body: response.text().await.ok(),
            });
        }

        Ok(response.bytes().await?)
    }

    pub async fn fetch_posts(&self, url: &str) -> FetchResult<Response> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(FetchError::HTTPError {
                code: response.status().as_u16(),
                message: "Site returned an error status.",
                body: response.text().await.ok(),
            });
        }

        Ok(response)
    }
}