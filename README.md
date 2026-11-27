# Inference engine teardown

Replay one prompt trace against a streaming completions endpoint. Use Python 3.10 or later. The runner uses the standard library.

## Quickstart

Run `make check`. Run `make demo` to exercise three requests against a loopback fixture. The test also verifies a connection failure during each stream.

## Run an engine

Start a compatible engine with streaming completions and usage reporting enabled. Copy `fixture-config.json` and replace all five fixture values with the actual run configuration.

```sh
python3 -m inference_teardown.bench --engine vllm --model MODEL --config config.json \
  --endpoint http://localhost:8000/v1/completions --out runs/vllm-001
```

Run the same command with a fresh output directory for each engine. Use the same workload, model revision, tokenizer, hardware, and precision. Set `--concurrency` to control simultaneous requests. The default is one.

## Data flow

Workload JSONL → HTTP client pool → engine SSE endpoint → manifest, trial records, and summary.

Each trial records the prompt, output, first visible text latency, total latency, and server-reported token count. A missing usage field produces a null count. One stream event can contain multiple tokens. The receiver measures time to first visible text. The timeout applies to socket operations.

A failed trial retains partial output and returns a failure status for the run. Select a new directory for each run to preserve earlier results.

## Measurement status

This release provides the transport harness and synthetic fixtures. Hardware measurements for vLLM, SGLang, and TensorRT-LLM require separate runs. The included workload is a smoke test.

Original code uses Apache-2.0. The author dedicates the original workload prompts to the public domain under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).

## Continuous integration and benchmarks

CI tests Python 3.10, 3.12, and 3.14. Each job builds a wheel and checks its installed entry point and default workload.

Run `python3 fixture_benchmark.py runs/transport-001` to measure loopback transport overhead. CI uploads its manifest, trial records, and summary. The fixture sends 60 requests with four concurrent clients and synthetic usage counts.

Use `--repetitions 20` for repeated engine trials. The summary reports nearest-rank p50, p95, and p99 latencies for successful requests. Throughput includes elapsed time for failures. Output-token throughput requires usage counts for every successful request.

## Design and operation

- [System design and UML diagrams](docs/system-design.md)
- [Experiment and troubleshooting guide](docs/operations.md)
- [Repository instructions](AGENTS.md)

## Graphify source map

Read the [source map report](graphify-out/GRAPH_REPORT.md) or query the [graph data](graphify-out/graph.json). Open the [interactive map](graphify-out/graph.html) locally in a browser.

Install Graphify 0.9.76 with `uv tool install graphifyy==0.9.76`. Run these commands from the repository root.

```sh
make graph
graphify query "run_trial"
```

The build reads tracked Python files from the working tree, including tests. Stage new Python files before you refresh the map. Graphify uses local syntax analysis with model backends disabled. The system design diagrams describe runtime behavior.
