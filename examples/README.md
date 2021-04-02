# Examples

You need:
* [direnv]
* [k6]
* a browser
* a terminal and shells
* some coffee or tea
* curiosity

First create an `.envrc` copy and enable it:

```sh
cp examples/.envrc.sample ./.envrc
direnv allow
```

This provides with env vars which makes the tracing export a bit happier when facing high traffic load on the servers. Otherwise you will loose traces as the apps cannot push to the agent (either the UDP packet is too big, or the amount of spans overwhelm the span processing/export).

This is an issue with the batch based export of the rusty otel crate, and there is ongoing work to improve the situation. Yet you should still prefer batch over simple export. The benefit of batch processing is that your agent/collector … well … gets batches of spans instead of each individually, making your whole tracing pipeline more performant.

```sh
# shell 1:
cargo run --example server

# shell 2:
cargo run --example front-server

# shell 3:
k6 run examples/k6.js

# open URL in browser
http://localhost:16686/
# and enjoy the traces flooding in
```

This example setup has no prometheus part wired up, this is a more complex exercise.
You can head over to [surfing-the-tide] for a full-blown docker-compose based environment.

<!-- links -->
[direnv]: https://direnv.net/
[k6]: https://k6.io/
[surfing-the-tide]: https://github.com/asaaki/surfing-the-tide
