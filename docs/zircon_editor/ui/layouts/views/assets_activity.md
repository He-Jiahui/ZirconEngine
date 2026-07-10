---
related_code:
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_nodes.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/responsive_layout.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/tests/ui/assets_activity/bootstrap_assets.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/visual_screenshot/assets_drawer.rs
implementation_files:
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_nodes.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/content_layout.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/responsive_layout.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Public/SAssetView.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Docking/STabDrawer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/STabDrawer.cpp
  - docs/ui-and-layout/ai-workbench-style/ai-blend-space-layout.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/zircon_editor/editor_layout/16-relative-layout-and-resolution-adaptation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --no-default-features asset_content_layout -- --nocapture
  - cargo test -p zircon_editor --lib --no-default-features assets_activity_content -- --nocapture
  - cargo test -p zircon_editor --lib --no-default-features assets_activity_regular_drawer -- --nocapture
  - direct zircon_editor test binary filter activity_asset_content_projector_ --nocapture
  - direct zircon_editor test binary filter activity_asset_content_scrollbar_ --nocapture
  - direct zircon_editor test binary filter assets_drawer_content_scroll_repaints_inside_content_without_touching_utility --exact --nocapture
  - direct zircon_editor test binary filter assets_activity --nocapture
  - direct zircon_editor test binary capture_m3_gui_acceptance_visual_artifacts --ignored --nocapture
doc_type: module-detail
status: in_progress
---

# Assets Activity responsive layout

`assets_activity.rs` projects the authored Assets drawer view and applies snapshot text/state. `responsive_layout.rs` owns the compact composition used when the resolved pane width is at or below the central `compact_left_drawer_max_width` density token. This keeps content semantics and responsive geometry separate and leaves the authored wide layout unchanged.

The compact toolbar is two relative rows. The first row gives the runtime-measured Browser button its complete label and assigns the remaining inner width to Search. The second row always preserves List, Thumb, and All, then adds the selected kind filter if it fits. Button widths use the retained runtime text measurement interface with the central Workbench body size and density padding; non-selected filters are removed as complete controls rather than clipped into partial text.

Selection is represented by the existing selected/focused surface and text-tone states. Labels no longer append `• Active`, which previously made 58-70px view buttons and short filter chips overflow despite already having a visual selected state. The Preview control-id projection is also corrected to `AssetsActivityPreviewTabButton`, so the authored button receives the actual active state.

At compact width the authored 248px folder tree is folded out and the asset content panel receives the full drawer width. Vertical space is recomputed from two toolbar rows, a token-derived four-row utility budget, and the remaining content height. Preview uses one visual well plus three readable text lines; References uses one full-width summary column and folds the secondary Used By column. Metadata/Plugins remain hidden in this two-tab activity view instead of overlapping the active panel.

`content_nodes.rs` now projects real `visible_folders` and `visible_assets` into compact list-row primitives instead of leaving the content surface blank. Each item is composed from a selected row surface, a dense resource-type badge, a runtime-token body label, and a caption revision/diagnostic marker. Empty workspaces receive the explicit `No assets in this folder` state. `content_layout.rs` arranges the same folder-first/item-second sequence used by the retained pointer bridge, retains every metric-derived row in full source geometry, and compacts file-like names with the existing runtime text measurement path while preserving their extension tail.

Compact vertical budgeting now reserves one complete token-derived content row before assigning the remaining height to Preview/References. This matters at the 640x420 shell tier, where the earlier fixed four-row utility budget left only 44px and caused the first 38px row plus padding to be folded out. Selection belongs only to the row surface; badge/name/meta children keep semantic text tones and do not create nested focus-colored boxes.

Visual and pointer row geometry now come from `ui/workbench/asset_content_layout`. Activity starts at its content-panel origin; Browser retains its separate 53px internal header offset. This removes the prior visual/pointer mismatch where Activity inherited a Browser-only blank header region.

Activity paint projection is transient and pane-scoped. The shared template-node pipeline accepts an optional owned-node transform, while `ActivityAssetContentProjector` is selected only for the Assets pane. It translates generated content rows by stored scroll, intersects their effective clip with `AssetsActivityContentPanel`, suppresses rows outside that viewport, and applies hover only to the single folder-first shared row index. The source model is not mutated. Empty-state text deliberately ignores stale scroll, while unrelated Preview/References controls pass through unchanged.

The content panel stores the shared metric-derived list extent in `value_number`. The native pane renderer consumes that extent and the panel viewport through the existing Starship vertical scrollbar owner, so overflow produces the same 8px track/4px thumb family used elsewhere; fitting and empty content produce no scrollbar. Browser interaction state is now retained at the host boundary, but Browser paint projection remains a separate future slice.

Current evidence:

- `assets_activity_regular_drawer_compacts_toolbar_and_reclaims_content_width` was RED on `Thumb • Active`, then passes while locking all visible controls inside 226px, a zero-width fixed tree, at least 210px content width, at least 120px content height, and active Preview state.
- `assets_activity_regular_drawer_references_use_one_readable_summary_column` locks one full-width compact References column and folds the secondary column.
- The latest Assets Activity bootstrap group passes 8/8, including populated geometry, explicit empty state, compact toolbar/utility composition, the 226x224 short-drawer row reserve, and a positive-size source row below the short viewport. Shared layout/text passes 4/4 and retained content scroll/click dispatch passes 1/1.
- Focused interaction-state tests pass 2/2, generic template-node transform tests pass 2/2, Activity projector tests pass 3/3, and Activity content scrollbar tests pass 4/4. The earlier setter-level content repaint/Preview-integrity test passed 1/1; review then correctly rejected it as insufficient because the native pane router never constructed `AssetContent`.
- The dedicated responsive capture passes 1/1 and writes only to `docs/tests/editor`: 640x420 is 53563 bytes/SHA256 `C40767672562795CF8590E31B27C262F2ECC9BB5BFC2750ED126F8215E65DEDE`; 900x620 is 71467 bytes/SHA256 `1AF3BA80CB924B1351E89DA3A5929EB41416743B045CE3C00BEF5FBF16E54C60`; 1260x780 is 99291 bytes/SHA256 `7BB6C59632A64B42B089BA9D23E2C64D09B1ACCD8A47BAD1DA2B453C9672BBFA`.
- Manual review confirms complete Search/Browser/List/Thumb/All labels, 1/3/6 visible asset rows across the three tiers, a single row-owned selected surface, and no content/Preview overlap. Repository `target` and external Cargo target scans contain zero matching PNG outputs.
- The accepted native-route capture at `docs/tests/editor/editor-window-m3-assets-drawer-scrolled-hover-900x620.png` is 74069 bytes/SHA256 `45359A656E5EEBADA47685E526C790C965BD2167724C3C85053A17056DC23533`. The fixture dispatches real native wheel/move events through the shared pointer bridge callback and host writeback; manual review confirms later rows (`unit.zshader`, `player_start.prefab`), one hovered visible row, a moved shared scrollbar thumb, and unchanged Preview utility pixels.
- Review-driven routing RED proved the old native wheel path returned the whole Assets body damage frame and never constructed `PanePointerTarget::AssetContent`. `native_pointer/routing/panes/pane/entry/asset_content.rs` now resolves the real content panel before template fallback. A second RED exposed `WorkbenchScenePlayerItem` stealing pointer move before pane routing; the final move order is native menu, Workbench popup rows, pane targets, then base Workbench template. The native wheel/move regression now passes 1/1 and requires the hovered later row to change by more than 40 interior pixels while every other visible row and Preview remain byte-identical.
- The final current-tree official validator passes its locked `zircon_editor` build. Its default-parallel test process exits 101 after creating thousands of threads without emitting a test result, so a full single-thread binary run is recorded separately rather than misreported as a product failure. The WSL `--locked --offline` editor library check passes; focused state, transform, projector, scrollbar, popup-priority, native-route, and capture groups are green.

Overlay drawer interaction and the wider Asset Browser window remain separate slices. This module does not claim complete editor-window visual parity.
