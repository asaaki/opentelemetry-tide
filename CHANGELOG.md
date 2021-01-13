# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Additional changes to original format:
- `Cosmetic` for changes without impact on the code/logic
- `Thank you for your contribution` for shout-outs to the community

## [Unreleased]

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

[Unreleased]: https://github.com/asaaki/opentelemetry-tide/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/asaaki/opentelemetry-tide/compare/v0.5.0...v0.5.2
[0.5.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.3.1...v0.5.0
[0.3.1]: https://github.com/asaaki/opentelemetry-tide/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/asaaki/opentelemetry-tide/compare/v0.0.0...v0.1.0

[fiag]: https://github.com/fiag
[arlyon]: https://github.com/arlyon
