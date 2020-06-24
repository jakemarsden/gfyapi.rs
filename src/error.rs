use crate::dto::ErrorResponse;
use reqwest::StatusCode;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Initialization(reqwest::Error),
    Connect(reqwest::Error),
    UnparsableResponseBody(reqwest::Error, Option<String>),
    HttpClientError(StatusCode, ErrorResponse),
    HttpServerError(StatusCode),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Error::Initialization(err) => err,
            Error::Connect(err) => err,
            Error::UnparsableResponseBody(err, _) => err,
            Error::HttpClientError(_, _) => return None,
            Error::HttpServerError(_) => return None,
        })
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
