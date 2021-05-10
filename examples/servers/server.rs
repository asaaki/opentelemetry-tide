/*!
Example server for testing

Basic call (always creates fresh traces):

```sh
curl 'http://127.0.0.1:3000/' -i
```

Call with parent trace (check request and response headers, trace ID should match):

```sh
curl 'http://127.0.0.1:3000/' -H 'traceparent: 00-00110022003300440055006600770088-0011223344556677-01' -i
```
*/

use opentelemetry_tide::{MetricsConfig, TideExt};

mod shared;

type MainResult = Result<(), Box<dyn std::error::Error>>;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() -> MainResult {
    // tide::log::with_level(tide::log::LevelFilter::Warn);
    tide::log::with_level(tide::log::LevelFilter::Debug);
    shared::init_global_propagator();
    let tracer = shared::jaeger_tracer(SVC_NAME, VERSION, "backend-123")?;

    let mut app = tide::new();
    app.with_middlewares(tracer, MetricsConfig::default());
    app.at("/").get(|_| async move { Ok("Hello, OpenTelemetry!") });

    app.listen("0.0.0.0:3000").await?;
    opentelemetry::global::force_flush_tracer_provider();
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
