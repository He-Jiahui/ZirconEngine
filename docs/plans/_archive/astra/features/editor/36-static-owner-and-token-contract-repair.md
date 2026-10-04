---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-editor-pointer-surface-delta-receipts.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
related_code:
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/paint_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/metrics.rs
tests:
  - tools/tests/test_editor_menu_interaction_projection_pressure.py
  - tools/tests/test_editor_runtime_render_command_transient_extraction_contract.py
  - tools/tests/test_editor_ui_device_pixel_aa_contract.py
---

# Static Owner And Token Contract Repair

The retained menu source now rebuilds authored frames directly, and Runtime UI
rendering uses the shared in-place transient paint projection helper. The
source contracts follow those owners rather than requiring retired direct calls.
Radius checks now derive their baseline values from the canonical Editor token
file, while continuing to verify scale propagation and all consuming surfaces.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E36 | Rebind menu/render transient and radius contracts to current source and token authorities | implemented_pending_validation | The focused three-module static batch passed 52/52; Python compilation and scoped diff checks passed. Managed Rust compilation remains blocked at immutable admission by dirty external worktree `E:\\Git\\zr_vm`; no Release p50/p95/p99 claim is made. |
