---
related_code:
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/session/v2_authoring.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.v2.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.v2.ui.toml
  - zircon_editor/assets/ui/editor/console.v2.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.v2.ui.toml
  - zircon_editor/assets/ui/editor/inspector.v2.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.v2.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.v2.ui.toml
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/editor/ui_asset_editor.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/asset_editor/session/v2_authoring.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_drawer_source.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/console_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/floating_window_source.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/module_plugins_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.v2.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.v2.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.v2.ui.toml
  - zircon_editor/assets/ui/editor/console.v2.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.v2.ui.toml
  - zircon_editor/assets/ui/editor/inspector.v2.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.v2.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.v2.ui.toml
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_popup.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_page_chrome.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_status_bar.v2.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.v2.ui.toml
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/editor/ui_asset_editor.v2.ui.toml
plan_sources:
  - user: 2026-05-11 hard-cut workbench host and core panes to UI v2
  - user: 2026-05-13 migrate UI Asset Editor authoring support to v2 so old schema assets can keep being removed
tests:
  - cargo check -p zircon_editor (2026-05-11: passed)
  - cargo test -p zircon_editor builtin_template_compile_cache_is_reused_across_runtime_instances -- --nocapture (2026-05-11: passed)
  - cargo test -p zircon_editor template_assets -- --nocapture (2026-05-11: passed, 10 passed)
  - cargo test -p zircon_editor viewport_toolbar -- --nocapture (2026-05-11: passed, 23 passed)
  - cargo test -p zircon_editor workbench_projection -- --nocapture (2026-05-11: passed, 12 passed)
  - cargo test -p zircon_editor bootstrap_assets -- --nocapture (2026-05-11: passed, 24 passed)
  - cargo test -p zircon_editor boundary -- --nocapture (2026-05-11: passed, 72 passed)
  - cargo test -p zircon_editor retained_menu_pointer -- --nocapture (2026-05-11: passed, 21 passed, 4 ignored)
  - cargo test -p zircon_editor retained_activity_rail_pointer -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor component_showcase -- --nocapture (2026-05-11: passed, 19 passed)
  - cargo test -p zircon_editor builtin_activity_window_documents_are_registered_in_host_runtime -- --nocapture (2026-05-11: passed, 1 passed)
  - cargo test -p zircon_runtime --lib component_catalog -- --nocapture (2026-05-11: passed, 43 passed)
  - cargo test -p zircon_editor --lib ui_asset_editor_v2_authoring_instantiates_imported_component_slots_for_preview --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 passed)
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 40 passed)
  - cargo test -p zircon_editor --lib global_material_surface_assets_follow_responsive_contracts --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 passed)
doc_type: module-detail
---

# Template Runtime Host

`EditorUiHostRuntime` now keeps a v2 prototype store and a compiled v2 document table beside the legacy template registry. Files ending in `.v2.ui.toml` are loaded through `UiV2AssetLoader`, inserted into `UiV2PrototypeStore`, and compiled with `UiV2DocumentCompiler::compile_with_prototype_store`. This makes composite component prototypes resident in heap-backed runtime state instead of reparsing a full recursive tree every time a document is projected.

## Projection Path

`project_document` and `project_pane_body` check the v2 compiled document table before falling back to the legacy template registry. V2 documents are projected from arena handles into retained host projections without re-instantiating the legacy `UiTemplateNode` tree. The arena projection uses an explicit stack, so deep v2 documents do not recurse through editor projection.

Pane payload injection is shared between old and v2 paths. Legacy panes still mutate a temporary `UiTemplateNode` before projection; v2 panes mutate the retained projection root and append any needed `HybridSlotAnchor` projection directly. This keeps existing Rust presenters and route IDs active while the pane body assets move to v2.

## Current Hard Cut

The builtin registry now routes these critical editor shell assets to v2:

- `editor_main_frame.v2.ui.toml`
- `workbench_shell.v2.ui.toml`
- `workbench_drawer_source.v2.ui.toml`
- `floating_window_source.v2.ui.toml`
- `scene_viewport_toolbar.v2.ui.toml`
- `animation_sequence_body.v2.ui.toml`
- `animation_graph_body.v2.ui.toml`
- `runtime_diagnostics_body.v2.ui.toml`
- `module_plugins_body.v2.ui.toml`
- `build_export_desktop_body.v2.ui.toml`
- `console_body.v2.ui.toml`
- `inspector_body.v2.ui.toml`
- `hierarchy_body.v2.ui.toml`

The view projection layer now routes these top-level pane assets to v2:

- `project_overview.v2.ui.toml`
- `asset_browser.v2.ui.toml`
- `console.v2.ui.toml`
- `hierarchy.v2.ui.toml`
- `inspector.v2.ui.toml`
- `assets_activity.v2.ui.toml`
- `animation_editor.v2.ui.toml`
- `welcome.v2.ui.toml`

The shared workbench chrome projection now routes these root chrome assets to v2:

- `workbench_menu_chrome.v2.ui.toml`
- `workbench_menu_popup.v2.ui.toml`
- `workbench_page_chrome.v2.ui.toml`
- `workbench_dock_header.v2.ui.toml`
- `workbench_status_bar.v2.ui.toml`
- `workbench_activity_rail.v2.ui.toml`

The runtime component showcase is also now routed through `component_showcase.v2.ui.toml`. It no longer imports the old recursive `component_widgets.ui.toml#ShowcaseSection` or `material_meta_components.ui.toml#Material*` references on the builtin path. The v2 asset uses flat arena nodes with formal catalog component ids, while retaining existing control ids, event route ids, and Material measurement props so Rust callback dispatch and retained host projection continue to work.

The UI Asset Editor bootstrap layout is now `ui_asset_editor.v2.ui.toml`. UI Asset Editor sessions detect v2 source through `UiV2AssetLoader`, keep the last valid v2 document resident on the session, and serialize edited/canonical source back as v2 instead of downgrading authoring output to the old recursive schema. The deleted `ui_asset_editor.ui.toml` path is covered by the asset boundary guard, so UI authoring can no longer quietly reopen the old bootstrap asset.

The UI Asset Editor authoring preview now mirrors the runtime v2 prototype path for registered imports. `v2_authoring.rs` builds a `UiV2PrototypeStore` from the current v2 document plus registered component/style imports, compiles through `compile_with_prototype_store`, and leaves the current asset source as a flat v2 view with import references. That gives authoring preview the same external component expansion, named slot fill, and props/state patch behavior as the runtime v2 path without re-entering the old recursive template builder.

`pane_data_conversion` now builds a shared surface and computes layout before building the host model for template pane bodies. This lets v2 pane bodies contribute frame, clip, z-order, component metadata, and event bindings through the same host model path as older shared surfaces.

Workbench shell and viewport toolbar bridges now keep their `UiSurface` instances resident after initial load. Recompute marks the surface root dirty and calls `rebuild_dirty(...)` before projecting the updated retained host model, so these high-frequency bridge layouts no longer rebuild a fresh shared surface for every pointer-adjacent host refresh.

## Remaining Scope

The runtime host still has old-template support for assets kept only as migration/test inputs. `ui_asset_editor.ui.toml` is no longer an exception and has been deleted; the remaining old-template inputs are legacy Material meta-component and additional authoring fixtures, such as `editor_widgets.ui.toml`, `material_meta_components.ui.toml`, `asset_browser.ui.toml`, `binding_browser.ui.toml`, `layout_workbench.ui.toml`, `preview_state_lab.ui.toml`, and `theme_browser.ui.toml`. These fixtures now live under `zircon_editor/src/tests/fixtures/ui_legacy/**`, outside the deployable asset roots. Staged `ZirconEngine/assets/ui/**` includes v2 UI templates only, and the guard test `packaged_ui_asset_roots_contain_only_v2_schema_files` prevents legacy `.ui.toml` files from returning to active editor/runtime asset roots.
