use opentelemetry::global;
use opentelemetry::sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_jaeger::Propagator as JaegerPropagator;

pub fn init_global_propagator() {
    global::set_text_map_propagator(composite_propagator());
    // OR you could use a single propagator only:
    // global::set_text_map_propagator(TraceContextPropagator::new());
}

fn composite_propagator() -> TextMapCompositePropagator {
    // W3C spec: https://w3c.github.io/baggage/ - very flexible KV format, can carry more than just trace context data
    let baggage_propagator = BaggagePropagator::new();

    // Uber's original format - probably only useful in a closed jaeger only setup
    let jaeger_propagator = JaegerPropagator::new(); // aka Uber headers

    // Yet another W3C spec: https://www.w3.org/TR/trace-context/ - only for trace context info
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

// make rust (analyzer) happy
#[allow(dead_code)]
fn main() {}
