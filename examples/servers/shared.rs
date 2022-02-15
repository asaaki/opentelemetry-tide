#![doc(hidden)]
#![allow(unused_imports)]

use opentelemetry::{global, trace::TraceError, KeyValue};
use opentelemetry::{
    global::BoxedTracer,
    sdk::{
        propagation::{BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator},
        trace::{self, Config, Sampler, Tracer},
        Resource,
    },
    trace::TracerProvider,
};
use opentelemetry_jaeger::Propagator as JaegerPropagator;
use opentelemetry_semantic_conventions::resource;

include!(concat!(env!("OUT_DIR"), "/build_vars.rs"));

pub fn init_global_propagator() {
    global::set_text_map_propagator(composite_propagator());
    // OR you could use a single propagator only:
    // global::set_text_map_propagator(TraceContextPropagator::new());
}

fn composite_propagator() -> TextMapCompositePropagator {
    // Uber's original format - probably only useful in a closed jaeger only setup
    let jaeger_propagator = JaegerPropagator::new(); // aka Uber headers

    // W3C spec: https://w3c.github.io/baggage/ - very flexible KV format, can carry more than just trace context data
    let baggage_propagator = BaggagePropagator::new();

    // W3C spec: https://www.w3.org/TR/trace-context/ - only for trace context info
    let trace_context_propagator = TraceContextPropagator::new();

    // NB! last wins (and overwrites!); so re-order based on your actual usage or preferences
    // or leave out propagators you definitely do no use;
    // of course, if you send all headers with identical values the order doesn't matter.
    TextMapCompositePropagator::new(vec![
        Box::new(jaeger_propagator),
        Box::new(baggage_propagator),
        Box::new(trace_context_propagator),
    ])
}

#[allow(dead_code)]
pub fn trace_config(version: &str, instance_id: &str) -> Config {
    let tags = [
        resource::SERVICE_VERSION.string(version.to_owned()),
        resource::SERVICE_INSTANCE_ID.string(instance_id.to_owned()),
        resource::PROCESS_EXECUTABLE_PATH.string(std::env::current_exe().unwrap().display().to_string()),
        resource::PROCESS_PID.string(std::process::id().to_string()),
        KeyValue::new("process.executable.profile", PROFILE),
    ];

    trace::config().with_resource(Resource::new(tags))
}

pub fn jaeger_tracer(svc_name: &str, version: &str, instance_id: &str) -> Result<Tracer, TraceError> {
    opentelemetry_jaeger::new_pipeline()
        .with_service_name(svc_name)
        .with_trace_config(trace_config(version, instance_id))
        .install_batch(opentelemetry::runtime::AsyncStd)
}

pub fn global_tracer(svc_name: &'static str, version: &str, instance_id: &str) -> Result<BoxedTracer, TraceError> {
    let _ = jaeger_tracer(svc_name, version, instance_id)?;
    Ok(global::tracer(svc_name))
}
