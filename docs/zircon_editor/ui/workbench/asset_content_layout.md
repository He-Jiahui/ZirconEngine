---
related_code:
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/profile.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/metrics.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/thumbnail_grid.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/text.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/compact_table_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/table_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/asset_content.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/asset_browser_content.rs
implementation_files:
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/profile.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/metrics.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/thumbnail_grid.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/text.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/compact_table_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/table_nodes.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_layout.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/asset_content.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/asset_browser_content.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/tests/host/retained_asset_pointer.rs
  - cargo test -p zircon_editor --lib asset_content_layout -- --nocapture
  - cargo test -p zircon_editor --lib shared_asset_content_pointer -- --nocapture
  - direct zircon_editor test binary filter activity_asset_content_ --nocapture
  - direct zircon_editor test binary filter asset_content_layout --nocapture
  - direct zircon_editor test binary filter browser_asset_content --nocapture
  - direct zircon_editor test binary filter asset_table_ --nocapture
  - direct zircon_editor test binary filter table_cell_text_ --nocapture
  - direct zircon_editor test binary filter asset_browser_list_scroll_repaints_rows_without_moving_header_or_preview --exact --nocapture
  - direct zircon_editor test binary filter asset_browser_thumbnail_scroll_repaints_grid_without_moving_fixed_controls --exact --nocapture
  - direct zircon_editor test binary filter shared_asset_content_pointer_bridge_hits_thumbnail_grid_columns_and_scrolls_rows --exact --nocapture
  - direct zircon_editor test binary filter browser_thumbnail_projector_scrolls_card_children_and_keeps_grid_fixed --exact --nocapture
  - direct zircon_editor test binary filter assets_drawer_content_scroll_repaints_inside_content_without_touching_utility --exact --nocapture
  - docs/tests/editor/editor-window-m3-asset-browser-list-scrolled-hover-900x620.png
  - docs/tests/editor/editor-window-m3-asset-browser-thumbnail-scrolled-hover-900x620.png
doc_type: module-detail
status: in_progress
---

# Shared asset content layout

`asset_content_layout` is the shared contract for asset rows that are both painted and pointer-addressable. Its navigational `mod.rs` delegates stable generated control ids to `controls.rs`, surface identity to `profile.rs`, geometry to `metrics.rs`, and runtime-measured file-name compaction to `text.rs`. Dense resource badge metadata now comes from the canonical asset-type presentation registry in `core/asset/type_registry/builtin.rs`; the deleted layout-local `labels.rs` owner is not retained as a facade or duplicate lookup. This directly addresses the repository structure findings instead of growing another mixed-responsibility root.

The metrics owner derives padding, gaps, row heights, and the Asset Browser header offset from `EditorDensityTokens` and `EditorControlTokens`; retained pointer code and Assets Activity visual projection consume the resulting `AssetContentLayoutMetrics` instead of maintaining parallel constants.

`list_height(...)` is also the single full-source extent owner. Activity layout keeps below-viewport rows at their real metric-derived positions and writes this extent onto the content panel. Paint projection and scrollbar code consume the same stable ids and extent, so neither reconstructs row heights or invents a second content list model.

`AssetContentSurfaceProfile` makes the intentional surface difference explicit. Assets Activity has no private content header, so its viewport begins at the content panel origin. Asset Browser list mode owns a fixed 24px table header and contiguous 28px asset rows. Browser folders are excluded because its Sources tree already owns folder navigation; the table projects every real catalog asset dynamically and never pads with fake `Empty Asset` rows.

`AssetThumbnailGridMetrics` is the single geometry owner for Browser Thumbnail mode. It derives the column count and card frame from the live content width, preserves an 8px token gap and padding, clamps cards to the 104-132px design range, and emits the full scrollable content extent for every visible catalog asset. Layout projection, retained pointer hit testing, native paint clipping, and scrollbar geometry all consume those item frames instead of reconstructing columns independently. This keeps the grid responsive without absolute per-card placement: a narrower surface reduces columns, a wider surface grows to six columns, and overflow remains vertical.

Browser list painting, pointer routing, and scrollbar geometry now share the table, header, preview, and row identities from `controls.rs`. Scrolling translates only row nodes. The effective row viewport starts below the fixed header and stops at the earlier of the table bottom or Preview top, so responsive layout drift cannot paint into fixed content. The generic table-cell text command intersects its own frame with the inherited viewport clip; fully excluded cells emit no command. This closes the lower-level text leak that previously allowed a scrolled row label to cross the table boundary even when the row surface was clipped correctly.

Browser thumbnail painting follows the same ownership rule: the grid surface remains fixed while card, preview, name, and info-band children receive the scroll translation and content clip. Hover styling applies only to the resolved card and info band. Native pointer routing deliberately prioritizes the visible thumbnail grid over retained collapsed table nodes, which prevents hidden List geometry from stealing scroll input after a view-mode switch.

The old retained-host-only `AssetListViewMode` and `asset_pointer/content/metrics.rs` owners were removed as a hard cutover. `AssetContentListPointerLayout` now carries the shared surface profile and the existing snapshot `AssetViewMode`, so a caller cannot silently apply Browser header geometry to Activity content.

The implementation follows Unreal Content Browser's responsibility split: the browser shell chooses a surface profile, while the asset-view row sequence owns item geometry. The primary source reference is `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp` and its `Public/SAssetView.h` contract.

Focused shared layout tests pass 4/4, dynamic table/normalization tests pass 7/7, Activity projection/scrollbar regressions pass 7/7, Browser native root routing passes 1/1, inherited table-text clipping passes 2/2, and the real-window Browser List scroll/hover/fixed-content pixel regression passes 1/1. The accepted List artifact is `docs/tests/editor/editor-window-m3-asset-browser-list-scrolled-hover-900x620.png`, 102051 bytes, SHA256 `6B035F5294EC7E0090B08BBC32A91BB9CCF494C20D7AF75F68E14A9CE2C28D7E`.

The Thumbnail slice passes the responsive-grid, dynamic-node, native-route-priority, two-dimensional pointer, projector, scrollbar, and real-window scroll isolation regressions. Its accepted 900x620 artifact is `docs/tests/editor/editor-window-m3-asset-browser-thumbnail-scrolled-hover-900x620.png`, 106508 bytes, SHA256 `7BAF3A64A6AA1BB109511E85F2775C35F97A7A63853B79EFE37248D08603D7FD`; matching repository and external target scans report zero hits. Manual review confirms a dense six-column grid, clipped scrolled rows, fixed toolbar/utility controls, and one hovered card without text or Preview overlap. Browser List and Thumbnail scrolling are complete for this slice; broader Content Browser operations and full-window visual convergence remain open.
