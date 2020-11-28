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
//! async-std = {version =  "1.7", features = ["attributes"]}
//! opentelemetry = { version = "0.10", features = ["async-std"] }
//! opentelemetry-jaeger = { version = "0.9", features = ["async-std"] }
//! opentelemetry-tide = "0.4"
//! tide = "0.15"
//! ```
//!
//! ## `server.rs`
//!
//! ```rust,no_run
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
//!     let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
//!         .with_service_name("example-server")
//!         .with_tags(tags.iter().map(ToOwned::to_owned))
//!         .install()
//!         .expect("pipeline install failure");
//!
//!     let mut app = tide::new();
//!     app.with(OpenTelemetryTracingMiddleware::new(tracer));
//!     app.at("/").get(|req: Request<()>| async move {
//!         eprintln!("req.version = {:?}", req.version());
//!         Ok("Hello, OpenTelemetry!")
//!     });
//!     app.listen("127.0.0.1:3000").await?;
//!
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
#![doc(test(attr(allow(unused_variables), deny(warnings))))]

use std::convert::TryFrom;

use opentelemetry::{
    trace::{FutureExt, SpanKind, StatusCode, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_semantic_conventions::trace;
use tide::{http::Version, Middleware, Next, Request, Result};
use url::Url;

/// The middleware struct to be used in tide
#[derive(Default, Debug)]
pub struct OpenTelemetryTracingMiddleware<T: Tracer> {
    tracer: T,
}

impl<T: Tracer> OpenTelemetryTracingMiddleware<T> {
    /// Instantiate the middleware
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let mut app = tide::new();
    /// let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline().install().unwrap();
    /// app.with(opentelemetry_tide::OpenTelemetryTracingMiddleware::new(tracer));
    /// app.at("/").get(|_| async { Ok("Traced!") });
    /// ```
    pub fn new(tracer: T) -> Self {
        Self { tracer }
    }
}

#[tide::utils::async_trait]
impl<T: Tracer + Send + Sync, State: Clone + Send + Sync + 'static> Middleware<State>
    for OpenTelemetryTracingMiddleware<T>
{
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        let method = req.method().clone();
        let url = req.url().clone();

        let mut attributes = vec![
            trace::HTTP_METHOD.string(method.to_string()),
            trace::HTTP_SCHEME.string(url.scheme().to_owned()),
            trace::HTTP_URL.string(url.to_string()),
            trace::HTTP_TARGET.string(http_target(&url)),
        ];

        attributes.reserve(6);

        if let Some(version) = req.version() {
            attributes.push(trace::HTTP_FLAVOR.string(http_version_str(version)));
        }

        if let Some(host) = url.host_str() {
            attributes.push(trace::HTTP_HOST.string(host.to_owned()));
        }

        if let Some(domain) = url.domain() {
            attributes.push(trace::HTTP_SERVER_NAME.string(domain.to_owned()));
        }

        if let Some(port) = url.port_or_known_default() {
            attributes.push(trace::NET_HOST_PORT.i64(port.into()));
        }

        if let Some(addr) = req.remote() {
            attributes.push(trace::NET_PEER_IP.string(net_addr_ip(addr)));
        }

        if let Some(addr) = req.peer_addr() {
            attributes.push(trace::HTTP_CLIENT_IP.string(net_addr_ip(addr)));
        }

        let span = self
            .tracer
            .span_builder(&format!("{} {}", method, url))
            .with_kind(SpanKind::Server)
            .with_attributes(attributes)
            .start(&self.tracer);
        let cx = Context::current_with_span(span);

        // call next in the chain
        let res = next.run(req).with_context(cx.clone()).await;

        let span = cx.span();

        span.set_status(span_status(res.status()), "".to_string());
        span.set_attribute(trace::HTTP_STATUS_CODE.i64(u16::from(res.status()).into()));

        if let Some(len) = res.len().and_then(|len| i64::try_from(len).ok()) {
            span.set_attribute(trace::HTTP_RESPONSE_CONTENT_LENGTH.i64(len));
        }

        Ok(res)
    }
}

#[inline]
fn http_version_str(version: Version) -> &'static str {
    use Version::*;
    match version {
        Http0_9 => "0.9",
        Http1_0 => "1.0",
        Http1_1 => "1.1",
        Http2_0 => "2.0",
        Http3_0 => "3.0",
        _ => "unknown",
    }
}

#[inline]
fn http_target(url: &Url) -> String {
    let mut target = String::from(url.path());
    if let Some(q) = url.query() {
        target.push_str("?");
        target.push_str(q)
    }
    if let Some(f) = url.fragment() {
        target.push_str("#");
        target.push_str(f);
    }
    target
}

#[inline]
fn net_addr_ip(input: &str) -> String {
    let (ip_string, _port) = addr_to_tuple(input);
    ip_string
}

#[inline]
fn addr_to_tuple(input: &str) -> (String, u16) {
    use std::net::SocketAddr;
    use std::str::FromStr;
    let addr: SocketAddr = SocketAddr::from_str(input).expect("malformet socket address str");
    (addr.ip().to_string(), addr.port())
}

#[inline]
fn span_status(http_status: tide::StatusCode) -> StatusCode {
    match http_status as u16 {
        100..=399 => StatusCode::Ok,
        _ => StatusCode::Error,
    }
}
