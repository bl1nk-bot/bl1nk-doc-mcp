.PHONY: all check test lint fmt clean build release setup hooks bench help

all: fmt lint test check

check:
	cargo check --workspace --all-targets --locked

test:
	cargo test --workspace --locked

lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings

fmt:
	cargo fmt --all -- --check

clean:
	cargo clean

build:
	cargo build --workspace --locked

release:
	cargo build --workspace --release --locked

setup:
	bash scripts/setup.sh

hooks:
	cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

bench:
	cargo bench

help:
	@echo "Available targets:"
	@echo "  all       - fmt + lint + test + check"
	@echo "  check     - cargo check --workspace --all-targets --locked"
	@echo "  test      - cargo test --workspace"
	@echo "  lint      - cargo clippy --workspace --all-targets -D warnings"
	@echo "  fmt       - cargo fmt --all -- --check"
	@echo "  clean     - cargo clean"
	@echo "  build     - cargo build --workspace"
	@echo "  release   - cargo build --workspace --release --locked"
	@echo "  setup     - run scripts/setup.sh"
	@echo "  hooks     - install git pre-commit hook"
	@echo "  bench     - cargo bench"
