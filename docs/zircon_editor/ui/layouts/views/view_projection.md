---
related_code:
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/console.rs
  - zircon_editor/src/ui/layouts/views/hierarchy.rs
  - zircon_editor/src/ui/layouts/views/inspector.rs
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/layouts/views/welcome.rs
  - zircon_editor/assets/ui/editor/project_overview.v2.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.v2.ui.toml
  - zircon_editor/assets/ui/editor/console.v2.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.v2.ui.toml
  - zircon_editor/assets/ui/editor/inspector.v2.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.v2.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.v2.ui.toml
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_base.v2.ui.toml
implementation_files:
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/console.rs
  - zircon_editor/src/ui/layouts/views/hierarchy.rs
  - zircon_editor/src/ui/layouts/views/inspector.rs
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/layouts/views/welcome.rs
  - zircon_editor/assets/ui/editor/project_overview.v2.ui.toml
  - zircon_editor/assets/ui/editor/asset_browser.v2.ui.toml
  - zircon_editor/assets/ui/editor/console.v2.ui.toml
  - zircon_editor/assets/ui/editor/hierarchy.v2.ui.toml
  - zircon_editor/assets/ui/editor/inspector.v2.ui.toml
  - zircon_editor/assets/ui/editor/assets_activity.v2.ui.toml
  - zircon_editor/assets/ui/editor/animation_editor.v2.ui.toml
  - zircon_editor/assets/ui/editor/welcome.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_base.v2.ui.toml
plan_sources:
  - user: 2026-05-11 hard-cut editor first screen and core panes to UI v2 schema
  - user: 2026-05-12 continue removing old schema fallback from editor UI v2 projection
tests:
  - cargo check -p zircon_editor (2026-05-11: passed)
  - cargo test -p zircon_editor asset_browser -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor template_assets -- --nocapture (2026-05-11: passed, 9 passed)
  - cargo test -p zircon_editor bootstrap_assets -- --nocapture (2026-05-11: passed, 24 passed)
  - cargo test -p zircon_editor boundary -- --nocapture (2026-05-11: passed, 72 passed)
  - cargo test -p zircon_editor --lib view_template_projection_rejects_legacy_asset_paths -- --nocapture --test-threads=1 (2026-05-12)
  - cargo test -p zircon_editor --lib critical_editor_shells_are_hard_cut_to_v2_assets -- --nocapture --test-threads=1 (2026-05-12)
  - cargo test -p zircon_editor --lib editor_v2_replacement_assets_do_not_keep_same_name_v1_sources --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo test -p zircon_editor --lib global_material_surface_assets_follow_responsive_contracts --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo test -p zircon_editor --lib editor_visual_density_contracts_keep_icons_and_chrome_professional_scale --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo check -p zircon_editor --lib --jobs 1 (2026-05-13: passed)
doc_type: module-detail
---

# View Projection

`view_projection` is the editor bridge that turns retained runtime UI surfaces into Slint-facing `ViewTemplateNodeData`. It now routes editor pane assets exclusively through `.v2.ui.toml` and `zircon_runtime::ui::v2`. Non-v2 asset paths return `ViewTemplateProjectionError::LegacyAssetPath` instead of falling back to `UiPrototypeStoreFileCache`, `UiDocumentCompiler`, or `UiTemplateSurfaceBuilder`.

## v2 Path

For `.v2.ui.toml` assets, the projection loader:

- resolves the asset and style source list through the v2 source-path helper,
- loads and reuses the heap-backed `UiV2PrototypeStoreFileCache`,
- compiles the view with imported component/style prototypes already resident in the v2 prototype store,
- builds a retained `UiSurface` with `UiV2SurfaceBuilder`,
- runs layout once for the requested pane size,
- extracts the same render commands into `ViewTemplateNodeData`.

There is no fallback from a v2 asset to the legacy recursive document path. If a v2 asset is malformed, projection fails for that pane instead of silently reparsing with the old schema.

The projection module also hard-rejects old `.ui.toml` view paths before attempting any load. This keeps editor view panes on the v2 heap-resident prototype cache and prevents accidental reintroduction of the recursive schema during later pane work.

The same-name v1 production assets for the converted editor panes, workbench chrome surfaces, host shell fragments, and editor window shells have been removed. Remaining v1 editor assets are restricted to UI Asset Editor authoring fixtures and old-schema component libraries that still have explicit tests, not the active host projection path.

## Current Cutover

These top-level editor pane projections now load from v2 assets:

- `ProjectOverview`: `zircon_editor/assets/ui/editor/project_overview.v2.ui.toml`
- `AssetBrowser`: `zircon_editor/assets/ui/editor/asset_browser.v2.ui.toml`
- `Console`: `zircon_editor/assets/ui/editor/console.v2.ui.toml`
- `Hierarchy`: `zircon_editor/assets/ui/editor/hierarchy.v2.ui.toml`
- `Inspector`: `zircon_editor/assets/ui/editor/inspector.v2.ui.toml`
- `AssetsActivity`: `zircon_editor/assets/ui/editor/assets_activity.v2.ui.toml`
- `AnimationEditor`: `zircon_editor/assets/ui/editor/animation_editor.v2.ui.toml`
- `Welcome`: `zircon_editor/assets/ui/editor/welcome.v2.ui.toml`

The conversions preserve existing authored control IDs and geometry semantics so host presenters and pointer routes continue to locate the same controls while loading through the v2 prototype cache and surface builder.

Dynamic text still comes from the existing Rust presenters through `text_overrides` or host payload attributes, preserving the no-embedded-script rule.
