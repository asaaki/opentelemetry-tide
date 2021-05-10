/*!
Add OpenTelemetry tracing and metrics support to your tide application.
Be part of the new observability movement!

# Notes

* It only implements very basic request tracing on the middleware layer.
  If you need spans for your executed code, you need to add them yourself.
* It provides basic prometheus metrics, based on the [RED method].
* This project got inspired by <https://github.com/OutThereLabs/actix-web-opentelemetry>.
* You probably do not want to use it in production. 🤷

# How to use

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

# Example

## `Cargo.toml`

```toml
# ...

[dependencies]
async-std = { version = "1.9", features = ["attributes"] }
opentelemetry = { version = "0.14", features = ["async-std", "rt-async-std"] }
opentelemetry-jaeger = { version = "0.13", features = ["async-std"] }
opentelemetry-tide = "0.9"
tide = "0.16"
```

## `server.rs`

```rust,no_run
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

# Cargo Features

|      flag | description |
| --------: | :---------- |
|   `trace` | enables **tracing** middleware; enabled by default via `full`
| `metrics` | enables **metrics** middleware; enabled by default via `full`
|    `full` | includes both `trace` and `metrics` features, enabled by default

# Safety

This crate uses ``#![forbid(unsafe_code)]`` to ensure everything is implemented in 100% Safe Rust.


<!-- links -->
[RED method]: https://www.weave.works/blog/the-red-method-key-metrics-for-microservices-architecture/
*/

// !!! GENERATE README WITH: cargo readme > README.md

#![forbid(unsafe_code)]
#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unused_imports)]
#![deny(missing_debug_implementations)]
#![warn(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![deny(unused_results)]
#![doc(
    test(attr(allow(unused_variables), deny(warnings))),
    html_favicon_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/favicon.ico",
    html_logo_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/docs.png"
)]

use opentelemetry::trace::Tracer;

const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

mod middlewares;

#[cfg(feature = "trace")]
pub use middlewares::tracing::OpenTelemetryTracingMiddleware;

#[cfg(feature = "metrics")]
pub use middlewares::metrics::{MetricsConfig, OpenTelemetryMetricsMiddleware};

/// this extension trait provides convenience methods for attaching middlewares of this crate
pub trait TideExt<S> {
    /**
    Attaches tracing middleware with provided tracer.

    See [OpenTelemetryTracingMiddleware::new] for details.
    */
    #[cfg(feature = "trace")]
    fn with_tracing_middleware<T>(&mut self, tracer: T) -> &mut Self
    where
        T: Tracer + Send + Sync,
        S: Clone + Send + Sync + 'static;

    /**
    Attaches metrics middleware with provided MetricsConfig.

    See [OpenTelemetryMetricsMiddleware::new] for details.
    */
    #[cfg(feature = "metrics")]
    fn with_metrics_middleware(&mut self, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches both middlewares with provided tracer and MetricsConfig.

    See [OpenTelemetryTracingMiddleware::new] and [OpenTelemetryMetricsMiddleware::new] for details.
    */
    #[cfg(all(feature = "trace", feature = "metrics"))]
    fn with_middlewares<T>(&mut self, tracer: T, config: MetricsConfig) -> &mut Self
    where
        T: Tracer + Send + Sync,
        S: Clone + Send + Sync + 'static;
}

impl<S> TideExt<S> for tide::Server<S> {
    #[cfg(feature = "trace")]
    fn with_tracing_middleware<T>(&mut self, tracer: T) -> &mut Self
    where
        T: Tracer + Send + Sync,
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::new(tracer))
    }

    #[cfg(feature = "metrics")]
    fn with_metrics_middleware(&mut self, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryMetricsMiddleware::new(config))
    }

    #[cfg(all(feature = "trace", feature = "metrics"))]
    fn with_middlewares<T>(&mut self, tracer: T, config: MetricsConfig) -> &mut Self
    where
        T: Tracer + Send + Sync,
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::new(tracer))
            .with(OpenTelemetryMetricsMiddleware::new(config))
    }
}
