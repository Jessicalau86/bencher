# 🐰 Bencher

[Bencher](https://bencher.dev) is a suite of continuous benchmarking tools designed to help catch performance regressions in CI.

It consists of:

- `bencher` CLI
- Bencher API Server
- Bencher Web UI

The best place to start is the [Bencher Quick Start](https://bencher.dev/docs/how-to/quick-start) tutorial.

## Run Bencher Locally

Pull and run Docker images:

```
docker compose up -d
```

OR

Build and run Docker images:

```
./scripts/docker_run.sh
```

Then open your browser to [localhost:3000](http://localhost:3000).

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a>
or <a href="LICENSE-MIT">MIT license</a> at your discretion.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Bencher by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.