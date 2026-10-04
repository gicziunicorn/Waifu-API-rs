use thiserror::Error;
use tokio::{sync::oneshot::error::RecvError};


// Custom errors (using thiserror)
#[derive(Error, Debug)]
/// An error that can happen during fetching
pub enum FetchError {
    #[error("Network error {0}")]
    NetworkError(#[source] wreq::Error),

    #[error("HTTP error ({code}): {message}")]
    HTTPError {
        code: u16,
        message: &'static str,
        body: Option<String>
    },

    #[error("Invalid response: {0}")]
    ResponseError(String),

    #[error("Network thread error: {0}")]
    ThreadError(&'static str),

    #[error("Network thread receiver failed. The sender was dropped before sending.")]
    ReceiverError(#[source] RecvError),

    #[error("Failed to get random element.")]
    RandomError,

    #[error("Rate-limit reached!")]
    RateLimitError,
}

pub type FetchResult<T> = Result<T, FetchError>;

// impl from a few types so ? can be used easily
impl From<wreq::Error> for FetchError {
    fn from(value: wreq::Error) -> Self {
        Self::NetworkError(value)
    }
}

impl From<RecvError> for FetchError {
    fn from(value: RecvError) -> Self {
        Self::ReceiverError(value)
    }
}