# Contributing to MEV Shield

We love your input! We want to make contributing to MEV Shield as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## Development Process

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code follows the existing style
6. Issue that pull request!

## Code Style

- Use `cargo fmt` to format your code
- Use `cargo clippy` to catch common mistakes
- Follow Rust naming conventions
- Write descriptive commit messages

## Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

## Pull Request Process

1. Update the README.md with details of changes to the interface
2. Update the CHANGELOG.md with your changes
3. The PR will be merged once you have the sign-off of at least one maintainer

## Any contributions you make will be under the Apache 2.0 License

When you submit code changes, your submissions are understood to be under the same [Apache 2.0 License](LICENSE) that covers the project.

## Report bugs using GitHub Issues

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/aurigraph/mev-shield/issues/new).

## License

By contributing, you agree that your contributions will be licensed under its Apache 2.0 License.
