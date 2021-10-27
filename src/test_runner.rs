use crate::test_dispatcher::{TestResult, TestSuiteRequest};
use crate::errors::{CliError};
use surf::{Response, Client, Request};
use async_std::channel::{Sender, Receiver};
use async_std::task;
use std::time::Instant;
use surf::middleware::{Next, Middleware};
use std::str::FromStr;
use surf::http::headers::{HeaderValue, HeaderName};
use serde::Serialize;

pub struct TestRunner {
    pub(crate) test_request_receiver: Receiver<TestSuiteRequest>,
    pub report_sender: Sender<TestResult>,
}

impl TestRunner {
    pub fn new(test_receiver: Receiver<TestSuiteRequest>, report_sender: Sender<TestResult>) -> Self {
        Self {
            test_request_receiver: test_receiver,
            report_sender,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TestStatus {
    Finished,
    Failed(CliError),
}

impl TestRunner {
    pub async fn listen_and_run(self) {
        while let Ok(task) = self.test_request_receiver.recv().await {
            log::debug!("Received new TestSuite {:?}", &task);
            task::spawn(Self::perform_test(task, self.report_sender.clone()));
        }
    }

    async fn perform_test(job: TestSuiteRequest, report_sender: Sender<TestResult>) {
        let request = TestRunner::build_request(&job);
        for &test_no in &job.request_count {
            let report = TestRunner::execute(&job, &request, test_no).await;
            if let Err(err) = report_sender.send(report).await {
                log::error!("Could not send back a report from test_case {} with err {}", test_no, err);
            }
        }
    }

    fn build_request(job: &TestSuiteRequest) -> Request {
        let mut request = surf::Request::builder(job.params.method, job.params.url.clone())
            .body(job.params.body.to_string());
        if let Some(headers) = &job.params.headers {
            for header in headers {
                let header = header.split_at(header.find(':').expect("Could not find ':' pattern in Header String"));
                let name = HeaderName::from_str(header.0).expect("Could not parse Header Name");
                let value = HeaderValue::from_str(header.1).expect("Could not parse Header Value");
                request = request.header(name, value);
            }
        }
        let request = request.build();
        log::debug!("Request Blueprint built: {:?}", request);
        request
    }

    async fn execute(job: &TestSuiteRequest, request: &Request, test: u32) -> TestResult {
        let time = Instant::now();
        let response = job.client.send(request.clone()).await;
        let time_elapsed = time.elapsed();
        log::debug!("FINISHED Job id: {}: with client {} time {:?}", test, job.client_id, time_elapsed);
        match response {
            Ok(response) => TestResult { client_id: job.client_id, test_id: test, job_status: TestStatus::Finished, duration: time_elapsed, status: Some(response.status()) },
            Err(err) => TestResult { client_id: job.client_id, test_id: test, job_status: TestStatus::Failed(CliError::from(err)), duration: time_elapsed, status: None }
        }
    }
}

struct Printer;

#[surf::utils::async_trait]
impl Middleware for Printer {
    async fn handle(
        &self,
        req: Request,
        client: Client,
        next: Next<'_>,
    ) -> surf::Result<Response> {
        log::debug!("sending a request!");
        let response = next.run(req, client).await?;
        log::debug!("request completed!");
        Ok(response)
    }
}


#[cfg(test)]
mod test {
    use crate::options::TargetParameters;
    use crate::test_dispatcher::TestSuiteRequest;
    use crate::test_runner::TestRunner;
    use futures::StreamExt;
    use surf::http::Method;
    use async_std::sync::Arc;

    #[async_std::test]
    async fn send_request_perf_test() -> std::io::Result<()> {
        let mut job_sender = async_std::channel::unbounded();
        let target = Arc::new(TargetParameters {
            body: "".to_string(),
            headers: None,
            method: Method::Get,
            url: "https://example.com".parse().unwrap(),
        });
        let mutex = surf::Client::new();
        let job_time = std::time::Instant::now();

        let test_request = TestSuiteRequest { client_id: 2, params: target.clone(), client: mutex.clone(), request_count: vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1] };
        TestRunner::perform_test(test_request, job_sender.0).await;
        if let Some(test_result) = job_sender.1.next().await {
            println!("{:?}", test_result);
        }
        println!("Test time {:?}", job_time.elapsed());

        Ok(())
    }
}
