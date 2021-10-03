#![doc = include_str!("../README.md")]
#![doc(
    test(attr(allow(unused_variables), deny(warnings))),
    html_favicon_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/favicon.ico",
    html_logo_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/docs.png"
)]
#![forbid(unsafe_code)]
#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unused_imports)]
#![deny(missing_debug_implementations)]
#![warn(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![deny(unused_results)]

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
