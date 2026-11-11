#!/bin/bash
set -e

echo "Running Quality Guard..."

echo "--- Checking formatting ---"
cargo fmt --all -- --check

echo "--- Running Clippy ---"
cargo clippy --workspace -- -D warnings

echo "--- Running Tests ---"
cargo test --workspace

echo "Quality Guard passed successfully!"
