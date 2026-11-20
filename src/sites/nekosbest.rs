// ----- Nekos.best -----
use bytes::Bytes;
use chrono::{DateTime, Utc};
use rand::{rng, seq::IteratorRandom};
use serde::Deserialize;
use strum::IntoEnumIterator;
use strum_macros::{Display as EnumDisplay, EnumIter};

use crate::error::FetchError::{self, RateLimitError, ResponseError};
use crate::sites::{Post, Posts};
use crate::{error::FetchResult, fetcher::Fetcher};


static CATEGORY_URL: &'static str = "https://nekos.best/api/v2";
static TEST_URL: &'static str = "https://nekos.best/api/v2/neko";


/// Nekosbest is an API that servers sfw images and gifs in categories
#[derive(Debug, Clone)]
pub struct NekosBest {
    pub name: &'static str,
    fetcher: Fetcher,
    pub ratelimit_remaining: u16,
    /// In UTC
    pub ratelimit_reset: Option<DateTime<Utc>>,
}


impl std::fmt::Display for NekosBest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


impl NekosBest {
    /// Create a new instance of this site
    pub fn new(fetcher: Fetcher) -> Self {
        NekosBest {
            name: "Nekosbest",
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

        let response = fetcher.fetch_url(TEST_URL).await?;

        Ok(response.status().is_success())
    }

    /// Fetch the image from an already requested post
    pub async fn get_image_from_post(&self, post: &NekosbestPost) -> FetchResult<Bytes> {
        let fetcher = &self.fetcher;

        let bytes = fetcher.fetch_image(&post.url).await?;

        Ok(bytes)
    }

    /// Get a number of random posts from a specific category (max 20)
    pub async fn get_random_posts_with_category(&mut self, category: NekosbestImageCategories, count: u16) -> FetchResult<NekosbestResponse> {
        if self.ratelimit_remaining == 0 { return Err(RateLimitError); }
        if count > 20 { return Err(FetchError::InvalidArgumentError("Count is more than allowed.")); }

        let fetcher = &self.fetcher;

        let url = format!("{}/{category}?amount={count}", CATEGORY_URL);
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

    /// Get a random image from a specific category
    pub async fn get_random_image_with_category(&mut self, category: NekosbestImageCategories) -> FetchResult<Bytes> {
        let posts = self.get_random_posts_with_category(category, 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok(image)
    }

    /// Get a posts from a random category
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// Also returns the chosen category
    pub async fn get_random_posts(&mut self, count: u16) -> FetchResult<(NekosbestResponse, NekosbestImageCategories)> {
        // Create rng in an inner scope so it is dropped right after and won't be a problem for async
        let rand_cat = {
            let mut rng = rng();
            NekosbestImageCategories::iter().choose(&mut rng)
                .ok_or(FetchError::RandomError)?
        };

        let posts = self.get_random_posts_with_category(rand_cat, count).await?;

        Ok( (posts, rand_cat) )
    }

    /// Get a random image from a random category
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// Also returns the chosen category
    pub async fn get_random_image(&mut self) -> FetchResult<(Bytes, NekosbestImageCategories)> {
        let (posts, rand_cat) = self.get_random_posts(1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok( (image, rand_cat) )
    }
}



//--- Response formats ---

#[derive(Debug, Clone, Deserialize)]
/// The structure of a post for Nekosbest
pub struct NekosbestPost {
    pub artist_name: String,
     #[serde(rename = "artist_href")]
    pub artist_profile: String,
     #[serde(rename = "source_url")]
    pub source: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
/// The structure that the API of the site will return
pub struct NekosbestResponse {
    pub results: Vec<NekosbestPost>
}


// Implement into generic post
impl Into<Post> for NekosbestPost {
    fn into(self) -> Post {
        let id = self.url.rsplit_once("/")
            .map(|(_, last)| last).unwrap_or_default()
            .rsplit_once(".")
            .map(|(first, _)| first).unwrap_or_default();

        Post {
            id: id.to_string(),
            image_url: self.url,
            image_source: Some(self.source),
            artist_name: Some(self.artist_name),
        }
    }
}

impl Into<Posts> for NekosbestResponse {
    fn into(self) -> Posts {
        self.results.into_iter().map(Into::into).collect()
    }
}


//--- The categories for the site ---

// implement Display from strum so categories can be accessed as lowercase
// and EnumIter so it can be converted into an iter and randomly chosen from
#[derive(Debug, Clone, Copy, EnumDisplay, EnumIter)]
#[strum(serialize_all = "lowercase")]
/// Nekosbest categories that return images
pub enum NekosbestImageCategories {
    Neko,
    Waifu,
    Husbando,
    Kitsune
}

#[derive(Debug, Clone, Copy, EnumDisplay, EnumIter)]
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
