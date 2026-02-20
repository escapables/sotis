.PHONY: build release test clippy fmt-check validate check clean

build:
	cargo build --workspace

release:
	cargo build --release --workspace

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt-check:
	cargo fmt --all -- --check

validate:
	./bin/validate-docs

check: build test clippy fmt-check validate

clean:
	cargo clean
