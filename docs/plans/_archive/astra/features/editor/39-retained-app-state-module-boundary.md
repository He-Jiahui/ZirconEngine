---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-ui-hotspot-ownership-review.md
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
tests:
  - tools/tests/test_editor17_settings_value_batch_contract.py
---

# Retained App State Module Boundary

The retained host composition root no longer owns the declarations for asset pointer state or
drawer-resize state. Asset pointer state now lives beside its constructors and lookup methods in
`app/asset_surface_pointer_state.rs`; workspace docking owns `ActiveDrawerResize`, with the app
module exposing only constrained re-exports for its descendants. Existing pointer and resize
callers keep the same fields and behavior, while the root file remains an orchestration boundary.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E39 | Move retained-host state declarations to their owning folder modules and keep app wiring/re-exports narrow | implemented_pending_validation | Editor17 settings/value batch `7/7` and merged Runtime/Editor source batch `125/125` passed; `app.rs` is 794 lines; focused new modules pass rustfmt and scoped diff checks. |

## Validation boundary

This is a structural and source-contract repair. No managed Cargo result, runtime/editor product
startup result, allocator sample, or p50/p95/p99 claim is made here. The next batched managed
validation must include the current app module tree after the shared worktree admission blocker is
cleared.

The five concurrent r3 submissions recorded in
`.codex/tmp/astra-runtime-editor-batch-20260911-r3.json` were rejected before
copy or Cargo with `validation_copy_overlay_not_owned`; the paths require
current Session attribution. No ticket was created, and this record does not
poll or retry that admission result.
