// ----- Waifu.im -----
// Community maintained version
// https://github.com/KorryKatti/waifu-api
#![allow(unused)]
use std::time::{Duration, Instant};

use bytes::Bytes;
use chrono::{DateTime, Local, Utc};
use rand::{rng, seq::IteratorRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display as EnumDisplay, EnumIter};

use crate::error::FetchError::{self, RateLimitError, ResponseError};
use crate::sites::{Post, Posts, Site};
use crate::{error::FetchResult, fetcher::Fetcher};


static RANDOM_URL: &'static str = "https://api.marek-server.com/images";
static TEST_URL: &'static str = "https://api.marek-server.com/";


/// Waifu.im is an API that servers sfw images and gifs in categories
#[derive(Debug, Clone)]
pub struct WaifuIm {
    pub name: &'static str,
    fetcher: Fetcher,
    // ratelimit is 20/s
    pub ratelimit_remaining: u16,
    /// In UTC
    pub ratelimit_reset: Option<Instant>,
}

impl std::fmt::Display for WaifuIm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


impl WaifuIm {
    /// Create a new instance of this site
    pub fn new(fetcher: Fetcher) -> Self {
        WaifuIm {
            name: "Waifu.im",
            fetcher,
            ratelimit_remaining: 20,
            ratelimit_reset: None,
        }
    }

    /// Get the corresponding value of [`Site`], consuming self
    pub fn into_site(self) -> Site { Site::WaifuIm(self) }

    /// Checks if the site's API is available
    pub async fn is_available(&self) -> FetchResult<bool> {
        let fetcher = &self.fetcher;

        let response = fetcher.fetch_url(TEST_URL).await?;

        Ok(response.status().is_success())
    }

    /// Get a number of random posts from a specific category (max 100)
    pub async fn get_random_posts(&mut self, count: u16) -> FetchResult<WaifuImResponse> {
        if self.ratelimit_remaining == 0 { return Err(RateLimitError); }
        if count > 100 { return Err(FetchError::InvalidArgumentError("Count is more than allowed.")); }

        let fetcher = &self.fetcher;

        let url = format!("{}?PageSize={count}", RANDOM_URL);
        let res = fetcher.fetch_posts(&url).await?;

        let current = Instant::now();
        if let Some(ratelimit_reset) = self.ratelimit_reset {
            if ratelimit_reset < current {
                self.ratelimit_reset = Some(current + Duration::from_secs(1));
                self.ratelimit_remaining = 20;
            } else {
                self.ratelimit_remaining -= 1;
            }
        }
        else {
            self.ratelimit_reset = Some(current + Duration::from_secs(1));
            self.ratelimit_remaining = 20;
        }

        // Convert the site's response to the struct
        let nekos_response = res.json::<WaifuImResponse>().await?;

        Ok(nekos_response)
    }

    /// Fetch the image from an already requested post
    pub async fn get_image_from_post(&self, post: &WaifuImPost) -> FetchResult<Bytes> {
        let fetcher = &self.fetcher;

        let bytes = fetcher.fetch_image(&post.url).await?;

        Ok(bytes)
    }

    /// Get a random image
    pub async fn get_random_image(&mut self) -> FetchResult<Bytes> {
        let posts = self.get_random_posts(1).await?;
        let post = &posts.items[0];

        let image = self.get_image_from_post(post).await?;

        Ok(image)
    }
}



//--- Response formats ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaifuImTag {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaifuImArtist {
    pub id: u64,
    pub name: String,
    pub patreon: Option<String>,
    pub pixiv: Option<String>,
    pub twitter: Option<String>,
    #[serde(rename = "devianArt")]
    pub deviantart: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The structure of a post for Nekosbest
pub struct WaifuImPost {
    pub id: u64,
    pub extension: String,
    pub source: String,
    pub artists: Vec<WaifuImArtist>,
    #[serde(rename = "isNsfw")]
    pub is_nsfw: bool,
    #[serde(rename = "isAnimated")]
    pub is_animated: bool,
    pub width: u64,
    pub height: u64,
    pub url: String,
    pub tags: Vec<WaifuImTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The structure that the API of the site will return
pub struct WaifuImResponse {
    pub items: Vec<WaifuImPost>,
    #[serde(rename = "pageNumber")]
    pub page_number: u64,
    #[serde(rename = "totalPages")]
    pub total_pages: u64,
    #[serde(rename = "totalCount")]
    pub total_count: u64,
    #[serde(rename = "hasPreviousPage")]
    pub has_previous_page: bool,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}


// Implement into generic post
impl From<WaifuImPost> for Post {
    fn from(post: WaifuImPost) -> Post {
        Post {
            id: post.id.to_string(),
            image_url: post.url,
            image_source: Some(post.source),
            artist_name: if post.artists.len() == 0 {
                None
            } else {
                Some(post.artists[0].name.clone())
            },
        }
    }
}

impl From<WaifuImResponse> for Posts {
    fn from(response: WaifuImResponse) -> Posts {
        response.items.into_iter().map(Into::into).collect()
    }
}
