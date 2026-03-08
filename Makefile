.PHONY: build release run test clean cross-arm64 static-arm64 docker docker-run lint fmt

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --check

# Cross-compile for Linux ARM64 (RPi5)
# Requires: cargo install cross --git https://github.com/cross-rs/cross
cross-arm64:
	cross build --release --target aarch64-unknown-linux-gnu

# Native Linux ARM64 (if running on ARM64 host with musl)
static-arm64:
	cargo build --release --target aarch64-unknown-linux-musl

# Docker
docker:
	docker build -t bch-api-rust .

docker-run:
	docker compose up
