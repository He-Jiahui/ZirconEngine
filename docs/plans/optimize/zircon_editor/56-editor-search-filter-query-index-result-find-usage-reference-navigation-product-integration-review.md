---
related_code:
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/hierarchy.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_effect_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_tags_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/ui/workbench_extension_icon_library_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/generated/workbench_generated_bottom_panel.zui
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/snapshot/asset
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/editor_asset_manager/reference_graph.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/dependency_index
  - zircon_editor/src/ui/layouts/views/asset_reference_rows.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/edit.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/hierarchy_projection.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference
  - zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/asset_reference.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/blend_space_search.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/generated_bottom_panel_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
tests:
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
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
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
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 56 · Editor Search / Filter / Query Index / Result / Find Usage / Reference Navigation 产品集成工程化差距

## 1. 结论

Zircon Editor当前没有统一的工程级搜索系统，而是并存三类完全不同的实现。Asset Browser搜索是真实可达的，但只在当前目录对`display_name`、`file_name`和`locator`做ASCII小写子串扫描，并附加单一`ResourceKind`过滤；Hierarchy搜索也真实可达，能保留命中节点祖先、处理Unicode小写并通过5,000行扁平/深层fixture，但过滤激活后会把每次稀疏层级变化升级为全量同步和重排；Blend Space则只过滤模板内写死的三个演示行。

更严重的是产品能力真实性。Workbench Shell的`WorkbenchSceneSearchField`没有事件，旁边Filter按钮发出`workbench.hierarchy.open_filter`，而preview allowlist和window binding登记的是另一个`workbench.hierarchy.filter.open`；Effect和Tags搜索事件虽被generic bridge识别，却既不读取query、也不更新结果或返回反馈。Icon Library的Find Usage固定显示`14 references`，Gameplay Tags的Reference Scan只切换到静态bottom-panel route。这些可见、可点击或可输入入口不能继续被当成已经存在的搜索产品。

当前`ReferenceGraph`、UI document dependency generation和动态reference row/pointer bridge可以保留为局部底座，但它们分别是资产catalog派生图、打开文档的刷新路由和一次性全量UI投影，不构成跨资产、Scene对象、脚本符号、Gameplay Tag、UI资源与插件定义的Find Usage服务。工程目标必须是一个带typed query、provider、index generation、取消、分页、结果地址、stale校验、导航receipt和能力真实性状态的Editor Search Runtime，而不是继续给每个Workspace手写字符串匹配。

本报告登记 **2项P0、44项P1、12项P2与38个资格门**。Editor56拥有跨Editor的query/result/provider/index-session/navigation产品合同及搜索入口真实性；Editor04继续拥有资产注册表、import、dependency/referencer数据权威，Editor03拥有Scene identity/hierarchy业务，Editor08拥有command palette与command基础设施，Editor10拥有通用background operation，Editor15/20/21/23/25拥有各业务域结果语义，Runtime24拥有qualified identity，Runtime51拥有Runtime asset registry/index持久化。不得把这些父owner的全部问题复制进本报告。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试 | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| 聚焦Zircon源码与资产 | 85 / 27,565 / 1,143,307 | 137个`#[test]`、1个ignore | E3 | asset search事件链、hierarchy filter/reflow、module搜索、reference graph/row/pointer/navigation |
| 真实搜索产品链 | Asset Browser与Hierarchy两条 | 两者均有行为测试；Hierarchy含5k规模fixture | E3 | query输入到snapshot、filtered rows、selection/navigation的实际效果 |
| 模板/preview搜索面 | Scene、Effect、Tags、Blend Space、Icon Usage、Tag Reference Scan | 大量binding/结构/截图测试；只有Blend Space有搜索行为测试 | E3 | visible control是否读取query、产生真实结果、报告真实operation状态 |
| 参考源码 | 17 / 12,198 / 483,650 | 53个词法test入口 | E2/E3 | compiled query、thread-safe filter、registry scan、async find、cancel/progress、provider/result UX |

85份聚焦文件按normalized relative path排序，将每个`path + NUL + lowercase file SHA-256 + LF`串联后计算working-tree fingerprint，结果为`28afd9d66c3667997612067aedc4a4ccf98d5818d5fefa591f3a5dd6a3812627`。17份参考源码按同一算法计算，fingerprint为`170aeb7da4ca9f7d684462ed8fa5f709fb4d76c790236e94e7c17161550f7f76`。冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

聚焦语料中有13份非本轮产生的在途文件，包括asset binding/normalization、reference pointer与若干presentation/tests；`asset_reference_rows.rs`当前差异只是格式与显式生命周期调整。本轮按working tree读取，不覆盖这些变更。实施前必须重算fingerprint、重查所有`Search|Filter|Find Usage|Reference Scan`caller，并确认P0入口仍可见且仍无真实consumer。

### 2.2 当前产品链矩阵

| 产品面 | 输入/入口 | 当前执行事实 | 工程结论 |
|---|---|---|---|
| Asset Browser | `SetSearchQuery` binding/event | 每次变化重建snapshot，扫描catalog，当前目录子串+单kind | 真实可用，仍是小规模局部过滤器 |
| Hierarchy pane | `HierarchySearchQuery` TextField | O(N)名称过滤并保留祖先；active filter使稀疏fragment走全量reflow | 真实可用，有正确性基础和增量退化 |
| Workbench Scene tree | `WorkbenchSceneSearchField` | 无events；Filter route与登记route不一致 | 可见但不可执行 |
| Gameplay Effect | Change/Submit route | generic action判定返回true，handler不读值，feedback返回`None` | 事件被吞掉的no-op |
| Gameplay Tags | Change/Submit route | 同Effect；且`Search tags...`写在`value`而非placeholder | no-op并污染初始query语义 |
| Blend Space | Change/Submit route | ASCII匹配`BS_Idle_Run`等三个常量行 | fixture级局部过滤，非资产搜索 |
| Icon Find Usage | 可见按钮 | 固定写入“queued / 14 references” | 伪造operation与结果数量 |
| Tag Reference Scan | 可见bottom row | 只选择静态panel route和Review mode | 没有scan/query/result |
| Asset References / Used By | utility tab + dynamic rows | catalog派生直接边，全量materialize，已知UUID可点击 | 局部直接边浏览器，不是Find Usage |
| Locate Selected Asset | toolbar action | 只打开`editor.assets`并请求preview refresh | 不保证清过滤、展开目录、滚动和聚焦 |

### 2.3 必须保留的工程基础

1. 保留Hierarchy反向祖先传播算法和一次parent-index构建；它已正确解决“子节点命中但祖先必须可见”的基本树过滤问题。
2. 保留Hierarchy的ASCII fast path、Unicode lowercase fallback、5k flat/deep测试与selection overlay分离；未来应接入compiled query，而不是删除现有正确性。
3. 保留Asset Browser从一个activity snapshot派生explorer snapshot的共享投影，避免恢复双重全量build；后续替换的是候选取得与结果合同。
4. 保留`ReferenceGraph`的UUID优先匹配、locator fallback与Editor04 catalog generation接线，但将其降为一个asset-reference provider，不让它继续冒充全局引用权威。
5. 保留`UiAssetDependencyGeneration`的generation、正反向open-document route和impact集合；它是文档刷新索引，不应被改名冒充project search index。
6. 保留reference row的prototype克隆、responsive layout与pointer bridge同步；工程化目标是在其上增加paged/virtual result model和typed target。
7. 保留Blend Space“不为每行分配lowercase字符串”的ASCII matcher作为极小静态列表优化，但真实资产列表必须由provider提供。

## 3. P0：必须先关闭的能力真实性问题

### P0-01 · Workbench Scene、Effect、Tags过滤入口对用户可见，却分别无事件、route漂移或静默no-op

`WorkbenchSceneSearchField`有真实输入外观但没有`events`；相邻Filter按钮发出`workbench.hierarchy.open_filter`，而preview action和window binding使用`workbench.hierarchy.filter.open`。Effect/Tags route被`is_workbench_module_action`接受，`apply_workbench_module_action`却只处理tab、row、command和dropdown，随后search action在feedback的默认分支返回`None`。调用成功不等于搜索发生。

必须在任何新增搜索功能前选择一种硬切：接入真实query owner/provider/result projection，或把控件标成Unavailable并不可交互。不得继续保留“事件成功分发但结果完全不变”的产品状态，也不得用模板binding测试作为完成证据。

### P0-02 · Icon Find Usage与Gameplay Tags Reference Scan宣称真实检索，却没有operation、scanner或结果集

Icon Library点击后固定输出`Icon usage search queued`与`14 references`，没有OperationId、provider调用、catalog generation、result rows或失败状态；Gameplay Tags的Reference Scan仅映射到`module-bottom-gameplay-tags:reference-scan`静态路由。固定数量会直接误导删除、重命名、迁移和依赖判断，风险高于普通占位UI。

必须删除固定结果文案，接入Editor Search Operation与业务provider；在provider未安装、索引未就绪、结果不完整或查询取消时，UI必须显示真实capability/completeness状态。找不到结果、尚未搜索与搜索失败必须是三个不同状态。

## 4. P1：工程级搜索与引用结果链必须补齐的44项差距

### 4.1 Search Runtime、query与provider合同

| ID | 差距 | 必须形成的合同 |
|---|---|---|
| E56-P1-01 | 没有唯一`EditorSearchService` owner | 明确service scope、project/session lifecycle、shutdown与多窗口共享规则 |
| E56-P1-02 | query只是一段原始字符串 | `SearchQuerySource -> parse -> typed AST -> compile`，带span、diagnostic与版本 |
| E56-P1-03 | 没有provider registry | `SearchProviderDescriptor`声明domain、capability、schema、priority与unregister generation |
| E56-P1-04 | query没有Project/World/Document/User scope | `SearchScopeId`与BuildSet/owner/generation必须进入每个请求 |
| E56-P1-05 | 结果没有统一schema | `SearchResultRecord`包含provider、qualified target、label、context、score、match ranges与actions |
| E56-P1-06 | 没有cursor/page协议 | page size、stable order、continuation、total estimate与terminal page receipt |
| E56-P1-07 | 没有operation lifecycle | OperationId、cancel、deadline、progress、terminal disposition与late-result rejection |
| E56-P1-08 | 没有index schema/lifecycle | index version、build generation、incremental update、rebuild、migration与persistence政策 |
| E56-P1-09 | UI不知道结果完整性 | `Complete/Partial/Indexing/Stale/ProviderFailed/Cancelled`必须是显式状态 |
| E56-P1-10 | 没有admission和预算 | 并发query、CPU时间、内存、provider fan-out、result bytes与UI apply budget |

### 4.2 Asset Browser搜索

| ID | 差距 | 必须形成的合同 |
|---|---|---|
| E56-P1-11 | 资产搜索被锁在当前目录直属项 | 当前目录/递归/全Project/package/plugin scope必须显式可选 |
| E56-P1-12 | 只匹配name/file/locator | 支持type、tag、source、status、owner、dependency、modified、platform等字段 |
| E56-P1-13 | kind filter只能保存一个`ResourceKind` | 多选facet、include/exclude与provider-defined facet schema |
| E56-P1-14 | `to_ascii_lowercase`导致国际化不一致 | Unicode case-fold/normalization policy与locale-independent identity matching |
| E56-P1-15 | 每次编辑都线性扫描folders/assets | 从catalog generation建立增量候选/index；小集合可保留scan fast path |
| E56-P1-16 | 所有命中一次性collect为`Vec` | bounded top-K、page/virtualization和早停，不能让结果数决定单帧分配 |
| E56-P1-17 | 只有布尔匹配，没有score/rank/highlight | 稳定相关性、exact/prefix/fuzzy权重、match ranges和确定性tie-break |
| E56-P1-18 | 没有saved query/history | 版本化query preset、recent history、项目共享/用户私有scope |
| E56-P1-19 | 资产类型与插件不能扩展字段 | provider enrichment与字段权限，不在central matcher手写每种asset |
| E56-P1-20 | catalog generation变化只触发重投影 | index delta、delete tombstone、rename/move continuity与currentness receipt |
| E56-P1-21 | Change事件没有debounce/coalescing/cancel | 输入generation、短debounce、旧任务取消和late page丢弃 |
| E56-P1-22 | activity/explorer共享数据但仍克隆整snapshot | 共享不可变result generation，surface只保存view cursor/selection |

### 4.3 Hierarchy与Workbench局部过滤

| ID | 差距 | 必须形成的合同 |
|---|---|---|
| E56-P1-23 | Hierarchy query只存在Retained host字符串字段 | Scene search request/result DTO，支持窗口恢复与同代投影 |
| E56-P1-24 | 只按节点名称匹配 | type、component、tag、visibility、lock、level/data-layer、owner与validation facet |
| E56-P1-25 | active filter明确禁用稀疏fragment | filtered tree增量维护或受预算重算，不能每次结构变化全量reflow |
| E56-P1-26 | 祖先可见但没有match reason | direct match、ancestor context、forced-visible selection分别标注 |
| E56-P1-27 | 没有flat result与path context | tree context/flat results切换，显示qualified scene path并稳定导航 |
| E56-P1-28 | Filter按钮route两种拼写且无菜单实现 | 单一typed action，真实facet menu或明确Unavailable |
| E56-P1-29 | 工作台Scene搜索与真实Hierarchy搜索是两套控件 | 绑定同一query session，避免shell fixture与pane authority分裂 |
| E56-P1-30 | 无result count/partial/error/empty区分 | 可访问的状态摘要、清除过滤与定位当前选择动作 |
| E56-P1-31 | Blend Space只过滤三个常量行 | Animation asset provider、真实catalog scope、paged rows与selection continuity |
| E56-P1-32 | Tags把placeholder写进`value` | placeholder与query state分离，空query不得搜索文字`Search tags...` |
| E56-P1-33 | Effect/Tags没有query generation/result owner | 业务provider返回typed target和同代结果，不由template bridge改静态可见性 |
| E56-P1-34 | 各模块重复route和matcher | 公共search session/widget adapter，domain只实现provider与result action |

### 4.4 Reference graph、Find Usage与结果导航

| ID | 差距 | 必须形成的合同 |
|---|---|---|
| E56-P1-35 | Editor `ReferenceGraph`重复保存outgoing/incoming且incoming输出源于`HashSet` | Editor04权威图提供稳定有序snapshot；Editor56只消费provider结果 |
| E56-P1-36 | direct reference提取靠`ImportedAsset`手写match且若干类型返回空 | 由Editor04/各asset domain注册typed reference extractor与field path |
| E56-P1-37 | References/Used By为每条结果克隆四个节点 | virtual row pool、page apply budget、百万边压力下的bounded UI |
| E56-P1-38 | 未知/缺失reference显示后不可点击也不可修复 | missing target结果类型、source locator、repair/rebind/remove动作 |
| E56-P1-39 | `navigate_to_asset`对未知UUID静默no-op | typed stale/not-found/permission/provider-error navigation receipt |
| E56-P1-40 | Locate Selected只打开Assets view | 清理冲突filter、展开folder、滚动、聚焦、选择确认与失败反馈 |
| E56-P1-41 | 结果目标只有asset UUID | object/document/symbol/property/line-column/subobject/reference-field地址 |
| E56-P1-42 | 点击时不校验result/index generation | re-resolve target、detect stale、offer refresh并拒绝跨Project误导航 |
| E56-P1-43 | 没有跨域Find Usage | 聚合assets、scene/prefab、scripts、tags、UI、materials、plugins，保留provider provenance |
| E56-P1-44 | 没有automation/query DTO | command/API可提交同一typed query、流式读page、取消并取得terminal receipt |

## 5. P2：实现质量与可维护性债务

| ID | 差距 | 收敛方向 |
|---|---|---|
| E56-P2-01 | route/control/property大量裸字符串 | 生成式descriptor与typed action key |
| E56-P2-02 | Asset、Hierarchy、Blend Space各写一套case-insensitive matcher | 共享compiled text primitive并保留小集合fast path |
| E56-P2-03 | Tags placeholder被编码为初始value | schema validator禁止placeholder/value混用 |
| E56-P2-04 | ReferenceGraph的`HashSet -> Vec`缺稳定排序 | provider contract要求deterministic order |
| E56-P2-05 | reference UUID藏在UI节点`value_text` | typed node payload/target handle |
| E56-P2-06 | dynamic row ID依赖数组index | stable result key与recycled row binding generation |
| E56-P2-07 | search没有统一metrics/trace | query parse、candidate、provider、page、apply、navigate分阶段指标 |
| E56-P2-08 | search优化测试用`include_str!`匹配源码文本 | 行为benchmark和allocation/candidate counters替代文本守卫 |
| E56-P2-09 | 1个Asset Browser视觉截图测试被ignore | 产物lane与普通test分离，required visual基线可重放 |
| E56-P2-10 | 模板测试主要证明route存在 | 增加route产生真实result generation变化的semantic assertion |
| E56-P2-11 | dependency generation用`saturating_add`隐藏终点 | generation exhaustion策略、epoch rollover与测试 |
| E56-P2-12 | 静态feedback嵌入资产名、计数和fps | 只呈现真实operation/result DTO，不从UI bridge伪造业务事实 |

## 6. 目标架构

### 6.1 核心数据模型

```text
SearchQuerySource
  -> SearchQueryParser -> CompiledSearchQuery { schema_version, AST, diagnostics }
  -> SearchRequest { scope, providers, generation, budget, deadline }
  -> EditorSearchService
       -> ProviderPlan[] -> candidate/index execution -> ranked merge
       -> SearchResultPage { cursor, records, completeness, provider receipts }
  -> SearchPresentation { virtual rows, facets, progress, errors }
  -> SearchNavigationRequest { result_key, target, observed_generation }
  -> NavigationReceipt { resolved target, focus/selection effects, disposition }
```

最低类型集合应包括：`SearchQueryId`、`SearchOperationId`、`SearchScopeId`、`SearchProviderId`、`SearchIndexGeneration`、`CompiledSearchQuery`、`SearchBudget`、`SearchResultKey`、`QualifiedSearchTarget`、`SearchResultPage`、`SearchCompleteness`、`ProviderSearchReceipt`与`SearchNavigationReceipt`。任何跨线程/跨窗口page都必须携带request generation；旧page不得覆盖新query。

### 6.2 Provider边界

| Provider | 数据权威 | 典型字段/结果 | 本报告边界 |
|---|---|---|---|
| Asset | Editor04/Runtime51 catalog与reference graph | asset、folder、type、tag、dependency、source | Editor56编排query/page/navigation，不复制registry |
| Scene | Editor03/Runtime05 scene snapshot | entity、path、component、tag、layer | 只消费同代只读snapshot，不直接扫mutable World |
| Script/Symbol | Script authoring owner | file、symbol、definition/reference、diagnostic | provider自带index/schema和line-column target |
| Domain authoring | Editor15/20/21/23/25等 | node、tag、icon、binding、diagnostic | domain负责语义和动作，Search Runtime负责公共生命周期 |
| Plugin | Editor extension owner | provider descriptor与custom fields | unregister generation、capability/permission、failure isolation |

### 6.3 性能政策

1. 小于配置阈值的静态列表可使用无分配linear scan；Project级资产/引用/符号搜索必须使用index或有界streaming candidate。
2. 输入编辑只发布query generation；parse/compile可同步但受字符/AST预算，provider执行不得阻塞UI线程。
3. merge必须有top-K heap或等价bounded算法，稳定tie-break不能依赖`HashMap/HashSet`迭代顺序。
4. UI每帧只应用有预算的page/row变化；取消、窗口关闭、Project切换和plugin unload都必须quiesce或拒绝late page。
5. “优于Unreal”只能由同语料、同字段、同冷/热状态、同完整性、同硬件的candidate latency、time-to-first-page、terminal latency、peak memory与UI frame cost证明，不能由源码行数或单个5k fixture推断。

## 7. 分层实施顺序

### M0 · Capability truth硬切

- 修复/下架P0-01的Scene/Effect/Tags控件，统一Hierarchy filter action ID。
- 删除P0-02固定`14 references`与静态scan成功暗示，显示Unavailable/Indexing/Failed等真实状态。
- 给所有Search/Find Usage/Reference Scan入口建立owner与capability inventory。

### M1 · Query、operation与result kernel

- 落地typed query AST、diagnostic、provider descriptor、request generation、budget/cancel/deadline。
- 落地paged result、completeness、provider receipt、qualified target与navigation receipt。
- 接入Editor10 operation lifecycle，但Search保留自己的query/page语义。

### M2 · Asset与Hierarchy迁移

- 先保持现有UI，将Asset matcher和Hierarchy matcher迁入provider。
- 为Asset接入catalog delta/index与facet；为Hierarchy接入filtered-tree增量更新。
- 统一Workbench Scene与pane Hierarchy query session，替换Blend Space常量列表。

### M3 · Reference / Find Usage闭环

- 由Editor04提供typed reference edge和稳定generation，建立asset/provider adapter。
- 引入source field path、missing target、stale resolve、repair与真实locate/focus。
- 先完成Asset、Gameplay Tag、Icon三个真实vertical，再扩展Scene/Script/UI/Material。

### M4 · Provider生态与产品统一

- Plugin注册/卸载、schema capability、权限、failure isolation和query preset。
- 公共Search widget、result view、keyboard/accessibility、history与automation API。
- 移除template bridge内所有静态业务结果文案。

### M5 · 规模、韧性与性能资格

- 100k/1m assets、百万reference edges、深层Scene、Unicode/fuzzy/compound query基准。
- query storm、cancel race、Project切换、plugin unload、index corruption/rebuild、OOM预算测试。
- 和参考引擎做同工作负载对比，保存可复现语料、配置、trace和receipt。

## 8. 资格门

| Gate | 验收条件 |
|---|---|
| E56-G01 | 所有可见Search/Filter/Find Usage/Reference Scan入口都有真实provider或明确Unavailable |
| E56-G02 | Effect/Tags query变化产生新的result generation，不再被generic handler静默吞掉 |
| E56-G03 | Workbench Scene与Hierarchy pane使用同一query session和action ID |
| E56-G04 | Icon Usage数量来自terminal result receipt，禁止固定计数 |
| E56-G05 | Gameplay Tags Reference Scan有OperationId、进度、取消、结果页和失败态 |
| E56-G06 | query parser覆盖AND/OR/NOT、group、field comparison、quote与invalid span |
| E56-G07 | compiled query可在多个provider worker安全复用或显式clone |
| E56-G08 | 每个request/page/result都有Project/World/Document scope与generation |
| E56-G09 | provider unregister后旧page被拒绝且无callback/owner泄漏 |
| E56-G10 | cancellation在deadline内terminal，late page不能修改当前UI |
| E56-G11 | result page受row/byte/time预算，空查询不能一次materialize全Project |
| E56-G12 | stable order在重复运行、线程数变化与Hash seed变化下不漂移 |
| E56-G13 | Asset支持current folder、recursive与whole project三种显式scope |
| E56-G14 | Asset支持多kind include/exclude和至少五类typed facet |
| E56-G15 | Unicode normalization/case-fold corpus跨Asset/Hierarchy/Module一致 |
| E56-G16 | exact/prefix/fuzzy score与match ranges有golden tests |
| E56-G17 | catalog add/remove/rename/move只增量更新受影响index segment |
| E56-G18 | index schema升级、损坏、缺失与重建都有receipt和回退状态 |
| E56-G19 | 100k asset warm query满足批准的TTFP、terminal和peak-memory预算 |
| E56-G20 | 1m asset/query storm下UI线程无未预算全量scan或row clone |
| E56-G21 | Hierarchy名称过滤继续保留祖先并标记direct/ancestor match |
| E56-G22 | filtered Hierarchy增量更新不再无条件触发full authoritative reflow |
| E56-G23 | 深层Scene、cycle-corruption guard与5k/100k层级压力测试通过 |
| E56-G24 | Blend Space搜索来自真实provider而非三个模板常量 |
| E56-G25 | placeholder/value schema test阻止`Search tags...`成为query |
| E56-G26 | Reference结果包含source asset/object、field path、target与edge kind |
| E56-G27 | missing reference可定位source并执行repair/rebind/remove受控动作 |
| E56-G28 | stale result导航返回typed disposition并可刷新，不静默no-op |
| E56-G29 | Locate Selected完成open、clear conflicting filter、reveal、scroll、focus、select receipt |
| E56-G30 | reference result view使用virtual rows/page，百万边不线性创建UI节点 |
| E56-G31 | Asset/Scene/Script/Tag/UI/Material至少六类provider可合并Find Usage |
| E56-G32 | provider partial failure不抹掉其他结果，并明确completeness |
| E56-G33 | automation API与UI使用同一query/result/navigation DTO |
| E56-G34 | keyboard、focus、screen-reader result count/progress/cancel通过产品测试 |
| E56-G35 | Project close/switch、window close、plugin unload均quiesce active queries |
| E56-G36 | fuzz覆盖query parser、cursor、stale target和malformed provider page |
| E56-G37 | benchmark记录cold/warm index、TTFP、terminal、CPU、peak memory与UI apply cost |
| E56-G38 | 同硬件同语料对比达到批准门槛前，不宣称搜索性能优于Unreal |

## 9. 参考引擎翻译决策

| 参考 | 可迁移的工程事实 | 不应照抄 |
|---|---|---|
| Unreal `TextFilterExpressionEvaluator` | compiled expression、basic/complex context、错误文本、single-token fast path | C++宏和具体parser实现不是Rust架构要求 |
| Unreal `AssetTextFilter` | 可供worker使用的compiled filter、saved queries、class/path/collection字段、memory reuse | 不复制Content Browser的历史UI耦合 |
| Unreal `FindInBlueprintManager` | versioned index/cache、active async query、begin/continue/end、progress/cancel、失败资产清单 | Zircon不应建立Blueprint专属全局单例 |
| Unreal `IAssetRegistry` | enumerate/query、dependency/referencer、sync/async scan、priority、completion与变更事件 | Runtime51拥有registry，不把接口搬进Editor56 |
| Unreal `SceneOutlinerFilters` | composable visibility filters与interactivity分离 | 不复刻Actor/Outliner类型层次 |
| Godot Quick Open | fuzzy candidate、base-type filter、history、max results、list/grid与selected path | 不把主线程全量遍历当作百万资产资格 |
| Godot Find in Files | include/exclude/root、whole-word/case、progress/stop、replace与多tab结果 | 文件文本替换不是所有domain的共同语义 |
| Fyrox Dependency/SearchBar | 依赖浏览器与可复用search bar证明局部产品下界 | 其局部scan不等于全局index架构 |
| Bevy `AssetPath`/`QueryState` | source/path/label typed identity与compiled/cached query state思路 | Bevy没有成熟Editor搜索产品面，不能作为UX完成证据 |
| Unity Graphics | UI highlight、debug filter、typed Search columns、Shader Graph provider/tree/result action | Graphics仓只是Unity搜索生态的消费者，不代表完整Unity Search源码 |

## 10. Owner边界与最终判定

- Editor56只拥有跨Editor query/runtime/provider/result/navigation与可见搜索入口真实性；不重写Editor04的资产注册/导入/引用抽取，也不接管Runtime51的持久化asset index。
- Hierarchy的Scene snapshot、identity和mutation继续由Editor03/Runtime05/Runtime24拥有；Search provider只能消费同代只读事实。
- Command Palette继续由Editor08拥有。它可以消费Search Runtime provider，但本报告不把palette快捷命令和内容搜索混成一个产品。
- Domain report继续拥有“一个Gameplay Tag引用是什么”“一个Shader节点如何创建”等业务语义；Editor56只要求它们以provider/result contract暴露。
- Tooling优化按用户要求暂停；本报告不新增tooling实现，只记录后续Rust实现必须满足的产品和运行时合同。

结论不是“Zircon没有搜索”。Asset Browser和Hierarchy已经具备可保留的真实实现，Blend Space也证明了template bridge能修改局部可见性。真正差距在于这些能力没有共享query/index/result/operation体系，并且同一产品里同时存在可用搜索、fixture搜索、静默no-op与固定伪结果。必须先关闭两项P0能力真实性问题，再按M1-M5把局部filter提升为可扩展、可取消、可分页、可审计、可导航并能接受百万级语料验证的Editor Search Runtime。
