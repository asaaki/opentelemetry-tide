use opentelemetry_semantic_conventions::resource;
use opentelemetry_tide::OpenTelemetryTracingMiddleware;
use tide::Request;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tide::log::start();

    let tags = [resource::SERVICE_VERSION.string(VERSION)];

    let (tracer, _uninstall) = opentelemetry_jaeger::new_pipeline()
        .with_service_name("example-server")
        .with_tags(tags.iter().map(ToOwned::to_owned))
        .install()
        .expect("pipeline install failure");

    let mut app = tide::new();
    app.with(OpenTelemetryTracingMiddleware::new(tracer));
    app.at("/").get(|req: Request<()>| async move {
        eprintln!("req.version = {:?}", req.version());
        Ok("Hello, OpenTelemetry!")
    });
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
