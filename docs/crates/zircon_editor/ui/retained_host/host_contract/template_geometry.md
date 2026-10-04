---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-geometry frame/bounds/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Template Geometry

`template_geometry.rs` is the retained-host boundary for Workbench template node geometry helpers. It now stays as a structural re-export entry and test hook.

`template_geometry/frame.rs` owns conversion from a retained `TemplatePaneNodeData` frame payload into a `FrameRect`. This keeps node DTO shape knowledge isolated from popup and pointer routing modules.

`template_geometry/bounds.rs` owns popup containment bounds. It prefers visible native window bounds, then falls back to the union of visible Workbench template nodes, and finally returns a 1x1 safe frame when no usable geometry exists. That fallback keeps popup dismiss and popup layout code from needing ad hoc empty-window behavior.

`template_geometry_tests.rs` owns the module-local regressions for native-window preference and template-node union fallback.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
