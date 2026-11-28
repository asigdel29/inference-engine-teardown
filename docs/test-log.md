# Test log

Record the revision, setup, command, and observed result for each measurement. The loopback fixture exercises transport and result handling. It echoes the prompt and supplies a synthetic token count.

| Date (UTC) | Revision and setup | Command | Observed result |
| --- | --- | --- | --- |
| 2026-11-27 | `caf763b`, macOS arm64, native release build | `make check` | Formatting and Clippy passed. One Rust test and two Python tests passed, including complete and interrupted streams. |
| 2026-11-27 | Same revision, local fixture, concurrency 4, 20 repetitions of three prompts | Fixture command in [the runbook](runbook.md) | 60 of 60 requests succeeded. Wall time was 7.823 ms. Client-observed total latency was 0.378 ms at p50 and 1.033 ms at p95. |

These timings describe one local run. Hardware throughput comparisons for vLLM, SGLang, and TensorRT-LLM require equivalent model-server runs.
