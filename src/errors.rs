use std::fmt::{Display, Formatter, Debug};
use http_client::http_types::Error;
use serde::{Serialize};
use CliError::ConnectionError;
use crate::errors::CliError::ValidationError;


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
            ValidationError(ref message) => {
                write!(f, "{}", message)
            }
            ConnectionError(ref message) => {
                write!(f, "{}", message)
            }
        }
    }
}
