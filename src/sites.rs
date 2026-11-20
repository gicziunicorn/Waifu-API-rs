pub mod nekosmoe;
pub mod nekosbest;
pub mod waifuim;

use chrono::{DateTime};
use serde::{Deserialize, Serialize};
use bytes::Bytes;
use strum_macros::{Display as EnumDisplay, EnumIter};

use crate::{
    error::FetchResult, fetcher::Fetcher, sites::{
        nekosbest::NekosBest, nekosmoe::NekosMoe, waifuim::WaifuIm,
    }
};


/// An enum just for the name of the websites, without holding any value
///
/// Can be used for listing or easier matching
///
/// Implements EnumIter and EnumDisplay from strum
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Hash, EnumDisplay, Serialize, Deserialize)]
pub enum Sites {
    NekosMoe,
    NekosBest,
    WaifuIm,
}

impl Sites {
    /// Create a [`Site`] corresponding to the value
    pub fn site(&self, fetcher: Fetcher) -> Site {
        match self {
            Sites::NekosMoe => Site::NekosMoe( NekosMoe::new(fetcher) ),
            Sites::NekosBest => Site::NekosBest( NekosBest::new(fetcher) ),
            Sites::WaifuIm => Site::WaifuIm( WaifuIm::new(fetcher) )
        }
    }
}


/// An enum for holding a specific site
#[derive(Debug, Clone)]
pub enum Site {
    NekosMoe(NekosMoe),
    NekosBest(NekosBest),
    WaifuIm(WaifuIm)
}


// Some general functions to make coding easier
impl Site {
    pub fn into_sites(self) -> Sites {
        self.into()
    }

    pub fn get_ratelimit_remaining(&self) -> u16 {
        match self {
            Self::NekosMoe(site) => site.ratelimit_remaining,
            Self::NekosBest(site) => site.ratelimit_remaining,
            Self::WaifuIm(site) => site.ratelimit_remaining,
        }
    }

    pub fn get_ratelimit_reset(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::NekosMoe(site) => site.ratelimit_reset,
            Self::NekosBest(site) => site.ratelimit_reset,
            Self::WaifuIm(site) => Some(Utc::now()),
        }
    }

    pub async fn is_available(&self) -> FetchResult<bool> {
        match self {
            Self::NekosMoe(site) => site.is_available().await,
            Self::NekosBest(site) => site.is_available().await,
            Self::WaifuIm(site) => site.is_available().await,
        }
    }

    pub async fn get_random_posts(&mut self, count: u16) -> FetchResult<Posts> {
        match self {
            Self::NekosMoe(site) => site.get_random_posts(count).await.map(Into::into),
            Self::NekosBest(site) => site.get_random_posts(count).await.map(|(posts, _)| posts.into()),
            Self::WaifuIm(site) => site.get_random_posts(count).await.map(Into::into),
        }
    }

    pub async fn get_random_image(&mut self) -> FetchResult<Bytes> {
        match self {
            Self::NekosMoe(site) => site.get_random_image().await,
            Self::NekosBest(site) => site.get_random_image().await.map(|i| i.0), // throw away the category chosen
            Self::WaifuIm(site) => site.get_random_image().await,
        }
    }
}

impl Into<Sites> for Site {
    fn into(self) -> Sites {
        match self {
            Self::NekosMoe(_) => Sites::NekosMoe,
            Self::NekosBest(_) => Sites::NekosBest,
            Self::WaifuIm(_) => Sites::WaifuIm,
        }
    }
}


/// A general post response type
///
/// Site responses will implement Into to this type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: String,
    pub image_url: String,
    pub image_source: Option<String>,
    pub artist_name: Option<String>,
}

pub type Posts = Vec<Post>;




// leftover ----------------------------------------
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct Rule34Post {
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
#[allow(dead_code)]
type Rule34Response = Vec<Rule34Post>;