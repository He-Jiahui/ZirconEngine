---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/07/2026-08-26-play-output-streaming-decode.md
  - docs/plans/optimize/zircon_editor/07/2026-09-15-drain-all-capacity.md
  - docs/plans/optimize/zircon_editor/257-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-working-tree-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/play/process_backend/output.rs
tests:
  - zircon_editor/src/core/play/process_backend/output/drain_all_capacity_tests.rs
  - tools/tests/test_editor_play_output_drain_capacity_performance_contract.py
---

# Editor764 · Play output drain-all capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor07 / Play output pump | Reserve the joined receiver length, optional deferred line, and five bounded budget-diagnostic slots before `drain_all` appends output; use the same diagnostic bound for limited drains. | TDD RED/GREEN source contract `3/3`; merged recent Runtime/Editor hot-path batch `15/15`; lower capacity/order regression; ignored `EDITOR07_PLAY_OUTPUT_DRAIN_ALL_CAPACITY_BENCH_V1` marker; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

Local source fingerprints: `output.rs`
`ED745C84BD89F86E0855D2C95061F7921A416CF0E83DBAD6843FD82A3CE763AD`, lower
regression `72BA2870CBAEB06CD1B715125A55068A8C1DB3360E90D0EA803431FC566D8635`,
and Python contract `2CE6864B841DD1579471236352FEB9027A32729983748DD39A0B251C0F5DFB8F`.

## 性能边界

The reservation is derived from the already bounded queue (`1024` lines), one
deferred slot, and the fixed counter/age diagnostic maximum; an entirely empty
finish keeps zero capacity. It removes result vector growth on a full finish
drain while preserving output ordering and counter semantics. The deterministic
model is not a CPU, allocator, RSS, or product p50/p95/p99 measurement.

## 受管验证

This feature joins the existing batched Runtime/Editor Windows Release lane.
No per-task Cargo run or coordinator status query was made. Keep
`implemented_pending_validation` until current-source compilation, lower
behavior tests, allocation evidence, and Play-output percentile gates arrive.
Tooling production work remains deferred.
