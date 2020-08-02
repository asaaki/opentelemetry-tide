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
async-std = {version =  "1.6", features = ["attributes"]}
opentelemetry = "0.7"
opentelemetry-jaeger = "0.6"
opentelemetry-tide = "0.3"
thrift = "0.13"
tide = "0.13"
```

#### `server.rs`

```rust
use opentelemetry::{api::KeyValue, global, sdk};
use opentelemetry_tide::OpenTelemetryTracingMiddleware;
use tide::Request;

#[async_std::main]
async fn main() -> thrift::Result<()> {
    tide::log::start();
    // Make sure to initialize the tracer
    init_tracer()?;

    let mut app = tide::new();
    // Here we add the middleware
    app.with(OpenTelemetryTracingMiddleware::new());
    app.at("/").get(|req: Request<()>| async move {
        eprintln!("req.version = {:?}", req.version());
        Ok("Hello, OpenTelemetry!")
    });
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().expect("not a valid endpoint"))
        .with_process(opentelemetry_jaeger::Process {
            service_name: "example-server".into(),
            tags: vec![KeyValue::new("exporter", "jaeger")],
        })
        .init()?;

    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.


<!-- links -->
[otel]: https://crates.io/crates/opentelemetry
[tide]: https://crates.io/crates/tide
