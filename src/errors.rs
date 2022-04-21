use crate::errors::CliError::ValidationError;
use http_client::http_types::Error;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};
use CliError::ConnectionError;

impl From<http_client::http_types::Error> for CliError {
    fn from(err: Error) -> Self {
        ConnectionError(err.to_string())
    }
}

impl From<std::string::String> for CliError {
    fn from(err: String) -> Self {
        ValidationError(err)
    }
}

#[derive(Debug, Serialize)]
pub enum CliError {
    ValidationError(String),
    ConnectionError(String),
}

impl std::error::Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError(ref message) | ConnectionError(ref message) => {
                write!(f, "{}", message)
            }
        }
    }
}
