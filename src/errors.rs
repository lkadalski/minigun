use std::fmt::{Display, Formatter, Debug};
use http_client::http_types::Error;
use serde::{Serialize};

// impl From<surf::Error> for CliError {
//     fn from(err: surf::Error) -> Self {
//         CliError::ConnectionError(err.to_string())
//     }
// }
impl From<http_client::http_types::Error> for CliError{
    fn from(err: Error) -> Self {
        CliError::ValidationError(err.to_string())
    }
}

#[derive(Debug, Serialize)]
pub enum CliError {
    ValidationError(String),
    ConnectionError(String),
}

impl std::error::Error for CliError {

}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
