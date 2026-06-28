---
related_code:
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints.rs
implementation_files:
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
plan_sources:
  - user: 2026-06-25 Optimize Zircon editor UI from primitive components upward before composing drawers and windows
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
tests:
  - cargo fmt -p zircon_editor --check
  - cargo test -p zircon_editor --lib compact_workbench_module_more_opens_overflow_menu_and_selects_hidden_module --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib workbench_toolbar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never
  - cargo test -p zircon_editor --lib capture_workbench_module_overflow_visual_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never -- --ignored --nocapture
  - docs/tests/editor/editor-window-m3-workbench-module-overflow-900x620.png
doc_type: module-detail
status: implemented-focused-passed
---

# Workbench Module Overflow Menu

## Purpose

`module_overflow_menu.rs` owns the compact-toolbar More Modules popup in the componentized Workbench template bridge. It turns the compact `WorkbenchModuleMore` affordance from a passive preview action into an interactive module selector for tabs that are hidden at 900px-style toolbar widths.

This module is intentionally small: it only maps hidden module menu rows to existing module tab controls. It does not introduce a second module-selection state model, a new module registry, or a toolbar-specific routing facade.

## Related Files

`workbench_window.zui` declares `WorkbenchModuleOverflowMenu` as a `WorkbenchPopupMenu` root child. The visible toolbar still owns the `WorkbenchModuleMore` trigger in `workbench_top_toolbar.zui`, while `toolbar_layout.rs` decides whether the trigger is visible at the current width.

`window_menu_state.rs` treats the overflow popup as a toolbar menu. Opening More closes the other toolbar menus, and resizing out of compact mode closes the overflow popup. `control.rs` handles popup item selection and merges the resulting hidden module-tab dispatch effects back into the retained host result.

## Behavior Model

The popup rows are the compact-hidden module tabs: Behavior, Render, Assets, VFX, and HUD. Before opening the menu, `refresh_workbench_module_overflow_menu_items(...)` rewrites the authored `menu_items` property so the currently selected hidden module receives a `checked` flag.

Selecting a row calls `dispatch_workbench_module_overflow_menu_item_state(...)`. The function verifies the control id is `WorkbenchModuleOverflowMenu`, maps the row action id to the corresponding hidden module tab control, and delegates to `dispatch_control_state(tab_control_id, Click)`. The visible and hidden tabs therefore share one selection path and one workspace visibility path.

Unknown menu action ids return `Ok(None)`. The generic popup selection still closes the popup, but no module selection is synthesized for actions outside the overflow map.

## Design And Rationale

The user request for this slice was to continue from primitive and composite controls before building more complex windows. The More Modules popup is a small composite-control step: a toolbar button, popup container, row list, checked state, and selection dispatch are wired together without broadening drawer/window behavior.

The design keeps ownership narrow. The mapping table lives in `module_overflow_menu.rs`; menu open/close exclusivity stays in `window_menu_state.rs`; compact visibility stays in `toolbar_layout.rs`; host-level item selection stays in `control.rs`. Root `mod.rs` only mounts the child module.

## Edge Cases And Constraints

- The overflow popup is closed when the toolbar is no longer compact, preventing stale off-screen or hidden-menu state after resize.
- The checked row is refreshed every time More opens, so selecting a hidden module and reopening the menu shows the active row.
- Hidden tabs are selected through their real control ids, so workspace visibility, checked state, and module feedback keep using existing module-navigation code.
- The screenshot test closes unrelated component-lab sample popups before opening More so the visual artifact isolates this control.

## Test Coverage

`compact_workbench_module_more_opens_overflow_menu_and_selects_hidden_module` verifies opening More, structured row contents, dispatch from the Behavior row to `WorkbenchModuleBehavior`, workspace visibility, popup close state, and checked-row refresh on reopen.

The `workbench_toolbar` Cargo filter verifies this test together with the compact-toolbar visibility regression and existing toolbar window-menu regressions. `capture_workbench_module_overflow_visual_artifact` writes `docs/tests/editor/editor-window-m3-workbench-module-overflow-900x620.png` and asserts non-background pixels inside the popup frame.

## Open Issues Or Follow-up

This closes the Workbench module toolbar More popup only. It does not implement the full main-page tab overflow menu from `15a`, tokenized popup anchor placement, or deeper 640/1260 responsive toolbar composition.
