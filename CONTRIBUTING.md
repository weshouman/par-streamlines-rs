# Contributing

Contributions are welcome. Please keep behavior changes covered by tests and
run the following before opening a pull request:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Compatibility fixes should include a small reference case that explains the
expected positions, indices, annotations, or error.
