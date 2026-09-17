install-rust-tooling:
	rustup component add rustfmt
	rustup component add clippy

build:
	cargo build --locked --release

build-dev:
	cargo build --locked

run-dev: build-dev
	valkey-server --save "" --loadmodule target/debug/libvalkey_array.so

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- -D clippy::all -D warnings

fix:
	cargo fmt --all
	cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged -- -D clippy::all -D warnings

docs:
	cargo doc --no-deps

# `test-prep` prepares for running tests
test-prep: build-dev  # Running `build-dev` before ensures the integration tests can load the newest module

test: test-prep
	cargo test

test-unit: test-prep
	cargo test --lib

test-int: test-prep
	cargo test --test '*'

full-monty: fix test docs
