# Graph Report - inference-engine-teardown  (2026-11-27)

## Corpus Check
- 8 files · ~1,834 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 89 nodes · 129 edges · 9 communities (3 shown, 6 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.95)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- reference_runner.py
- test_bench.py
- main.rs
- stream.rs
- bench.py
- RunnerTests
- .parse

## God Nodes (most connected - your core abstractions)
1. `main()` - 10 edges
2. `Fixture` - 7 edges
3. `main()` - 6 edges
4. `run()` - 6 edges
5. `run_trial()` - 6 edges
6. `run_trial()` - 5 edges
7. `RunnerTests` - 5 edges
8. `events()` - 5 edges
9. `Event` - 4 edges
10. `load_workload()` - 4 edges

## Surprising Connections (you probably didn't know these)
- `RunnerTests` --uses--> `Fixture`  [INFERRED]
  test_bench.py → fixture_benchmark.py

## Import Cycles
- None detected.

## Communities (9 total, 6 thin omitted)

### Community 0 - "reference_runner.py"
Cohesion: 0.17
Nodes (4): load_workload(), main(), percentiles(), run_trial()

### Community 2 - "main.rs"
Cohesion: 0.18
Nodes (7): load_workload(), main(), Options, percentiles(), run(), run_trial(), write_json()

### Community 3 - "stream.rs"
Cohesion: 0.22
Nodes (3): Choice, Event, Usage

## Knowledge Gaps
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Fixture` connect `test_bench.py` to `RunnerTests`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **Why does `RunnerTests` connect `RunnerTests` to `test_bench.py`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **Why does `Observation` connect `.parse` to `stream.rs`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._