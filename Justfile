# List available recipes.
default:
    @just --list

# Format all Rust sources.
fmt:
    cargo fmt --all

# Run Clippy with all targets and features.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests.
test:
    cargo test --all-features

# Run everything the CI runs: fmt check, lint, test, deny.
check:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-features

# Run supply-chain checks.
deny:
    cargo deny check

# Build the release binary.
build:
    cargo build --release --locked

# Run the binary with arguments.
run *args:
    cargo run -- {{args}}

# Clean build artifacts.
clean:
    cargo clean
