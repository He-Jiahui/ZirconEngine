---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-viewport-toolbar-pointer-surface-reuse.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-editor-pointer-surface-delta-receipts.md
  - docs/plans/optimize/zircon_editor/01/2026-09-18-viewport-toolbar-control-projection-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
tests:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
  - tools/tests/test_editor_retained_viewport_toolbar_pointer_generation_performance_contract.py
---

# Editor799 · viewport-toolbar control projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 viewport-toolbar pointer projection | Reserve the prior retained-control count for the mandatory controls projection; retain a lazy sparse geometry-change vector. | TDD RED→GREEN source contract `6/6`; deterministic 64-control zero-capacity doubling model `7→0` growth events for the mandatory vector; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs` | `01F23ABF184CEDCB69EE8D23AD95ED8E800CD91316463B649AA73F33F8296C60` |
| `tools/tests/test_editor_retained_viewport_toolbar_pointer_generation_performance_contract.py` | `B0BE17FE18515E6BD2EBFB5FC832F8F071B12C365774F676C6B1273CD6E086DB` |

## 性能与受管验证边界

Only the every-frame control projection receives the exact retained bound.
The geometry-change vector stays lazy because its cardinality is the changed
subset and is often zero. Existing hit-grid identity, action-key topology, and
geometry authority are unchanged. The combined non-tooling contract batch passes
`2205/2205` across 598 modules in `6.538s`; Rustfmt, Python AST, scoped diff,
whitespace, Wiki validation, and the listed source fingerprints also pass.
The deterministic model is not a product performance claim. Keep this record
`implemented_pending_validation` until a single owner-attributed Windows batch
supplies Cargo, Release allocation, and viewport-toolbar p50/p95/p99 receipts.
Tooling production is unchanged and this session does not poll the coordinator.
