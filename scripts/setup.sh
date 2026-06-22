#!/bin/bash
set -euo pipefail

echo "=== bl1nk-doc-mcp setup ==="

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust toolchain not found. Please install Rust first."
    echo "  Termux: pkg install rust"
    echo "  Debian/Ubuntu: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "Rust: $(rustc --version)"
echo "Cargo: $(cargo --version)"

echo "=== Install hooks ==="
if [ -f scripts/pre-commit ]; then
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "pre-commit hook installed"
fi

echo "=== Run quality gate ==="
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

echo "=== setup complete ==="
