---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/marker.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/top_bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/union.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/root_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_impl/skeleton.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/marker.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/top_bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/union.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract paint-diagnostics marker/top-bar/union/visibility/test ownership scan
  - scoped trailing whitespace scan
  - scoped git diff --check
  - milestone testing stage: cargo check/test deferred until runtime render-history errors are resolved
doc_type: module-detail
---

# Paint Diagnostics

`paint_diagnostics.rs` owns the retained-host software-paint diagnostics geometry boundary. It is now a structural module entry that preserves the existing root imports for workbench root frames and diagnostic skeleton painting.

`paint_diagnostics/marker.rs` owns the debug refresh overlay marker rectangle. It turns overlay text into a bounded top-right marker frame and rejects empty labels or invisible top-bar frames before geometry is emitted.

`paint_diagnostics/top_bar.rs` owns top-bar frame selection. It prefers the current componentized scene layout when visible, falls back to the host layout when the scene layout is empty, and finally computes a conservative height from the window size when no layout frame is usable.

`paint_diagnostics/union.rs` owns diagnostic frame union math for callers that need to enlarge damage regions around two diagnostic rectangles. `paint_diagnostics/visibility.rs` owns the shared finite-size visibility predicate used by marker and top-bar selection.

`paint_diagnostics_tests.rs` owns the module-local regressions for top-right marker geometry, scene-layout top-bar preference, and empty-layout fallback sizing.

## Validation Notes

This slice is implementation-first. Formatting, root ownership scans, scoped whitespace scans, and scoped diff checks cover the handoff. Full Cargo check/test validation remains deferred to the milestone testing stage because current package checks fail in unrelated `zircon_runtime` render-history code before editor diagnostics are reached.
