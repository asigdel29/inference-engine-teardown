# Run and compare experiments

## Verify the client

1. Run `make check` from the repository root.
2. Run `INFERENCE_TEARDOWN_BIN=$PWD/target/release/inference-engine-teardown python3 fixture_benchmark.py runs/transport-001`.
3. Inspect `runs/transport-001/summary.json`.
4. Confirm that all 60 fixture requests succeeded.

The fixture echoes prompts and returns synthetic usage counts. Use the fixture to diagnose client transport or packaging changes.

## Measure an inference engine

1. Start the engine and identify its streaming completions URL.
2. Copy `fixture-config.json` to a local configuration file.
3. Record the engine, model, tokenizer, hardware, and precision in that file.
4. Select a workload and record the engine launch command.
5. Run the command below once per engine.
6. Select a fresh output directory for every run.

```sh
inference-engine-teardown --engine vllm --model MODEL \
  --endpoint http://localhost:8000/v1/completions --config config.json \
  --workload inference_teardown/workload.jsonl --concurrency 4 \
  --repetitions 20 --out runs/vllm-001
```

Keep the trace hash, concurrency, repetitions, precision, and hardware fixed for a paired comparison. Record any warmup policy. Keep warmup runs in separate directories.

## Inspect a failed run

| Symptom | Action |
|---|---|
| Output directory exists | Select a new directory. Preserve the first run. |
| Trial contains partial text | Inspect its error and finish reason. Check the engine log for a disconnect. |
| Output-token throughput is null | Check the usage field in each successful response. |
| Summary is absent | Inspect the manifest and complete JSONL rows. Repeat the experiment in a fresh directory. |
| A request exceeds the timeout in total | Check the gaps between chunks. The timeout applies to each socket operation. |

The artifacts contain complete prompts, generated text, endpoint URLs, and the supplied configuration. Use publishable inputs for public benchmark runs.
