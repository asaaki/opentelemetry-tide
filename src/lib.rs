//! # opentelemetry-tide
//!
//! OpenTelemetry integration for Tide
//!
//! # Example
//!
//! ## `Cargo.toml`
//!
//! ```toml
//! [dependencies]
//! async-std = { version = "1.9", features = ["attributes"] }
//! opentelemetry = { version = "0.13", features = ["async-std", "rt-async-std"] }
//! opentelemetry-jaeger = { version = "0.12", features = ["async-std"] }
//! opentelemetry-tide = "0.7"
//! tide = "0.16"
//! ```
//!
//! ## `server.rs`
//!
//! ```rust,no_run
//! use opentelemetry::{global, runtime};
//! use opentelemetry_semantic_conventions::resource;
//! use opentelemetry_tide::OpenTelemetryTracingMiddleware;
//! use tide::Request;
//!
//! const VERSION: &'static str = env!("CARGO_PKG_VERSION");
//!
//! #[async_std::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     tide::log::start();
//!
//!     let tags = [resource::SERVICE_VERSION.string(VERSION)];
//!
//!     let tracer = opentelemetry_jaeger::new_pipeline()
//!         .with_service_name("example-server")
//!         .with_tags(tags.iter().map(ToOwned::to_owned))
//!         .install_batch(runtime::AsyncStd)
//!         .expect("pipeline install failure");
//!
//!     let mut app = tide::new();
//!     app.with(OpenTelemetryTracingMiddleware::new(tracer));
//!     app.at("/").get(|req: Request<()>| async move {
//!         eprintln!("req.version = {:?}", req.version());
//!         Ok("Hello, OpenTelemetry!")
//!     });
//!     app.listen("127.0.0.1:3000").await?;
//!     global::shutdown_tracer_provider();
//!     Ok(())
//! }
//! ```

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

// const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

mod middlewares;

#[cfg(feature = "trace")]
pub use middlewares::tracing::OpenTelemetryTracingMiddleware;

#[cfg(feature = "metrics")]
pub use middlewares::metrics::OpenTelemetryMetricsMiddleware;
