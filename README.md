```markdown
# rust-ci-example

Tiny Rust project to demonstrate GitHub Actions CI.

## What is included

- A simple library (an `add` function) with unit tests.
- A small binary that uses the library.
- A GitHub Actions workflow (.github/workflows/ci.yml) that:
  - sets up the Rust toolchain,
  - caches dependencies and target,
  - runs `cargo build`, `cargo test`,
  - checks formatting with `cargo fmt -- --check`,
  - runs clippy with `cargo clippy` (treating warnings as errors).

## Run locally

1. Install Rust (rustup): https://rustup.rs
2. Build: `cargo build`
3. Test: `cargo test`
4. Format check: `cargo fmt -- --check` (install rustfmt if needed)
5. Lint: `cargo clippy --all-targets -- -D warnings` (install clippy if needed)

## CI

When you push this repo to GitHub, Actions will run the workflow on pushes and pull requests to main (or whichever branch you choose). You can inspect the runs on the "Actions" tab of your repository.

## Next steps

- Try adding a new test or deliberately introduce a formatting/clippy warning to see CI fail.
- Add a badge to this README after creating the repo to show CI status.
```