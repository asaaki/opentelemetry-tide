# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Additional changes to original format:
- `Cosmetic` for changes without impact on the code/logic
- `Thank you for your contribution` for shout-outs to the community

## [Unreleased]
### Fixed
- `MetricsConfig` would cause a stack overflow on construction

## [0.8.0] - 2021-05-10

### Added
- `MetricsConfig` to provide a more convenient and publicly accessible way of
  configuring the metrics middleware;
  this allows you to also align histogram boundaries across your services, even if you do not use
  this crate at all (prometheus loves to have a defined set of buckets for identical metrics)

  Usage example:

  ```rust
  use opentelemetry_tide::{MetricsConfig, TideExt};
  // ... snip ...
  app.with_middlewares(tracer, MetricsConfig::default());
  ```

### Changed
- metrics endpoint is configurable now
- histogram boundaries default buckets
  - more granular steppings
  - lower bound is now `0.001` instead of `0.0001`
- (new) summary quantiles default, which has more different nines buckets than the upstream default;
  currently the summary is not really used anywhere yet, otel rust/prom need some changes/features
  exposed to users, yet we still want to communicate a more desired standard nature *if* we would
  use summaries somewhere
- Update dependencies and adapt code accordingly

### Cosmetic
- Fix formatting and notes in README.md
- Ignore "RUSTSEC-2020-0056: stdweb is unmaintained" (#11)
- Ignore aes related audits until upstream dependencies have been updated
  - Ignore "RUSTSEC-2021-0059: `aesni` has been merged into the `aes` crate"
  - Ignore "RUSTSEC-2021-0060: `aes-soft` has been merged into the `aes` crate"
- Use cargo audit directly, as `actions-rs/audit-check` does not support ignore option

## [0.7.0] - 2021-04-03
### Added
- middleware for metrics (`OpenTelemetryMetricsMiddleware`)

  Simplest example to get it up and running:

  ```rust
  // setup
  app.with(opentelemetry_tide::OpenTelemetryMetricsMiddleware::new(None));
  // the rest
  ```

  Note: it will respond to `/metrics` in the same app. This routes is currently hardcoded.
  If that clashes for you, please open an issue or send me a PR with a change.

- tide::Server trait extension `TideExt` to set up middlewares more conveniently:

  ```rust
  use opentelemetry_tide::TideExt;

  // for tracing only
  app.with_tracing_middleware(tracer);
  // for metrics only
  app.with_metrics_middleware(None);
  // using both together
  app.with_middlewares(tracer, None);
  ```

  If you use `.with_middlewares`, keep in mind that the order is _trace -> metrics,_
  so that the tracing middleware can also observe and trace calls to the `/metrics` route.
  If that is an undesired behaviour and/or you want this configurable, please open an issue
  or send me a PR with a change.
  Also the method names are open for debate, but I wouldn't expect people to use many extensions, or that tide would add those names anytime soon.

- feature flags `trace`, `metrics`, and `full`, with "full" being the default.
  If you want to scope it down, use
  ```toml
  [dependencies]
  opentelemetry-tide = { version = "0.7", default-features = false, features = ["trace"]
  ```
  for example.

### Changed
- Update dependencies and adapt code accordingly

  This is a breaking change!
  Most notably: The "uninstall" guard is gone; see examples for how to do it with current otel crates.

### Cosmetic
- "Fix" the issue with examples' shared module
- Improve the example code (move more setup and config to shared module)
- Adds k6.io script and .envrc sample for load testing purposes
- Generate readme from crate documentation and a template (using `cargo-readme`)

## [0.6.2] - 2021-03-08
### Changed
- Update dependencies (tide 0.16)

### Cosmetic
- Use auto merge action for dependabot (patch level updates)

## [0.6.1] - 2021-01-26
### Added
- Dependabot v2 configuration

### Changed
- Update dependencies (#3)
- Include "The opentelemetry-tide Contributors" in the authors list of the crate

### Thank you for your contribution
- [@fiag][fiag]

## [0.6.0] - 2021-01-13
### Added
- Changelog with basic historical summaries
### Changed
- Middleware takes the uninstall guard to support different setup styles and ensures the provider lives long enough.

  Example for an alternative tracing middleware init in [this PR comment](https://github.com/asaaki/opentelemetry-tide/pull/4#issuecomment-757456319).
- dependency updates and adjustment of code and example
### Thank you for your contribution
- [@fiag][fiag]

## [0.5.2] - 2020-12-16
### Fixed
- This patch release fixes an issue around missing PROFILE env var. (#3)
### Thank you for your contribution
- [@arlyon][arlyon]

## 0.5.1 - 2020-12-06
_(untagged crates.io release)_

## [0.5.0] - 2020-12-04
### Changed
- Align span data with specification
- Internal improvements
### Cosmetic
- CI setup improvements
### Thank you for your contribution
- [@arlyon][arlyon]

## 0.4.0
_(skipped)_

## [0.3.1] - 2020-08-02
### Fixed
- doctests

## [0.3.0] - 2020-08-02
_(not released to crates.io)_

## [0.2.0] - 2020-08-01
### Cosmetic
- Readme polishing
- Project cleanups

## [0.1.0] - 2020-08-01
**Initial release**

[Unreleased]: https://github.com/asaaki/opentelemetry-tide/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.6.2...v0.7.0
[0.6.1]: https://github.com/asaaki/opentelemetry-tide/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/asaaki/opentelemetry-tide/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/asaaki/opentelemetry-tide/compare/v0.5.0...v0.5.2
[0.5.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.3.1...v0.5.0
[0.3.1]: https://github.com/asaaki/opentelemetry-tide/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.0.0...v0.1.0

[fiag]: https://github.com/fiag
[arlyon]: https://github.com/arlyon
