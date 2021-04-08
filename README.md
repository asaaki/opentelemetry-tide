<h1 align="center"><img src="https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/opentelemetry-tide-logo.svg" width=128 height=128><br>opentelemetry-tide</h1>
<div align="center"><strong>

[OpenTelemetry] integration for [Tide]

</strong></div><br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/opentelemetry-tide">
    <img src="https://img.shields.io/crates/v/opentelemetry-tide.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- docs.rs -->
  <a href="https://docs.rs/opentelemetry-tide">
    <img src="https://img.shields.io/badge/docs.rs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
    <!-- <img src="https://docs.rs/opentelemetry-tide/badge.svg"
      alt="docs.rs docs" /> -->
  </a>
  <!-- CI -->
  <a href="https://crates.io/crates/opentelemetry-tide">
    <img src="https://img.shields.io/github/workflow/status/asaaki/opentelemetry-tide/CI/main?style=flat-square"
      alt="CI status" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/opentelemetry-tide">
    <img src="https://img.shields.io/crates/d/opentelemetry-tide.svg?style=flat-square"
      alt="Download" />
  </a>
</div>

Add OpenTelemetry tracing and metrics support to your tide application.
Be part of the new observability movement!

## Notes

* It only implements very basic request tracing on the middleware layer.
  If you need spans for your executed code, you need to add them yourself.
* The majority of the implementation is based on <https://github.com/OutThereLabs/actix-web-opentelemetry>.
* It provides basic prometheus metrics, based on the [RED method].
* You probably do not want to use it in production. 🤷

## How to use

```sh
# Run jaeger in background
docker run -d \
  -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 \
  jaegertracing/all-in-one:latest

# Run server example with tracing middleware
cargo run --example server

# Make a request or two ...
curl http://localhost:3000/

# Open browser and view the traces
firefox http://localhost:16686/

# Check the prometheus metrics endpoint
curl http://localhost:3000/metrics
```

## Example

### `Cargo.toml`

```toml
# ...

[dependencies]
async-std = { version = "1.9", features = ["attributes"] }
opentelemetry = { version = "0.13", features = ["async-std", "rt-async-std"] }
opentelemetry-jaeger = { version = "0.12", features = ["async-std"] }
opentelemetry-tide = "0.7"
tide = "0.16"
```

### `server.rs`

```rust
use opentelemetry::{global, KeyValue, runtime};
use opentelemetry_semantic_conventions::resource;
use opentelemetry_tide::TideExt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::with_level(tide::log::LevelFilter::Warn);

    let tags = [resource::SERVICE_VERSION.string(VERSION)];

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("example-server")
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install_batch(runtime::AsyncStd)
        .expect("pipeline install failure");

    let metrics_kvs = vec![KeyValue::new("K", "V")];

    let mut app = tide::new();

    app.with_middlewares(tracer, Some(metrics_kvs));

    app.at("/").get(|_| async move {
        Ok("Hello, OpenTelemetry!")
    });

    app.listen("0.0.0.0:3000").await?;

    global::shutdown_tracer_provider();

    Ok(())
}
```

## Cargo Features

|      flag | description |
| --------: | :---------- |
|   `trace` | enables **tracing** middleware; enabled by default via `full`
| `metrics` | enables **metrics** middleware; enabled by default via `full`
|    `full` | includes both `trace` and `metrics` features, enabled by default

## Safety

This crate uses ``#![forbid(unsafe_code)]`` to ensure everything is implemented in 100% Safe Rust.


<!-- links -->
[RED method]: https://www.weave.works/blog/the-red-method-key-metrics-for-microservices-architecture/

## License

<sup>
Licensed under either of
  <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or
  <a href="LICENSE-MIT">MIT license</a>
at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

<!-- links -->
[OpenTelemetry]: https://crates.io/crates/opentelemetry
[Tide]: https://crates.io/crates/tide
