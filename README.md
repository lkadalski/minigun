
# <p align="center" color="red"> PROJECT IS IN ARCHIVED STATE </p>
# <p align="center"> MINIGUN  🦀 </p>

<p align="center">
  <img src="docs/images/minigun.svg" width="256" height="256" />
</p>

<p align="center">Developed by <a href="https://lkadalski.github.io/" target="_blank">@lkadalski</a></p>

<p align="center">
  <a href="https://opensource.org/licenses/MIT"><img
      src="https://img.shields.io/badge/License-MIT-teal.svg"
      alt="License-MIT"
  /></a>
  <a href="https://github.com/lkadalski/minigun/stargazers">
  <img
      src="https://img.shields.io/github/stars/lkadalski/minigun.svg"
      alt="Repo stars"
  /></a>
    <a href="https://github.com/lkadalski/minigun/releases">
    <img
      src="https://img.shields.io/github/v/release/lkadalski/minigun?sort=semver"
      alt="Latest version"
  /></a>
      <a href="https://github.com/lkadalski/minigun/actions/workflows/build.yml">
      <img
      src="https://github.com/lkadalski/minigun/actions/workflows/build.yml/badge.svg"
      alt="Continous Integration"
  /></a>
      <a href="https://github.com/lkadalski/minigun/releases">
      <img
      src="https://img.shields.io/badge/semantic--release-conventional-e10079?logo=semantic-release"
      alt="Semantic Release: conventional"
  /></a>
  <a href="https://codecov.io/gh/lkadalski/minigun"><img src="https://codecov.io/gh/lkadalski/minigun/branch/master/graph/badge.svg?token=VICVX5EAEM"/></a>

<!--   <a href="https://crates.io/crates/minigun"
    ><img
      src="https://img.shields.io/crates/d/minigun.svg"
      alt="Downloads counter"
  /></a>
  <a href="https://crates.io/crates/minigun"
    ><img
      src="https://img.shields.io/crates/v/minigun.svg"
      alt="Latest version"
  /></a>
   -->

---

- [About minigun 👑](#about-minigun-)
  - [Get started 🏁](#get-started-)
      - [Installation](#installation)
        - [docker](#docker)
        - [binary](#binary)
      - [Usage](#usage)
  - [Documentation 📚](#documentation-)
  - [Contributing and issues 🤝](#contributing-and-issues-)
  - [Changelog ⏳](#changelog-)
  - [License 📃](#license-)


---

## About Minigun 👑

Minigun is a experimental HTTP(S) benchmarking tool. It is written in Rust programming language.
It's highly inspired by https://github.com/codesenberg/bombardier but it's far from it's usefulness. <br>
From my personal tests, current implementation is far more faster(lower latencies) that `bombardier`s. <br>
Please test it yourself!

---

# Get started 🏁

## Installation
There are few methods to obtain binary:
<!--
### cargo
TBD -->
### docker
Images are available [here](https://hub.docker.com/r/lkadalski/minigun/tags)
### binary
See [releases](https://github.com/lkadalski/minigun/releases)

or build from source!

---

## Usage

`minigun [<flags>] <url>` <br>
or  <br>
`minigun --help` <br>
To see all the options and possibilities.
You can either see report of a test or consume a output from each test in json or ron format!

Example: <br>
`$ minigun -c 10 -r 100 http://localhost:8080 -o json` <br>

Other possibilites: <br>
```
CLI Multipurpose HTTP benchmarking tool written in Rust

USAGE:
    minigun [FLAGS] [OPTIONS] <url>

FLAGS:
    -d, --debug      Enable debug mode
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --body <body>
        --client <client>                        Use different type of HTTP client from Surf [default: isahc]
    -c, --connection-count <connection-count>    Total connections count which should be used in test [default: 1]
    -h, --headers <headers>...                   HTTP Headers to use K: V
    -m, --method <method>                        HTTP Method [default: GET]
    -o, --output <output>                        Output type: ron or json
    -r, --request-count <request-count>          Total amount of request which should be executed [default: 10]

ARGS:
    <url>    Target URL which should Minigun aim for
```

---

## Documentation 📚

Minigun was designed using [Technical Design Document](docs/TDD.md)

<!-- The developer documentation can be found on Rust Docs at <https://docs.rs/minigun> -->

---

## Contributing and issues 🤝

Contributions, bug reports, new features and questions are welcome! 😉 <br>
If you have any question or concern, or you want to suggest a new feature, or you want just want to improve minigun, feel free to create ticket!

How to improve this repo ? Please follow [our contributing guidelines](docs/CONTRIBUTING.md)

---

## Changelog ⏳
View minigun's changelog [HERE](docs/CHANGELOG.md)

---

## License 📃

Minigun is licensed under the MIT license.

You can read the entire license [HERE](LICENSE)

Artwork created by Izet
