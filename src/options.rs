use std::str::FromStr;
use http_client::http_types::headers::{HeaderName, HeaderValue};
use structopt::StructOpt;
use surf::http::{Method};
use crate::errors::CliError;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "Minigun", about = "CLI Multipurpose HTTP benchmarking tool written in Rust")]
pub struct Options {
    #[structopt(flatten)]
    pub target_parameters: TargetParameters,
    #[structopt(flatten)]
    pub test_parameters: TestParameters,
}

impl Options {
    pub fn new() -> Options {
        Options::from_args()
    }
    #[cfg(test)]
    pub(crate) fn from_params(params: TestParameters, target_params: TargetParameters) -> Self {
        Self {
            target_parameters: target_params,
            test_parameters: params,
        }
    }
}

//TODO validate headers upfront
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "TargetParameters")]
pub struct TargetParameters {
    #[structopt(short, long)]
    pub body: Option<String>,
    /// HTTP Headers to use K: V
    #[structopt(short, long)]
    pub headers: Header,
    /// HTTP Method
    #[structopt(short, long, default_value = "GET")]
    pub method: Method,
    /// Target URL which should Minigun aim for
    pub url: surf::Url,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: HeaderName,
    pub value: HeaderValue,
}

impl FromStr for Header {
    type Err = CliError;

    fn from_str(header: &str) -> Result<Self, Self::Err> {
        Err(CliError::ValidationError("problem".to_string()))
        // let header_split = header.find(":").ok_or_else(||
        // format!("Invalid format for header. Missing ':' token."))?;
        // let header = header.split_at(header_split);
        // let name = HeaderName::from_str(header.0)?;
        // let value = HeaderValue::from_str(header.1)?;
        // return Ok(Self {
        //     name,
        //     value,
        // });
    }

}

fn index_of_comma(header: &str)-> Result<usize, CliError> {
    match header.find(':') {
        None => { Err(CliError::ValidationError("Could not find ':' pattern in Header String'".to_string())) }
        Some(index) => { Ok(index) }
    }
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "TestParameters")]
pub struct TestParameters {
    #[structopt(short, long, default_value = "1")]
    /// Total connections count which should be used in test
    pub connection_count: u32,
    #[structopt(short, long, default_value = "10")]
    /// Total amount of request which should be executed
    pub request_count: u32,
    #[structopt(short, long)]
    /// Enable debug mode
    pub debug: bool,
    /// Output type: ron or json
    #[structopt(short, long)]
    pub output: Option<OutputType>,
    /// Use different type of HTTP client from Surf
    #[structopt(long, default_value = "isahc")]
    pub client: HttpClientType,
}

#[derive(Debug, StructOpt, Clone, PartialEq)]
pub enum HttpClientType {
    Isahc,
    H1,
    Hyper,
}

impl FromStr for HttpClientType {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "isahc" => { Ok(HttpClientType::Isahc) }
            "h1" => { Ok(HttpClientType::H1) }
            "hyper" => { Ok(HttpClientType::Hyper) }
            _ => { Err(CliError::ValidationError(format!("Could not choose client {}", s))) }
        }
    }
}


#[derive(Debug, StructOpt, Clone, PartialEq)]
pub enum OutputType {
    #[structopt(name = "Json")]
    Json,
    #[structopt(name = "Ron")]
    Ron,
}

impl FromStr for OutputType {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => { Ok(OutputType::Json) }
            "ron" => { Ok(OutputType::Ron) }
            _ => { Err(CliError::ValidationError(format!("'{}' There is no such output type. Possible values 'ron' or 'json'", s))) }
        }
    }
}
