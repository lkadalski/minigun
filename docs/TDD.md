# Technical Design Document

#### Author: lkadalski
#### Created Date: 2021-06-11
#### Last Update: 2021-06-11

## Overview

This is design document for `minigun` application. Main goal is to create load testing tool for HTTP servers. 
Typical user should be able to:
* Perform `n` HTTP requests with `m` concurrent connections against ip address with any HTTP method(GET,POST...)
* Perform as much as possible requests against ip address in `k` seconds  
* CLI should return Report with average request times and other relevant statistics

Main inspiration is [bombardier](http://google.pl) program.

> Implementation should be carried out with Rust language. <br>
> Use Ockham's  [Coding Standard](https://www.ockam.io/learn/how-to-guides/contributing/ockam_rust_code_standard)


## Context

Every back-end developer from time to time hits a performance problem with his service. To test such service there should be a tool which is very performant and produce informative reports.
Solution designed here should consist of:
* Technical Design Document
* Examples
* CLI which handles communication with server over HTTP/HTTPS

Rust Community will assess this project.

## Proposed solution

### CLI

To test a web server, user has to provide properties like:
* IP address
* Port
* Headers(Optionally)
* Body(Optionally)  
* Test Parameters
* Output format(stdout or file?)

Extra features:
* progress bar
* latency distribution
* fine-grained statistics

Beside server address, rest of properties should have a defaults.
Authentication should be provided by a user via `Headers` option.

Test Parameters should allow for such scenarios:
#### Test `n` times with `m` threads
* Request count - how many requests should be performed
* Concurrency - how many concurrent connections should be utilised
#### Test for a `k` seconds with everything you got
* Timeout - how many seconds should `minigun` shoot at


### Example usage:
* Test `n` times with `m` threads <br>
`$ minigun -a 127.0.0.1 -p 8080 -n 1000 -m 100`<br>
```
Report:
  Performed 1000 requests with 100 connections
  Avg. time 20.3 ms
  Responses:
              5xx 0
              4xx 0
              3xx 0
              2XX 1000
              1xx 0
```
* Test for a `k` seconds with everything you got <br>
  `$ minigun -a 127.0.0.1 -p 8080 -t 100`<br>
```
Report:
  Performed 1932 requests with 100 connections for 100 s
  Avg. time 20.3 ms
  Responses:
              5xx 0
              4xx 0
              3xx 0
              2XX 1932
              1xx 0
```

### Test runner

Whole test should be wrapped with `async` runtime. <br>
There should be a such components:
* `job dispatcher` - which dispatch and receives updates from job workers and subsequently produce final report.
  Also keeps the application state. 
* `job worker` - which performs requests and push messages to `job dispatcher`
Communication between those should be handled by channels.
  
Provide also graceful shutdown operation in case of SIGINT.
