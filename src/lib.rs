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
//! async-std = {version =  "1.9", features = ["attributes"]}
//! opentelemetry = { version = "0.13", features = ["rt-async-std"] }
//! opentelemetry-jaeger = { version = "0.12", features = ["async-std"] }
//! opentelemetry-tide = "0.7"
//! tide = "0.16"
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
//!     let tracer = opentelemetry_jaeger::new_pipeline()
//!         .with_service_name("example-server")
//!         .with_tags(tags.iter().map(ToOwned::to_owned))
//!         .install_batch(opentelemetry::runtime::AsyncStd)
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
#![doc(
    test(attr(allow(unused_variables), deny(warnings))),
    html_logo_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/opentelemetry-tide-logo.svg"
)]

use http_types::headers::{HeaderName, HeaderValue};
use kv_log_macro as log;
use opentelemetry::{
    global,
    trace::{FutureExt, Span, SpanKind, StatusCode, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_semantic_conventions::{resource, trace};
use std::collections::HashMap;
use std::{convert::TryFrom, net::IpAddr, net::SocketAddr, str::FromStr};
use tide::{http::Version, Middleware, Next, Request, Result};
use url::Url;

// const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    /// let tracer = opentelemetry_jaeger::new_pipeline().install_simple().unwrap();
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
        // gather trace data from request, used later to conditionally add remote trace info from upstream service
        let mut req_headers = HashMap::new();
        for (k, v) in req.iter() {
            let _ = req_headers.insert(k.to_string(), v.last().to_string());
        }
        let parent_cx = global::get_text_map_propagator(|propagator| propagator.extract(&req_headers));
        drop(req_headers);

        let method = req.method();
        let url = req.url().clone();

        let mut attributes = Vec::with_capacity(13); // 7 required and 6 optional values
        attributes.push(resource::TELEMETRY_SDK_NAME.string(CRATE_NAME));
        attributes.push(resource::TELEMETRY_SDK_VERSION.string(VERSION));
        attributes.push(resource::TELEMETRY_SDK_LANGUAGE.string("rust"));
        attributes.push(trace::HTTP_METHOD.string(method.to_string()));
        attributes.push(trace::HTTP_SCHEME.string(url.scheme().to_owned()));
        attributes.push(trace::HTTP_URL.string(url.to_string()));
        attributes.push(trace::HTTP_TARGET.string(http_target(&url)));

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

        if let Some(addr) = req.remote().and_then(socket_str_to_ip) {
            attributes.push(trace::NET_PEER_IP.string(addr.to_string()));
        }

        if let Some(addr) = req.peer_addr().and_then(socket_str_to_ip) {
            attributes.push(trace::HTTP_CLIENT_IP.string(addr.to_string()));
        }

        let mut span_builder = self
            .tracer
            .span_builder(&format!("{} {}", method, url))
            .with_kind(SpanKind::Server)
            .with_attributes(attributes);

        // make sure our span can be connected to a currently open/active (remote) trace if existing
        if let Some(remote_span_ctx) = parent_cx.remote_span_context() {
            if remote_span_ctx.is_remote() {
                span_builder = span_builder.with_parent_context(parent_cx.clone());
            }
        }

        let span = span_builder.start(&self.tracer);
        span.add_event("request.started".to_owned(), vec![]);
        let cx = &Context::current_with_span(span);

        // call next in the chain
        let mut res = next.run(req).with_context(cx.clone()).await;

        let span = cx.span();
        span.add_event("request.completed".to_owned(), vec![]);

        span.set_status(span_status(res.status()), "".to_string());
        span.set_attribute(trace::HTTP_STATUS_CODE.i64(u16::from(res.status()).into()));

        if let Some(len) = res.len().and_then(|len| i64::try_from(len).ok()) {
            span.set_attribute(trace::HTTP_RESPONSE_CONTENT_LENGTH.i64(len));
        }

        // write trace info to response, so it can be picked up by downstream services
        let mut injector = HashMap::new();
        global::get_text_map_propagator(|propagator| propagator.inject_context(&cx, &mut injector));

        for (k, v) in injector {
            let header_name = HeaderName::from_bytes(k.clone().into_bytes());
            let header_value = HeaderValue::from_bytes(v.clone().into_bytes());
            if let (Ok(name), Ok(value)) = (header_name, header_value) {
                res.insert_header(name, value);
            } else {
                log::error!("Could not compose header for pair: ({}, {})", k, v);
            }
        }

        span.add_event("request.finished".to_owned(), vec![]);
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
        target.push('?');
        target.push_str(q)
    }
    if let Some(f) = url.fragment() {
        target.push('#');
        target.push_str(f);
    }
    target
}

#[inline]
fn socket_str_to_ip(socket: &str) -> Option<IpAddr> {
    SocketAddr::from_str(socket).ok().map(|s| s.ip())
}

#[inline]
fn span_status(http_status: tide::StatusCode) -> StatusCode {
    match http_status as u16 {
        100..=399 => StatusCode::Ok,
        400..=599 => StatusCode::Error,
        _ => StatusCode::Unset,
    }
}
