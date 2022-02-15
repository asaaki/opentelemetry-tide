#![doc = include_str!("../README.md")]
#![doc(
    test(attr(allow(unused_variables), deny(warnings))),
    html_favicon_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/favicon.ico",
    html_logo_url = "https://raw.githubusercontent.com/asaaki/opentelemetry-tide/main/.assets/docs.png"
)]
#![forbid(unsafe_code)]
#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![deny(clippy::unwrap_used)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unused_imports)]
#![deny(unused_results)]
#![warn(clippy::expect_used)]

use opentelemetry::global::BoxedTracer;
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

mod middlewares;

#[cfg(any(feature = "trace", doc))]
pub use middlewares::tracing::OpenTelemetryTracingMiddleware;

#[cfg(any(feature = "metrics", doc))]
pub use middlewares::metrics::{MetricsConfig, OpenTelemetryMetricsMiddleware};

/// this extension trait provides convenience methods for attaching middlewares of this crate
pub trait TideExt<S> {
    /**
    Attaches tracing middleware with provided tracer.

    See [OpenTelemetryTracingMiddleware::new] for details.
    */
    #[cfg(any(feature = "trace", doc))]
    fn with_tracing_middleware(&mut self, tracer: BoxedTracer) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches tracing middleware with the global tracer as default.

    See [OpenTelemetryTracingMiddleware::new] for details.
    */
    #[cfg(any(feature = "trace", doc))]
    fn with_default_tracing_middleware(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches metrics middleware with provided MetricsConfig.

    See [OpenTelemetryMetricsMiddleware::new] for details.
    */
    #[cfg(any(feature = "metrics", doc))]
    fn with_metrics_middleware(&mut self, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches metrics middleware with default MetricsConfig.

    See [OpenTelemetryMetricsMiddleware::new] for details.
    */
    #[cfg(any(feature = "metrics", doc))]
    fn with_default_metrics_middleware(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches both middlewares with provided tracer and MetricsConfig.

    See [OpenTelemetryTracingMiddleware::new] and [OpenTelemetryMetricsMiddleware::new] for details.
    */
    #[cfg(any(all(feature = "trace", feature = "metrics"), doc))]
    fn with_middlewares(&mut self, tracer: BoxedTracer, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;

    /**
    Attaches both middlewares with their defaults.

    See [OpenTelemetryTracingMiddleware::default] and [OpenTelemetryMetricsMiddleware::default] for details.
    */
    #[cfg(any(all(feature = "trace", feature = "metrics"), doc))]
    fn with_default_middlewares(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static;
}

impl<S> TideExt<S> for tide::Server<S> {
    #[cfg(any(feature = "trace", doc))]
    fn with_tracing_middleware(&mut self, tracer: BoxedTracer) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::new(tracer))
    }

    #[cfg(any(feature = "trace", doc))]
    fn with_default_tracing_middleware(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::default())
    }

    #[cfg(any(feature = "metrics", doc))]
    fn with_metrics_middleware(&mut self, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryMetricsMiddleware::new(config))
    }

    #[cfg(any(feature = "metrics", doc))]
    fn with_default_metrics_middleware(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryMetricsMiddleware::default())
    }

    #[cfg(any(all(feature = "trace", feature = "metrics"), doc))]
    fn with_middlewares(&mut self, tracer: BoxedTracer, config: MetricsConfig) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::new(tracer))
            .with(OpenTelemetryMetricsMiddleware::new(config))
    }

    #[cfg(any(all(feature = "trace", feature = "metrics"), doc))]
    fn with_default_middlewares(&mut self) -> &mut Self
    where
        S: Clone + Send + Sync + 'static,
    {
        self.with(OpenTelemetryTracingMiddleware::default())
            .with(OpenTelemetryMetricsMiddleware::default())
    }
}
