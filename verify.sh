#!/usr/bin/env bash
set -euo pipefail

echo "=== cargo check ==="
cargo check

echo "=== cargo test ==="
cargo test

echo "=== cargo build (release) ==="
cargo build --release

echo "=== smoke test ==="
./target/release/saints-mile --version

echo "✓ All checks passed"
