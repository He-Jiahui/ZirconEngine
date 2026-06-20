---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/native_panes/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/panes.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-debug-reflector-overlay color/draw/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Paint Debug Reflector Overlay

`paint_debug_reflector_overlay.rs` is the retained-host boundary for drawing runtime debug-reflector primitives over native pane content. It stays as a structural module entry and re-exports the single draw entry used by the Workbench native-pane diagnostics painter.

`paint_debug_reflector_overlay/colors.rs` owns the overlay palette and alpha adjustment rules. It maps `UiDebugOverlayPrimitiveKind` to the selected frame, clip frame, wireframe, hit, rejected-bounds, overdraw, material batch, text debug, resource atlas, and damage-region colors. It also owns the label text color and the solid-border alpha boost used for filled debug cells.

`paint_debug_reflector_overlay/draw.rs` owns the draw sequencing. It rejects empty or invisible overlay batches, translates each primitive from pane-local space into the active origin, intersects it with the active clip, draws either an outline-only or filled debug primitive, and emits a small label marker when the primitive carries non-empty text.

`paint_debug_reflector_overlay_tests.rs` owns the module-local regression that proves a damage-region primitive inside the clip mutates the output RGBA frame. The test remains module-local because it verifies retained-host software painting behavior rather than a public runtime UI contract.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
