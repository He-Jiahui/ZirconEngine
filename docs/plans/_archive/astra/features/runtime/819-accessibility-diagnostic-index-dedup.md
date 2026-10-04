---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-08-26-accessibility-indexed-focus.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-19-accessibility-diagnostic-index-dedup.md
related_records:
  - docs/plans/astra/features/runtime/03-accessibility-indexed-focus.md
implementation_files:
  - zircon_runtime/src/ui/accessibility/diagnostics.rs
  - zircon_runtime/src/ui/accessibility/diagnostics/index_dedup_tests.rs
tests:
  - zircon_runtime/src/ui/accessibility/diagnostics/index_dedup_tests.rs
  - tools/tests/test_runtime_accessibility_diagnostic_index_capacity_performance_contract.py
---

# Runtime819 · accessibility diagnostic node-index deduplication

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime03 / snapshot diagnostics | Replace the parallel duplicate-ID `BTreeSet` and lookup `BTreeMap` with one entry-based node index while preserving the first index and duplicate callback semantics. | TDD source/model contract `3/3`; lower duplicate/order regression and ignored `RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1` marker are wired; deterministic 4,096-node model changes auxiliary index allocations `2→1`. | implemented_pending_validation |

## 性能边界

This is an ordered-index allocation and lookup-shape optimization for
accessibility snapshot validation. It does not claim allocator bytes, CPU,
RSS, or product accessibility latency until the managed Release lane supplies
those measurements.

## 受管验证

Keep this record at `implemented_pending_validation` until the owner-attributed
Windows Release batch compiles the Runtime accessibility diagnostics module,
executes the lower regression and ignored marker, and supplies allocation plus
accessibility p50/p95/p99 evidence. Tooling production work remains deferred.
