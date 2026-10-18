# Zircon Editor UI: Material + Fyrox + JetBrains + Unreal

## Summary

- 第一阶段先做 UI 基础元组件与 `Material Dark` 样式系统，用 demo window 覆盖按钮、输入、菜单、tab、列表、树、滚动、面板、分割器、弹窗等基础视觉状态。
- 第二阶段做复合组件和布局能力：`Slot`/`Component` 承载子组件，基于 taffy 验证 `FlexGroup`、`HorizontalGroup`、`VerticalGroup`、`GridGroup`、`ListView`、`VirtualList`、`TreeView`。
- 第三阶段组装完整新 Workbench：Unreal 式 Window/Asset Editor 逻辑，JetBrains 式侧边抽屉和多页签外壳，Fyrox 式 Scene/Game/Hierarchy/Inspector/AssetBrowser/Console/PluginManager 面板内容。
- 现有界面作为 legacy 暂时弃用，不作为新 Workbench 的设计来源；除非明确要求删除，否则实现时不硬删未提交旧资源。

## Key Changes

- 新增可切换的 UI skin/preset 分层：
  - `Material Dark` 作为默认视觉 skin，参考 `dev/material-ui`、`dev/material-components`、`dev/slint` material widgets。
  - `FyroxPanelPreset` 定义面板内容结构和交互：World/Hierarchy、Inspector、Asset Browser、Scene Viewer、Console。
  - `JetBrainsShellPreset` 定义侧边抽屉、多页签、拖出独立窗体、拖回侧边隐藏。
  - `UnrealWindowModel` 定义功能编辑器即 window/workbench：Prefab Editor、Material Editor、UI Asset Editor、Animation Editor 等都是独立 window 单元。
- 扩展现有 workbench layout 模型，而不是写死单一页面：
  - `Window` 是顶层编辑单位。
  - `View` 是可停靠/可拖出的页签内容。
  - `Drawer` 是侧边或底部可折叠停靠区。
  - `DocumentNode`/`TabStack`/`FloatingWindow` 继续承载 split、tab、detach、attach 行为。
- 基础组件优先组件化：
  - primitive：Button、IconButton、TextField、Checkbox、Switch、Dropdown、Slider、Tabs、Menu、Tooltip、Scrollbar、Splitter、Panel、Modal。
  - layout：FlexGroup、HorizontalGroup、VerticalGroup、GridGroup、Overlay、ScrollView。
  - data/view：ListView、VirtualList、TreeView、PropertyGrid、InspectorSection。
  - shell：Drawer、ViewTab、WindowFrame、DockHost、WorkbenchShell。
- 编辑器面板全部用复合组件拼装，不直接绑定某个样式实现：
  - `Inspector` 用 PropertyGrid + Section + FieldEditor。
  - `Hierarchy` 用 TreeView + search/filter + context menu。
  - `AssetBrowser` 用 folder tree + asset grid/list + preview/metadata pane。
  - `Scene/Game` 用 ViewportHost + toolbar + gizmo controls。
  - `Console` 用 filter bar + virtual log list + severity chips。
  - `PluginManager` 用 categorized list + status/action controls。

## Implementation Plan

- Phase 1: Build `Material Dark` foundation and demo window.
  - Add semantic tokens for palette, text, action states, surfaces, dividers, focus, elevation, radius, spacing, typography, icon sizes.
  - Implement all primitive component visual states: normal, hover, pressed, selected, disabled, focused, error, warning.
  - Create one demo window that renders every primitive and verifies style consistency.
- Phase 2: Build composition and layout demos.
  - Formalize component slot behavior so composite widgets own named content slots.
  - Route flex/grid/wrap layout through taffy where supported, with Zircon-owned fallback for overlay, popup, scroll, and virtualized list semantics.
  - Create demo windows for layout stress cases, nested slots, list/tree virtualization, and inspector-style property editing.
- Phase 3: Build editor domain windows and final Workbench.
  - Create separate functional windows for Scene/Game, Prefab, Material, UI Asset, Animation, Asset Browser, and Diagnostics.
  - Add drawer/tab/window synchronization: active tab, selected drawer item, floating window focus, persisted layout, data refresh, repaint invalidation.
  - Assemble the default Workbench preset with side drawers, central Scene/Game tabs, bottom Console/Diagnostics tabs, and detachable feature windows.

## Public Interfaces

- Add or extend UI descriptors for `Window`, `View`, `Drawer`, `DockHost`, `Slot`, `Composite`, `VirtualList`, `TreeView`, and `PropertyGrid`.
- Add skin/preset identifiers such as `material_dark`, `fyrox_panel`, `jetbrains_shell`, and `unreal_window_model`.
- Keep runtime/editor boundary intact: editor UI owns authoring state and projections; runtime world data remains under runtime scene authority.

## Test Plan

- Component visual contract tests for every primitive state and token resolution.
- UI v2 asset tests for slot filling, composite expansion, selector specificity, and missing-slot diagnostics.
- Taffy layout tests for flex, grid, wrap, nested groups, min/max sizing, and fallback reporting.
- Virtualization tests for large lists/trees and scrolling range correctness.
- Workbench behavior tests for drawer selection, tab focus, detach/attach, floating window persistence, reset layout, and data refresh.
- Editor integration tests for Inspector, Hierarchy, AssetBrowser, Console, Scene/Game, PluginManager projections.
- Snapshot/fixture tests for the demo windows and final default Workbench preset.

## Assumptions

- Default visual mode is `Material Dark`.
- Runtime HUD/dialog UI is not part of this round, but the token/component system should be reusable later.
- New UI is componentized first; full Workbench assembly waits until primitive, layout, and composite demos pass.
- Existing UI assets are deprecated for this path, not deleted during planning.

## Current Progress

- Editor/runtime production UI entry assets have been hard-cut to `.v2.ui.toml` roots; staged build output no longer carries legacy `.ui.toml` under deployable UI asset roots.
- Runtime fixture loading uses `UiV2PrototypeStoreFileCache -> UiV2SurfaceBuilder -> UiSurface`, and editor runtime-preview tests consume the same runtime v2 fixture sources instead of editor-owned legacy runtime preview assets.
- Asset importer matching now reserves `.v2.ui.toml` as a dedicated v2 UI document suffix. It no longer falls through to the legacy `.ui.toml` importer; production parses v2 documents into first-class `UiV2ViewAsset`, `UiV2ComponentAsset`, or `UiV2StyleAsset` payloads and project scan/artifact restore preserves those payload variants.
- UI Asset Editor v2 authoring now keeps local `[components.*]` definitions in the authoring projection. A v2 `component` asset can open as an editable component tree, preview through a transient component root, show hierarchy/selection without a view root, and canonical-save back to v2 component schema instead of downgrading to legacy widget TOML.
- UI Asset Editor v2 component projection now rejects malformed component authoring graphs instead of silently dropping them: missing component roots and cyclic component child graphs produce explicit v2 asset errors before authoring state is built.
- UI Asset Editor sessions now expose selected-node props/state literal patching for v2 component authoring. Edits go through the existing schema-aware document edit path, so component assets canonical-save as `kind = "component"` with flat `[nodes.*].props/state` instead of falling back to legacy widget schema.
- The `asset_editor` component adapter now routes `widget.prop.*` and `widget.state.*` commits into the same session patch APIs, giving retained/v2 UI fields a generic component-parameter path instead of hard-coding one editor action per prop.
- Selected node props/state now project into structured `inspector_widget_prop_state_rows` rows and retained-host `UiAssetInspectorWidgetData.prop_state_rows` (`kind/path/value/display`), while the display strings still flow into the legacy summary stream. This gives v2 inspector controls a real data source for generic component parameters instead of scraping plain text.
- Retained UI Asset Editor conversion now renders Widget/Slot/Layout/Binding inspector fields as dynamic Material-style `InputField` nodes through the dedicated `ui_asset_detail_fields.rs` owner. Widget covers control id/text/root-class-policy plus generic props/state rows; Slot covers mount, padding, preferred size, and visible linear/overlay/grid/flow fields; Layout covers preferred size, semantic, box, scroll, virtualization, and clip fields; Binding covers id/event/route/route target/action target. The converter grows each populated section and shifts following inspector sections to avoid overlap. Generated inputs use `dispatch_kind = "commit_only"`: keypresses only update native focus and repaint the input damage region, while Enter/commit dispatches `ui_asset_detail|<instance>|<detail>|<action>|<index>` through the pane surface dispatcher into the generic UI Asset detail path. Draft edit ids stay under `ui_asset_detail_draft|...`, so visible inspector typing no longer triggers per-key asset patch/rebuild.
- Direct v2 view projection now preserves interaction metadata into retained-host nodes. `TextField`/`InputField` and related component roles no longer fall through as passive panels/mounts; `component_role`, `value_text`, binding id, edit action id, and commit action id survive `ViewTemplateNodeData -> TemplatePaneNodeData`. Asset Browser and Assets Activity `SearchEdited` are now authored as asset-dispatched v2 `TextField` controls, and the native host contract test confirms keyboard edits route through `on_asset_control_changed` instead of a dead mount node.
- Asset Browser and Assets Activity v2 toolbar buttons now route static patches through direct v2 metadata instead of visual-only selection. Filter chips, view-mode buttons, utility tabs, and search fields author canonical dispatch metadata; native pointer/text dispatch recognizes `asset` and `asset:<surface>` so Asset Browser updates the browser surface while existing Assets Activity controls keep the activity default.
- Editor asset reference analysis now handles first-class `UiV2ViewAsset`, `UiV2ComponentAsset`, and `UiV2StyleAsset` payloads, so v2 imports/resources participate in direct-reference analysis instead of hitting a non-exhaustive legacy-only match.
- Editor extension registration now rejects legacy `.ui.toml` UI template and component drawer document paths. Plugin/editor descriptor registrations must point at `.v2.ui.toml`, so stale recursive-template documents cannot re-enter Inspector or workbench projection.
- Runtime plugin UI component registration now applies the same v2 document guard: `UiComponentDescriptor` manifests must point at `.v2.ui.toml`, so plugin component metadata cannot reintroduce recursive `.ui.toml` prototypes through the runtime component catalog.
- Runtime asset importer registration now applies a registry-level legacy UI guard. Non-fixture importer descriptors cannot register `.ui.toml`, so stale plugin/package manifests cannot bring the old recursive `UiAssetDocument` importer back into production; the exact legacy matcher remains only in unit-test migration coverage.
- Plugin editor UI assets under `zircon_plugins/navigation`, `zircon_plugins/particles`, and `zircon_plugins/sound` have been converted from recursive `.ui.toml` documents into flat `.v2.ui.toml` graph documents; the old plugin UI files were removed so they are no longer package candidates.
- The default runtime asset importer no longer registers a `.ui.toml` plugin-required fallback. The `ui_document_importer` plugin typed-TOML importer and package manifest are now v2-only (`.v2.ui.toml`) and produce `UiV2ViewAsset`/`UiV2ComponentAsset`/`UiV2StyleAsset` payloads instead of routing plugin TOML through the legacy `UiAssetDocument` migration chain. The plugin's legacy serialized `.ui.json`, `.zui`, and `.uidoc` importers and their `serde_json`/`bincode` dependencies were removed, so legacy UI schema import now stays constrained to explicit migration/fixture test importers.
- Retained event effects now have a paint-only request helper. Standalone `AssetPreviewRefreshRequested` effects refresh visible previews without setting `presentation_dirty`, while asset details refresh remains presentation-domain because it mutates projected snapshot data.
- Backend `PreviewChanged` refreshes now follow the same dirty-domain split: the editor catalog snapshot is synchronized without the old `sync_asset_catalog()` presentation side effect, the host records `PAINT_ONLY`, and the native shell is redrawn without rebuilding the retained presentation tree. Structural catalog/reference/resource changes still request presentation/render domains.
- Retained `RenderChanged` has been narrowed to the render domain instead of implicitly forcing presentation. Viewport pointer execution now reports dirty domains from actual feedback: idle moves and press bookkeeping stay dirty-domain idle, camera scroll/orbit/pan can be render-only, and gizmo transform/hover or structural viewport commands still add presentation/reflection explicitly.
- Runtime v2 fixture input now consumes pointer dirty requests through the persistent `UiSurface`. `RuntimeUiManager` applies `UiPointerDispatchEffect::RequestDirty(...)` to the invoking surface node and calls `rebuild_dirty(root_size)` only when dirty domains are present, so render-only component updates avoid layout/hit-grid work and layout changes use the incremental pass without reloading v2 TOML.
- Runtime pointer and focus interaction now starts reducing component state into the retained surface: primary press/release updates `UiTreeNode::state_flags.pressed`, clears the previous pressed node when a new primary press moves target, and updates retained component state instead of relying only on emitted event reports. Hover/focus/press are mirrored into `UiSurfaceComponentStateStore`; programmatic, autofocus, pointer, keyboard navigation, and focus-clear paths keep that store in sync, detached node-pool subtrees clear their component state before reuse, and repeated same-target hover movement remains clean after the first repaint.
- v2 style projection now has a runtime pseudo-state style index instead of resolving hover/focus/press only at compile time. `UiV2SurfaceBuilder` bakes only non-pseudo rules into the retained tree baseline, stores pseudo-state selector rules in `UiV2RuntimeStyleIndex`, seeds authored hover/focus/press state into `UiSurfaceComponentStateStore`, and re-overlays matching runtime declarations on state changes. Hover/active/focus Material variants can now turn on and off from retained component state without reparsing TOML or rebuilding the UI tree; render-only visual keys stay render-dirty while text/unknown style changes escalate to the appropriate dirty domain.
- The shared `editor_material.zui` theme now defines runtime pseudo-state variants for imported editor assets instead of leaving that behavior in the standalone Material demo. Common `.material-control`, button, primary button, text button, outlined field, line edit, menu item, list item, table row, and tab classes have hover/pressed/focus/selected/disabled variants where applicable. The v2 style resolver also recursively resolves token references inside nested style arrays/tables, so table-form declarations such as `background = { color = "$material_surface_hover" }` reach render extraction as concrete colors.
- Imported Material pseudo-state rendering is now guarded at the render-extract boundary. The Welcome startup button test drives real pointer hover and primary press through `UiSurface`, rebuilds dirty state, and asserts the render command background/border colors switch to hover, pressed, and focus variants while layout and arranged-tree rebuild flags remain false.
- Runtime pseudo-state re-resolution now maintains selector paths on an explicit traversal stack instead of rebuilding a parent chain for every affected node. Deep descendant selectors are guarded with a 512-level fixture where a parent hover rule restyles the leaf label and marks only render dirty.
- Programmatic property mutation now drives the same runtime pseudo-state style path. Accepted `checked`, `selected`, `disabled`, `enabled`, and `pressed` mutations re-apply the v2 runtime style index to the affected subtree; `pressed` mutations also synchronize `UiSurfaceComponentStateStore`. A v2 Toggle fixture verifies Rust-side `checked` and `enabled = false` patches immediately resolve `:checked` and `:disabled` Material colors without pointer/focus events or layout dirtiness.
- v2 runtime checked/disabled pseudo-state now has a retained surface source of truth. Authored `checked`, `enabled`, and `disabled` values seed `UiTreeNode::state_flags`, runtime selector matching ignores stale TOML props for those states, and primary-click default actions for `Toggle`/`Checkbox`/`Switch`/`ToggleButton` flip `checked` through `UiSurface::mutate_property(...)`. Bound `onChange` handlers receive a `ValueChanged { property = "checked" }` envelope and the affected Material `:checked` style rebuilds render-only.
- v2 expanded component interaction now follows the same retained path. `Group`, `Foldout`, `InspectorSection`, `TreeRow`, and `TreeView` primary-click default actions flip `expanded`, emit bound `ToggleExpanded` envelopes on `onToggle`, and drive `:expanded` Material variants render-only. The runtime style baseline now strips retained state keys before overlaying pseudo-state declarations, so authored `expanded = true` no longer reappears from static TOML after the user collapses the component.
- v2 popup selection interaction now uses typed default events. `Dropdown`/`ComboBox`/`EnumField`/`FlagsField`/`SearchSelect`/`ContextActionMenu`/`Popup` flip `popup_open` on primary click, re-resolve `:open`/`:popup_open` Material variants from retained state, and route only the matching `OpenPopup` or `ClosePopup` binding instead of broadcasting a generic click commit to both open and close routes.
- v2 range controls now have a retained default value interaction. `RangeField`/`Slider` map primary-click position to `min..max`, apply authored `step`, patch `value`, and emit `ValueChanged { property = "value" }`; press starts pointer capture and emits `BeginDrag`, captured move updates continue outside the hit frame with `DragDelta`, release emits `EndDrag`, focused controls consume left/right/up/down as step changes, and accessibility `Increment`/`Decrement` on slider roles uses the same retained step path. Numeric `value` changes on Range/Slider/Progress are classified render-only so track-fill updates avoid layout and hit-grid rebuilds.
- Runtime surface default interactions have been split out of `surface.rs` into `surface/default_interactions.rs`. The main surface file stays focused on dispatch/rebuild/render orchestration, while Toggle/Foldout/Popup/Range default state mutation and typed component event filtering now have a dedicated owner for the next component-interaction productization pass.
- Runtime surface rebuild orchestration has been split into `surface/rebuild.rs`. `UiSurfaceRebuildReport`, dirty flag aggregation, full/incremental rebuild, layout recompute, render extract cache refresh, and rebuild timing/accounting now have a dedicated owner, leaving `surface.rs` focused on the retained surface state and input/focus/pointer orchestration.
- The runtime manager render-dirty guard now asserts the exact de-duplicated dirty source set instead of assuming only the explicit dispatcher target can be dirty. This preserves the render-only/no-layout/no-hit-grid contract while allowing retained hover component state to contribute its own render dirty node on first pointer entry.
- UI Asset Editor `OpenAsset` production routing is now v2-only. `.v2.ui.toml` opens through schema-aware session construction and `from_v2_source(...)`; legacy `.ui.toml` no longer creates an `editor.ui_asset` view from the asset-open event. Restore/reload/clean external-change refresh use the same schema-aware helper so v2 authoring sessions do not fall back to the old parser after opening.
- Still open: broad dirty-domain propagation across every editor/runtime surface, productized visual component authoring controls for import management and slot picking, and deleting the remaining legacy schema support outside migration and fixture tests.
