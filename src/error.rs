use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    Init(reqwest::Error),
    Network(reqwest::Error),
    Parse(reqwest::Error),
    Status(StatusCode),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            Init(source) => Some(source),
            Network(source) => Some(source),
            Parse(source) => Some(source),
            Status(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
