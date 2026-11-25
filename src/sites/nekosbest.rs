use std::time::Instant;

// ----- Nekos.best -----
use bytes::Bytes;
use rand::{rng, seq::IteratorRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display as EnumDisplay, EnumIter};

use crate::error::FetchError::{self, RateLimitError, ResponseError};
use crate::helper::iso_8601_to_instant;
use crate::sites::{Post, Posts, Site};
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
    pub ratelimit_reset: Option<Instant>,
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

    /// Get the corresponding value of [`Site`], consuming self
    pub fn into_site(self) -> Site { Site::NekosBest(self) }

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
    pub async fn get_random_posts_with_category(&mut self, category: NekosbestCategories, count: u16) -> FetchResult<NekosbestResponse> {
        if let Some(ratelimit_reset) = self.ratelimit_reset &&
        self.ratelimit_remaining == 0 && Instant::now() < ratelimit_reset {
            return Err(RateLimitError);
        }
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
        let rl_rst = iso_8601_to_instant(
            headers.get("x-rate-limit-reset")
                .ok_or(ResponseError("'x-rate-limit-reset' field not found".to_string()))?
                .to_str()
                .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?
        )
        .map_err(|e| ResponseError(format!("Failed to convert header value: {e}")))?;

        self.ratelimit_remaining = rl_rem;
        self.ratelimit_reset = Some(rl_rst);

        // Convert the site's response to the struct
        let nekos_response = res.json::<NekosbestResponse>().await?;

        Ok(nekos_response)
    }


    /// Get a random image or gif from a specific category
    pub async fn get_random_image_or_gif_with_category(&mut self, category: NekosbestCategories) -> FetchResult<Bytes> {
        let posts = self.get_random_posts_with_category(category, 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok(image)
    }

    /// Get a random image from a specific category
    pub async fn get_random_image_with_category(&mut self, category: NekosbestImageCategories) -> FetchResult<Bytes> {
        let posts = self.get_random_posts_with_category(category.into(), 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok(image)
    }

    /// Get a random gif from a specific category
    pub async fn get_random_gif_with_category(&mut self, category: NekosbestGifCategories) -> FetchResult<Bytes> {
        let posts = self.get_random_posts_with_category(category.into(), 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok(image)
    }


    /// Get a posts from a random category (can be both image or gif)
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// The chosen category is returned
    pub async fn get_random_posts(&mut self, count: u16) -> FetchResult<(NekosbestResponse, NekosbestCategories)> {
        // Create rng in an inner scope so it is dropped right after and won't be a problem for async
        let rand_cat = {
            let mut rng = rng();
            NekosbestCategories::iter().choose(&mut rng)
                .ok_or(FetchError::RandomError)?
        };

        let posts = self.get_random_posts_with_category(rand_cat, count).await?;

        Ok( (posts, rand_cat) )
    }

    /// Get a random image or gif from a random category
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// The chosen category is returned
    pub async fn get_random_image_or_gif(&mut self) -> FetchResult<(Bytes, NekosbestCategories)> {
        let (posts, rand_cat) = self.get_random_posts(1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok( (image, rand_cat) )
    }

    /// Get a random image from a random category
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// The chosen category is returned
    pub async fn get_random_image(&mut self) -> FetchResult<(Bytes, NekosbestImageCategories)> {
        let rand_cat = {
            let mut rng = rng();
            NekosbestImageCategories::iter().choose(&mut rng)
                .ok_or(FetchError::RandomError)?
        };

        let posts = self.get_random_posts_with_category(rand_cat.into(), 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok( (image, rand_cat) )
    }

    /// Get a random gif from a random category
    ///
    /// One category will be selected and all posts will be from that category
    ///
    /// The chosen category is returned
    pub async fn get_random_gif(&mut self) -> FetchResult<(Bytes, NekosbestGifCategories)> {
        let rand_cat = {
            let mut rng = rng();
            NekosbestGifCategories::iter().choose(&mut rng)
                .ok_or(FetchError::RandomError)?
        };

        let posts = self.get_random_posts_with_category(rand_cat.into(), 1).await?;
        let post = &posts.results[0];

        let image = self.get_image_from_post(post).await?;

        Ok( (image, rand_cat) )
    }
}



//--- Response formats ---

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The structure of a post for Nekosbest
pub struct NekosbestPost {
    pub anime_name: Option<String>,
    pub artist_name: Option<String>,
     #[serde(rename = "artist_href")]
    pub artist_profile: Option<String>,
     #[serde(rename = "source_url")]
    pub source: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// The structure that the API of the site will return
pub struct NekosbestResponse {
    pub results: Vec<NekosbestPost>
}


// Implement into generic post
impl From<NekosbestPost> for Post {
    fn from(nekosbestpost: NekosbestPost) -> Self {
        let id = nekosbestpost.url.rsplit_once("/")
            .map(|(_, last)| last).unwrap_or_default()
            .rsplit_once(".")
            .map(|(first, _)| first).unwrap_or_default();

        Post {
            id: id.to_string(),
            image_url: nekosbestpost.url,
            image_source: Some(nekosbestpost.source),
            artist_name: nekosbestpost.artist_name,
        }
    }
}

impl From<NekosbestResponse> for Posts {
    fn from(nekosbestres: NekosbestResponse) -> Self {
        nekosbestres.results.into_iter().map(Into::into).collect()
    }
}


//--- The categories for the site ---

// implement Display from strum so categories can be accessed as lowercase
// and EnumIter so it can be converted into an iter and randomly chosen from
#[derive(Debug, Clone, Copy, PartialEq, EnumDisplay, EnumIter, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
/// Nekosbest categories that return images
pub enum NekosbestImageCategories {
    Neko,
    Waifu,
    Husbando,
    Kitsune
}

#[derive(Debug, Clone, Copy, PartialEq, EnumDisplay, EnumIter, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, EnumDisplay, EnumIter, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
/// All of the Nekosbest categories
pub enum NekosbestCategories {
    // Image categories
    Neko,
    Waifu,
    Husbando,
    Kitsune,

    // GIF categories
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

impl NekosbestCategories {
    pub fn is_image(&self) -> bool {
        matches!(self,  Self::Neko | Self::Waifu | Self::Husbando | Self::Kitsune)
    }

    pub fn is_gif(&self) -> bool {
        !self.is_image()
    }
}

impl From<NekosbestImageCategories> for NekosbestCategories {
    fn from(category: NekosbestImageCategories) -> Self {
        match category {
            NekosbestImageCategories::Neko => Self::Neko,
            NekosbestImageCategories::Waifu => Self::Waifu,
            NekosbestImageCategories::Husbando => Self::Husbando,
            NekosbestImageCategories::Kitsune => Self::Kitsune,
        }
    }
}

impl From<NekosbestGifCategories> for NekosbestCategories {
    fn from(category: NekosbestGifCategories) -> Self {
        match category {
            NekosbestGifCategories::Angry => Self::Angry,
            NekosbestGifCategories::Baka => Self::Baka,
            NekosbestGifCategories::Bite => Self::Bite,
            NekosbestGifCategories::Bleh => Self::Bleh,
            NekosbestGifCategories::Blowkiss => Self::Blowkiss,
            NekosbestGifCategories::Blush => Self::Blush,
            NekosbestGifCategories::Bonk => Self::Bonk,
            NekosbestGifCategories::Bored => Self::Bored,
            NekosbestGifCategories::Carry => Self::Carry,
            NekosbestGifCategories::Clap => Self::Clap,
            NekosbestGifCategories::Confused => Self::Confused,
            NekosbestGifCategories::Cry => Self::Cry,
            NekosbestGifCategories::Cuddle => Self::Cuddle,
            NekosbestGifCategories::Dance => Self::Dance,
            NekosbestGifCategories::Facepalm => Self::Facepalm,
            NekosbestGifCategories::Feed => Self::Feed,
            NekosbestGifCategories::Handhold => Self::Handhold,
            NekosbestGifCategories::Handshake => Self::Handshake,
            NekosbestGifCategories::Happy => Self::Happy,
            NekosbestGifCategories::Highfive => Self::Highfive,
            NekosbestGifCategories::Hug => Self::Hug,
            NekosbestGifCategories::Kabedon => Self::Kabedon,
            NekosbestGifCategories::Kick => Self::Kick,
            NekosbestGifCategories::Kiss => Self::Kiss,
            NekosbestGifCategories::Lappillow => Self::Lappillow,
            NekosbestGifCategories::Laugh => Self::Laugh,
            NekosbestGifCategories::Lurk => Self::Lurk,
            NekosbestGifCategories::Nod => Self::Nod,
            NekosbestGifCategories::Nom => Self::Nom,
            NekosbestGifCategories::Nope => Self::Nope,
            NekosbestGifCategories::Nya => Self::Nya,
            NekosbestGifCategories::Pat => Self::Pat,
            NekosbestGifCategories::Peck => Self::Peck,
            NekosbestGifCategories::Poke => Self::Poke,
            NekosbestGifCategories::Pout => Self::Pout,
            NekosbestGifCategories::Punch => Self::Punch,
            NekosbestGifCategories::Run => Self::Run,
            NekosbestGifCategories::Salute => Self::Salute,
            NekosbestGifCategories::Shake => Self::Shake,
            NekosbestGifCategories::Shocked => Self::Shocked,
            NekosbestGifCategories::Shoot => Self::Shoot,
            NekosbestGifCategories::Shrug => Self::Shrug,
            NekosbestGifCategories::Sip => Self::Sip,
            NekosbestGifCategories::Slap => Self::Slap,
            NekosbestGifCategories::Sleep => Self::Sleep,
            NekosbestGifCategories::Smile => Self::Smile,
            NekosbestGifCategories::Smug => Self::Smug,
            NekosbestGifCategories::Spin => Self::Spin,
            NekosbestGifCategories::Stare => Self::Stare,
            NekosbestGifCategories::Tableflip => Self::Tableflip,
            NekosbestGifCategories::Teehee => Self::Teehee,
            NekosbestGifCategories::Think => Self::Think,
            NekosbestGifCategories::ThumbsUp => Self::ThumbsUp,
            NekosbestGifCategories::Tickle => Self::Tickle,
            NekosbestGifCategories::Wag => Self::Wag,
            NekosbestGifCategories::Wave => Self::Wave,
            NekosbestGifCategories::Wink => Self::Wink,
            NekosbestGifCategories::Yawn => Self::Yawn,
            NekosbestGifCategories::Yeet => Self::Yeet,
        }
    }
}
