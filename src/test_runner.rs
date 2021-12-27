use crate::test_dispatcher::{Error, TestResult, TestSuiteRequest};
use crate::errors::{CliError};
use surf::{Request};
use async_std::channel::{Sender, Receiver};
use async_std::task;
use std::time::Instant;
use futures::StreamExt;
use serde::Serialize;

pub struct TestRunner;

#[derive(Debug, Serialize)]
pub enum TestStatus {
    Finished,
    Failed(CliError),
}

impl TestRunner {
    pub async fn run(rx_dispatcher: Receiver<TestSuiteRequest>) -> Result<Receiver<TestResult>, Error> {
        let (tx, rx) = async_std::channel::unbounded();
        task::spawn(Self::listen(rx_dispatcher, tx));
        Ok(rx)
    }
    async fn listen(mut rx_dispatcher: Receiver<TestSuiteRequest>, tx: Sender<TestResult>) {
        let mut request = None;
        while let Some(suite) = rx_dispatcher.next().await {
            if request.is_none() {
                request = Some(TestRunner::build_request(&suite));
            }
            let request = request.clone().unwrap();
            let tx_result = tx.clone();
            task::spawn(TestRunner::perform_test(suite, request, tx_result));
        }
    }

    async fn perform_test(job: TestSuiteRequest, request: Request, report_sender: Sender<TestResult>) {
        for &test_no in &job.request_count {
            let report = TestRunner::execute(&job, &request, test_no).await;
            report_sender.send(report).await.expect(format!("Could not send back a report from test_case {}", test_no).as_str());
        }
    }

    fn build_request(job: &TestSuiteRequest) -> Request {
        let mut request = surf::Request::builder(job.params.method, job.params.url.clone());
        if let Some(body) = &job.params.body {
            request = request.body(body.as_str())
        }
        for header in &job.params.headers {
            request = request.header(&header.name, header.value.clone());
        }
        // request = request.header(&job.params.headers.name, job.params.headers.value.clone());

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


#[cfg(test)]
mod test {
    use crate::options::TargetParameters;
    use crate::test_dispatcher::TestSuiteRequest;
    use crate::test_runner::TestRunner;
    use surf::http::Method;
    use async_std::sync::Arc;
    use http_client::http_types::StatusCode;
    use mockito::{mock};

    #[async_std::test]
    async fn send_request_perf_test() -> std::io::Result<()> {
        let mock = mock("GET", "/hello")
            .with_status(200)
            .with_body("OK")
            .create();
        let job_sender = async_std::channel::unbounded();
        let target = Arc::new(TargetParameters {
            body: None,
            headers: vec![],
            method: Method::Get,
            url: format!("{}/hello", mockito::server_url()).parse().unwrap(),
        });
        let client = surf::Client::new();

        let test_request = TestSuiteRequest { client_id: 2, params: target, client, request_count: vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1] };
        job_sender.0.send(test_request).await.expect("Could not send a message");
        let output = TestRunner::run(job_sender.1).await;
        if let Ok(receiver) = output {
            if let Ok(result) = receiver.recv().await {
                assert_eq!(result.status.unwrap(), StatusCode::Ok)
            }
        }
        mock.assert();
        Ok(())
    }
}
