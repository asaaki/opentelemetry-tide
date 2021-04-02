use http_types::{Body, StatusCode};
use opentelemetry::{
    global,
    metrics::{Counter, ValueRecorder},
    sdk::Resource,
    Key, KeyValue, Unit,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::time::SystemTime;
use tide::{Middleware, Next, Request, Response, Result};

const DEFAULT_METRICS_ROUTE: &str = "/metrics";
// adapted to be aligned with naming convention for traces (dots become underscores)
const ROUTE_KEY: Key = Key::from_static_str("http_route");
const METHOD_KEY: Key = Key::from_static_str("http_method");
const STATUS_KEY: Key = Key::from_static_str("http_status_code");

// I chose the 1, 5, 10, 50, … stepping as a compromise between enough details and small data set size
#[rustfmt::skip]
const HISTOGRAM_BOUNDARIES: [f64; 15] = [
    0.000100, 0.000500,                       // nanoseconds on ms base, ms on a seconds base
    0.001, 0.005, 0.010, 0.050, 0.100, 0.500, // μs on ms base, ms on seconds base
    1.000, 5.000, 10.000,                     // ms or seconds hereafter, depending on base
    50.000, 100.000, 500.000,
    1000.000
];
const SUMMARY_QUANTILES: [f64; 6] = [0.50, 0.75, 0.90, 0.95, 0.99, 0.999];

/// The middleware struct to be used in tide
#[derive(Debug)]
pub struct OpenTelemetryMetricsMiddleware {
    exporter: PrometheusExporter,
    request_count: Counter<u64>,
    error_count: Counter<u64>,
    duration: ValueRecorder<f64>,
    duration_ms: ValueRecorder<f64>,
}

fn init_meter(custom_kvs: Option<Vec<KeyValue>>) -> PrometheusExporter {
    if let Some(kvs) = custom_kvs {
        opentelemetry_prometheus::exporter()
            .with_default_histogram_boundaries(HISTOGRAM_BOUNDARIES.to_vec())
            .with_default_summary_quantiles(SUMMARY_QUANTILES.to_vec())
            .with_resource(Resource::new(kvs))
            .init()
    } else {
        opentelemetry_prometheus::exporter()
            .with_default_histogram_boundaries(HISTOGRAM_BOUNDARIES.to_vec())
            .with_default_summary_quantiles(SUMMARY_QUANTILES.to_vec())
            .init()
    }
}

impl OpenTelemetryMetricsMiddleware {
    /// Instantiate the middleware
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let mut app = tide::new();
    /// app.with(opentelemetry_tide::OpenTelemetryMetricsMiddleware::new());
    /// app.at("/").get(|_| async { Ok("Metricized!") });
    /// ```
    pub fn new() -> Self {
        let exporter = init_meter(None);
        // as a starting point we use RED method:
        // https://www.weave.works/blog/the-red-method-key-metrics-for-microservices-architecture/
        let meter = global::meter("red-metrics");
        let request_count = meter
            .u64_counter("http_server_requests_count")
            .with_description("total request count (since start of service)")
            .init();
        let error_count = meter
            .u64_counter("http_server_errors_count")
            .with_description("failed request count (since start of service)")
            .init();
        let duration = meter
            .f64_value_recorder("http_server_request_duration_seconds")
            .with_unit(Unit::new("seconds"))
            .with_description("request duration histogram (in seconds, since start of service)")
            .init();

        let duration_ms = meter
            .f64_value_recorder("http_server_request_duration_ms")
            .with_unit(Unit::new("milliseconds"))
            .with_description("request duration histogram (in milliseconds, since start of service)")
            .init();

        Self {
            exporter,
            request_count,
            error_count,
            duration,
            duration_ms,
        }
    }
}

impl Default for OpenTelemetryMetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for OpenTelemetryMetricsMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        if req.url().path() == DEFAULT_METRICS_ROUTE {
            let encoder = TextEncoder::new();
            let metric_families = self.exporter.registry().gather();
            let mut result = Vec::new();
            encoder.encode(&metric_families, &mut result)?;
            let mut res = Response::new(StatusCode::Ok);
            res.set_content_type(tide::http::mime::PLAIN);
            res.set_body(Body::from_bytes(result));
            Ok(res)

        // regular request came in, handle and serve it
        } else {
            let mut labels = Vec::with_capacity(3);
            labels.push(ROUTE_KEY.string(req.url().path().to_string()));
            labels.push(METHOD_KEY.string(req.method().to_string()));

            let timer = SystemTime::now();

            // call next in the chain
            let res = next.run(req).await;

            let elapsed = timer.elapsed();
            let elapsed_sec = elapsed.clone().map(|t| t.as_secs_f64()).unwrap_or_default();
            let elapsed_ms = elapsed.map(|t| t.as_secs_f64() * 1_000f64).unwrap_or_default();

            labels.push(STATUS_KEY.i64(u16::from(res.status()).into()));

            if res.status().is_server_error() {
                self.error_count.add(1, &labels)
            }
            self.request_count.add(1, &labels);
            self.duration.record(elapsed_sec, &labels);
            self.duration_ms.record(elapsed_ms, &labels);
            Ok(res)
        }
    }
}
