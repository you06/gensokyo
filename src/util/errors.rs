use crate::request::Request;
use std::result::Result as stdResult;
use thiserror::Error;

pub type Result<T> = stdResult<T, Error>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("tso error {0}")]
    TSOError(TSOError),
    #[error("request error {0}")]
    RequestError(RequestError),
    #[error("shard error {0}")]
    ShardError(ShardError),
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ShardError {
    #[error("split on {0} failed")]
    SplitError(String),
}

impl From<ShardError> for Error {
    fn from(e: ShardError) -> Error {
        Error::ShardError(e)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TSOError {}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RequestError {
    #[error("channel send error {0}")]
    SendError(String),
}

impl From<RequestError> for Error {
    fn from(e: RequestError) -> Error {
        Error::RequestError(e)
    }
}
