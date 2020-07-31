#![forbid(unsafe_code)]
#![cfg_attr(feature = "docs", feature(doc_cfg))]
// #![deny(missing_docs)]
#![deny(unused_imports)]
#![deny(missing_debug_implementations)]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]
#![allow(clippy::all)]

use {
    http_types::{
        headers::{HeaderName, HeaderValue},
        Version,
    },
    kv_log_macro as log,
    opentelemetry::{
        api::{
            trace::b3_propagator::B3Encoding, B3Propagator, Context, HttpTextCompositePropagator, HttpTextFormat,
            KeyValue, SpanKind, StatusCode, TraceContextPropagator, Tracer, Value,
        },
        global,
    },
    tide::{Middleware, Next, Request, Result},
    url::Url,
};

static HTTP_METHOD_ATTRIBUTE: &str = "http.method";
static HTTP_URL_ATTRIBUTE: &str = "http.url";
static HTTP_TARGET_ATTRIBUTE: &str = "http.target";
static HTTP_SCHEME_ATTRIBUTE: &str = "http.scheme";
static HTTP_STATUS_CODE_ATTRIBUTE: &str = "http.status_code";
static HTTP_STATUS_TEXT_ATTRIBUTE: &str = "http.status_text";
static HTTP_FLAVOR_ATTRIBUTE: &str = "http.flavor";
// TODO: parse UA header
// static HTTP_USER_AGENT_ATTRIBUTE: &str = "http.user_agent";

static HTTP_HOST_ATTRIBUTE: &str = "http.host";
static HTTP_SERVER_NAME_ATTRIBUTE: &str = "http.server_name";
// needs access to the framework's router:
// static HTTP_ROUTE_ATTRIBUTE: &str = "http.route";
static HTTP_CLIENT_IP_ATTRIBUTE: &str = "http.client_ip";
static NET_PEER_IP_ATTRIBUTE: &str = "net.peer.ip";
static NET_HOST_PORT_ATTRIBUTE: &str = "net.host.port";
static UNKNOWN: &str = "unknown";
static EMPTY: &str = "";

#[derive(Default, Debug)]
pub struct OpenTelemetryTracingMiddleware {
    _priv: (),
}

impl OpenTelemetryTracingMiddleware {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for OpenTelemetryTracingMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        let mut req_headers = std::collections::HashMap::new();
        for (k, v) in req.iter() {
            req_headers.insert(k.to_string(), v.last().to_string());
        }

        let b3_propagator = B3Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);
        let trace_context_propagator = TraceContextPropagator::new();
        let composite_propagator =
            HttpTextCompositePropagator::new(vec![Box::new(b3_propagator), Box::new(trace_context_propagator)]);
        let _parent = composite_propagator
            .extract_with_context(&Context::current(), &req_headers)
            .attach();
        let tracer = global::tracer("opentelemetry-tide");
        let mut builder = tracer.span_builder("http.request");
        builder.span_kind = Some(SpanKind::Server);

        let url = req.url();
        let attributes = vec![
            KeyValue::new(HTTP_METHOD_ATTRIBUTE, req.method().to_string()),
            KeyValue::new(HTTP_FLAVOR_ATTRIBUTE, http_version_str(req.version())),
            KeyValue::new(HTTP_SCHEME_ATTRIBUTE, url.scheme()),
            KeyValue::new(HTTP_URL_ATTRIBUTE, url.as_str()),
            KeyValue::new(HTTP_HOST_ATTRIBUTE, url.host_str().unwrap_or(UNKNOWN)),
            KeyValue::new(HTTP_TARGET_ATTRIBUTE, http_target(url)),
            KeyValue::new(HTTP_SERVER_NAME_ATTRIBUTE, url.domain().unwrap_or(UNKNOWN)),
            KeyValue::new(
                NET_HOST_PORT_ATTRIBUTE,
                Value::U64(url.port_or_known_default().unwrap_or(0u16) as u64),
            ),
            KeyValue::new(HTTP_CLIENT_IP_ATTRIBUTE, net_addr_ip(req.peer_addr())),
            KeyValue::new(NET_PEER_IP_ATTRIBUTE, net_addr_ip(req.remote())),
        ];
        builder.attributes = Some(attributes);
        let span = tracer.build(builder);
        let _guard = tracer.mark_span_as_active(span);

        // call next in the chain
        let mut res = next.run(req).await;

        tracer.get_active_span(|span| {
            span.set_attribute(KeyValue::new(
                HTTP_STATUS_CODE_ATTRIBUTE,
                Value::U64(res.status() as u16 as u64),
            ));
            span.set_attribute(KeyValue::new(
                HTTP_STATUS_TEXT_ATTRIBUTE,
                res.status().canonical_reason(),
            ));
            span.set_attribute(KeyValue::new("http.body.length", res.len().map_or(0u64, |v| v as u64)));

            if let Some(ct) = res.content_type() {
                span.set_attribute(KeyValue::new("http.content_type", ct.to_string()));
            }

            span.set_status(span_status(res.status()), EMPTY.to_string());
        });

        let mut carrier = std::collections::HashMap::new();
        composite_propagator.inject_context(&Context::current(), &mut carrier);
        for (k, v) in carrier {
            let header_name = HeaderName::from_bytes(k.clone().into_bytes());
            let header_value = HeaderValue::from_bytes(v.clone().into_bytes());
            if let (Ok(name), Ok(value)) = (header_name, header_value) {
                res.insert_header(name, value);
            } else {
                log::error!("Could not compose header for pair: ({}, {})", k, v);
            }
        }

        Ok(res)
    }
}

#[inline]
fn http_version_str(version: Option<Version>) -> &'static str {
    use Version::*;
    if let Some(v) = version {
        match v {
            Http0_9 => "0.9",
            Http1_0 => "1.0",
            Http1_1 => "1.1",
            Http2_0 => "2.0",
            Http3_0 => "3.0",
            _ => UNKNOWN,
        }
    } else {
        "1.1 (assumed)" // tide seems to not set the version correctly, but states it's 1.1 only
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
fn net_addr_ip(input: Option<&str>) -> String {
    if let Some(addr) = input {
        let (ip_string, _port) = addr_to_tuple(addr);
        ip_string
    } else {
        UNKNOWN.to_string()
    }
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
        100..=399 => StatusCode::OK,
        401 => StatusCode::Unauthenticated,
        403 => StatusCode::PermissionDenied,
        404 => StatusCode::NotFound,
        429 => StatusCode::ResourceExhausted,
        #[allow(clippy::match_overlapping_arm)]
        400..=499 => StatusCode::InvalidArgument,
        501 => StatusCode::Unimplemented,
        503 => StatusCode::Unavailable,
        504 => StatusCode::DeadlineExceeded,
        #[allow(clippy::match_overlapping_arm)]
        500..=599 => StatusCode::Internal,
        _ => StatusCode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
