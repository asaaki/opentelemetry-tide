//! Example "frontend" server for testing distributed traces
//!
//! Start backend before making any calls to the frontend
//! ```sh
//! cargo run --example server
//! ```
//!
//! Basic call
//! ```sh
//! curl 'http://127.0.0.1:4000/' -i
//! ```
//!
//! And then check jaeger to see multiple spans across services.

use http_types::headers::{HeaderName, HeaderValue};
use opentelemetry::{
    global,
    trace::{FutureExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_semantic_conventions::resource;
use opentelemetry_tide::OpenTelemetryTracingMiddleware;
use std::collections::HashMap;
use tide::Request;

const SVC_NAME: &str = env!("CARGO_CRATE_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
include!(concat!(env!("OUT_DIR"), "/build_vars.rs"));

mod shared;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::start();
    shared::init_global_propagator();

    let tags = [
        resource::SERVICE_VERSION.string(VERSION),
        resource::SERVICE_INSTANCE_ID.string("frontend-753"),
        resource::PROCESS_EXECUTABLE_PATH.string(std::env::current_exe().unwrap().display().to_string()),
        resource::PROCESS_PID.string(std::process::id().to_string()),
        KeyValue::new("process.executable.profile", PROFILE),
    ];

    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name(SVC_NAME)
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install()
        .expect("pipeline install failure");

    let mut app = tide::with_state(surf::client());
    app.with(OpenTelemetryTracingMiddleware::new(tracer));

    app.at("/").get(|req: Request<surf::Client>| async move {
        // collect current tracing data, so we can pass it down
        let cx = Context::current();
        let mut injector = HashMap::new();
        global::get_text_map_propagator(|propagator| propagator.inject_context(&cx, &mut injector));

        let client = req.state();
        let mut surf_request = client.get("http://localhost:3000/").build();

        for (k, v) in injector {
            let header_name = HeaderName::from_bytes(k.clone().into_bytes());
            let header_value = HeaderValue::from_bytes(v.clone().into_bytes());
            if let (Ok(name), Ok(value)) = (header_name, header_value) {
                surf_request.insert_header(name, value);
            } else {
                eprintln!("Could not compose header for pair: ({}, {})", k, v);
            }
        }

        let upstream_res = async {
            let tracer = global::tracer("(child)");
            let span = tracer.start("surf.client.send");
            let cx = Context::current_with_value(span);
            client.send(surf_request).with_context(cx).await
        };

        // let mut ures = ur.with_context(cx).await?;

        let body = format!(
            "upstream responded with: \n{}",
            upstream_res
                .with_context(cx)
                .await?
                .take_body()
                .into_string()
                .await
                .unwrap()
        );

        Ok(body)
    });
    app.listen("127.0.0.1:4000").await?;

    Ok(())
}
