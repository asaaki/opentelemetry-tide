use opentelemetry::{api::KeyValue, global, sdk};
use opentelemetry_tide::OpenTelemetryTracingMiddleware;
use tide::Request;

#[async_std::main]
async fn main() -> thrift::Result<()> {
    tide::log::start();
    // Make sure to initialize the tracer
    init_tracer()?;

    let mut app = tide::new();
    // Here we add the middleware
    app.with(OpenTelemetryTracingMiddleware::new());
    app.at("/").get(|req: Request<()>| async move {
        eprintln!("req.version = {:?}", req.version());
        Ok("Hello, OpenTelemetry!")
    });
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().expect("not a valid endpoint"))
        .with_process(opentelemetry_jaeger::Process {
            service_name: "example-server".into(),
            tags: vec![KeyValue::new("exporter", "jaeger")],
        })
        .init()?;

    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}
