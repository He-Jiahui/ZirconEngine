---
title: Runtime91 Shared Include Analysis
category: zircon_runtime
report_id: Runtime91-shared-include-analysis-2026-08-28
date: 2026-08-28
session_id: root-runtime91-shared-include-analysis-20260828
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime91 Shared Include Analysis

## Scope

`shader_sources_with_module_dependency_hashes_and_changed_paths` previously invoked the affected
source closure and topology hash consumers independently. Both consumers rebuilt the same strongly
connected components and condensed component graph, so every incremental shader-prewarm batch paid
twice for the O(V + E) include analysis before performing consumer-specific work.

The batch now builds one `IndexedIncludeAnalysis` and lends it to both consumers. The analysis owns
the SCC member lists, source-to-component projection, dependency edges, and reverse dependent edges.
Topology hashing and changed-source closure keep their existing ordering, cycle compression,
changed-path, and deterministic hash behavior while sharing the structural graph work.

## Performance Evidence

The isolated Rust model uses 24,000 source nodes with four include edges per source. It compares a
legacy batch that constructs the include analysis separately for topology hashing and changed-source
closure with the final batch that constructs it once and reuses it. Each variant uses 21 paired
samples after three warmups and was compiled with `rustc --edition 2021 -C opt-level=3` on Windows.

| Metric | Separate analyses | Shared analysis | Change |
|---|---:|---:|---:|
| Include-analysis probes | 768,000 | 384,000 | -50.000% |
| Include-analysis allocation events | 48,012 | 24,006 | -50.000% |
| P50 | 30,451,000 ns | 13,453,700 ns | -55.819% |
| P95 | 38,801,400 ns | 20,648,500 ns | -46.784% |

The baseline and optimized checksums both remained `288,012,001`. The acceptance gates were
include-analysis probes and allocation events at or below 50% of baseline, plus P50 and P95 at or
below 65% of baseline; all four passed. The first latency run contained a Windows scheduling outlier
in the optimized P95, so the unchanged binary was rerun rather than filtering samples; the table
reports that complete second run.

Model source:

- `.codex/state/session-coordinator/runtime91-shared-include-analysis-model.rs`

The model isolates shared graph-analysis work. It does not replace managed Cargo behavior tests,
product shader-manifest profiling, or allocator measurements of the full prewarm process.

## Contracts And Validation

- `tools/tests/test_runtime91_shared_include_analysis_performance_contract.py` locks one analysis
  construction per batch, the owned reusable projection, both borrowed consumer calls, and the
  absence of SCC/component-graph reconstruction inside either consumer.
- Initial TDD RED failed because both consumers still called `strongly_connected_components` and
  `component_graph`; the final implementation passes all four source-contract tests through the
  repository-standard `unittest` entry point.
- Scoped `rustfmt --edition 2021 --check`, Python contracts, and `git diff --check` are preflight
  gates for one three-task batch with provider deduplication and the material override index.
- Cargo behavior, the release model, terminal integration, record finalization, and automatic WeCom
  delivery remain pending in that managed asynchronous coordinator batch; no direct Cargo command
  is run by the owner session.

## Remaining Parent-Plan Work

Runtime91 still requires the broader material/shader module graph, permutation compiler,
reflection/layout, pipeline and PSO cache, prewarm, hot-reload, and product-integration gates from
the parent plan. This slice removes one repeated include-analysis pass from the existing prewarm
manifest path; it does not close the parent plan or claim product-scale shader pipeline parity.
