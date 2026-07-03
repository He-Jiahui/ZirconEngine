---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout_tests.rs
  - zircon_editor/src/ui/retained_host/popup_anchor_metrics.rs
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
  - zircon_editor/src/ui/retained_host/popup_anchor_metrics.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-20 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor --check
  - host_contract template-popup-layout dropdown/template/row/metric/bounds/test ownership scan
  - cargo test -p zircon_editor --lib template_popup --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib clamp_popup_x_to_bounds_preserves_shared_edge_margin_when_space_allows --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib workbench_toolbar_window_menus_anchor_to_toolbar_controls_across_widths --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib capture_workbench_component_slate_atlas_visual_artifact --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never -- --ignored --nocapture --test-threads=1
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-text-tabs-0701 --message-format short --color never
  - scoped whitespace scan
  - scoped git diff --check
doc_type: module-detail
---

# Template Popup Layout

`template_popup_layout.rs` is the retained-host popup geometry entry for projected template dropdowns and popup menus. It keeps the stable geometry helpers used by popup dismiss and hit testing while splitting dropdown placement, projected popup behavior, menu row geometry, shared metrics, bounds validation, and regressions into child modules.

## Dropdown Ownership

`template_popup_layout/dropdown.rs` owns dropdown option popup placement and row frame construction. It computes below-control placement, opens above when bounded vertical space requires it, uses the shared popup anchor gap and shared edge-margin x clamp from `popup_anchor_metrics.rs`, and exposes row frames for dropdown-style popups.

## Template Popup Ownership

`template_popup_layout/template.rs` owns projected popup behavior. Nodes declared as `DropdownPopup` or `dropdown-popup` use their projected control frame as the popup frame; other option lists use dropdown popup geometry. It also maps rows inside the projected popup frame.

## Shared Geometry Ownership

`template_popup_layout/metrics.rs` owns the template popup anchor gap, delegated to shared retained-host popup anchor metrics, and minimum-row-height metrics plus dropdown/menu row-height helpers.

`popup_anchor_metrics.rs` owns cross-popup anchor tokens shared by template dropdown popups and toolbar/page/menu popups: edge margin, anchor gap, toolbar render gap, and the bounded x clamp helper. This keeps popup spacing adjustments out of individual dropdown/menu paint owners.

`template_popup_layout/bounds.rs` owns finite positive bounds validation used before popup clamping.

`template_popup_layout/rows.rs` owns menu item row frame construction for menu-style popup bodies.

## Root Boundary

The root `template_popup_layout.rs` only declares child modules, re-exports existing geometry helpers, and attaches the external test module. It should not regain popup placement math, bounds checks, row-height helpers, projected popup checks, menu row geometry, or inline tests.

## Test Ownership

`template_popup_layout_tests.rs` owns local regressions for vertical overflow placement, default placement fallback, right-edge clamping, shared anchor margin tokens, and projected `DropdownPopup` frame/row geometry. `popup_anchor_metrics.rs` owns the focused shared x-clamp edge-margin regression.

## Validation Notes

The 2026-07-01 popup-anchor tokenization slice passed `cargo fmt -p zircon_editor --check`, focused `template_popup` regressions 13/13, the shared clamp regression 1/1, the toolbar popup anchor regression 1/1, component atlas screenshot generation 1/1, M3 screenshot generation 1/1, and the editor-host build in external target directory `D:\cargo-targets\zircon-editor-text-tabs-0701`. Screenshot artifacts were refreshed under `docs/tests/editor`; scans of repo `target`, `zircon_editor/target`, and the external Cargo target found no matching editor screenshots.
