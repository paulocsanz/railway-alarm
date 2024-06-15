use chrono::{DateTime, Utc};
use std::num::{ParseFloatError, ParseIntError};
use tracing::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CtrlC(#[from] ctrlc::Error),
    #[error("date out of range: {0} - {1}")]
    DateOutOfRange(DateTime<Utc>, u64),
    #[error(transparent)]
    DotEnv(#[from] dotenv::Error),
    #[error(transparent)]
    HMacInvalidLength(#[from] hmac::digest::InvalidLength),
    #[error("invalid time delta: secs = {0}, nano = {1}")]
    InvalidTimeDelta(u64, u64),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("missing env var: {0}")]
    MissingEnvVar(&'static str),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("railway responded with: {0:?}")]
    Railway(Vec<String>),
    #[error("railway data missing: {0}")]
    RailwayDataMissing(&'static str),
    #[error("railway request failed with status {0}: {1}")]
    RailwayStatusFailure(u16, String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("webhook request failed with status {0}: {1}")]
    WebHookStatusFailure(u16, String),
}
