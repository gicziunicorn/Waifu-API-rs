use async_trait::async_trait;
use serde::Deserialize;
use bytes::Bytes;
use log::{debug, info};
use tokio::sync::{mpsc, oneshot};
use std::thread;
use rand::{Rng, RngExt, rngs::ThreadRng};
use wreq::{Client, Response, header};

use crate::{fetcher::Fetcher, types::{FetchError::InvalidDataError, FetchResult}};




#[async_trait]
/// A trait for sites
pub trait Site {
    /// Checks if the site's API is available
    async fn is_available(&self, fetcher: &Fetcher) -> FetchResult<bool>;

    /// The response structure of the site
    type ResponseType;

    /// Get a random post from the site
    async fn get_random_post(&self, fetcher: &Fetcher) -> FetchResult<Self::ResponseType>;

    /// Get a random image from the site
    async fn get_random_image(&self, fetcher: &Fetcher) -> FetchResult<Bytes>;

    async fn get_random_image_from_post(&self, fetcher: &Fetcher, post: Self::ResponseType) -> FetchResult<Bytes>;
}



// ----- Nekos.moe -----

/// struct for the 'Nekos.moe' website
pub struct NekosMoe {
    pub name: &'static str,
    pub url_for_random: &'static str,
    pub url_for_image: &'static str,
    pub url_for_test: Option<&'static str>,
}

// create a 'new()' function for the site, so the user can initialize it's data
impl NekosMoe {
    /// Create a new instance of this site
    pub fn new() -> Self {
        NekosMoe {
            name: "Nekos.moe",
            url_for_random: "https://nekos.moe/api/v1/random/image",
            url_for_image: "https://nekos.moe/image",
            url_for_test: Some("https://nekos.moe/api/v1"),
        }
    }
}

#[async_trait]
impl Site for NekosMoe {
    type ResponseType = NekosMoeResponse;

    async fn is_available(&self, fetcher: &Fetcher) -> FetchResult<bool> {
        let Some(url_for_test) = &self.url_for_test
        else { return Err(InvalidDataError("The url_for_test is None!")); };

        let response = fetcher.fetch_url(url_for_test).await?;

        Ok(response.status().is_success())
    }

    async fn get_random_post(&self, fetcher: &Fetcher) -> FetchResult<Self::ResponseType> {
        let url_for_random = self.url_for_random.to_string();

        let res = fetcher.fetch_posts(&url_for_random).await?;

        // Convert the site's response to the struct
        let nekos_response = res.json::<NekosMoeResponse>().await?;

        Ok(nekos_response)
    }

    async fn get_random_image(&self, fetcher: &Fetcher) -> FetchResult<Bytes> {
        // Get a list of random post (1 post here but still a list)
        let post = self.get_random_post(fetcher).await?;

        Ok(self.get_random_image_from_post(fetcher, post).await?)
    }

    async fn get_random_image_from_post(&self, fetcher: &Fetcher, post: Self::ResponseType) -> FetchResult<Bytes> {
        // The website returns random posts already with each request, so there's no need to do it here
        let image_id = &post.images[0].id;
        let image_url = format!("{}/{}", self.url_for_image, image_id);

        let image_bytes = fetcher.fetch_image(&image_url).await?;

        Ok(image_bytes)
    }
}


// Deserialize allows the response to be parsed into this struct
// (unneeded fields can be left out)
#[derive(Deserialize)]
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

#[derive(Deserialize)]
/// The structure that the API of the site will return
pub struct NekosMoeResponse {
    images: Vec<NekosPost>
}

// ---------------------


// ----- Nekos.moe -----

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