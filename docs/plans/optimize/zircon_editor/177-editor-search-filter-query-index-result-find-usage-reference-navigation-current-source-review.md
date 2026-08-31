---
related_code:
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/assets_activity.zui
  - zircon_editor/assets/ui/editor/hierarchy.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_command_palette.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/diagnostics/workbench_extension_runtime_diagnostics_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/document/mod.rs
  - zircon_editor/src/core/document/scene_route.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/ui/workbench/snapshot/asset
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/dependency_index
  - zircon_editor/src/ui/layouts/views/asset_reference_rows.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/hierarchy_projection.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/asset_reference.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_search.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/ui_diagnostics/observability.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/ui_diagnostics/observability.rs
  - zircon_runtime/src/asset/registry
tests:
  - zircon_editor/src/core/commands/palette/performance_tests.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/host/retained_asset_pointer.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch
  - zircon_editor/src/tests/host/retained_hierarchy_template_body.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/asset_browser_controls_visual_screenshot.rs
  - zircon_editor/src/tests/ui/asset_browser/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/hierarchy/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/dependency_index/tests.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/tests/reference_lists.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session_tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/52-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/51-runtime-asset-registry-index-persistence-rebuild-incremental-query-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/TextFilterExpressionEvaluator.h
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Public/AssetTextFilter.h
  - dev/UnrealEngine/Engine/Source/Editor/Kismet/Public/FindInBlueprintManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Public/AssetRegistry/IAssetRegistry.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/SceneOutlinerFilters.h
  - dev/godot/editor/gui/editor_quick_open_dialog.h
  - dev/godot/editor/gui/editor_quick_open_dialog.cpp
  - dev/godot/editor/script/find_in_files.h
  - dev/godot/editor/script/find_in_files.cpp
  - dev/fyrox/editor/src/asset/dependency.rs
  - dev/fyrox/fyrox-ui/src/searchbar.rs
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.SearchFilter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Utilities/UIElementSearchFilter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightingSearchColumnProviders.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/SearchWindowProvider.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/56-editor-search-filter-query-index-result-find-usage-reference-navigation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/129-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 177 · Editor Search / Filter / Query Index / Result / Find Usage / Reference Navigation 当前源码复核

## 1. 结论与状态

Editor56/129要求的统一工程级搜索系统仍未出现。生产源码精确符号审计没有找到`EditorSearchService`、`SearchProviderDescriptor`、`CompiledSearchQuery`、`SearchOperationId`、`SearchScopeId`、`SearchIndexGeneration`、`SearchResultRecord`、`SearchResultPage`、`SearchCompleteness`、`ProviderSearchReceipt`或`SearchNavigationReceipt`。当前仍是多个局部实现并存，且能力级别差异很大：Command Palette和Scene Picker已有有界窗口与generation基础；Asset Browser和Hierarchy是真实但未分页的局部过滤；Blend Space只处理三个模板常量；Workbench Scene、Effect、Tags、Generated Bottom与多项diagnostics filter仍无真实结果consumer；Icon Find Usage和Gameplay Tags Reference Scan继续发布固定或静态事实。

当前源码有两项值得保留的新进展。第一，`EditorCommandPaletteCatalogGeneration`已经使用倒排byte posting、rarest posting、稳定fuzzy score、bounded `BinaryHeap` top-K、offset/limit窗口和查询metrics。第二，Scene Picker已经复用Command Palette产品面，持有project-bound `ScenePickerTicket`、catalog generation、稳定排序去重的Scene列表、12行窗口，并在提交时按当前query和window重新校验command。这些证明Zircon具备实现局部有界搜索与stale拒绝的能力，但它们仍是Editor08/Scene Picker的私有合同，不能关闭跨域Search Runtime、provider registry、completeness、cancel、Find Usage或typed navigation差距。

Asset链也有真实进展：`AssetWorkspaceItemGeneration`以64行`Arc` chunk共享不可变结果并支持已存在条目的chunk级替换；Runtime `AssetRegistryIndex`继续提供持久化、增量更新、UUID/path依赖与稳定referencer查询。不过Asset Browser仍对当前目录全量扫描并一次收集全部命中；新增/删除/移动导致可见性变化时仍退回整代重建。References/Used By仍全量克隆四个UI节点/结果，DTO没有source field path或edge kind，unknown target不可修复，Locate Selected仍只打开Assets view。

本轮不新增canonical finding，继续刷新Editor56的 **2项P0、44项P1、12项P2与38个资格门**。当前判定为：

| 等级 | Open | Partial | Closed/Pass |
|---|---:|---:|---:|
| P0 | 2 | 0 | 0 |
| P1 | 31 | 13 | 0 |
| P2 | 10 | 2 | 0 |
| Gate | 27 Fail | 11 Partial | 0 Pass |

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据 |
|---|---:|---|
| Zircon Editor/Runtime聚焦源码、资产与测试 | **218 / 42,682 / 39,242 / 1,664,115 / 363 / 15** | 当前磁盘产品入口、query/projection、palette/picker、registry、reference UI/navigation与聚焦测试；fingerprint `83ffdd596e94bab77c08c7252ee5cd3143bfd8951d1951fcd1acef6e8fdcc47c` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **17 / 12,215 / 10,502 / 483,650 / 53 / 0** | compiled filter、registry、async find、cancel/progress、provider/result/navigation参考；fingerprint `53e58e1d593219ca90f76231deadf1b9e675ba22b92c7d6febc39f2a660fd382` |

统计按normalized relative path的ordinal顺序，将每个`path + NUL + raw bytes + NUL`串联后计算SHA-256；行数按CRLF/LF/CR split并保留终止空行。`tests/ignored`是词法计数，不是执行receipt。读取时Git HEAD为`ea35974cdf64068f6789010451d20bbf69e0a29d`；共享工作树存在大量在途修改，因此本报告以当前磁盘fingerprint而不是HEAD tree作为证据锚点。

### 2.2 产品链真实性矩阵

| 产品面 | 当前事实 | 判定 |
|---|---|---|
| Command Palette | immutable catalog generation、posting候选、bounded top-K、stable rank、offset/limit、metrics | 可保留的局部工程底座；owner仍是Editor08，不是统一Search Runtime |
| Scene Picker | project Scene scope、ticket/session fence、catalog generation、稳定去重、12行窗口、提交时重验当前query/window | 可保留的产品vertical；仍两次线性扫描、ASCII substring、无cancel/provider/completeness |
| Asset Browser/Activity | 当前目录，name/file/locator ASCII子串，单kind；全catalog扫描后一次materialize；64行共享chunk与局部replacement | 真实可用但不是全项目索引搜索 |
| Hierarchy pane | O(N)名称过滤、Unicode fallback、保留祖先、5k flat/deep fixture与阶段counter | 正确性基础存在；active filter仍把稀疏更新升级为full authoritative reflow |
| Workbench Scene tree | SearchField无events；Filter发`workbench.hierarchy.open_filter`，binding登记`workbench.hierarchy.filter.open` | 可见但不可执行且route漂移 |
| Gameplay Effect/Tags | Change/Submit被generic field action接受；没有query owner/result generation/feedback；Tags把提示文字写入`value` | 静默no-op |
| Generated Bottom | Filter Change/Submit只把status改为`Generated bottom filter updated` | 有路由，无筛选 |
| Runtime Diagnostics | Filter字段有Change/Submit绑定，field action没有结果consumer；Performance/Telemetry/Console filter只写固定feedback | 参数外观与业务结果脱节 |
| Blend Space | 仅匹配`BS_Idle_Run`、`BS_Strafe_Grid`、`BS_Sprint_Lean` | fixture级过滤 |
| Icon Find Usage | 固定发布`14 references` | 伪造operation/result count |
| Gameplay Tags Reference Scan | 只选择`module-bottom-gameplay-tags:reference-scan`静态route | 没有scanner/result set |
| Asset References/Used By | Runtime registry直接边投影、稳定排序；全部materialize并克隆四节点/结果；known UUID可导航 | 局部依赖浏览器，不是跨域Find Usage |
| Locate Selected | 只`open_view("editor.assets")`并请求preview refresh | 没有clear/reveal/scroll/focus/select receipt |

### 2.3 当前源码新发现

1. `asset_workspace_state.rs`生产区现在有两次精确的`self.search_query.to_ascii_lowercase()`调用，分别服务full build和catalog replacement patch；同文件测试`asset_snapshot_normalizes_search_once_and_streams_parent_paths`仍通过`include_str!`断言生产区精确命中次数为1。未运行测试，但按当前源码该文本断言会拒绝当前实现。这是P2-08所描述的源码文本守卫脆弱性，也是当前基线不可直接宣称green的具体证据。
2. `AssetWorkspaceItemGeneration::replace_existing_items`只接受locator不变的已存在条目替换；新增、删除、move或过滤可见性改变仍不能通过该path收敛。它减少投影clone，不等于增量search index。
3. Scene Picker提交会重新计算当前query window并拒绝隐藏/陈旧command，Core document route还会拒绝stale project session。这可作为未来`SearchNavigationReceipt`的局部先例，但Asset reference点击仍没有result/index generation或typed disposition。
4. Hierarchy新增parse/source/name/ancestor/visible counters，Command Palette也暴露query metrics；两者没有共同trace schema，不能合并为统一搜索SLO证据。
5. Runtime registry的corruption rebuild、incremental candidate swap、stable dependency/referencer query继续成立；Editor Search层仍没有消费它的provider receipt、scope、index generation或completeness。

## 3. P0：能力真实性

### E56-P0-01 · Open · Workbench Scene、Effect、Tags过滤入口仍无真实查询结果链

Workbench Scene继续无输入事件并存在两个filter route拼写；Effect/Tags继续只进入generic field bridge。相同缺陷类别还出现在Generated Bottom和Runtime Diagnostics字段：事件被登记并可返回成功，但没有改变候选集、row generation、result count或completeness。M0必须为每个可见入口安装真实provider/session，或以明确Unavailable状态禁用；不得把route存在、field value变化或status文字更新作为搜索完成证据。

### E56-P0-02 · Open · Icon Find Usage与Gameplay Tags Reference Scan仍发布虚假或静态结果

Icon Library仍固定输出`Icon usage search queued`和`14 references`，Tag Reference Scan仍只切换静态bottom route。两者没有OperationId、scanner、provider、catalog/index generation、page、cancel、failure或terminal receipt。固定事实必须硬切删除，未安装能力时只能显示Unavailable/Indexing/Failed等真实状态。

## 4. P1：44项工程差距当前判定

### 4.1 Search Runtime、query与provider合同

| ID | 状态 | 当前证据与必须收敛的合同 |
|---|---|---|
| E56-P1-01 | Open | 无唯一`EditorSearchService`、project/session lifecycle或多窗口共享owner |
| E56-P1-02 | Open | 仍无typed AST、span diagnostic、schema/version与compiled query |
| E56-P1-03 | Open | 无provider registry、descriptor、capability、priority与unregister generation |
| E56-P1-04 | Partial | Scene Picker ticket绑定project root/session，局部结果携带catalog generation；没有公共Project/World/Document/User `SearchScopeId` |
| E56-P1-05 | Partial | `WorkbenchCommandPaletteOpenState`是局部结果投影；没有provider provenance、qualified target、score、match ranges与actions的统一record |
| E56-P1-06 | Partial | Command Palette与Scene Picker有offset/limit窗口；没有cursor、byte/time预算、continuation、total estimate语义或terminal page receipt |
| E56-P1-07 | Open | 无搜索OperationId、cancel、deadline、progress、terminal disposition与late-page拒绝 |
| E56-P1-08 | Partial | Runtime51 registry有持久化/增量/损坏重建；Editor跨域search index仍无schema、migration、generation与lifecycle |
| E56-P1-09 | Open | UI仍不区分Complete/Partial/Indexing/Stale/ProviderFailed/Cancelled |
| E56-P1-10 | Partial | Command Palette对candidate/result数量有界；没有跨provider并发、CPU、内存、fan-out、result bytes与UI apply admission |

### 4.2 Asset Browser搜索

| ID | 状态 | 当前证据与必须收敛的合同 |
|---|---|---|
| E56-P1-11 | Open | 仍锁在当前目录直属项；必须显式支持current/recursive/project/package/plugin scope |
| E56-P1-12 | Open | 仍只匹配name/file/locator；缺type/tag/source/status/owner/dependency/modified/platform字段 |
| E56-P1-13 | Open | 仍是单一`ResourceKind`；缺多选include/exclude与provider facet schema |
| E56-P1-14 | Open | 仍用ASCII lowercase；缺跨Asset/Hierarchy/Module统一Unicode normalization/case-fold政策 |
| E56-P1-15 | Partial | catalog change可对可见性不变的已有行做chunk replacement；初始/查询/可见性变化仍全catalog扫描 |
| E56-P1-16 | Open | Asset命中仍全部collect；没有bounded top-K、page/virtualization或early stop |
| E56-P1-17 | Partial | Command Palette已有稳定exact/subsequence ranking与tie-break先例；Asset仍只有布尔substring且无highlight |
| E56-P1-18 | Open | 无saved query、recent history、project/user scope preset |
| E56-P1-19 | Open | 资产类型和插件仍不能注册搜索字段/enrichment/权限 |
| E56-P1-20 | Partial | Runtime registry可增量替换/删除，Asset item generation可局部替换；无search segment delta、tombstone与rename/move continuity receipt |
| E56-P1-21 | Open | Change路径无input generation、debounce/coalescing、cancel或late-result fence |
| E56-P1-22 | Partial | Activity/Explorer共享64行`Arc` chunks并复用未变chunk；surface仍持有完整命中generation而非page cursor |

### 4.3 Hierarchy与Workbench局部过滤

| ID | 状态 | 当前证据与必须收敛的合同 |
|---|---|---|
| E56-P1-23 | Open | query仍是Retained host字符串；无Scene search request/result DTO与generation恢复 |
| E56-P1-24 | Open | 仍只按名称；缺type/component/tag/visibility/lock/layer/owner/validation facet |
| E56-P1-25 | Open | active filter继续禁用稀疏fragment并触发authoritative reflow |
| E56-P1-26 | Open | 祖先虽保留，但row没有direct/ancestor/forced-selection match reason |
| E56-P1-27 | Open | 无flat result、qualified scene path与稳定导航 |
| E56-P1-28 | Open | 两个Filter action ID仍漂移，且无真实facet menu/Unavailable合同 |
| E56-P1-29 | Open | Workbench Scene和Hierarchy pane仍不是同一query session |
| E56-P1-30 | Open | 无result count、partial/error/empty区分、clear与locate summary |
| E56-P1-31 | Open | Blend Space仍是三个常量行，未接Animation asset provider |
| E56-P1-32 | Open | Tags仍把`Search tags...`放在`value`而非placeholder |
| E56-P1-33 | Open | Effect/Tags仍无query generation、provider result owner与typed target |
| E56-P1-34 | Partial | Scene Picker复用Command Palette产品控件，证明widget adapter可行；尚无公共search session/provider adapter |

### 4.4 Reference graph、Find Usage与导航

| ID | 状态 | 当前证据与必须收敛的合同 |
|---|---|---|
| E56-P1-35 | Partial | Runtime registry是asset direct/reverse edge权威并稳定排序；Editor仍无统一asset search provider、scope、generation与receipt |
| E56-P1-36 | Partial | `EditorAssetReferenceRecord`保留UUID/locator/display/kind/known；缺source object/property path、edge kind与resolve diagnostic |
| E56-P1-37 | Open | References/Used By仍按结果克隆四个节点；无virtual row pool/page apply budget |
| E56-P1-38 | Partial | unknown target有`known_project_asset=false`并可见；仍不可定位source或repair/rebind/remove |
| E56-P1-39 | Open | `navigate_to_asset`找不到UUID仍返回false；无typed stale/not-found/permission/provider-error receipt |
| E56-P1-40 | Open | Locate Selected仍只打开Assets view并刷新preview |
| E56-P1-41 | Open | 目标仍只有asset UUID；无object/document/symbol/property/line/subobject/reference-field地址 |
| E56-P1-42 | Open | Scene Picker有局部stale校验，但Reference点击仍不校验result/index generation |
| E56-P1-43 | Open | 无Asset/Scene/Script/Tag/UI/Material/Plugin跨域Find Usage聚合 |
| E56-P1-44 | Open | 无UI/command/API共用的typed query、page stream、cancel与terminal DTO |

## 5. P2：实现质量与维护债务

| ID | 状态 | 当前证据与收敛方向 |
|---|---|---|
| E56-P2-01 | Open | route/control/property继续大量裸字符串；改为生成descriptor与typed action key |
| E56-P2-02 | Open | Asset/Hierarchy/Blend/Scene Picker各有case-insensitive matcher；共享compiled text primitive并保留小集合fast path |
| E56-P2-03 | Open | Tags placeholder/value混用仍存在；schema validator必须拒绝 |
| E56-P2-04 | Partial | Runtime reverse query稳定排序；reference UI仍用数组index生成row ID，缺stable result key |
| E56-P2-05 | Open | reference UUID继续藏在node `value_text`；改用typed payload/target handle |
| E56-P2-06 | Open | dynamic row ID继续依赖数组index；需要stable key与recycled binding generation |
| E56-P2-07 | Partial | Hierarchy和Command Palette有局部counter；缺统一parse/candidate/provider/page/apply/navigate trace |
| E56-P2-08 | Open | `include_str!`文本测试当前期望一次lowercase而生产区已有两次；改为行为benchmark与allocation/candidate counter |
| E56-P2-09 | Open | 选择集仍含ignored视觉测试；required visual lane需可重放receipt |
| E56-P2-10 | Open | 多数模板测试只证明route/binding存在；必须断言真实result generation变化 |
| E56-P2-11 | Open | dependency generation仍用`saturating_add`；需要exhaustion/epoch rollover合同 |
| E56-P2-12 | Open | 多处feedback继续嵌入固定资产名、计数、阈值与结果；只能呈现真实operation DTO |

## 6. 参考引擎约束翻译

| 参考 | 必须吸收的工程约束 | Zircon不得照抄的部分 |
|---|---|---|
| Unreal Text Filter / Asset Text Filter | compiled expression、字段context、parse diagnostic、single-token fast path、worker-safe compiled filter、saved query | C++宏、Content Browser历史耦合 |
| Unreal Find in Blueprint / Asset Registry | versioned cache/index、async begin/continue/end、progress/cancel/failure list、dependency/referencer与scan lifecycle | Blueprint专属单例；Runtime51已有registry权威 |
| Unreal Scene Outliner | composable filter、filter changed event、visibility/interactivity分离 | Actor/Outliner具体类型层次 |
| Godot Quick Open | fuzzy、base type、history、max result、stable selection、list/grid与accessibility | 主线程全量遍历不能成为百万资产资格证据 |
| Godot Find in Files | root/include/exclude、case/whole-word、progress/stop、多result set | replace语义不能强加给所有domain |
| Fyrox | 可复用SearchBar与dependency browser的局部产品下界 | 局部scan不是统一index/runtime |
| Bevy | typed asset path/source/label与cached query-state思想 | ECS QueryState不是Editor搜索产品实现 |
| Unity Graphics | hierarchical filter、throttle/highlight、typed search columns、Shader Graph provider/tree/action | Graphics仓只是Unity搜索生态consumer，不代表完整Unity Search |

## 7. 目标架构与硬切顺序

```text
SearchQuerySource
  -> parser / typed AST / diagnostic / compiled query
  -> SearchRequest { scope, provider set, request generation, budget, deadline }
  -> EditorSearchService
       -> provider registry + index generation
       -> bounded candidate execution + ranked merge + cancel
       -> SearchResultPage { cursor, records, completeness, provider receipts }
  -> virtual SearchPresentation
  -> SearchNavigationRequest { stable result key, qualified target, observed generation }
  -> SearchNavigationReceipt { disposition, reveal/focus/select effects }
```

1. **M0 Capability truth**：删除固定`14 references`和静态scan成功暗示；修复或禁用Scene/Effect/Tags/Generated/Diagnostics假入口；建立完整入口inventory。
2. **M1 Kernel**：实现typed query、provider descriptor、request/index generation、budget/cancel/deadline、paged result、completeness、qualified target与navigation receipt。
3. **M2 Asset/Hierarchy迁移**：保持现有UI，先把matcher接到provider；Asset接catalog delta/facet/page，Hierarchy接filtered-tree delta和match reason；Workbench Scene绑定同一session。
4. **M3 Find Usage**：以Runtime51 direct edge为首个provider，补source field path、missing/stale target、repair、真实locate；再接Tag、Icon、Scene、Script、UI、Material。
5. **M4 生态**：plugin register/unregister generation、failure isolation、public widget adapter、history/preset、keyboard/a11y与automation DTO。
6. **M5 资格**：100k/1m assets、百万edges、100k hierarchy、Unicode/compound/fuzzy、query storm/cancel/project switch/plugin unload/corruption/OOM与同负载跨引擎基准。

## 8. 38项资格门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| E56-G01 | Fail | 多个可见入口仍无provider或明确Unavailable |
| E56-G02 | Fail | Effect/Tags query不产生result generation |
| E56-G03 | Fail | Workbench Scene与Hierarchy仍非同一session/action |
| E56-G04 | Fail | Icon Usage继续固定14 |
| E56-G05 | Fail | Tag Reference Scan无operation/progress/cancel/page/failure |
| E56-G06 | Fail | 无AND/OR/NOT/group/field/quote parser |
| E56-G07 | Fail | 无公共compiled query worker合同 |
| E56-G08 | Partial | Scene Picker有project ticket与catalog generation；公共request/page/result scope仍缺失 |
| E56-G09 | Fail | 无provider unregister late-page/leak fence |
| E56-G10 | Fail | 无搜索cancel terminal与late-page rejection |
| E56-G11 | Partial | Palette/Scene Picker有row window；Asset/Reference仍全量materialize，且无byte/time page预算 |
| E56-G12 | Partial | Palette/Scene Picker/registry局部稳定排序；跨provider merge未定义 |
| E56-G13 | Fail | Asset没有current/recursive/project三scope |
| E56-G14 | Fail | Asset没有multi-kind include/exclude与typed facets |
| E56-G15 | Fail | 各模块Unicode政策不一致 |
| E56-G16 | Partial | Palette有exact/subsequence score测试；无公共score与match-range golden |
| E56-G17 | Partial | Registry与item chunk有局部delta；search index segment/rename/move continuity未闭合 |
| E56-G18 | Partial | Runtime registry有persist/corrupt rebuild；Editor search index schema/receipt仍缺失 |
| E56-G19 | Fail | 无100k Asset warm-query批准预算receipt |
| E56-G20 | Fail | 无1m/query-storm UI scan/clone资格 |
| E56-G21 | Partial | Hierarchy保留祖先；没有direct/ancestor reason |
| E56-G22 | Fail | active filter仍触发full authoritative reflow |
| E56-G23 | Partial | 5k flat/deep通过源码fixture；无100k/cycle-corruption资格 |
| E56-G24 | Fail | Blend Space仍是三个模板常量 |
| E56-G25 | Fail | Tags placeholder/value仍错误 |
| E56-G26 | Fail | Reference结果无source object/field path/edge kind |
| E56-G27 | Fail | missing reference无source定位与repair动作 |
| E56-G28 | Partial | Scene Picker拒绝stale query/window/session；Asset reference仍静默false且无typed receipt |
| E56-G29 | Fail | Locate Selected无clear/reveal/scroll/focus/select receipt |
| E56-G30 | Fail | Reference rows无virtual page，仍线性克隆节点 |
| E56-G31 | Fail | 无六域Find Usage merge |
| E56-G32 | Fail | 无provider partial-failure/completeness |
| E56-G33 | Fail | 无UI/automation共用DTO |
| E56-G34 | Partial | Palette/Picker有局部focus/a11y/window；无公共result count/progress/cancel产品测试 |
| E56-G35 | Partial | Scene Picker有project-session fence；active query的window/plugin/project统一quiesce仍缺失 |
| E56-G36 | Fail | 无parser/cursor/provider page/stale target fuzz |
| E56-G37 | Fail | 无统一cold/warm TTFP/terminal/CPU/RSS/UI apply benchmark |
| E56-G38 | Fail | 没有同硬件同语料对比receipt，不得宣称优于Unreal |

## 9. Owner边界与实施前置

- Editor177只刷新Editor56/129的跨Editor query/provider/result/navigation和入口真实性，不复制Editor04资产registry/import/reference extractor，也不接管Runtime51持久化asset index。
- Command Palette由Editor08拥有；Scene Picker是其产品consumer。可抽取公共primitive，但不得用一次局部复用宣称跨域Search Runtime已经存在。
- Scene snapshot/identity由Editor03与Runtime owner维护；search provider只消费generation-qualified只读snapshot。
- Gameplay、UI、Diagnostics等domain报告拥有“结果是什么”和业务动作；Editor177拥有公共operation/page/completeness/navigation合同以及禁止伪结果的产品规则。
- Tooling继续按用户要求排除；后续实现应在Rust产品crate中完成，不新增脚本旁路。

本轮只进行源码与本地参考审查、文档写入和静态一致性检查，不运行Cargo、Editor GUI、视觉测试、索引构建、race/fault/scale/soak或跨引擎性能基准。由于已发现`asset_workspace_state.rs`源码文本断言与生产区精确命中次数不一致，任何实现阶段开始前都必须先取得当前工作树的真实test receipt；本报告不把未执行测试推断为已通过。
