---
related_code:
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/assets/icons/zircon_editor_shell/toolbar
  - zircon_editor/assets/icons/zircon_editor_shell/controls/check.svg
  - zircon_editor/assets/icons/zircon_editor_shell/status/disabled.svg
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/glyph.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/aliases.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/toolbar_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_overflow_menu.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/glyph.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates/aliases.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - cargo test -p zircon_editor --lib compact_workbench_file_and_module_commands_use_toolbar_icon_family --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib toolbar_shell_svg_icons_load_as_real_pixels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib semantic_shell_icon_aliases_load_as_real_pixels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib full_workbench_run_mode_uses_toolbar_dropdown_icon --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor capture_full_workbench_run_mode_visual_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib compact_workbench_module_more_uses_toolbar_overflow_icon --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib compact_workbench_toolbar_separates_command_and_module_tab_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib workbench_toolbar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib production_zui_axis_layout_stretch_semantics_are_unambiguous --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib l4_surfaces_keep_runtime_region_topology_snapshot --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib workbench_shell_surface_component_assets_keep_bottom_up_composition_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo fmt -p zircon_editor --check
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture
doc_type: module-detail
status: implemented-focused-passed-build-screenshot-passed
---

# Workbench Toolbar Layout

`workbench_top_toolbar.zui` owns the authored two-row shell for the componentized Workbench top toolbar. The toolbar is a 72 px `VerticalGroup`: a 36 px command row contains file icons, primary module commands, tool commands, stretch spacers, and run controls; a 34 px module tab row below it contains Scene, Effect, Ability, Tags, Perception, Material, and the More overflow entry.

`toolbar_layout.rs` owns the responsive visibility rules. At compact and regular widths it keeps the primary authoring modules readable, hides secondary module tabs and secondary command groups, and exposes `WorkbenchModuleMore` as the overflow entry. `componentized_window.rs` only wires this owner after layout recomputation.

`workbench_window.v2.ui.toml` keeps the top toolbar region and static popup defaults aligned to the 72 px toolbar bottom. `window_menu_state.rs` still computes live menu frames from trigger frames, toolbar bottom, popup constraints, and root bounds, so Main, Run Mode, Layout, and Module More menus follow the final arranged toolbar instead of depending on hand-authored open-state coordinates.

## Compact Behavior

At widths up to `COMPACT_TOOLBAR_MAX_WIDTH`, the toolbar keeps Scene, Effect, Ability, Tags, Perception, and Material visible with readable widths. Behavior, Render, Assets, VFX, and HUD are hidden behind the overflow entry. Diff, Simulate, and secondary tool/run groups are hidden until full-toolbar width.

`WorkbenchModuleMore` dispatches `workbench.module.more.open` and participates in preview action registration. The action now opens `WorkbenchModuleOverflowMenu`, a retained `WorkbenchPopupMenu` node declared in the Workbench window template. The overflow menu lists the hidden module tabs and routes selection back through the same tab dispatch path used by visible module tabs.

`WorkbenchModuleMore` uses `zircon_editor_shell/toolbar/more-vertical.svg` so the compact module overflow trigger reads as a toolbar menu/overflow affordance. It no longer reuses the tab/file-shaped `editor_pages/workbench/tabs/tab-overflow.svg` placeholder. The rendering path still uses the existing SVG visual asset raster path first and the established glyph fallback only when an asset is missing.

`WorkbenchRunMode` uses `zircon_editor_shell/toolbar/dropdown.svg` so the full-width run mode trigger reads as a toolbar dropdown affordance. It no longer reuses the tab/file-shaped `editor_pages/workbench/tabs/tab-overflow.svg` placeholder. Compact 900px layouts intentionally hide the run group; the full toolbar route is locked at 1672x941 by `full_workbench_run_mode_uses_toolbar_dropdown_icon` and the focused wide screenshot artifact.

`WorkbenchToolbarOpen` and `WorkbenchModuleBrowse` use `zircon_editor_shell/toolbar/folder-open.svg`, while `WorkbenchToolbarSave` and `WorkbenchModuleSave` use `zircon_editor_shell/toolbar/save.svg`. These controls now share the toolbar icon family instead of borrowing Workbench menu icon assets, so file-group icon buttons and module command buttons follow the same primitive visual identity.

The follow-up shell icon asset pass centralizes toolbar and compound-control icon rasterization in `template_icon_assets.rs`. Toolbar icon buttons and icon-bearing buttons now share the same asset-first helper as list/table/Inspector row glyphs, so semantic shell SVGs such as dropdown, more, check, chevron-right, disabled, mesh, and material render as real pixels before any local fallback glyph is used.

When the toolbar is not compact, `toolbar_layout.rs` collapses the overflow trigger and closes `WorkbenchModuleOverflowMenu` so a stale hidden popup cannot remain open after resizing. `window_menu_state.rs` also includes the overflow popup in the toolbar menu exclusivity set, so More, File/Edit, Run Mode, and Layout menus cannot stay open together.

## Visual Evidence

The screenshot harness refreshes `docs/tests/editor/editor-window-m3-workbench-900x620.png`. The validated 900px toolbar no longer displays `Sc...`, `Eff...`, or `Abili...` in the top module strip. The focused overflow screenshot writes `docs/tests/editor/editor-window-m3-workbench-module-overflow-900x620.png`, showing the More popup anchored below the toolbar with Behavior, Render, Assets, VFX, and HUD rows.

The 2026-06-26 two-row toolbar pass refreshed `docs/tests/editor/editor-window-m3-workbench-900x620.png` at 16:13:41 and `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` at 16:13:47. The validated 900px toolbar no longer places module tabs and Save/Browse/Compile in the same row; document tabs begin below the two-row toolbar. Build output used `D:\cargo-targets\zircon-editor-components-0626`, and screenshots were written under `docs/tests/editor`, not Cargo `target`.

The 2026-06-26 toolbar More icon follow-up refreshed `docs/tests/editor/editor-window-m3-workbench-900x620.png` at 16:34:03 and `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` at 16:34:09. The validated 900px toolbar shows the module More trigger as a vertical three-dot overflow icon instead of the old tab/file-shaped placeholder. Build output again used `D:\cargo-targets\zircon-editor-components-0626`, and screenshots were written under `docs/tests/editor`, not Cargo `target`.

The 2026-06-26 toolbar Run Mode icon follow-up refreshed `docs/tests/editor/editor-window-m3-workbench-900x620.png` at 16:48:11 and `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` at 16:48:17. Because the run group is hidden in the 900px tier, `capture_full_workbench_run_mode_visual_artifact` also writes `docs/tests/editor/editor-window-m3-workbench-run-mode-1672x941.png` at 16:53:12 to keep a visible full-toolbar artifact for the dropdown trigger. Build output again used `D:\cargo-targets\zircon-editor-components-0626`, and screenshots were written under `docs/tests/editor`, not Cargo `target`.

The 2026-06-26 Open/Browse/Save toolbar icon-family follow-up refreshed `docs/tests/editor/editor-window-m3-workbench-900x620.png` at 17:23:52 and `docs/tests/editor/editor-window-m3-asset-browser-900x620.png` at 17:23:58. `capture_full_workbench_run_mode_visual_artifact` also refreshed `docs/tests/editor/editor-window-m3-workbench-run-mode-1672x941.png` at 17:24:33 so the full toolbar shares the same evidence set. The wide artifact still exposes remaining X placeholder icon work for later slices; this pass only closes the Open/Browse/Save icon family mismatch. Build output again used `D:\cargo-targets\zircon-editor-components-0626`, and screenshots were written under `docs/tests/editor`, not Cargo `target`.

The next shell icon asset pass refreshed `docs/tests/editor/editor-window-m3-workbench-run-mode-1672x941.png` after routing toolbar, list-row, table-row action, and Inspector row glyphs through `template_icon_assets.rs`. The focused evidence covers toolbar SVG pixels, semantic alias pixels, list-row adornments, table-row actions, and Inspector row resource/shadow/disclosure glyphs; build output again used `D:\cargo-targets\zircon-editor-components-0626`, with screenshots under `docs/tests/editor` rather than Cargo `target`.
