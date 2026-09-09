install-rust-tooling:
	rustup component add rustfmt
	rustup component add clippy

build:
	cargo build --locked --release

build-dev:
	cargo build --locked

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- -D clippy::all -D warnings

fix:
	cargo fmt --all
	cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged -- -D clippy::all -D warnings
