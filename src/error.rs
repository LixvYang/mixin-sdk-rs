use crate::auth::AuthError;
use crate::request::ApiError;
use http::method::InvalidMethod;
use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Request error: {0}")]
    Request(reqwest::Error),
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
    #[error("Authentication error: {0}")]
    Auth(AuthError),
    #[error("API error: {0}")]
    Api(ApiError),
    #[error("Input error: {0}")]
    Input(String),
    #[error("Data not found in API response: {0}")]
    DataNotFound(String),
    #[error("Server error: {0}")]
    Server(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Self {
        Error::Auth(err)
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Error::Api(err)
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(err: InvalidHeaderValue) -> Self {
        Error::Server(err.to_string())
    }
}

impl From<InvalidMethod> for Error {
    fn from(err: InvalidMethod) -> Self {
        Error::Server(err.to_string())
    }
}
