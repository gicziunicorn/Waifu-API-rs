use chrono::{DateTime, Utc};
use serde::Deserialize;
use bytes::Bytes;
use rand::{rng, seq::IteratorRandom};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{error::{FetchError::{self, RateLimitError, ResponseError}, FetchResult}, fetcher::Fetcher};


// ----- Nekos.moe -----

/// Nekos.moe serves sfw/nsfw images of catgirls
#[derive(Debug)]
pub struct NekosMoe {
    pub name: &'static str,
    pub url_for_random: &'static str,
    pub url_for_image: &'static str,
    pub url_for_test: &'static str,
    fetcher: Fetcher,
}

impl NekosMoe {
    /// Create a new instance of this site with its data
    pub fn new(fetcher: Fetcher) -> Self {
        NekosMoe {
            name: "Nekos.moe",
            url_for_random: "https://nekos.moe/api/v1/random/image",
            url_for_image: "https://nekos.moe/image",
            url_for_test: "https://nekos.moe/api/v1",
            fetcher,
        }
    }

    /// Checks if the site's API is available
    pub async fn is_available(&self) -> FetchResult<bool> {
        let fetcher = &self.fetcher;

        let response = fetcher.fetch_url(self.url_for_test).await?;

        Ok(response.status().is_success())
    }

    /// Get a number of random post from the site (max 100)
    pub async fn get_random_posts(&self, count: u16) -> FetchResult<NekosMoeResponse> {
        let fetcher = &self.fetcher;

        let complete_url = format!("{}?count={}", self.url_for_random, count);

        let res = fetcher.fetch_posts(&complete_url).await?;

        // Convert the site's response to the struct
        let nekos_response = res.json::<NekosMoeResponse>().await?;

        Ok(nekos_response)
    }

    /// Get a random image from an already requested post
    pub async fn get_random_image_from_post(&self, post: &NekosMoePost) -> FetchResult<Bytes> {
        let fetcher = &self.fetcher;
        let image_id = &post.id;
        let image_url = format!("{}/{}", self.url_for_image, image_id);

        let image_bytes = fetcher.fetch_image(&image_url).await?;

        Ok(image_bytes)
    }

    /// Get a random image from the site
    pub async fn get_random_image(&self) -> FetchResult<Bytes> {
        // Get a list of random post (1 post here but still a list)
        // The website returns random posts already with each request, so there's no need to randomize here
        let posts = self.get_random_posts(1).await?;
        let post = &posts.images[0];

        Ok(self.get_random_image_from_post(post).await?)
    }
}


// Deserialize allows the response to be parsed into this struct
// (unneeded fields can be left out)
#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
/// The structure that the API of the site will return
pub struct NekosMoeResponse {
    images: Vec<NekosMoePost>
}

// ---------------------



// ----- Nekos.best -----

#[derive(Debug)]
/// Nekosbest is an API that servers sfw images and gifs in categories
pub struct NekosBest {
    pub name: &'static str,
    pub category_url: &'static str,
    pub url_for_test: &'static str,
    fetcher: Fetcher,
    pub ratelimit_remaining: u16,
    /// In UTC
    pub ratelimit_reset: Option<DateTime<Utc>>,
}

impl NekosBest {
    /// Create a new instance of this site
    pub fn new(fetcher: Fetcher) -> Self {
        NekosBest {
            name: "Nekosbest",
            category_url: "https://nekos.best/api/v2",
            url_for_test: "https://nekos.best/api/v2/neko",
            fetcher,
            ratelimit_remaining: 200,
            ratelimit_reset: None,
        }
    }

    /// Checks if the site's API is available
    ///
    /// This'll attempt to request a random post
    pub async fn is_available(&self) -> FetchResult<bool> {
        let fetcher = &self.fetcher;

        let response = fetcher.fetch_url(self.url_for_test).await?;

        Ok(response.status().is_success())
    }

    /// Get a number of random posts from a specific category (max 20)
    pub async fn get_random_posts_with_category(&mut self, category: NekosbestImageCategories, count: u16) -> FetchResult<NekosbestResponse> {
        if self.ratelimit_remaining == 0 { return Err(RateLimitError); }

        let fetcher = &self.fetcher;

        let url = format!("{}/{category}?amount={count}", self.category_url);
        let res = fetcher.fetch_posts(&url).await?;

        let headers = res.headers();
        let rl_rem = headers.get("x-rate-limit-remaining")
            .ok_or(ResponseError("'x-rate-limit-remaining' field not found".to_string()))?
            .to_str()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
            .parse::<u16>()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?;
        let rl_rst = headers.get("x-rate-limit-reset")
            .ok_or(ResponseError("'x-rate-limit-reset' field not found".to_string()))?
            .to_str()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
            .parse::<DateTime<Utc>>()
            .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?;

        self.ratelimit_remaining = rl_rem;
        self.ratelimit_reset = Some(rl_rst);

        // Convert the site's response to the struct
        let nekos_response = res.json::<NekosbestResponse>().await?;

        Ok(nekos_response)
    }

    /// Get a random image from an already requested post
    pub async fn get_random_image_from_post(&self, post: &NekosbestPost) -> FetchResult<Bytes> {
        let fetcher = &self.fetcher;

        let bytes = fetcher.fetch_image(&post.url).await?;

        Ok(bytes)
    }

    /// Get a random image from a specific category
    pub async fn get_random_image_with_category(&mut self, category: NekosbestImageCategories) -> FetchResult<Bytes> {
        let posts = self.get_random_posts_with_category(category, 1).await?;
        let post = &posts.results[0];

        let image = self.get_random_image_from_post(post).await?;

        Ok(image)
    }

    /// Get a random image from a random category
    ///
    /// Also returns the chosen category
    pub async fn get_random_image(&mut self) -> FetchResult<(Bytes, NekosbestImageCategories)> {
        // Create rng in an inner scope so it is dropped right after and won't be a problem for async
        let rand_cat = {
            let mut rng = rng();
            NekosbestImageCategories::iter().choose(&mut rng)
                .ok_or(FetchError::RandomError)?
        };

        let bytes = self.get_random_image_with_category(rand_cat).await?;

        Ok( (bytes, rand_cat) )
    }
}


#[derive(Debug, Deserialize)]
/// The structure of a post for Nekosbest
pub struct NekosbestPost {
    pub artist_name: String,
     #[serde(rename = "artist_href")]
    pub artist_profile: String,
     #[serde(rename = "source_url")]
    pub source: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
/// The structure that the API of the site will return
pub struct NekosbestResponse {
    pub results: Vec<NekosbestPost>
}


// -- The categories for the site

// implement Display from strum so categories can be accessed as lowercase
// and EnumIter so it can be converted into an iter and randomly chosen from
#[derive(Debug, Display, EnumIter, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
/// Nekosbest categories that return images
pub enum NekosbestImageCategories {
    Neko,
    Waifu,
    Husbando,
    Kitsune
}

#[derive(Debug, Display, EnumIter, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
/// Nekosbest categories that return gifs
pub enum NekosbestGifCategories {
    Angry,
    Baka,
    Bite,
    Bleh,
    Blowkiss,
    Blush,
    Bonk,
    Bored,
    Carry,
    Clap,
    Confused,
    Cry,
    Cuddle,
    Dance,
    Facepalm,
    Feed,
    Handhold,
    Handshake,
    Happy,
    Highfive,
    Hug,
    Kabedon,
    Kick,
    Kiss,
    Lappillow,
    Laugh,
    Lurk,
    Nod,
    Nom,
    Nope,
    Nya,
    Pat,
    Peck,
    Poke,
    Pout,
    Punch,
    Run,
    Salute,
    Shake,
    Shocked,
    Shoot,
    Shrug,
    Sip,
    Slap,
    Sleep,
    Smile,
    Smug,
    Spin,
    Stare,
    Tableflip,
    Teehee,
    Think,
    ThumbsUp,
    Tickle,
    Wag,
    Wave,
    Wink,
    Yawn,
    Yeet,
}


// ---------------------











// leftover ----------------------------------------
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