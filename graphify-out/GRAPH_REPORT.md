# Graph Report - inference-engine-teardown  (2026-11-26)

## Corpus Check
- 4 files · ~860 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 47 nodes · 71 edges · 8 communities (0 shown, 8 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 1 edges (avg confidence: 0.95)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- events
- Fixture
- main
- json
- run_trial

## God Nodes (most connected - your core abstractions)
1. `main()` - 10 edges
2. `Fixture` - 7 edges
3. `run_trial()` - 6 edges
4. `events()` - 5 edges
5. `RunnerTests` - 5 edges
6. `load_workload()` - 4 edges
7. `percentiles()` - 4 edges
8. `Measure client transport overhead with a deterministic local HTTP server.` - 1 edges
9. `Serve deterministic completion streams for transport tests.` - 1 edges
10. `Keep fixture requests out of the test output.` - 1 edges

## Surprising Connections (you probably didn't know these)
- `RunnerTests` --uses--> `Fixture`  [INFERRED]
  test_bench.py → fixture_benchmark.py

## Import Cycles
- None detected.

## Communities (8 total, 8 thin omitted)

## Knowledge Gaps
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Fixture` connect `Fixture` to `test_bench.py`, `events`?**
  _High betweenness centrality (0.238) - this node is a cross-community bridge._
- **Why does `RunnerTests` connect `events` to `test_bench.py`, `Fixture`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._
- **Why does `events()` connect `events` to `test_bench.py`, `bench.py`, `run_trial`?**
  _High betweenness centrality (0.100) - this node is a cross-community bridge._