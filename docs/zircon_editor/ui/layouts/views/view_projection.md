---
related_code:
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection/tests.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/compact_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout.rs
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
  - zircon_editor/src/ui/layouts/views/view_projection/tests.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/ui/layouts/views/project_overview.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/compact_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/toolbar_layout.rs
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
  - user: 2026-06-24 implement editor_layout visual architecture and screenshot acceptance
tests:
  - cargo check -p zircon_editor (2026-05-11: passed)
  - cargo test -p zircon_editor asset_browser -- --nocapture (2026-05-11: passed, 6 passed)
  - cargo test -p zircon_editor template_assets -- --nocapture (2026-05-11: passed, 9 passed)
  - cargo test -p zircon_editor bootstrap_assets -- --nocapture (2026-05-11: passed, 24 passed)
  - cargo test -p zircon_editor boundary -- --nocapture (2026-05-11: passed, 72 passed)
  - cargo test -p zircon_editor --lib view_template_projection_rejects_non_v2_asset_paths -- --nocapture --test-threads=1 (2026-06-05: pending rerun while editor Cargo lanes are active)
  - cargo test -p zircon_editor --lib critical_editor_shells_are_hard_cut_to_v2_assets -- --nocapture --test-threads=1 (2026-05-12)
  - cargo test -p zircon_editor --lib editor_v2_replacement_assets_do_not_keep_same_name_v1_sources --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo test -p zircon_editor --lib global_material_surface_assets_follow_responsive_contracts --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo test -p zircon_editor --lib editor_visual_density_contracts_keep_icons_and_chrome_professional_scale --jobs 1 -- --nocapture --test-threads=1 (2026-05-13: passed, 1 test)
  - cargo check -p zircon_editor --lib --jobs 1 (2026-05-13: passed)
  - cargo test -p zircon_editor --lib text_input_components_own_generated_text_commands_for_native_painter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-visual-fix-0624 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24: passed, 1 passed)
  - cargo test -p zircon_editor --lib projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-visual-fix-0624 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-24: passed, 7 passed)
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-visual-fix-0624 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-24: passed, refreshed Asset Browser and Workbench PNGs)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-visual-fix-0624 --message-format short --color never (2026-06-24: passed with existing warning noise)
  - E:\cargo-targets\zircon-editor-layout-visual-fix-0624\debug\deps\zircon_editor-820618fe5427109a.exe tests::ui::project_overview::bootstrap_assets::project_overview_projection_maps_bootstrap_asset_into_template_nodes --exact --test-threads=1 --nocapture (2026-06-24: passed, one `OpenAssetsView` Button and one `OpenAssetBrowser` Button)
  - E:\cargo-targets\zircon-editor-layout-visual-fix-0624\debug\deps\zircon_editor-820618fe5427109a.exe tests::host::retained_menu_pointer::visual_screenshot::capture_m3_gui_acceptance_visual_artifacts --ignored --exact --test-threads=1 --nocapture (2026-06-24: passed, refreshed `target/visual-layout` screenshots at 09:43 +08)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --message-format short --color never (2026-06-24 09:59 +08: passed with existing warning noise)
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-completion-audit-0624 --message-format short --color never -- --test-threads=1 (2026-06-24: timed out after 20 minutes during compile; not counted as passing)
  - cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never (2026-06-25: passed, 1 passed)
  - cargo test -p zircon_editor --lib capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never -- --ignored --test-threads=1 --nocapture (2026-06-25: passed, refreshed `docs/tests/editor`)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 12 passed)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never (2026-06-26: passed with existing warning noise)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor`)
  - cargo fmt -p zircon_editor (2026-06-26: passed)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 12 passed; compact content header/table/preview duplicate-container guards)
  - cargo test -p zircon_editor --lib asset_preview_selected_surface_uses_slate_outline_emphasis --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 1 passed)
  - cargo test -p zircon_editor --lib template_style --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 6 passed; asset-preview selected emphasis guard)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 12 passed; selected preview low-emphasis style still preserves compact layout)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed with existing warning noise)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor`; no repo target; selected preview card dark fill + thin outline verified)
  - cargo fmt -p zircon_editor --check (2026-06-26: passed after Asset Browser compact toolbar slice)
  - cargo test -p zircon_editor --lib asset_browser_projection_compacts_preview_utility_for_short_viewport --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, compact toolbar/search/kind/view/Quick Import geometry guard)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 16 passed; compact toolbar owner integrated with existing Asset Browser projection guards)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed with existing warning noise after compact toolbar slice)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`; no repo target)
  - cargo test -p zircon_editor --lib asset_browser_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, Quick Import placeholder text projects while value_text stays empty)
  - cargo test -p zircon_editor --lib asset_browser_projection_compacts_preview_utility_for_short_viewport --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, compact Quick Import placeholder geometry/value guard)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 16 passed after Quick Import placeholder slice)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed with existing warning noise after Quick Import placeholder slice)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`; no repo target; Quick Import placeholder visible)
  - cargo test -p zircon_editor --lib asset_browser_projection_compacts_preview_utility_for_short_viewport --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: red then passed, compact utility duplicate projection collapse guard)
  - cargo test -p zircon_editor --lib asset_browser --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-26: passed, 16 passed after compact utility duplicate projection collapse)
  - cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never (2026-06-26: passed with existing warning noise after compact utility duplicate projection collapse)
  - cargo test -p zircon_editor capture_m3_gui_acceptance_visual_artifacts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0626 --message-format short --color never -- --ignored --test-threads=1 --nocapture (2026-06-26: passed, refreshed `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`; no repo target; collapsed utility Preview residual hidden)
doc_type: module-detail
---

# View Projection

`view_projection` is the editor bridge that turns retained runtime UI surfaces into Slint-facing `ViewTemplateNodeData`. It now routes editor pane assets exclusively through `.v2.ui.toml` and `zircon_runtime::ui::v2`. Non-v2 asset paths return `ViewTemplateProjectionError::NonV2AssetPath` instead of falling back to `UiPrototypeStoreFileCache`, `UiDocumentCompiler`, or `UiTemplateSurfaceBuilder`.

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

## Component-Owned Text

The 2026-06-24 retained-host visual pass treats `Button`, `InputField`, `TextField`, and `NumberField` as component-owned text controls. `view_projection.rs` first collects generated `UiRenderCommandKind::Text` content by `control_id`, pairs it with the largest non-text command frame for the same control, and emits the text carrier as the semantic node with the full component frame restored. The non-text generated commands for that component are skipped, so native retained painting keeps one node per interactive control while preserving labels, placeholders, and live input text.

This rule fixed the Asset Browser and Assets Activity search fields: the `SearchEdited` control now projects one visible `TextField` node with a single text payload, not a component plus a separate generated text node. The same projection path now covers Project Overview buttons: `OpenAssetsView` and `OpenAssetBrowser` each project exactly one `Button` node with stable text, variant, binding, and dispatch metadata. The focused projection tests assert those counts and fields, and the M3 screenshot harness verifies the 900x620 Asset Browser and Workbench PNGs render without duplicate or clipped text.

The projection cache lock also now returns `ViewTemplateProjectionError::V2StoreCachePoisoned` instead of panicking through a production `expect(...)`. The inline regression tests were moved to `view_projection/tests.rs`, keeping the production owner focused on v2 loading, command projection, component metadata, and duplicate-text filtering.

## Display Text Versus Dispatch Value

The 2026-06-25 Asset Browser projection pass keeps authored labels and internal values separate. View-mode buttons and utility tabs project display text such as `List`, `Thumb`, `Preview`, `References`, `Metadata`, and `Plugins`, while `value_text` remains the lowercase dispatch value (`list`, `thumbnail`, `preview`, and related tab keys). Table header and row nodes also carry column `options` so retained painting can treat table text as component-owned cell content instead of duplicated free text.

`asset_browser_projection_maps_bootstrap_asset_into_mount_nodes` asserts the text/value split and table options directly, and the M3 screenshot harness confirms the 900x620 Asset Browser render uses the display labels.

## Asset Browser Component Composition

The 2026-06-26 Asset Browser projection pass keeps the pane closer to the component-first plan. The Rust projection now removes inactive utility-tab content before painting, so `Preview`, `References`, `Metadata`, and `Plugins` do not stack hidden panels in the same short viewport. Empty and secondary preview areas use `asset-placeholder` surfaces, while the actual selected asset preview keeps the stronger selected treatment.

Short-viewport layout now lives in `asset_browser/compact_layout.rs` instead of growing the main `asset_browser.rs` owner. That child owner collapses the bottom utility body to a 28px tab strip, collapses the right details column when the main content height is too small, and expands the primary content column so the table and selected-preview summary stay readable. It also collapses the empty sources column when the pane's logical width drops below the compact source threshold, then lets the content panel take that horizontal space.

The compact toolbar/search/import stack now lives in `asset_browser/toolbar_layout.rs`, a sibling owner to the short-viewport content layout. It collapses the old title/subtitle rows, places search and Locate In Assets in one 28px row, places kind chips and List/Thumb view buttons in a second 24px row, hides low-priority kind chips when width runs out, and keeps Quick Import as a single 32px strip with label, path field, and Import command. The main content y is derived from the import strip bottom plus a root gap, so the slice stays relative/adaptive instead of relying on screenshot-specific absolute coordinates.

The Quick Import path field now follows the same display/value split as other component-owned controls. `asset_browser.v2.ui.toml` authors a placeholder for `AssetBrowserImportPathField`; `view_projection.rs` uses that placeholder as the visible text when no label or value is present, while `resolve_node_value_text` keeps `value_text` empty when the display text equals the placeholder. This gives the retained painter a real text payload for the empty field without turning placeholder copy into dispatch state.

The same compact owner now lays out the primary content stack directly: a 20px content header row with padded title/path text, a 4px header-to-table gap, a table stack limited to the header plus four visible asset rows with rows clamped to 30px, and a selected-preview summary card anchored inside the content panel below the table. Duplicate projected `AssetBrowserContentPanel` and `AssetBrowserAssetTablePanel` containers are collapsed to zero-size `frame_only` nodes so stale high-z surfaces cannot draw narrow content or table frames over the rows and preview.

The compact utility drawer also treats duplicate projected `control_id`s as one visual component. `asset_browser/compact_layout.rs` now applies frame and height updates to every matching projection node, not just the first match. When the short viewport collapses the utility content to a 28px tab strip, all `AssetBrowserPreview*` panel, visual, and text projections are moved to the collapsed frame with height 0 so hidden Preview content cannot remain visible under the tab row.

The selected-preview summary card now keeps its selected semantics without using a full cyan fill. The `asset-preview` and `asset-preview-visual` surfaces resolve selected/focused states to the low-emphasis pressed surface and preserve the authored 1px border, matching the darker Slate-like row/card selection pattern while leaving the selected asset identity visible.

The focused Asset Browser tests assert these frames directly: compact toolbar/search/kind/view/import strip geometry, utility content height 0, all duplicate Preview utility projections collapsed in short viewports, details width 0, sources width 0, the content panel expanded across the reclaimed columns, one visible content panel, one visible table panel, compact header geometry, table closure on the fourth asset row, and the selected-preview card staying inside the content panel. The M3 screenshot harness refreshes `docs/tests/editor/editor-window-m3-workbench-900x620.png` and `docs/tests/editor/editor-window-m3-asset-browser-900x620.png`.
