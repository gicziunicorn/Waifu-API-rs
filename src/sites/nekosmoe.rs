// ----- Nekos.moe -----
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{error::{FetchError::{self, RateLimitError, ResponseError}, FetchResult}, fetcher::Fetcher, sites::{Post, Posts}};


static RANDOM_URL: &'static str = "https://nekos.moe/api/v1/random/image";
static IMAGE_URL: &'static str = "https://nekos.moe/image";
static TEST_URL: &'static str = "https://nekos.moe/api/v1";


/// Nekos.moe serves sfw/nsfw images of catgirls
#[derive(Debug, Clone)]
pub struct NekosMoe {
    pub name: &'static str,
    fetcher: Fetcher,
    pub ratelimit_limit: u16,
    pub ratelimit_remaining: u16,
    /// In UTC
    pub ratelimit_reset: Option<DateTime<Utc>>,
}


impl std::fmt::Display for NekosMoe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


impl NekosMoe {
    /// Create a new instance of this site with its data
    pub fn new(fetcher: Fetcher) -> Self {
        NekosMoe {
            name: "Nekos.moe",
            fetcher,
            ratelimit_limit: 5,
            ratelimit_remaining: 5,
            ratelimit_reset: None,
        }
    }

    /// Checks if the site's API is available
    pub async fn is_available(&self) -> FetchResult<bool> {
        let fetcher = &self.fetcher;

        let response = fetcher.fetch_url(TEST_URL).await?;

        Ok(response.status().is_success())
    }

    /// Get a number of random post from the site (max 100)
    pub async fn get_random_posts(&mut self, count: u16) -> FetchResult<NekosMoeResponse> {
        if let Some(ratelimit_reset) = self.ratelimit_reset && self.ratelimit_remaining == 0 && Utc::now() < ratelimit_reset {
            return Err(RateLimitError);
        }
        if count > 100 { return Err(FetchError::InvalidArgumentError("Count is more than allowed.")); }

        let fetcher = &self.fetcher;

        let complete_url = format!("{}?count={}", RANDOM_URL, count);
        let res = fetcher.fetch_posts(&complete_url).await?;

        // Get the ratelimits from the headers
        let headers = res.headers();
        let rl_rem = headers.get("x-ratelimit-remaining")
            .ok_or(ResponseError("'x-ratelimit-remaining' field not found".to_string()))?
            .to_str()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
            .parse::<u16>()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?;
        let rl_rst = DateTime::<Utc>::from_timestamp_millis(
            headers.get("x-ratelimit-reset")
                .ok_or(ResponseError("'x-ratelimit-reset' field not found".to_string()))?
                .to_str()
                .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
                .parse::<i64>()
                .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
        )
        .ok_or(ResponseError(format!("Failed to convert header value: Timestamp is out of range.")))?;

        self.ratelimit_remaining = rl_rem;
        self.ratelimit_reset = Some(rl_rst);

        // Convert the site's response to the struct
        let nekos_response = res.json::<NekosMoeResponse>().await?;

        Ok(nekos_response)
    }

    /// Get a random image from an already requested post
    pub async fn get_random_image_from_post(&self, post: &NekosMoePost) -> FetchResult<Bytes> {
        let fetcher = &self.fetcher;
        let image_id = &post.id;
        let image_url = format!("{}/{}", IMAGE_URL, image_id);

        let image_bytes = fetcher.fetch_image(&image_url).await?;

        Ok(image_bytes)
    }

    /// Get a random image from the site
    pub async fn get_random_image(&mut self) -> FetchResult<Bytes> {
        // Get a list of random post (1 post here but still a list)
        // The website returns random posts already with each request, so there's no need to randomize here
        let posts = self.get_random_posts(1).await?;
        let post = &posts.images[0];

        Ok(self.get_random_image_from_post(post).await?)
    }
}



// Deserialize allows the response to be parsed into this struct
// (unneeded fields can be left out)
#[derive(Debug, Clone, Deserialize)]
/// The structure of a post for the site
pub struct NekosMoePost {
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
#[derive(Debug, Deserialize)]
pub struct NekosMoeResponse {
    pub images: Vec<NekosMoePost>
}


// Implement into generic post
impl Into<Post> for NekosMoePost {
    fn into(self) -> Post {
        let image_url = format!("{}/{}", IMAGE_URL, self.id);

        Post {
            id: self.id,
            image_url: image_url,
            image_source: None,
            artist_name: self.artist,
        }
    }
}

impl Into<Posts> for NekosMoeResponse {
    fn into(self) -> Posts {
        self.images.into_iter().map(Into::into).collect()
    }
}