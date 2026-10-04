---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-authored-geometry-delta-publication.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-19-surface-frame-patch-range-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/surface/frame_publication.rs
tests:
  - tools/tests/test_runtime_surface_frame_patch_range_capacity_performance_contract.py
---

# Runtime833 · surface frame patch-range capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime UI surface-frame partial render publication | The exact changed-node set now supplies a single `node_ids.len()` upper bound for the temporary render patch-range vector; `filter_map` extends directly into the reserved buffer. `None` full-snapshot fallback, empty-set zero capacity, BTree order, command filtering, range merging, and frame authority are unchanged. | TDD source/model contract `2/2`; lower source regression and ignored `RUNTIME833_SURFACE_FRAME_PATCH_RANGE_CAPACITY_BENCH_V1` marker wired; deterministic 4,096-node model `12→0` geometric growth events; the current one-process smoke batch covers `169` files and passes `626/626` tests in `5.379s`; scoped rustfmt passes. | implemented_pending_validation |

## 性能与受管验证边界

The `12→0` result is an allocation-shape model, not a product timing claim.
Managed Windows Cargo/Release compilation, lower Rust execution, allocator
receipts, and surface-frame/input product p50/p95/p99 evidence remain pending
in the asynchronous owner-attributed batch. Tooling production is unchanged
and remains deferred for the later Rust migration.
