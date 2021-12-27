use std::time::{Duration, Instant};
use assert_cmd::Command;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use predicates::prelude::*;


#[async_std::test]
async fn test_defaults_with_10_requests() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(ResponseTemplate::new(200))
        .expect(10)
        .mount(&mock_server)
        .await;

    let url = format!("{}", format!("{}/index", &mock_server.uri()));

    let cli = Command::cargo_bin("minigun").unwrap().arg(url).assert();

    cli.success();
}

#[async_std::test]
async fn test_advanced_with_20_requests_2_connections() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(ResponseTemplate::new(200))
        .expect(20)
        .mount(&mock_server)
        .await;

    let url = format!("{}", format!("{}/index", &mock_server.uri()));

    let cli = Command::cargo_bin("minigun").unwrap().arg(url)
        .args(&["-r", "20"])
        .args(&["-c", "2"])
        .args(&["-h", "Authorization: SomeKey"])
        .assert();

    cli.success();
}

#[async_std::test]
async fn test_output_as_ron() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}", format!("{}/index", &mock_server.uri()));

    Command::cargo_bin("minigun").unwrap().arg(url)
        .args(&["-r", "1"])
        .args(&["-c", "1"])
        .args(&["-o", "ron"])
        .assert()
        .stdout(predicate::str::starts_with("(client_id:0,test_id:1,job_status:Finished,duration:(secs:0,"))
        .stdout(predicate::str::ends_with("),status:Some(200))\n"))
        .success();
}

#[async_std::test]
async fn test_advanced_with_2k_requests_20_connections() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/index"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2000)
        .mount(&mock_server)
        .await;

    let url = format!("{}", format!("{}/index", &mock_server.uri()));
    Command::cargo_bin("minigun").unwrap().arg(url)
        .args(&["-r", "2000"])
        .args(&["-c", "20"])
        .args(&["-o", "ron"])
        .assert()
        .stdout(predicate::str::starts_with("(client_id:"))
        .stdout(predicate::str::ends_with("),status:Some(200))\n"))
        .success();
}
