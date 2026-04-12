# List available recipes
default:
    @just --list

# Run all tests
test:
    cargo test --workspace

# Check formatting and lints
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features

# Format in place
fmt:
    cargo fmt

# Build all workspace members
build:
    cargo build --workspace
