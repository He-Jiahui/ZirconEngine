---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/180/2026-09-15-runtime-pointer-hit-capacity.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
tests:
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter/hit_capacity_tests.rs
  - tools/tests/test_editor_runtime_picking_hit_capacity_performance_contract.py
---

# Editor765 · Runtime pointer hit capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor180 / Runtime47 viewport picking | Reserve the saturating stacked-plus-renderer candidate bound before appending runtime hit records, while retaining source order and empty-hit behavior. | TDD RED/GREEN source contract `3/3`; lower empty/combined/overflow-safe capacity regression; ignored `EDITOR765_RUNTIME_POINTER_HIT_CAPACITY_BENCH_V1` marker; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

Local source fingerprints are recorded after the final source-contract batch. The implementation is
intentionally limited to the adapter's result-vector allocation; ordered-hit authority, visibility
qualification, and multi-pointer policy remain owned by the parent Editor180/Runtime47 work.

Current source fingerprints: `runtime_picking_adapter.rs`
`9C80BBD04C83C7A13A1D7C78361C17259DBB258B4802D82C884C3FD746227715`, lower regression
`DCD5AFDF5CC14CF8ADAD4AEE6FB7442C88C1BAE3A78743D7A28861AAA73C02EA`, and Python contract
`0D367B34F196A0352E859034A6D9922DC7C8C16C4115BF825A97E16F90EA66A2`.

The current single-process Runtime+Editor performance-contract discovery
loaded `548` modules and passed `1963/1963` tests in `10.100s`, with zero
failures, errors, or skips; the focused pointer/navigation/style/capacity
probe passed `99/99`.

## 性能边界

The reservation is an upper bound already present in the two input slices and uses saturating
addition. It removes geometric growth when many stacked and renderer candidates score as hits;
filtered misses still return the existing empty `Vec`/no-wrapper result. The ignored benchmark
reports capacity and paired p95 samples. In the deterministic 32,768 stacked + 4,096 renderer
model, the legacy vector grows 17 times to capacity 65,536 while the candidate reserves exact
capacity 36,864; this is a model, not a product CPU, allocator, RSS, or pointer p50/p95/p99
measurement.

## 受管验证

This slice joins the existing batched Runtime/Editor Windows Release lane. No per-task Cargo run,
coordinator retry, or status query was made. Keep `implemented_pending_validation` until the
current-source compile/test, allocation, and pointer percentile gates arrive. Tooling production
work remains deferred.
