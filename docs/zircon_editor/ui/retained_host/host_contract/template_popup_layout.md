---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout_tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-popup-layout dropdown/template/row/metric/bounds/test ownership scan
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Template Popup Layout

`template_popup_layout.rs` is the retained-host popup geometry entry for projected template dropdowns and popup menus. It keeps the stable geometry helpers used by popup dismiss and hit testing while splitting dropdown placement, projected popup behavior, menu row geometry, shared metrics, bounds validation, and regressions into child modules.

## Dropdown Ownership

`template_popup_layout/dropdown.rs` owns dropdown option popup placement and row frame construction. It computes below-control placement, opens above when bounded vertical space requires it, clamps the right edge inside valid bounds, and exposes row frames for dropdown-style popups.

## Template Popup Ownership

`template_popup_layout/template.rs` owns projected popup behavior. Nodes declared as `DropdownPopup` or `dropdown-popup` use their projected control frame as the popup frame; other option lists use dropdown popup geometry. It also maps rows inside the projected popup frame.

## Shared Geometry Ownership

`template_popup_layout/metrics.rs` owns row-gap and minimum-row-height metrics plus dropdown/menu row-height helpers.

`template_popup_layout/bounds.rs` owns finite positive bounds validation used before popup clamping.

`template_popup_layout/rows.rs` owns menu item row frame construction for menu-style popup bodies.

## Root Boundary

The root `template_popup_layout.rs` only declares child modules, re-exports existing geometry helpers, and attaches the external test module. It should not regain popup placement math, bounds checks, row-height helpers, projected popup checks, menu row geometry, or inline tests.

## Test Ownership

`template_popup_layout_tests.rs` owns local regressions for vertical overflow placement, default placement fallback, right-edge clamping, and projected `DropdownPopup` frame/row geometry.

## Validation Notes

This slice used `cargo fmt -p zircon_editor --check`, a root ownership scan confirming `template_popup_layout.rs` no longer owns dropdown, template, row, metric, bounds, or inline test bodies, a scoped trailing-whitespace scan, and scoped `git diff --check`. Full Cargo check/test validation remains deferred because current package checks are blocked before editor diagnostics by unrelated `zircon_runtime` render-history errors, and the active instruction is to implement functionality first.
