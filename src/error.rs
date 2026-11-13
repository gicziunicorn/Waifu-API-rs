use thiserror::Error;
use tokio::{sync::oneshot::error::RecvError};


// Wrapper error for source fields
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ResponseBody(String);

impl Into<ResponseBody> for String {
    fn into(self) -> ResponseBody {
        ResponseBody(self)
    }
}


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
        #[source]
        body: Option<ResponseBody>
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

    #[error("Invalid argument! {0}")]
    InvalidArgumentError(&'static str),
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