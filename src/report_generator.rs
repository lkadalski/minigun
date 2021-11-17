use std::mem;
use std::ops::{Add, Div};
use std::time::{Duration};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use prettytable::{Table};
use prettytable::{row, cell};
use crate::test_dispatcher::{TestState, TestResult};
use async_std::channel::Receiver;
use crate::options::OutputType;
use futures::{StreamExt};
use ron::to_string;


pub struct ReportGenerator {
    test_state: Arc<Mutex<TestState>>,
    pub report_receiver: Receiver<TestResult>,
    output: Option<OutputType>,
}

impl ReportGenerator {
    pub async fn listen_and_generate(self) {
        let handle;
        if let None = &self.output {
            handle = task::spawn(Self::listen_for_a_reports(self));
        } else {
            handle = task::spawn(Self::print_report_to_tty(self));
        }
        handle.await;
    }

    async fn listen_for_a_reports(mut self) {
        //TODO add progress bar
        while let Some(report) = self.report_receiver.next().await {
            log::debug!("Received report {:?}", &report);
            let mut data = self.test_state.clone().lock_arc().await;
            data.test_results.push(report);
        }
        if self.report_receiver.is_closed() {
            log::info!("Closing receiver");
            let mut data = self.test_state.clone().lock_arc().await;
            data.stop_timer();
        }
        self.generate_report().await;
        log::error!("Could not read more reports");
    }


    async fn print_report_to_tty(mut self) {
        while let Some(report) = self.report_receiver.next().await {
            let output_type = self.output.take().expect("There should be an OutputType");
            let message = match output_type {
                OutputType::Json => {
                    serde_json::to_string(&report).expect("Could not write json to std output")
                }
                OutputType::Ron => {
                    to_string(&report).expect("Could not write ron to std output")
                }
            };
            println!("{}", message);
        }

        if self.report_receiver.is_closed() {
            log::error!("Closing Message queue");
            // self.test_sender.close();
        }
    }

    pub(crate) async fn generate_report(self) {
        let mut table = Table::new();
        let mut status_table = Table::new();

        let state = Arc::try_unwrap(self.test_state);
        if let Ok(state) = state {
            let mut data = state.into_inner();
            let data = mem::take(&mut data);
            let statistics = calculate_statistics(&data);
            table.add_row(row!["Total Time", "Average Request Time ", "Total Requests"]);
            table.add_row(row![format!("{:?}", data.calculate_duration()), format!("{:?}", statistics.avg_time), format!("{}", data.test_results.len())]);
            status_table.add_row(row!["StatusCodes", "1xx", "2xx", "3xx", "4xx" , "5xx", "Others"]);
            status_table.add_row(row!["Count", statistics.test_statuses.val_100, statistics.test_statuses.val_200, statistics.test_statuses.val_300, statistics.test_statuses.val_400, statistics.test_statuses.val_500, statistics.test_statuses.err_val]);
        } else {
            //TODO FIX IT
            log::error!("COULD NOT AQUIRE STATE FOR REPORT");
        }

        table.printstd();
        status_table.printstd();
    }

    pub fn new(test_state: Arc<Mutex<TestState>>, report_receiver: Receiver<TestResult>, output: Option<OutputType>) -> Self {
        Self {
            test_state,
            report_receiver,
            output,
        }
    }
}


fn calculate_statistics(data: &TestState) -> TestStatistics {
    let mut statistics = TestStatistics {
        test_statuses: TestStatuses {
            val_100: 0,
            val_200: 0,
            val_300: 0,
            val_400: 0,
            val_500: 0,
            err_val: 0,
        },
        avg_time: Duration::new(0, 0),
    };
    if !data.test_results.is_empty() {
        for result in &data.test_results {
            if let Some(success_test) = result.status {
                match success_test as u16 {
                    100..=199 => {
                        statistics.test_statuses.inc_100();
                    }
                    200..=299 => {
                        statistics.test_statuses.inc_200();
                    }
                    300..=399 => {
                        statistics.test_statuses.inc_300();
                    }
                    400..=499 => {
                        statistics.test_statuses.inc_400();
                    }
                    500..=599 => {
                        statistics.test_statuses.inc_500();
                    }
                    _ => {
                        statistics.test_statuses.inc_err_val();
                    }
                }
            }
            statistics.avg_time = statistics.avg_time.add(result.duration);
        }
        statistics.avg_time = statistics.avg_time.div(data.test_results.len() as u32);
    }
    statistics
}

struct TestStatistics {
    test_statuses: TestStatuses,
    avg_time: Duration,
}

struct TestStatuses {
    val_100: u32,
    val_200: u32,
    val_300: u32,
    val_400: u32,
    val_500: u32,
    err_val: u32,
}

impl TestStatuses {
    fn inc_err_val(&mut self) {
        self.err_val += 1;
    }
    fn inc_100(&mut self) {
        self.val_100 += 1;
    }
    fn inc_200(&mut self) {
        self.val_200 += 1;
    }
    fn inc_300(&mut self) {
        self.val_300 += 1;
    }
    fn inc_400(&mut self) {
        self.val_400 += 1;
    }
    fn inc_500(&mut self) {
        self.val_500 += 1;
    }
}
