use crate::options::{Options, TargetParameters, HttpClientType};
use crate::test_runner::{TestRunner, TestStatus};
use crate::report_generator::ReportGenerator;
use std::time::{Duration, Instant};
use futures::{join};
use async_std::task;
use async_std::sync::{Arc, Mutex};
use async_std::channel::{Sender};
use itertools::{Itertools};
use surf::{Client, StatusCode};
use std::ops::{Sub};
use femme::LevelFilter;
use serde::{Serialize};
use futures::executor::block_on;
use async_std::task::JoinHandle;
use http_client::HttpClient;


pub fn initialize(test_command: TestCommand)  {
    // Create application state
    let test_state = Arc::new(Mutex::new(TestState::new()));
    // Create two async channels
    let (test_request_sender, test_request_receiver) = async_std::channel::unbounded();
    let (report_sender, report_receiver) = async_std::channel::unbounded();
    //create two components job_dispatcher and job_runner
    let test_dispatcher = TestDispatcher::new(test_request_sender);
    let test_runner = TestRunner::new(test_request_receiver, report_sender);
    let report_generator = ReportGenerator::new(test_state, report_receiver, test_command.options.test_parameters.output.clone());

    if let None = test_command.options.test_parameters.output {
        log::info!("Targeting {} with ammo of {} bullets using {} connections", &test_command.options.target_parameters.url, &test_command.options.test_parameters.request_count, &test_command.options.test_parameters.connection_count);
    }

    let runner_handle = task::spawn(test_runner.listen_and_run());
    let dispatcher_handle = task::spawn(test_dispatcher.prepare_and_send(test_command));
    let generator_handle = task::spawn(report_generator.listen_and_generate());
    block_on(start(runner_handle, dispatcher_handle, generator_handle));
}

async fn start(runner_handle: JoinHandle<()>, dispatcher_handle: JoinHandle<()>, generator_handle: JoinHandle<()>) {
    join!(runner_handle,dispatcher_handle, generator_handle );
}

impl TestDispatcher {
    async fn prepare_and_send(self, command: TestCommand) {
        let test_requests = Self::generate_test_request_suites(&command);
        for test in test_requests {
            task::spawn(Self::dispatch_test_suite(test, self.test_sender.clone()));
        }
    }

    fn generate_test_request_suites(command: &TestCommand) -> Vec<TestSuiteRequest> {
        let clients = (1..=command.options.test_parameters.connection_count).into_iter().map(|_x| {
            let client = choose_client_backend(command);
            let mut client = surf::Client::with_http_client(client);
            if command.options.test_parameters.debug {
                log::debug!("DEBUG IS ON");
                client = client.with(surf::middleware::Logger::new());
            }
            client
        }).collect_vec();
        let target_params = Arc::new(command.options.target_parameters.clone());
        let mut test_requests = clients.into_iter().enumerate().map(|(id, client)| TestSuiteRequest {
            client_id: id as u32,
            params: target_params.clone(),
            client,
            request_count: vec![],
        }).collect_vec();

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

    async fn dispatch_test_suite(test: TestSuiteRequest, test_sender: Sender<TestSuiteRequest>) {
        log::debug!("Sending {:?}", &test);
        if let Err(err) = test_sender.send(test).await {
            log::error!("Could not request a job with err {:?}", err.to_string());
        }
    }
}

fn choose_client_backend(command: &TestCommand) -> Box<dyn HttpClient> {
    match command.options.test_parameters.client {
        HttpClientType::H1 => {
            Box::new(http_client::h1::H1Client::new())
        }
        HttpClientType::Isahc => {
            Box::new(http_client::isahc::IsahcClient::new())
        }
        HttpClientType::Hyper => {
            Box::new(http_client::hyper::HyperClient::new())
        }
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
        Self {
            options
        }
    }
}

#[derive(Debug)]
pub struct TestState {
    pub test_results: Vec<TestResult>,
    pub start_time: Instant,
    pub finish_time: Instant,
}

impl TestState {
    pub fn new() -> Self {
        TestState {
            test_results: vec![],
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
        TestState::new()
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

#[derive(Debug)]
struct TestDispatcher {
    test_sender: Sender<TestSuiteRequest>,
}

impl TestDispatcher {
    pub fn new(test_sender: Sender<TestSuiteRequest>) -> Self {
        Self {
            test_sender,
        }
    }
}

#[derive(Debug)]
struct Report {
    http_status: String,
    time: Duration,
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use crate::test_dispatcher::{TestDispatcher, TestCommand};
    use crate::options::{Options, TargetParameters, TestParameters, OutputType, HttpClientType};
    use surf::http::Method;

    #[test]
    fn run_with_defaults() {
        Command::cargo_bin("minigun")
            .expect("Binary exists")
            .arg("http://127.0.0.1")
            .assert().success();
    }

    #[async_std::test]
    async fn test_surf_performance() {
        for _x in 0..10 {
            let time = std::time::Instant::now();
            let response = surf::get("https://example.org").await;
            println!("Time for {} status is {:?}", response.unwrap().status(), time.elapsed())
        }
    }

    #[test]
    fn test_generating_test_requests() {
        let options = Options::from_params(TestParameters {
            connection_count: 2,
            request_count: 10,
            debug: false,
            output: None,
            client: HttpClientType::Isahc,
        }, TargetParameters {
            body: None,
            headers: Some(vec!["Authorization: SomeKey".to_string()]),
            method: Method::Get,
            url: surf::Url::parse("https://example.org").unwrap(),
        });
        let command = TestCommand::new(Box::new(options));
        let tests = TestDispatcher::generate_test_request_suites(&command);
        assert_eq!(tests[0].request_count.len(), 5);
        assert_eq!(tests[1].request_count.len(), 5)
    }

    #[test]
    fn test_generating_test_requests_with_uniformity() {
        let options = Options::from_params(TestParameters {
            connection_count: 3,
            request_count: 40,
            debug: false,
            output: OutputType::Cli,
            client: HttpClientType::Isahc
        }, TargetParameters {
            body: None,
            headers: None,
            method: Method::Get,
            url: surf::Url::parse("https://example.org").unwrap(),
        });
        let command = TestCommand::new(Box::new(options));
        let tests = TestDispatcher::generate_test_request_suites(&command);
        assert_eq!(tests[0].request_count.len(), 14);
        assert_eq!(tests[1].request_count.len(), 13);
        assert_eq!(tests[1].request_count.len(), 13)
    }
}
