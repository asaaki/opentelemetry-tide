<h1 align="center">opentelemetry-tide</h1>
<div align="center"><strong>

[OpenTelemetry][otel] integration for [Tide][tide]

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

## Notes

* It only implements very basic request tracing on the middleware layer.
  If you need spans for your executed code, you need to add them yourself.
* The majority of the implementation is based on <https://github.com/OutThereLabs/actix-web-opentelemetry>.
* It does not provide metrics, so it cannot be used for Prometheus metrics. Yet. Maybe I'll add it in the future.
  Or you want to contribute the extension. ;-)
* You probably do not want to use it in production. 🤷

## How to use

```sh
# Run jaeger in background
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 jaegertracing/all-in-one:latest

# Run server example with tracing middleware
cargo run --example server

# Make a request or two ...
curl http://localhost:3000/

# Open browser and view the traces
firefox http://localhost:16686/
```

![example jaeger trace](https://github.com/asaaki/opentelemetry-tide/blob/main/.assets/jaeger-trace.png)

### Code example

#### `Cargo.toml`

```toml
[dependencies]
async-std = { version = "1.7", features = ["attributes"] }
opentelemetry = { version = "0.10", features = ["async-std"] }
opentelemetry-jaeger = { version = "0.9", features = ["async-std"] }
opentelemetry-tide = "0.4"
thrift = "0.13"
tide = "0.13"
```

#### `server.rs`

```rust
use opentelemetry::global as otel_global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry_semantic_conventions::resource;
use opentelemetry_tide::OpenTelemetryTracingMiddleware;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::start();
    otel_global::set_text_map_propagator(TraceContextPropagator::new());

    let tags = [resource::SERVICE_VERSION.string(VERSION)];

    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name("example-server")
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install()
        .expect("pipeline install failure");

    let mut app = tide::new();
    app.with(OpenTelemetryTracingMiddleware::new(tracer));
    app.at("/").get(|_| async move { Ok("Hello, OpenTelemetry!") });
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
```



## Cargo Features:

## Safety

This crate uses ``#![forbid(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>



<!-- links -->
[otel]: https://crates.io/crates/opentelemetry
[surf]: https://crates.io/crates/surf
[tide]: https://crates.io/crates/tide
