#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.

set -e
clear || true

echo "==> cargo check"
cargo check --workspace --all-features

echo "==> cargo check (wasm, ui_panels)"
cargo check --all-features --lib --target wasm32-unknown-unknown -p ui_panels

echo "==> cargo test"
cargo test --workspace --lib

echo "==> cargo fmt --check"
cargo fmt --all 

echo "==> cargo clippy"
cargo clippy --workspace -- -D warnings

echo "==> trunk build"
(cd ui_panels && trunk build)

echo "✅ All checks passed"
