# Build and exercise the native streaming client.
.PHONY: check demo bench graph
BIN := $(CURDIR)/target/release/inference-engine-teardown
check:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings
	cargo test --locked
	cargo build --release --locked
	INFERENCE_TEARDOWN_BIN=$(BIN) python3 -m unittest -v
demo:
	cargo build --release --locked
	INFERENCE_TEARDOWN_BIN=$(BIN) python3 -m unittest -v test_bench.RunnerTests.test_cli
bench:
	cargo run --release --locked -- $(ARGS)
graph:
	bash scripts/graphify.sh
