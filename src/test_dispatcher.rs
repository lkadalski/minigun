use crate::options::{HttpClientType, Options, TargetParameters};
use crate::report_generator::ReportGenerator;
use crate::test_runner::{TestRunner, TestStatus};
use async_std::channel::Receiver;
use async_std::sync::Arc;
use async_std::task;
use femme::LevelFilter;
use http_client::HttpClient;
use itertools::Itertools;
use serde::Serialize;
use std::ops::Sub;
use std::time::{Duration, Instant};
use surf::{Client, StatusCode};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn initialize(test_command: TestCommand) {
    let test_state = TestState::new(u64::from(
        test_command.options.test_parameters.request_count,
    ));
    let output_type = test_command.options.test_parameters.output.clone();

    async_std::task::block_on(async move {
        ReportGenerator::run(
            TestRunner::run(TestDispatcher::run(&test_command).await?).await?,
            test_state,
            output_type,
        )
        .await
    })
    .expect("Could not block on async std runtime");
}

impl TestDispatcher {
    async fn run(command: &TestCommand) -> Result<Receiver<TestSuiteRequest>, Error> {
        let (tx, rx) = async_std::channel::bounded(1000);
        let test_requests = Self::generate_test_request_suites(command);
        task::spawn(async move {
            for suite in test_requests {
                tx.send(suite)
                    .await
                    .expect("Could not send the generated TestSuite over the channel");
            }
        });
        Ok(rx)
    }

    fn generate_test_request_suites(command: &TestCommand) -> Vec<TestSuiteRequest> {
        let clients = (1..=command.options.test_parameters.connection_count)
            .into_iter()
            .map(|_x| {
                let client = choose_client_backend(command);
                surf::Client::with_http_client(client)
            })
            .collect_vec();
        let target_params = Arc::new(command.options.target_parameters.clone());
        let mut test_requests = clients
            .into_iter()
            .enumerate()
            .map(|(id, client)| TestSuiteRequest {
                client_id: id as u32,
                params: Arc::clone(&target_params),
                client,
                request_count: vec![],
            })
            .collect_vec();

        let mut iterator = test_requests.iter_mut();
        for number in 1..=command.options.test_parameters.request_count {
            if let Some(test_request) = iterator.next() {
                test_request.inc_request_count(number);
            } else {
                iterator = test_requests.iter_mut();
                if let Some(test_request) = iterator.next() {
                    test_request.inc_request_count(number);
                }
            }
        }
        test_requests
    }
}

fn choose_client_backend(command: &TestCommand) -> Box<dyn HttpClient> {
    match command.options.test_parameters.client {
        HttpClientType::H1 => Box::new(http_client::h1::H1Client::new()),
        HttpClientType::Isahc => Box::new(http_client::isahc::IsahcClient::new()),
        HttpClientType::Hyper => Box::new(http_client::hyper::HyperClient::new()),
    }
}

pub struct TestCommand {
    pub options: Box<Options>,
}

impl TestCommand {
    pub fn new(options: Box<Options>) -> Self {
        if options.test_parameters.debug {
            femme::with_level(LevelFilter::Debug);
        }
        Self { options }
    }
}

#[derive(Debug)]
pub struct TestState {
    pub test_results: Vec<TestResult>,
    pub expected_request_count: u64,
    pub start_time: Instant,
    pub finish_time: Instant,
}

impl TestState {
    pub fn new(request_count: u64) -> Self {
        TestState {
            test_results: vec![],
            expected_request_count: request_count,
            start_time: Instant::now(),
            finish_time: Instant::now(),
        }
    }
    pub fn calculate_duration(&self) -> Duration {
        self.finish_time.sub(self.start_time)
    }
    pub fn stop_timer(&mut self) {
        self.finish_time = Instant::now();
    }
}

impl Default for TestState {
    fn default() -> Self {
        TestState::new(0)
    }
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub(crate) client_id: u32,
    pub(crate) test_id: u32,
    pub(crate) job_status: TestStatus,
    pub(crate) duration: Duration,
    pub(crate) status: Option<StatusCode>,
}

#[derive(Debug, Clone)]
pub struct TestSuiteRequest {
    pub(crate) client_id: u32,
    pub(crate) params: Arc<TargetParameters>,
    pub(crate) client: Client,
    pub(crate) request_count: Vec<u32>,
}

impl TestSuiteRequest {
    pub fn inc_request_count(&mut self, test_id: u32) {
        self.request_count.push(test_id);
    }
}

struct TestDispatcher;

#[cfg(test)]
mod tests {
    use crate::options::{Header, HttpClientType, Options, TargetParameters, TestParameters};
    use crate::test_dispatcher::{TestCommand, TestDispatcher};
    use assert_cmd::Command;
    use http_client::http_types::Method;
    use mockito::mock;
    use std::str::FromStr;

    #[test]
    fn run_with_defaults() {
        Command::cargo_bin("minigun")
            .expect("Binary exists")
            .arg(mockito::server_url())
            .assert()
            .success();
    }

    #[async_std::test]
    async fn test_surf_performance() {
        let mock = mock("GET", "/")
            .with_status(200)
            .with_body("OK")
            .expect_at_least(10)
            .create();
        for _x in 0..10 {
            let time = std::time::Instant::now();
            let response = surf::get(mockito::server_url())
                .await
                .expect("Could not send request");
            println!(
                "Time for {} status is {:?}",
                response.status(),
                time.elapsed()
            );
        }
        mock.assert();
    }

    #[test]
    fn test_generating_test_requests() {
        let options = Options::from_params(
            TestParameters {
                connection_count: 2,
                request_count: 10,
                debug: false,
                output: None,
                client: HttpClientType::Isahc,
            },
            TargetParameters {
                body: None,
                headers: vec![Header::from_str("Authorization: SomeKey").unwrap()],
                method: Method::Get,
                url: mockito::server_url().parse().unwrap(),
            },
        );
        let command = TestCommand::new(Box::new(options));
        let tests = TestDispatcher::generate_test_request_suites(&command);
        assert_eq!(tests[0].request_count.len(), 5);
        assert_eq!(tests[1].request_count.len(), 5);
    }

    #[test]
    fn test_generating_test_requests_with_uniformity() {
        let options = Options::from_params(
            TestParameters {
                connection_count: 3,
                request_count: 40,
                debug: false,
                output: None,
                client: HttpClientType::Isahc,
            },
            TargetParameters {
                body: None,
                headers: vec![],
                method: Method::Get,
                url: surf::Url::parse("https://example.org").unwrap(),
            },
        );
        let command = TestCommand::new(Box::new(options));
        let tests = TestDispatcher::generate_test_request_suites(&command);
        assert_eq!(tests[0].request_count.len(), 14);
        assert_eq!(tests[1].request_count.len(), 13);
        assert_eq!(tests[1].request_count.len(), 13);
    }
}
