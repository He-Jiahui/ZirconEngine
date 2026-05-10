---
related_code:
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench_drawer_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/retained_host/root_shell_projection.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/tab_drag/bridge.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
implementation_files:
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench_drawer_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench_drawer_source/layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
plan_sources:
  - .codex/plans/Zircon Editor Runtime UI Rust-Owned Retained Host 重构计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_layout_paths.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/drawer_source_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/floating_window_source.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/workbench/template_bridge.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - cargo check -p zircon_editor --lib --locked --message-format=short
  - cargo check -p zircon_editor --lib --tests --locked --message-format=short
  - cargo test -p zircon_editor --lib native_root_menu_pointer_click_dispatches_shared_menu_action_in_real_host --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_frame_request_recomputes_dirty_layout_before_presentation --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib --locked --target-dir target\codex-shared-a -- --test-threads=1
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir target\codex-shared-a (2026-05-09: workspace build passed, workspace test blocked in zircon_plugin_navigation_runtime --lib)
  - cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1 (2026-05-09: reproduced external navigation runtime blocker, 5 passed / 8 failed)
doc_type: module-detail
---

# Editor Workbench Shell

## Purpose

This document describes the current Rust-owned retained editor workbench shell. The active editor shell is owned by `zircon_editor::ui::retained_host`, consumes `.ui.toml` host assets, and projects editor state into Rust host-contract DTOs. It is not a compatibility layer for deleted generated UI sources.

## Ownership

- `zircon_editor::ui::host` owns `EditorManager`, module wiring, layout/view registration, startup state, asset manager access, and editor-side service boundaries.
- `zircon_editor::ui::workbench` owns `EditorState`, `WorkbenchLayout`, workbench snapshots, model projection, menu models, and editor pane semantics.
- `zircon_editor::ui::retained_host` owns native window glue, retained input bridges, presenter state, host-contract DTOs, template projection application, and native painting.
- `zircon_runtime::ui` and `zircon_runtime_interface::ui` own the shared UI surface, tree, layout, dispatch, component, and binding contracts that the editor host consumes.

The boundary is intentionally split this way so the retained host can draw and dispatch editor UI without taking ownership of workbench business state or runtime UI contracts.

## Host Runtime Flow

`run_editor(...)` creates a `UiHostWindow`, constructs `RetainedEditorHost`, wires retained callbacks, refreshes the initial state, and enters the host window run loop. `RetainedEditorHost` holds the editor runtime, manager, asset/resource channels, workbench chrome metrics, shared pointer bridges, template bridges, viewport controller, native window presenter store, and dirty flags.

The recompute path builds a `WorkbenchViewModel`, computes workbench geometry, refreshes builtin host template bridges, resolves floating-window projection bundles, and then calls `apply_presentation(...)`. Presentation application converts editor-owned workbench data into `HostWindowPresentationData` plus `PaneSurfaceHostContext` state. Existing live host interaction state, menu state, close prompts, text focus, and viewport images are preserved across full presentation replacement.

## Template Authority

The current shell structure comes from source-controlled `.ui.toml` assets, not from generated UI files. Important host assets include:

- `zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml`
- `zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml`
- `zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml`
- `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml`
- `zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml`
- `zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml`
- `zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml`
- `zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml`

`EditorUiHostRuntime` loads these assets, builds shared UI surfaces, registers stable bindings, and exposes retained host projections. `RetainedUiHostAdapter` maps generic host models into retained host node models with component kind, frame, clip, z, style, state, validation, popup, selection, drag/drop, and route metadata.

Current root-shell frame authority is the host `.ui.toml` geometry, not old workbench metrics. `workbench_shell.ui.toml` gives the menu bar `24px`, the page strip `32px`, the separator `1px`, the status bar `24px`, and the activity rail `44px`. That makes the body and document top boundary `57px`; at 1280x720 the document host frame is `44,57,1236,639`, and at 960x540 it is `44,57,916,459`.

Dedicated source assets must match that root-shell authority unless they are deliberately exercising a standalone fallback. `floating_window_source.ui.toml` therefore uses a `57px` top spacer and `44px` rail so floating-window default/clamp frames line up with the document host. `workbench_drawer_source.ui.toml` still supports its metric-only fallback, but `BuiltinHostWindowTemplateBridge` passes `WorkbenchBody` and `StatusBarRoot` anchors into `workbench_drawer_source/layout.rs` for real workbench projections so visible drawer frames are recomputed from the current root shell.

The source bridges keep their shared surfaces alive across layout recompute. Drawer-source construction still builds the initial surface from `EditorUiHostRuntime`, but subsequent standalone, workbench-model, and anchored recomputes mutate the existing `UiSurface`, mark root layout dirty when shell size is an input, and call `UiSurface::rebuild_dirty(...)`. Floating-window source recompute follows the same retained-surface pattern. This keeps node ids, render-cache state, and bridge-local surface state stable while still letting runtime layout/render rebuild only the dirty domains.

## Input Authority

All high-frequency workbench input is host-owned but shared-surface-first. The retained host uploads pointer facts and lets shared `UiSurface` / `UiPointerDispatcher` routes decide hits, capture, and semantic delivery.

Current retained bridge families include:

- menu and popup routes through `retained_host::menu_pointer` and `callback_dispatch::shared_pointer::menu`
- activity rail, host page, document tab, drawer header, and viewport toolbar routes through their retained pointer bridge modules
- shell drag/drop and splitter routes through `retained_host::shell_pointer`
- hierarchy, asset tree/content/reference, welcome recent, and scroll-only pane routes through retained list and detail pointer bridges
- viewport body and toolbar routes through `callback_dispatch::viewport` and `RetainedViewportController`

Stable editor events are produced after route resolution through template bindings and editor runtime dispatch. The host does not keep a second direct business callback path for list selection, tab activation, drawer toggles, menu selection, or pane surface actions.

## Host Contract And Painter

`retained_host::host_contract` is the Rust-owned DTO and native-window seam. It contains the presentation data, pane/context globals, input translation, surface hit testing, native pointer dispatch, redraw decisions, presenter, and painter modules.

The painter consumes host-contract data and shared template render commands. It is allowed to provide native fallback pixels for shell chrome, text, icons, viewport images, diagnostics overlays, close prompts, and retained template nodes. It must not introduce a second layout or business-state authority; arranged frames and stable action ids come from `.ui.toml`, shared surface projection, and editor workbench data.

Visual asset pixels stay inside that host-contract seam. `painter/visual_assets.rs` resolves runtime `UiVisualAssetRef` values and template image/icon metadata through the editor asset tree, then rasterizes SVG sources with `resvg` at the requested host paint target size. Missing assets still fall back to native placeholder behavior, but decoded SVG or bitmap pixels are clipped and alpha-blended by the Rust-owned painter rather than by a restored generated UI layer.

## Hard Cutover From Deleted Slint Host

The old owner path was `zircon_editor::ui::slint_host` and the old source tree included `zircon_editor/ui/**/*.slint`. Those paths are historical only. They must not be restored as a compatibility module, shim, facade, re-export, generated include, build dependency, or active documentation owner.

Remaining references to Slint are allowed only as historical cutover context, no-Slint guard wording, or dependency-deletion evidence. Current code, tests, docs, and validation commands should use `retained_host`, `.ui.toml`, and Rust-owned `host_contract` names.

## Validation

The retained shell is guarded by:

- source tests that reject active deleted UI source files and generated build seams
- retained host window and boundary tests under `zircon_editor/src/tests/host/retained_window`
- retained pointer tests under `zircon_editor/src/tests/host/retained_*`
- template-runtime tests under `zircon_editor/src/tests/host/template_runtime`
- integration-contract readers for `workbench_retained*`
- editor boundary tests for `.ui.toml` host assets and workbench projection cutover

The milestone validation target remains `cargo check -p zircon_editor --lib --locked --message-format=short`, `cargo check -p zircon_editor --lib --tests --locked --message-format=short`, and then the repository validator when unrelated active-workstream blockers are clear or classified.

The 2026-05-09 workspace rerun reached the retained-host test build after the earlier reflection blocker moved forward. The retained-host slice removed all obsolete `i_retained_backend_testing::init_no_event_loop()` calls and kept `ModelRc` construction on the Rust-owned `VecModel`/`ModelRc` path. This keeps the hard cutover honest: retained tests instantiate `UiHostWindow` directly and do not reintroduce a generated or toolkit-backed test backend dependency.

The same 2026-05-09 geometry cleanup revalidated focused retained-host authority tests after the shell moved to `24 + 32 + 1 = 57` top chrome and `44px` rail sizing. Focused passes covered `workbench_projection`, `drawer_source_projection`, `floating_window_source`, `shared_surface`, `retained_host_page_pointer`, `retained_activity_rail_pointer`, and `retained_callback_dispatch::workbench::template_bridge` using `target\codex-shared-a`; warnings remain existing unused/dead-code warnings.

The 2026-05-10 retained/template performance follow-up strengthens `drawer_source_projection` and `floating_window_source` reuse guards. They now require stable source-surface node ids, stable render command counts, layout recomputation through `rebuild_dirty(...)`, and positive render-command reuse. A bridge that replaces its surface with a newly instantiated deterministic tree no longer satisfies the test.

The later menu-pointer timeout was classified as a command-budget artifact rather than a retained menu-pointer hang. The focused `native_root_menu_pointer_click_dispatches_shared_menu_action_in_real_host` test passed in isolation, and the next visible full-suite stop points, `native_frame_request_recomputes_dirty_layout_before_presentation` and `child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch`, also passed in isolation. A redirected serial run of `cargo test -p zircon_editor --lib --locked --target-dir target\codex-shared-a -- --test-threads=1` completed with `1162 passed; 0 failed; 4 ignored` in 2018.92s. A fresh 2026-05-09 recheck repeated the native menu-pointer case with 1 passed / 0 failed in 14.05s after compile, then repeated the full redirected serial gate with `1162 passed; 0 failed; 4 ignored` in 2126.68s.

The final retained-host workspace validator attempt used `./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir target\codex-shared-a`. It passed `cargo build --workspace --locked`, then failed during `cargo test --workspace --locked` in `zircon_plugin_navigation_runtime --lib`. The focused reproduction `cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1` failed with `5 passed / 8 failed`; the failure cluster is world-driven navigation scans seeing missing/default dynamic authoring components after world mutation, while direct manager/navmesh checks still pass. That blocker is outside the retained editor host and does not justify restoring `slint_host`, generated UI, or backend-testing dependencies.
