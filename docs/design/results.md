# Result files and measurements

Each run owns a new directory. The writer flushes trial records in workload order, even when later requests finish first. The source of the calculations is [the native runner](../../src/main.rs).

| File | Contents | Use |
| --- | --- | --- |
| `manifest.json` | Schema version, endpoint, model, engine, concurrency, repetitions, timeout, workload SHA-256, and supplied configuration | Check that two runs used comparable inputs. |
| `trials.jsonl` | Input fields, repetition, response, duration, first-text delay, finish reason, reported tokens, and success or error | Inspect individual failures or recompute distributions. |
| `summary.json` | Counts, wall time, throughput, and latency percentiles | Compare runs after checking success counts. |

## Measurement population

- Request throughput divides successful requests by the wall time for the entire run.
- Output throughput divides reported completion tokens from successful requests by that same duration.
- Missing usage on any successful trial leaves output throughput null.
- Total latency percentiles include successful trials. First-text percentiles include successful trials that produced visible text.
- For percentile P, sort the observations and select rank `ceil(P * count / 100)`, starting at rank one. An empty population produces null.

A short failed request still occupies a concurrency slot and contributes to wall time. Compare success rates before comparing throughput. A stream event can contain several tokens, so first-text latency measures a client-visible chunk boundary.
