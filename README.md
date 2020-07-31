# opentelemetry-tide

[OpenTelemetry][otel] integration for [Tide][tide]

## Client and Server example

```sh
# Run jaeger in background
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 jaegertracing/all-in-one:latest

# Run server example with tracing middleware
cargo run --example server

# Make a request:
curl http://localhost:3000/

# TODO:
# # (In other tab) Run client example with request tracing
# cargo run --example client

# Open browser and view the traces
firefox http://localhost:16686/
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.


<!-- links -->
[otel]: https://crates.io/crates/opentelemetry
[tide]: https://crates.io/crates/tide
