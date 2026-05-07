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

- Status: accepted with warnings-only validation after spec-compliance review fixes.
- Milestone 1 Rust implementation files with current diffs: `zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs`, `zircon_editor/src/ui/slint_host/host_contract/mod.rs`, `zircon_editor/src/ui/slint_host/host_contract/presenter.rs`, `zircon_editor/src/ui/slint_host/host_contract/window.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs`, and `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`.
- Milestone 1 planned formatting gate coverage also includes `zircon_editor/src/ui/slint_host/app/invalidation.rs`, `zircon_editor/src/tests/host/slint_window/shell_window.rs`, and rustfmt child-module coverage reached through `zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs`. `invalidation.rs` and `shell_window.rs` are retained in the gate because the Milestone 1 diagnostics path crosses invalidation snapshots and the native host snapshot fixture, but they do not currently show Milestone 1 diffs in this worktree. The rustfmt child-module coverage produced formatter-only cleanup in `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs` and `zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs`.
- Milestone 1 documentation and acceptance evidence: `docs/editor-and-tooling/editor-workbench-shell.md` section `2026-05-06 Native Visual Assets And Debug Overlay`, `docs/ui-and-layout/slate-style-ui-surface-frame.md` section `Editor Native Fast Path` paragraphs beginning `The top-right debug readout...` and `That live overlay...`, `docs/superpowers/plans/2026-05-06-global-ui-material-responsive-diagnostics.md` Milestone 1 diagnostics gate, and this acceptance section. Other lifecycle, reflector, property, and M7 material currently present in `docs/ui-and-layout/slate-style-ui-surface-frame.md` belongs to unrelated active-session work and is not Milestone 1 overlay evidence.
- Runtime text/layout and runtime-interface diffs visible in the current worktree are active-session work outside this Milestone 1 acceptance record. This diagnostics gate does not accept or claim `zircon_runtime/src/ui/text/layout_engine.rs` behavior changes.
- Unrelated active-session dirty work may exist elsewhere in the repository and is outside this Milestone 1 acceptance record.
- Behavior covered by source tests: overlay text changes after two recorded presents, the startup fallback uses the same `FPS/present/full/region/pixels/slow/render/paint-only` field shape as live overlay text, region/full paint counters split correctly, invalidation paint-only/render counters appear in overlay text, presenter damage expansion uses the presentation-derived top-bar frame instead of fallback-only geometry, the presenter diagnostics plan makes same-frame visible `pixels` match expanded region damage, the host window state overlay update path stores returned diagnostics text, and the existing top-right marker snapshot keeps checking visible pixels.
- Formatting: `rustfmt --edition 2021 --check "zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs" "zircon_editor/src/ui/slint_host/host_contract/mod.rs" "zircon_editor/src/ui/slint_host/host_contract/presenter.rs" "zircon_editor/src/ui/slint_host/host_contract/window.rs" "zircon_editor/src/ui/slint_host/app/invalidation.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs" "zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs" "zircon_editor/src/tests/host/slint_window/shell_window.rs"` passed with no output.
- Snapshot validation: `cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed: 1 passed, 0 failed, 1022 filtered out.
- Diagnostics validation: `cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed: 20 passed, 0 failed, 1003 filtered out. This includes presenter-path, presentation-top-bar geometry, startup-field-shape, and host-state overlay tests, not only DTO formatting.
- Compile validation: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never` passed. It emitted existing unused-import/dead-code warnings in `zircon_runtime`, plus editor dead-code warnings, but no errors.
- Support-layer note: the shared runtime surface root currently re-exports `layout_text` from `ui::text`, matching the text subsystem ownership. Broader runtime text-layout behavior changes remain outside this Milestone 1 editor diagnostics acceptance.

## Milestone 2 Text/Material Gate

- Status: accepted with existing warning noise after the reopened compliance testing stage.
- Coordination script: `.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4`.
- Coordination timestamp: 2026-05-06 08:54:28 +08:00.
- Active sessions read before source edits: `.codex/sessions/20260506-0445-ui-lifecycle-reflection-reflector.md`, `.codex/sessions/20260506-0520-ui-render-slate-contract.md`, `.codex/sessions/20260506-0414-widget-behavior-closure.md`, `.codex/sessions/20260506-0428-ui-m7-invalidation-performance.md`, `.codex/sessions/20260506-0446-ui-complete-input-events.md`, `.codex/sessions/20260506-0424-hit-test-unreal-slate.md`, and stale-but-overlapping `.codex/sessions/20260506-0112-material-layout-foundation.md`.
- Reopened compliance implementation files: `zircon_runtime/src/ui/surface/render/resolve.rs`, `zircon_runtime/src/ui/tests/material_layout.rs`, `zircon_editor/assets/ui/editor/material_meta_components.ui.toml`, `zircon_editor/assets/ui/editor/component_showcase.ui.toml`, `zircon_editor/assets/ui/runtime/runtime_hud.ui.toml`, `zircon_editor/assets/ui/runtime/pause_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/settings_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/inventory_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml`, `zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs`, and `zircon_editor/src/tests/ui/boundary/template_assets.rs`.
- Documentation files: `docs/ui-and-layout/runtime-ui-component-showcase.md`, `docs/ui-and-layout/shared-ui-core-foundation.md`, `docs/superpowers/plans/2026-05-06-global-ui-material-responsive-diagnostics.md`, and this acceptance file.
- Runtime coverage inventory: long Material button text expands from shared text measurement plus Material padding, icon-only Material button keeps a square frame, menu item text uses list-row min height and horizontal padding, tab text uses control height and text width plus padding, plain non-Material label remains text-only, Material label with layout attributes receives padding/min-height, table-row role uses list-row metrics, field controls measure visible `value`/`placeholder`/default option text, numeric fields measure numeric `value`, scalar/object options measure deterministic visible labels including boolean options, Image/Icon nodes with asset `value` metadata still render as image commands, `SvgIcon.source` resolves as an image command instead of text/group, and common native roles (`ProgressBar`, `Spinner`, `ContextActionMenu`) consume authored Material layout metrics.
- Editor/runtime projection coverage inventory: Component Showcase Material Button/Input/Number/Combo/List/Table/Menu/VirtualList/IconButton controls carry projected `layout_*` metrics, text/value controls assert shared desired widths are at least as wide as projected visible text plus Material horizontal padding before checking final arranged frames, and runtime HUD/dialog Button controls in the five runtime `.ui.toml` assets carry authored Material metrics and compute a 40px Material desired height through shared layout.
- Separate Component Showcase projection regression found during validation: `component_showcase` initially failed because `ContextActionMenuDemo` projected no popup anchor after the showcase row moved through `MaterialMenuFrame`; the shared reducer/projection path for `OpenPopupAt` was already intact, so the fix was wrapper metadata forwarding. Popup-anchor forwarding remains covered as a projection regression fix, but it is not counted as Milestone 2 shared Material measurement acceptance evidence. The reopened review then found field visible text, runtime dialog button metrics, table-row meta coverage, and wide-stretch projection-evidence gaps; these were fixed in shared visible-text resolution, TOML metadata/import coverage, and shared desired-size projection assertions instead of screen-specific host/painter fallbacks.
- `pane_body_documents.rs` remains a broad pre-existing integration bucket near the split threshold. This follow-up did not add a new responsibility there; future unrelated projection coverage should move to a focused module instead of further expanding that file.
- Formatting command passed with no output after an initial line-wrap-only rustfmt check failure in the new editor helper: `rustfmt --edition 2021 --check "zircon_runtime/src/ui/layout/pass/material.rs" "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/surface/render/resolve.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs" "zircon_editor/src/tests/ui/boundary/template_assets.rs"`.
- Runtime test command passed with existing warning noise after the first cold-compile attempt timed out before tests: `cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture` ran 17 tests, 17 passed, 0 failed, 872 filtered out.
- Editor projection test command passed with existing warning noise after adding `MaterialTableRow` to the Component Showcase import list: `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture` ran 19 tests, 19 passed, 0 failed, 1019 filtered out.
- Focused runtime dialog/HUD measurement command passed with existing warning noise: `cargo test -p zircon_editor --lib runtime_dialog_and_hud_buttons_participate_in_material_measurement --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never -- --nocapture`.
- Runtime compile command passed with existing warning noise: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.
- Editor compile command passed with existing warning noise: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.

## Milestone 3 Responsive Gate

- Status: accepted with scoped validation and existing warning noise. The final post-portability-fix `global_material_surface_assets` rerun succeeded after the earlier explicit-marker Cargo process exit and after the low-disk cleanup of `E:\zircon-build\targets\global-ui`.
- Coordination timestamps: validation closeout at 2026-05-06 23:40 +08:00 and documentation/evidence closeout at 2026-05-07 00:18:40 +08:00. Active overlap remained in editor/runtime validation lanes, so this record accepts only the Milestone 3 scoped files and tests; it does not claim the full workspace is clean or green.
- Implementation files: `zircon_runtime_interface/src/ui/layout/scroll.rs`, `zircon_runtime_interface/src/ui/layout/mod.rs`, `zircon_runtime/src/ui/template/build/parsers.rs`, `zircon_runtime/src/ui/template/build/layout_contract.rs`, `zircon_runtime/src/ui/layout/pass/measure.rs`, `zircon_runtime/src/ui/layout/pass/arrange.rs`, `zircon_runtime/src/ui/tests/material_layout.rs`, `zircon_runtime/src/ui/tests/shared_core.rs`, `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs`, and `zircon_editor/src/tests/ui/boundary/mod.rs`.
- UI asset files touched in this gate: `zircon_editor/assets/ui/editor/console.ui.toml`, `zircon_editor/assets/ui/editor/welcome.ui.toml`, `zircon_editor/assets/ui/editor/host/console_body.ui.toml`, `zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml`, and `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml`.
- Lockfile consistency: `Cargo.lock` now records the dirty manifest dependencies already present in this worktree, `resvg` for `zircon_editor` and `unicode-segmentation` for `zircon_runtime`; this keeps all scoped commands on `--locked`.
- Runtime responsive coverage: `WrapBox` is parsed through `layout.container.kind = "WrapBox"`, represented by `UiWrapBoxConfig`, measured by shared runtime layout using available width/bounds, arranged into rows with horizontal/vertical gaps and item minimum width, and covered by `shared_core` wrap tests.
- Global asset conformance coverage: `global_material_surface_assets` enumerates the 54 editor, host, window, and runtime `.ui.toml` surfaces; checks Material theme import reachability, responsive root/stretch or bounded-root exceptions, plain interactive Material contracts, fixed-axis allow reasons, and collection-heavy viewport/scroll metadata. The latest harness change normalizes `res://ui/...` import graph paths so the test is not Windows-separator-only.
- Formatting command passed with no output: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/layout/scroll.rs" "zircon_runtime_interface/src/ui/layout/mod.rs" "zircon_runtime/src/ui/template/build/parsers.rs" "zircon_runtime/src/ui/template/build/layout_contract.rs" "zircon_runtime/src/ui/layout/pass/measure.rs" "zircon_runtime/src/ui/layout/pass/arrange.rs" "zircon_runtime/src/ui/tests/material_layout.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs" "zircon_editor/src/tests/ui/boundary/mod.rs"`.
- Superseded blocked evidence: after adding `material_import_graph_uses_normalized_res_paths`, the explicit-marker command on `D:\cargo-targets\global-ui-m3-validation-2` exited with `M3_GLOBAL_MATERIAL_SURFACE_ASSETS_EXIT=-1` before a test summary and no Rust diagnostic. This was treated as an environmental/process interruption, not accepted evidence.
- Final global conformance command passed with existing warning noise: `cargo test -q -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture` passed with `2 passed; 0 failed; 1071 filtered out`.
- Runtime Material regression command passed with existing warning noise: `cargo test -q -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Runtime shared-core regression command passed with existing warning noise: `cargo test -q -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Runtime interface compile command passed: `cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.
- Runtime compile command passed with existing warning noise: `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.
- Editor compile command passed with existing warning noise: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.
- Disk evidence: `E:` fell below the 50 GB validation threshold during this closeout, so `cargo clean --target-dir "E:\zircon-build\targets\global-ui"` was run and removed `15114 files, 35.5GiB total` before the final scoped validation completed.

## Milestone 4 Theme Gate

- Status: accepted with scoped validation and existing warning noise. This record closes the Material theme/native painter consistency scope only; it does not claim full workspace validation or a clean working tree.
- Coordination timestamps: implementation coordination at 2026-05-07 00:44:37 +08:00 and documentation closeout coordination at 2026-05-07 01:56:33 +08:00. The later scan found a newer Slate layout plan and active Slate sessions, so this closeout stayed in the existing M4 Material evidence/docs lane.
- Implementation files: `zircon_editor/assets/ui/theme/editor_material.ui.toml`, `zircon_editor/assets/ui/editor/material_meta_components.ui.toml`, `zircon_editor/assets/ui/runtime/runtime_hud.ui.toml`, `zircon_editor/assets/ui/runtime/pause_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/settings_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/inventory_dialog.ui.toml`, `zircon_editor/assets/ui/runtime/quest_log_dialog.ui.toml`, `zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs`, `zircon_editor/src/tests/host/slint_window/native_material_painter.rs`, `zircon_editor/src/tests/host/slint_window/mod.rs`, and `zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs`.
- Documentation files: `docs/superpowers/plans/2026-05-06-global-ui-material-responsive-diagnostics.md`, `docs/ui-and-layout/shared-ui-template-runtime.md`, `docs/ui-and-layout/runtime-ui-component-showcase.md`, `docs/editor-and-tooling/editor-workbench-shell.md`, this acceptance file, and `.codex/sessions/20260505-2334-asset-browser-material-svg-fps.md`.
- Material token coverage: the editor Material theme now owns the M4 state palette vocabulary for `material_surface_pressed`, `material_surface_selected`, `material_surface_disabled`, `material_accent_soft`, `material_text_disabled`, `material_warning`, `material_error`, and `material_focus_ring`, while the native painter mirrors those values through a shared private palette instead of per-screen color constants.
- Material component coverage: `material_meta_components.ui.toml` now projects stable Material classes and state parameters for control roots, including hover/press/focus/selected/checked/disabled style metadata. Runtime HUD/dialog assets use the same Material vocabulary and layout metadata as editor surfaces instead of a separate runtime visual language.
- Native painter coverage: `template_nodes.rs` honors an explicit `surface_variant = "inset"` before the generic Button hover fallback, and the fallback now applies `PALETTE.surface_hover` only when no explicit surface variant is authored. This preserves authored Material state while keeping deterministic native painter fallback colors aligned with the theme asset.
- Global asset coverage: `global_material_surface_assets` keeps the 54-file global `.ui.toml` inventory under the Material import/class/layout contract and adds M4 state-token/runtime-surface assertions.
- Formatting command passed with no output after focused M4 painter/test formatting: `rustfmt --edition 2021` on `zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs`, `zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs`, `zircon_editor/src/tests/host/slint_window/native_material_painter.rs`, and `zircon_editor/src/tests/host/slint_window/mod.rs`.
- Disk evidence before final scoped validation: `(Get-PSDrive -Name E).Free` returned `67526467584`.
- Final global conformance command passed with existing warning noise: `cargo test -q -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Focused native painter command passed with existing warning noise: `cargo test -q -p zircon_editor --lib native_template_painter_uses_material_state_palette_for_controls --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Editor compile command passed with existing warning noise: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" --message-format short --color never`.
- Component Showcase command passed with existing warning noise: `cargo test -q -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Runtime dialog/HUD measurement command passed with existing warning noise: `cargo test -q -p zircon_editor --lib runtime_dialog_and_hud_buttons_participate_in_material_measurement --locked --jobs 1 --target-dir "E:\zircon-build\targets\global-ui" -- --nocapture`.
- Accepted risks: existing `zircon_runtime` warning noise remains; visual-asset cache hot reload is still not implemented; M5 damage/invalidation performance is not accepted by this M4 theme gate; full workspace validation was not run.
