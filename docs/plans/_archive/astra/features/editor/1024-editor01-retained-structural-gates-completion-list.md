---
related_code:
  - zircon_editor/src/tests/host/retained_window/native_mode.rs
  - zircon_editor/src/tests/host/retained_window/activity_rail_template_boundary.rs
  - zircon_editor/src/tests/host/retained_tab_drag/surface_contract.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/runtime_fixtures.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/retained_projection.rs
  - zircon_editor/src/tests/ui/boundary/template_assets/host_shells.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-09-27-retained-window-menu-template-structural-gates.md
status: implemented_pending_validation
---

# Editor1024 / Editor01 retained structural gates completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| Native window, side rail, and tab drag owners | Assertions now read presentation, store, window handle/snapshot, DTO, dock patch and committed drag-route children. Root-shell fallback exclusions remain. | Pre-repair source replay identified obsolete parent markers. Managed Editor lib behavior execution is pending. | implemented_pending_validation |
| Menu chrome and popup route | Slot frames, disabled labels, one popup node per open layer, geometry item hit, and published route index are checked at their current owners. Existing 10,000-item and nested-popup behavior regressions stay wired. | Static source checks passed; actual pointer behavior and allocation/latency remain pending in managed validation. | implemented_pending_validation |
| `.zui` fixtures and shell projection | Folder-backed fixture and DTO owners, cached builtin registration, `UiSurface` node projection, and welcome action are checked. Legacy schema negatives remain. | Seven test owners are repaired without production edits. Managed Editor lib execution is pending. | implemented_pending_validation |
| Editor01 product performance | This slice repairs structural test ownership. | CPU, allocator, RSS, and latency gates remain open; no dynamic pass or speedup is claimed. | product_gate_pending |

Batch T preimages, source replay, inverse diff and source manifest are stored
under `.codex/state/session-coordinator/async-validation-batches/`.
