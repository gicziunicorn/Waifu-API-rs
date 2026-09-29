use async_trait::async_trait;
use serde::Deserialize;
use bytes::Bytes;
use log::{debug, info};
use tokio::sync::{mpsc, oneshot};
use std::thread;
use rand::{Rng, RngExt, rngs::ThreadRng};
use wreq::{Client, header};

use crate::{fetcher::{FetchJob, FetchRequestType, FetchResponse, Fetcher}, types::{FetchError::{self, UnexpectedResponseError}, FetchResult}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sites {
    Rule34,
    Nekos,
}

// implement Display to easily display the site name in the dropdown menu
impl std::fmt::Display for Sites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}


pub struct SiteDetails {
    pub name: &'static str,
    pub icon_name: &'static str,
    pub url_random: &'static str,
    pub url_image: &'static str,
}


impl Sites {
    pub fn details(&self) -> SiteDetails {
        match self {
            Sites::Rule34 => SiteDetails {
                name: "Rule 34",
                url_random: "https://rule34.xxx/index.php?page=post&s=random",
                url_image: "",
                icon_name: "rule34_favicon.ico",
            },
            Sites::Nekos => SiteDetails {
                name: "Nekos.moe",
                url_random: "https://nekos.moe/",
                url_image: "",
                icon_name: "nekos_favicon.ico",
            },
        }
    }

    pub fn name(&self) -> &'static str {
        self.details().name
    }
    pub fn url(&self) -> &'static str {
        self.details().url_image
    }
    pub fn image_path(&self) -> &'static str {
        self.details().icon_name
    }
}

// Give each site's response a struct so it can be parsed later
// use deserialize so if the api returns fileUrl it'll be file_url
#[derive(Debug, Clone, Deserialize)]
pub struct Rule34Post {
    pub preview_url: String,
    pub sample_url: String,
    pub file_url: String,
    pub directory: u32,
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub id: u32,
    pub image: String,
    pub change: u32,
    pub owner: String,
    pub parent_id: u32,
    pub rating: String,
    pub sample: bool,
    pub sample_height: u32,
    pub sample_width: u32,
    pub score: u32,
    pub tags: String,
    pub source: String,
    pub status: String,
    pub has_notes: bool,
    pub comment_count: u32,
}
pub type Rule34Response = Vec<Rule34Post>;


#[derive(Debug, Clone, Deserialize)]
pub struct NekosResponse {
    pub images: Vec<NekosPost>,
}


pub enum Responses {
    Rule34(Rule34Response),
    Nekos(NekosResponse),
}




// ------------------------


pub struct RandomPost {
    image_bytes: Vec<u8>,
}


/// A trait for sites
#[async_trait]
pub trait Site {
    /// Checks if the site is available
    async fn is_available(&self) -> bool;

    /// Get a random image from the site;
    /// T: site return structure
    async fn get_random_image<T>(&self, fetcher: Fetcher) -> FetchResult<T>;
}


// create a struct for the site
/// Nekos.moe
pub struct NekosMoe {
    pub name: &'static str,
    pub url_random: &'static str,
    pub url_image: &'static str,
    pub test_url: Option<&'static str>,
}

// create a new function for the site, so the user can initialize it's data
impl NekosMoe {
    /// Create a new instance of this site
    pub fn new() -> Self {
        NekosMoe {
            name: "Nekos.moe",
            url_random: "https://nekos.moe/api/v1/random/image",
            url_image: "https://nekos.moe/image",
            test_url: None,
        }
    }
}

#[async_trait]
impl Site for NekosMoe {
    async fn is_available(&self) -> bool {
        todo!()
    }

    async fn get_random_image<NekosMoeResponse>(&self, fetcher: Fetcher) -> FetchResult<NekosMoeResponse> {
        let url_random = self.url_random.to_string();

        let (sender, receiver) = oneshot::channel();
        let job = FetchJob {
            fetch_type: FetchRequestType::Posts,
            url: url_random,
            sender,
        };

        if fetcher.job_sender.send(job).is_err() {
            Err(FetchError::ThreadError("Network thread worker crashed or closed!"))?
        }

        let res: FetchResult<FetchResponse<NekosMoeResponse>> = receiver.await?;

        let posts = match res {
            Ok(FetchResponse::Posts(result)) => result,
            Ok(_) => Err(UnexpectedResponseError),
            Err(e) => Err(e),
        }?;

        info!("Fetched posts from Nekos: \n{:#?}", posts);

        // get the image
        let url = posts[0].image_url.clone();
        let bytes = fetcher.fetch_image_link(url).await?;
        let image_vec = bytes.to_vec();

        Ok(RandomPost {
            image_bytes: image_vec,
        })
    }
}


#[derive(Debug, Clone, Deserialize)]
/// The structure of a post for the site
pub struct NekosPost {
    pub id: String,
    pub nsfw: bool,
    pub artist: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub likes: u32,
    pub favorites: u32,
}

/// The structure that the API of the site will return
type NekosMoeResponse = Vec<NekosPost>;