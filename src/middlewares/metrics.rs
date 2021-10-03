use http_types::{Body, StatusCode};
use opentelemetry::{
    global,
    metrics::{Counter, Unit, ValueRecorder},
    sdk::resource::Resource,
    Key, KeyValue,
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

#[rustfmt::skip]
const HISTOGRAM_BOUNDARIES: [f64; 31] = [
    0.001, 0.002, 0.004, 0.006, 0.008,
    0.01, 0.02, 0.04, 0.06, 0.08,
    0.1, 0.2, 0.4, 0.6, 0.8,
    1.0, 2.0, 4.0, 6.0, 8.0,
    10.0, 20.0, 40.0, 60.0, 80.0,
    100.0, 200.0, 400.0, 600.0, 800.0,
    1000.0,
];

#[rustfmt::skip]
const SUMMARY_QUANTILES: [f64; 8] = [
    0.5, 0.75,
    0.90, 0.95,
    0.99, 0.999, 0.9999, 0.99999, // the nines
];

/**
Configuration for the metrics middleware

Unless you need specific values, [MetricsConfig::default()] should be fine for most use cases.

In your application you can shortcut that further down to `Default::default()`,
so you don't have to bring this struct into scope with a `use`.
*/
#[derive(Debug)]
// cannot use #[non_exhaustive] if we want to allow struct expression construction
pub struct MetricsConfig {
    /// Optional vec of key value pairs which then get added as labels to all metrics
    pub global_labels: Option<Vec<KeyValue>>,
    /// A vec of histogram boundaries; set your own fine-tuned buckets for your services
    pub boundaries: Vec<f64>,
    /// A vec of summary quantiles (currently no prometheus-exportable metric is using them)
    pub quantiles: Vec<f64>,
    /// The route which will be used for metrics scraping by prometheus
    pub route: String,
}

impl MetricsConfig {
    /// Initializes a MetricsConfig
    pub fn new(global_labels: Option<Vec<KeyValue>>, boundaries: Vec<f64>, quantiles: Vec<f64>, route: String) -> Self {
        Self {
            global_labels,
            boundaries,
            quantiles,
            route,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self::new(
            None,
            HISTOGRAM_BOUNDARIES.to_vec(),
            SUMMARY_QUANTILES.to_vec(),
            DEFAULT_METRICS_ROUTE.to_owned(),
        )
    }
}

/// The middleware struct to be used in tide
#[derive(Debug)]
pub struct OpenTelemetryMetricsMiddleware {
    route: String,
    exporter: PrometheusExporter,
    request_count: Counter<u64>,
    error_count: Counter<u64>,
    duration: ValueRecorder<f64>,
    duration_ms: ValueRecorder<f64>,
}

#[allow(dead_code)]
fn build_exporter_and_init_meter(config: MetricsConfig) -> PrometheusExporter {
    let mut builder = opentelemetry_prometheus::exporter()
        .with_default_histogram_boundaries(config.boundaries)
        .with_default_summary_quantiles(config.quantiles);
    if let Some(global_labels) = config.global_labels {
        builder = builder.with_resource(Resource::new(global_labels));
    }
    builder.init()
}

impl OpenTelemetryMetricsMiddleware {
    /// Instantiate the middleware
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let mut app = tide::new();
    /// app.with(opentelemetry_tide::OpenTelemetryMetricsMiddleware::new(None));
    /// app.at("/").get(|_| async { Ok("Metricized!") });
    /// ```
    ///
    /// ## with custom metrics configuration
    /// ```rust,no_run
    /// let mut app = tide::new();
    /// let mut config = opentelemetry_tide::MetricsConfig::default();
    /// config.global_tags = Some(vec![opentelemetry::KeyValue::new("K","V")];)
    /// app.with(opentelemetry_tide::OpenTelemetryMetricsMiddleware::new(Some(custom_kvs)));
    /// app.at("/").get(|_| async { Ok("Metricized!") });
    /// ```
    pub fn new(config: MetricsConfig) -> Self {
        let route = config.route.clone();
        let exporter = build_exporter_and_init_meter(config);
        // As a starting point we use RED method:
        // * https://www.weave.works/blog/the-red-method-key-metrics-for-microservices-architecture/
        // * https://grafana.com/files/grafanacon_eu_2018/Tom_Wilkie_GrafanaCon_EU_2018.pdf
        // * http://www.brendangregg.com/usemethod.html
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
            route,
            exporter,
            request_count,
            error_count,
            duration,
            duration_ms,
        }
    }
}

impl Default for OpenTelemetryMetricsMiddleware {
    /// Instantiate the middleware with defaults
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let mut app = tide::new();
    /// app.with(opentelemetry_tide::OpenTelemetryMetricsMiddleware::default());
    /// app.at("/").get(|_| async { Ok("Metricized!") });
    /// ```
    fn default() -> Self {
        Self::new(MetricsConfig::default())
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for OpenTelemetryMetricsMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        if req.url().path() == self.route {
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
