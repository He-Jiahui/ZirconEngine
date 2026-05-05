# Global UI Material, Responsive Layout, And Diagnostics Acceptance

## Milestone 0 Baseline Gate

- Status: baseline inventory recorded.
- Coordination script: `.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4`.
- Coordination timestamp: 2026-05-06 03:56:08 +08:00.
- Cargo build/test: intentionally not run for Milestone 0; this milestone is documentation and coordination only.
- Milestone 0 changed only coordination/documentation files: `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md`, `docs/superpowers/plans/2026-05-06-global-ui-material-responsive-diagnostics.md`, and `tests/acceptance/global-ui-material-responsive-diagnostics.md`.
- Source behavior files were not modified by Milestone 0. The repository may still contain unrelated dirty work from active sessions; this record does not claim the full working tree is clean.

## Active Conflicting Sessions Read

- `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md`: active owner note for Asset Browser Material/SVG/FPS and this global convergence handoff.
- `.codex/sessions/20260506-0355-material-ui-e3-binding-events.md`: active runtime event binding session touching `zircon_runtime/src/ui/surface/surface.rs` and `docs/ui-and-layout`.
- `.codex/sessions/20260506-0112-material-layout-foundation.md`: active Material layout session touching `zircon_runtime/src/ui/layout/pass`, `zircon_editor/assets/ui/editor/material_meta_components.ui.toml`, and `docs/ui-and-layout`.
- `.codex/sessions/20260505-1106-editor-native-text-input-regression.md`: active native text/input session touching `zircon_runtime/src/ui`, `zircon_editor/src/ui/slint_host`, and `docs/ui-and-layout`.
- `.codex/sessions/20260505-1502-editor-ui-layout-regression.md`: active shared Slate-style UI layout session touching `zircon_runtime/src/ui`, `zircon_editor/src/ui/slint_host`, and `docs/ui-and-layout`.

## Inventory Comparison

- Inventory source: `Glob` for `zircon_editor/assets/ui/**/*.ui.toml`.
- Scope globs matched: `zircon_editor/assets/ui/editor/*.ui.toml`, `zircon_editor/assets/ui/editor/host/*.ui.toml`, `zircon_editor/assets/ui/editor/windows/*.ui.toml`, `zircon_editor/assets/ui/runtime/*.ui.toml`, and `zircon_editor/assets/ui/theme/*.ui.toml`.
- Plan correction made: `zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml` existed in the current inventory and was added to the plan's concrete inventory list.

## Final `.ui.toml` Inventory

```text
zircon_editor/assets/ui/editor/animation_editor.ui.toml
zircon_editor/assets/ui/editor/asset_browser.ui.toml
zircon_editor/assets/ui/editor/assets_activity.ui.toml
zircon_editor/assets/ui/editor/binding_browser.ui.toml
zircon_editor/assets/ui/editor/component_showcase.ui.toml
zircon_editor/assets/ui/editor/component_widgets.ui.toml
zircon_editor/assets/ui/editor/console.ui.toml
zircon_editor/assets/ui/editor/editor_widgets.ui.toml
zircon_editor/assets/ui/editor/hierarchy.ui.toml
zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml
zircon_editor/assets/ui/editor/host/console_body.ui.toml
zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
zircon_editor/assets/ui/editor/host/inspector_body.ui.toml
zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
zircon_editor/assets/ui/editor/host/workbench_bottom_dock_header.ui.toml
zircon_editor/assets/ui/editor/host/workbench_document_dock_header.ui.toml
zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
zircon_editor/assets/ui/editor/host/workbench_menu_chrome.ui.toml
zircon_editor/assets/ui/editor/host/workbench_page_chrome.ui.toml
zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
zircon_editor/assets/ui/editor/host/workbench_side_dock_header.ui.toml
zircon_editor/assets/ui/editor/inspector.ui.toml
zircon_editor/assets/ui/editor/layout_workbench.ui.toml
zircon_editor/assets/ui/editor/material_meta_components.ui.toml
zircon_editor/assets/ui/editor/preview_state_lab.ui.toml
zircon_editor/assets/ui/editor/project_overview.ui.toml
zircon_editor/assets/ui/editor/theme_browser.ui.toml
zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
zircon_editor/assets/ui/editor/welcome.ui.toml
zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
zircon_editor/assets/ui/editor/workbench_menu_popup.ui.toml
zircon_editor/assets/ui/editor/workbench_page_chrome.ui.toml
zircon_editor/assets/ui/editor/workbench_status_bar.ui.toml
zircon_editor/assets/ui/runtime/inventory_dialog.ui.toml
zircon_editor/assets/ui/runtime/pause_dialog.ui.toml
zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml
zircon_editor/assets/ui/runtime/runtime_hud.ui.toml
zircon_editor/assets/ui/runtime/settings_dialog.ui.toml
zircon_editor/assets/ui/theme/editor_base.ui.toml
zircon_editor/assets/ui/theme/editor_material.ui.toml
```

## Milestone 1 Diagnostics Gate

- Status: accepted with warnings-only validation.
- Implementation files: `zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs`, `zircon_editor/src/ui/slint_host/host_contract/presenter.rs`, `zircon_editor/src/ui/slint_host/host_contract/window.rs`, `zircon_editor/src/ui/slint_host/app/invalidation.rs`, `zircon_editor/src/ui/slint_host/app/host_lifecycle.rs`, `zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs`, `zircon_editor/src/tests/host/slint_window/shell_window.rs`, `zircon_runtime/src/ui/surface/mod.rs`, `zircon_runtime/src/ui/text/mod.rs`, `zircon_runtime/src/ui/text/layout_engine.rs`, and `zircon_runtime_interface/src/ui/surface/mod.rs`.
- Behavior covered by source tests: overlay text changes after two recorded presents, region/full paint counters split correctly, invalidation paint-only/render counters appear in overlay text, and the existing top-right marker snapshot keeps checking visible pixels.
- Formatting: `rustfmt --edition 2021 --check --config skip_children=true "zircon_runtime/src/ui/surface/mod.rs" "zircon_runtime/src/ui/text/mod.rs" "zircon_runtime/src/ui/text/layout_engine.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/surface/diagnostics.rs" "zircon_runtime_interface/src/ui/surface/mod.rs"` passed with no output.
- Snapshot validation: `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed: 1 passed, 0 failed, 1009 filtered out.
- Diagnostics validation: `cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed: 13 passed, 0 failed, 997 filtered out.
- Compile validation: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed. It emitted existing unused-import/dead-code warnings in `zircon_runtime`, plus editor dead-code warnings, but no errors.
- Support-layer correction: the shared runtime surface root now re-exports `layout_text` from `ui::text`, matching the text subsystem ownership and preserving the boundary expectation that `zircon_runtime::ui::surface` exposes runtime surface behavior without importing it from `surface::render`.
