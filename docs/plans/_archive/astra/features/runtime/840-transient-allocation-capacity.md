---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/214/2026-09-19-transient-allocation-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/render_graph/graph/transient_allocation.rs
  - zircon_runtime/src/render_graph/graph/transient_allocation/capacity_tests.rs
tests:
  - tools/tests/test_runtime_transient_allocation_capacity_performance_contract.py
---

# Runtime840 · transient allocation output capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime render-graph transient compiler | Reserve the exact filtered lifetime count before emitting compiled transient allocations, preserving interval sort, slot reuse, IDs, and empty behavior. | TDD source/model contract `2/2`; lower source/order regression and ignored `RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-lifetime model removes `11→0` growth events; focused batch `46/46` and broad non-tooling loader `3907/3907` across `933` modules pass in `62.574s` with zero failures/errors/load errors/skips. Managed Cargo/Release and render-graph product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the temporary output vector capacity in the
compiler-local transient lifetime allocator. It does not change bucket keys,
aliasing or interval validation, physical allocation identity, backend
materialization, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/render_graph/graph/transient_allocation.rs` | `D4BDBF1BA9E9B43382C90FB1144739CBB20FF61B0D7AEE3337C92947499C6015` |
| `zircon_runtime/src/render_graph/graph/transient_allocation/capacity_tests.rs` | `12C81B13BC8FA1BCC8C31FC2F52605481D43F7A4006E704B228BC0D54DCEB79F` |
| `tools/tests/test_runtime_transient_allocation_capacity_performance_contract.py` | `7311AEF4259E0AA8A56C421B656038F6FFEF9A2254ACAE74E2BAE8B3D1CB59DA` |

## Managed gate

No Cargo process is started locally and the coordinator is not polled. Keep
this entry `implemented_pending_validation` until the owner-attributed batched
Windows Release lane proves current-source compilation, allocation behavior,
and Runtime render-graph product p50/p95/p99 evidence.
