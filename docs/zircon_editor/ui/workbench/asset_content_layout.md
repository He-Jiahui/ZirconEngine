---
related_code:
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/profile.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/metrics.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/labels.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/text.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/asset_content.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
implementation_files:
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/profile.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/metrics.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/labels.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/text.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes/pane/entry/asset_content.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
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
  - direct zircon_editor test binary filter assets_drawer_content_scroll_repaints_inside_content_without_touching_utility --exact --nocapture
doc_type: module-detail
status: in_progress
---

# Shared asset content layout

`asset_content_layout` is the shared contract for asset rows that are both painted and pointer-addressable. Its navigational `mod.rs` delegates stable generated control ids to `controls.rs`, surface identity to `profile.rs`, geometry to `metrics.rs`, dense resource badges to `labels.rs`, and runtime-measured file-name compaction to `text.rs`. This directly addresses the repository structure findings instead of growing another mixed-responsibility root.

The metrics owner derives padding, gaps, row heights, and the Asset Browser header offset from `EditorDensityTokens` and `EditorControlTokens`; retained pointer code and Assets Activity visual projection consume the resulting `AssetContentLayoutMetrics` instead of maintaining parallel constants.

`list_height(...)` is also the single full-source extent owner. Activity layout keeps below-viewport rows at their real metric-derived positions and writes this extent onto the content panel. Paint projection and scrollbar code consume the same stable ids and extent, so neither reconstructs row heights or invents a second content list model.

`AssetContentSurfaceProfile` makes the intentional surface difference explicit. Assets Activity has no private content header, so its viewport begins at the content panel origin. Asset Browser retains its dense 28px toolbar plus two 12px vertical gaps and a 1px divider, yielding the existing 53px viewport offset. List and thumbnail folder/item heights preserve the established pointer behavior while being expressed through central row, gap, and border tokens.

The old retained-host-only `AssetListViewMode` and `asset_pointer/content/metrics.rs` owners were removed as a hard cutover. `AssetContentListPointerLayout` now carries the shared surface profile and the existing snapshot `AssetViewMode`, so a caller cannot silently apply Browser header geometry to Activity content.

The implementation follows Unreal Content Browser's responsibility split: the browser shell chooses a surface profile, while the asset-view row sequence owns item geometry. The primary source reference is `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp` and its `Public/SAssetView.h` contract.

Focused token-derivation tests pass 4/4, short full-source geometry passes 2/2, Activity projection/scrollbar tests pass 7/7, and the shared retained content scroll/click dispatch regression passes 1/1. The production native Activity route plus callback/writeback/content-only repaint regression passes 1/1 and also passes inside the full single-thread editor test run. Scoped Rustfmt, diff, hard-cutover, named-constant, and repository/external-target scans pass. The plan-output audit reports zero `editor_layout` violations while retaining 23 unrelated violations in other active plan owners. Browser state writeback is present, but Browser paint projection is intentionally not claimed by this milestone.
