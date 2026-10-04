---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-30-editor-pointer-surface-delta-receipts.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-viewport-toolbar-pointer-surface-reuse.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
related_code:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs
tests:
  - tools/performance/editor/editor_viewport_toolbar_pointer_surface_reuse_pressure.py
  - tools/tests/test_editor_viewport_toolbar_pointer_surface_reuse.py
  - tools/performance/editor/editor_viewport_overlay_pointer_surface_reuse_pressure.py
  - tools/tests/test_editor_viewport_overlay_pointer_surface_reuse.py
---

# Viewport Pointer Delta Contract Repair

The Toolbar and Overlay pointer performance models previously asserted an
obsolete single-function retained-patch topology. Current source uses explicit
`NoChange`, `Geometry`, and `Topology` delta classification: stable geometry
publishes local authored-frame changes, while topology or ordering changes take
the typed full-rebuild path.

The repaired contracts validate retained identity, route-change capture release,
exact geometry publication, and the absence of allocation-bearing route-map
work in the stable classification branch. They retain the existing deterministic
operation/allocation model and do not claim product timing.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| E34 | Rebind Toolbar and Overlay pointer-surface pressure contracts to the delta classifier | implemented_pending_validation | The focused four-file test batch passed 12/12; Python compilation and scoped diff checks passed. Managed Rust compilation is still blocked at immutable admission by the dirty external `E:\\Git\\zr_vm` worktree. Windows Release allocation/time p50/p95/p99 evidence remains pending. |
