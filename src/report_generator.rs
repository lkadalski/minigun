use crate::options::OutputType;
use crate::test_dispatcher::{Error, TestResult, TestState};
use async_std::channel::Receiver;
use async_std::task;
use futures::StreamExt;
use indicatif::ProgressStyle;
use prettytable::Table;
use prettytable::{cell, row};
use std::ops::{Add, Div};
use std::time::Duration;

pub struct ReportGenerator;

impl ReportGenerator {
    pub async fn run(
        rx_result: Receiver<TestResult>,
        test_state: TestState,
        output: Option<OutputType>,
    ) -> Result<(), Error> {
        let handle;
        if let None = output {
            handle = task::spawn(Self::listen_for_a_reports(rx_result, test_state));
        } else {
            handle = task::spawn(Self::print_report_to_tty(rx_result, output));
        }
        handle.await
    }

    async fn listen_for_a_reports(
        mut rx_result: Receiver<TestResult>,
        mut test_state: TestState,
    ) -> Result<(), Error> {
        let progress_bar = indicatif::ProgressBar::new(test_state.expected_request_count).with_style(ProgressStyle::default_bar().template("[Total: {pos:>3}/{len}] [{per_sec}] [{percent}%] [ETA: {eta_precise}] [Elapsed: {elapsed_precise}]\n{wide_bar:.cyan/blue}"));
        progress_bar.set_draw_rate(10);
        while let Some(report) = rx_result.next().await {
            progress_bar.inc(1);
            log::debug!("Received report {:?}", &report);
            test_state.test_results.push(report);
        }
        progress_bar.finish();
        test_state.stop_timer();
        Self::generate_report(test_state).await
    }

    async fn print_report_to_tty(
        mut rx_result: Receiver<TestResult>,
        mut output: Option<OutputType>,
    ) -> Result<(), Error> {
        let output_type = output.take().expect("There should be an OutputType");
        while let Some(report) = rx_result.next().await {
            let message = match output_type {
                OutputType::Json => {
                    serde_json::to_string(&report).expect("Could not write json to std output")
                }
                OutputType::Ron => {
                    ron::to_string(&report).expect("Could not write ron to std output")
                }
            };
            println!("{}", message);
        }
        Ok(())
    }

    async fn generate_report(test_state: TestState) -> Result<(), Error> {
        let mut table = Table::new();
        let mut status_table = Table::new();

        let statistics = calculate_statistics(&test_state);
        table.add_row(row![
            "Total Time",
            "Average Request Time ",
            "Total Requests"
        ]);
        table.add_row(row![
            format!("{:?}", test_state.calculate_duration()),
            format!("{:?}", statistics.avg_time),
            format!("{}", test_state.test_results.len())
        ]);
        status_table.add_row(row![
            "HTTP codes",
            "1xx",
            "2xx",
            "3xx",
            "4xx",
            "5xx",
            "Others"
        ]);
        status_table.add_row(row![
            "Count",
            statistics.test_statuses.val_100,
            statistics.test_statuses.val_200,
            statistics.test_statuses.val_300,
            statistics.test_statuses.val_400,
            statistics.test_statuses.val_500,
            statistics.test_statuses.err_val
        ]);

        table.printstd();
        status_table.printstd();
        Ok(())
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
            } else {
                statistics.test_statuses.inc_err_val();
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
