/*!
Example "frontend" server for testing distributed traces

Start backend before making any calls to the frontend

```sh
cargo run --example server
```

Basic call

```sh
curl 'http://127.0.0.1:4000/' -i
```

And then check jaeger to see multiple spans across services.
*/

use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
    Context,
};
use opentelemetry_tide::{MetricsConfig, TideExt};
use std::collections::HashMap;
use tide::Request;

mod shared;

type MainResult = Result<(), Box<dyn std::error::Error>>;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const UPSTREAM_SERVICE: &str = "http://localhost:3000/";

#[async_std::main]
async fn main() -> MainResult {
    tide::log::with_level(tide::log::LevelFilter::Warn);
    shared::init_global_propagator();
    let tracer = shared::jaeger_tracer(SVC_NAME, VERSION, "frontend-753")?;

    let mut app = tide::with_state(surf::client());
    let route = std::env::var("METRICS_ROUTE").unwrap_or_else(|_| "/metrics".into());
    let config = MetricsConfig {
        route,
        global_labels: Some(vec![opentelemetry::KeyValue::new("K", "V")]),
        ..MetricsConfig::default()
    };
    app.with_middlewares(tracer, config);

    app.at("/").get(|req: Request<surf::Client>| async move {
        // collect current tracing data, so we can pass it down
        let cx = Context::current();
        let span = cx.span();
        let mut injector = HashMap::new();
        global::get_text_map_propagator(|propagator| propagator.inject_context(&cx, &mut injector));

        let client = req.state();
        let mut surf_request = client.get(UPSTREAM_SERVICE).build();

        for (k, v) in injector {
            surf_request.insert_header(k.as_str(), v.as_str());
        }

        span.add_event("upstream.request.started".into(), vec![]);
        let upstream_res = async {
            let tracer = global::tracer("(child)");
            let span = tracer.start("surf.client.send");
            let cx = Context::current_with_value(span);
            client.send(surf_request).with_context(cx).await
        };

        let body = format!(
            "upstream responded with: \n{}",
            upstream_res
                .with_context(cx.clone())
                .await?
                .take_body()
                .into_string()
                .await
                .unwrap()
        );
        span.add_event("upstream.request.finished".into(), vec![]);

        Ok(body)
    });

    app.listen("0.0.0.0:4000").await?;
    opentelemetry::global::force_flush_tracer_provider();
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
