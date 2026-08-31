---
title: Editor Scene Hierarchy、Outliner Tree Projection、Expansion、Selection、Rename、Reparent、Drag Drop、Visibility、Lock 与 Multi-World Product Integration 当前源码复核
category: zircon_editor
report_id: Editor181
review_date: 2026-08-27
baseline_head: e6bfb5c0240fb62434c4ba86a1dc2525c0434d96
related_code:
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/hierarchy_topology.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_runtime_interface/src/world_sync
  - zircon_editor/src/core/editor_message/message/scene_inspection
  - zircon_editor/src/core/editor_event/hierarchy_host_event.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene/selection
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/host/play_hierarchy_projection.rs
  - zircon_editor/src/ui/host/editor_event_execution/hierarchy_event.rs
  - zircon_editor/src/ui/workbench/snapshot/data/scene_entry
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/hierarchy
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy
tests:
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/state/selection.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
  - zircon_editor/src/tests/host/retained_asset_refresh/scene_reload.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/hierarchy.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/interaction.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_fragment.rs
  - zircon_editor/src/tests/host/retained_list_pointer/bridge_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/162-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
  - docs/plans/optimize/zircon_editor/176-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-current-source-review.md
  - docs/plans/optimize/zircon_editor/177-editor-search-filter-query-index-result-find-usage-reference-navigation-current-source-review.md
  - docs/plans/zircon_editor/editor_layout/09/failure-2026-08-05-retained-hierarchy-dirty-refresh-full-snapshot-fallback.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-27-retained-template-table-row-identity-selection.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-17-editor-selection-state-runtime-session-boundary.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-transaction-selection-history-wide-snapshot.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerTreeItem.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerHierarchy.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerMode.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SOutlinerTreeView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/ActorMode.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/Fyrox/editor/src/world/mod.rs
  - dev/Fyrox/editor/src/world/item.rs
  - dev/Fyrox/editor/src/world/graph.rs
  - dev/bevy/crates/bevy_ecs/src/hierarchy.rs
  - dev/bevy/crates/bevy_ecs/src/relationship/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/macro_logic/src/component.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/Window/RenderPipelineConverterVisualElement.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/Window/RenderPipelineConverterVisualElementListFilter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/ConverterState.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 181 · Editor Scene Hierarchy / Outliner / Selection / Rename / Reparent 当前源码复核

## 1. 结论与状态

Editor60登记的两条可达P0仍然全部Open。Hierarchy primary press仍立即写入`active_scene_drag_payload`和`active_hierarchy_drag_node_ids`，primary release只要命中另一row或list surface就直接派发`ReparentNodes`；期间没有drag threshold、pointer capture、source generation、drop preflight或terminal receipt。World/project replacement也只替换authoring World并发布inspection resync，没有退休retained host中的active drag、double-click或inline rename focus。event、intent和rename focus继续只携裸`u64 NodeId`，因此World A留下的旧交互仍可能修改World B中复用同值ID的对象。

本轮也确认了不能抹掉的真实进展。Runtime hierarchy artifact已删除Editor focus死字段，Editor selection是唯一focus来源；`SelectionSnapshot`和`SceneSelection`已改为typed immutable `Arc` payload；Scene inspection支持generation、selection revision、稀疏row/selection patch、缺口resync与persistent row patch；Scene tree物理control固定为10个，native paint和pointer route按viewport/算术索引工作。实际control dispatch还会通过`control -> EntityId`的无损投影解析实体，不再完全依赖clamped `scene_node_id`。因此旧P1-03和P1-09关闭，P1-02、P1-11、P1-13、P1-15、P1-24降为Partial。

这些进展没有形成工程级World Outliner。逻辑`SceneNodeData`仍只含`id/name/depth/selected`，Runtime row的kind、active和has-children在最终pane丢失；`expanded`仍错误等于`has_children`，所有后代始终可见；结构变化和active filter仍触发完整reflow，逻辑`ModelRc`、entity index和filter仍按全树规模；点击仍replace-only；rename仍只有trim/nonempty；drop仍没有anchor、transform、owner/instance/lock/reference政策。当前产品不能支持“性能和表现优于Unreal”的声明。

本轮不新增、不删除、不重排Editor60的canonical finding，只按当前磁盘重判其2项P0、24项P1、8项P2和40个资格门：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 2 | 0 | 0 |
| P1 | 17 | 5 | 2 |
| P2 | 6 | 2 | 0 |
| Gate | 26 Fail | 10 Partial | 4 Pass |

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Runtime inspection、world hierarchy与world-sync interface | **26 / 7,797 / 7,083 / 278,943 / 58 / 3** | hierarchy authority、artifact、delta、subscription、generation与mutation；fingerprint `2256f9433d4b16fa4e1c857fe63feb608275b3cf169131fc9f94e505f97a33a2` |
| Editor publication、projection、bridge与native paint | **96 / 20,704 / 19,533 / 785,677 / 53 / 1** | message到logical row、固定control和viewport paint的依赖闭包；fingerprint `3dbe65139411587020a18da1f9113624213392aa6e6419430f585d36e0843ee9` |
| Hierarchy input、selection、event、intent与transaction | **55 / 5,626 / 5,145 / 201,139 / 32 / 0** | press/release、rename、World replacement、selection和command链；fingerprint `76dcf12407a9d9c88cabe5aa6b245e0305af5845957d0ef9a0dbdebca664e64a` |
| 聚焦测试 | **22 / 7,156 / 6,654 / 256,238 / 110 / 0** | sparse patch、10K逻辑行、selection、transaction与drag source；fingerprint `274f87e98689de91d877ac94fe44a078b506367a116f59098dd3b40d6f4a0add` |
| Zircon去重选择集 | **199 / 41,283 / 38,415 / 1,521,997 / 253 / 4** | 当前磁盘静态依赖闭包；fingerprint `4612035deb3780bac035b49941d3d487af52e2ec4f3f491664a66005a2690af9` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **18 / 19,930 / 17,191 / 706,227 / 49 / 0** | Outliner contract、drag/drop、reparent、identity、recycled tree cell；fingerprint `b5cac140004338922e81f4435e10273dd7f6eba8d27763489ffacf6325a7fbf6` |

统计按lowercase normalized relative path的ordinal顺序，将`path + NUL + raw bytes + NUL`串联后计算SHA-256。tests/ignored为词法属性计数，不是执行receipt。冻结时主仓HEAD为`e6bfb5c0240fb62434c4ba86a1dc2525c0434d96`；Godot、Fyrox、Bevy和Unity Graphics分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`和`a7e4c051d256a781ab362c64316b125a1e104694`。共享工作树存在其他在途修改，本报告以当前磁盘为事实源，实施前必须重算fingerprint。

### 2.2 当前产品链

```text
World mutation
  -> WorldInspectionArtifact { generation, Arc hierarchy, sparse overrides }
  -> SceneInspectionMessage { artifact generation, selection revision }
  -> sparse fragment or authoritative reflow
  -> SceneHierarchyProjectionState { entity/index/control/selection }
  -> fixed 10 authored controls + O(N) logical SceneNodeData model
  -> viewport-bounded native paint and arithmetic pointer routing

primary press
  -> immediately arm UiDragPayload("scene://node/{id}") + Vec<NodeId>
primary release
  -> resolve row/root from current projection
  -> EditorHierarchyEvent::ReparentNodes { bare ids }
  -> EditorIntent::SetParents
  -> normalized-root command group / rollback / history

F2 or local 500 ms second click
  -> text focus dispatch string carries bare NodeId
  -> clear draft before command result
  -> EditorHierarchyEvent::RenameNode
  -> mutate whichever edit World is current
```

### 2.3 已有基础必须保留

1. Runtime artifact的immutable generation、Arc row/index、sparse override、non-recursive traversal和ancestor hash patch。
2. Latest delivery缺口后的authoritative reflow、selection revision修复和generation拒旧。
3. `SceneHierarchyProjectionState`的无损entity/control路由；clamped property不能再作为真实identity回退。
4. 固定10个authored control、viewport-bounded paint和O(1) pointer index；不要恢复per-item retained control。
5. typed `SelectionSnapshot` / `SceneSelection`共享payload，以及Edit/Play domain selection隔离。
6. multi-node reparent先折叠selected parent/child为top-level roots，再以一个transaction group提交；cycle失败会rollback全部command。
7. Runtime reparent前后各记录一次fact以覆盖旧/新祖先链，再在subscription内合并成一个对外事实；这不是重复事件缺陷。
8. source-shape guard只能作补充，不能替代真实pointer、IME、World replacement、窗口和规模测试。

## 3. 旧报告current-source校正

| 旧条目 | current-source变化 | 裁决 |
|---|---|---|
| ED60-P1-02 lossless identity | product control dispatch已通过`scene_entity_for_control()`读取无损map；但`scene_node_id()`仍把所有大于`i64::MAX`的ID压成同值，property和fallback合同仍不安全 | Partial |
| ED60-P1-03 Runtime focus双authority | `WorldHierarchyRow`已无focused字段，Runtime测试反向守卫legacy focus；Editor selection primary独立驱动focused fields artifact | Closed |
| ED60-P1-09 physical virtualization | `SCENE_TREE_STATIC_CONTROLS`固定10个，逻辑第11行不会生成control；native paint只遍历visible range，pointer用item count算术命中 | Closed；O(N)逻辑model/reflow继续由P1-10/11/12约束 |
| ED60-P1-11 filter | ASCII路径避免逐name分配lowercase，parent index单次构造，并增加5K深/平树和profile counter | Partial；仍是全量scan/clone且无index/cancel/query generation |
| ED60-P1-13 row/cell | 物理control有界，generic callback按最新control/entity map解析，不存在逐item闭包遗留 | Partial；icon/columns/warning/visibility/lock等产品能力仍缺 |
| ED60-P1-15 selection | `SelectionModel`使用ordered `IndexSet`、Edit/Play domain和revision，transaction selection使用typed Arc snapshot | Partial；没有DocumentKey/WorldSessionId/range anchor/hidden count |
| ED60-P1-24 metrics/tests | hierarchy fragment、filter、materialization、row patch和10K逻辑行已有counter/tests | Partial；P0交互、100K/1M、latency、rejection、retirement与真实窗口receipt仍缺 |

四份failure记录不能冒充完成。Layout09的pure tree dirty fragment已有static forward repair但managed validation仍pending；EditorUI01 table row identity仍Open；Runtime10已删除错误的runtime `selected_node`状态但上行门未完整闭合；Editor03 typed selection handle已落地，compact/paged history和bounded lifecycle stream仍未完成。

## 4. P0：当前仍可达

### ED60-P0-01 · Open · 普通press/release仍可直接变成reparent

`hierarchy_pointer_event(kind=0, button=1)`在press分支立即构造authoritative selection IDs和`scene://node/{id}`payload。`kind=2` release先`take()` payload和IDs，再以release位置解析target并派发reparent。没有任何motion threshold、drag-detected transition、pointer/window identity、capture lease、drag-over validation或drop decorator。现有测试甚至把“pointer down应武装payload”固化为正向合同。

这意味着按下A、在B释放即可执行结构修改，即使平台从未判定drag。必须硬切为`Idle -> PressedCandidate -> Dragging(capture lease) -> Terminal(receipt)`；click、double click、rename和drag必须互斥。只有越过平台metrics且取得capture后才能创建mutation-capable transfer session，release只能提交已验证且仍current的immutable drop plan。

### ED60-P0-02 · Open · 旧World交互仍能修改新World的同值ID

Retained host长期保存`active_scene_drag_payload`、`active_hierarchy_drag_node_ids`、`last_hierarchy_rename_click`和text focus dispatch string。`EditorHostEventController::replace_world()`与`clear_project()`只替换state、发布resync并刷新Workbench；当前Hierarchy路径没有OwnerLost或显式清理。`EditorHierarchyEvent`、`EditorIntent`和command capture都只接收裸NodeId。

因此World A中的press/rename可跨project/scene replacement存活，随后对World B中复用的数值ID通过存在性检查。修复必须让interaction、event、intent、plan和receipt共同携`DocumentKey + WorldSessionId + WorldGeneration + item generation + interaction generation`；World close/replacement必须同步terminalize全部session，迟到event必须产生typed stale rejection且B零变化。

## 5. P1：状态与重构要求

| Finding | 状态 | 当前源码事实 | 必须重构为 |
|---|---|---|---|
| ED60-P1-01共同document/world/session identity | Open | artifact只有World内部generation；message、selection、event和active interaction没有共同owner key | qualified `OutlinerOwnerScope`贯穿全链 |
| ED60-P1-02 `u64 -> i64`碰撞 | Partial | control dispatch已有无损map；`scene_node_id()`仍saturating clamp | opaque typed key或checked admission，删除clamp fallback |
| ED60-P1-03 Runtime focus双authority | Closed | Runtime row无focus，Editor selection primary驱动focused field artifact | 保留当前owner边界并补qualified selection identity |
| ED60-P1-04 row字段在pane丢失 | Open | Runtime有kind/active/has_children，`SceneNodeData`只有id/name/depth/selected | typed row capability/column DTO无损投影 |
| ED60-P1-05 invalid topology伪装普通root | Open | unreachable/cycle fallback以depth 0输出但保留冲突parent，无typed diagnostic | invalid topology item、reason与repair action |
| ED60-P1-06 subtree hash语义不足 | Open | hash只含name、child count和ordered child identity/hash | field revision或覆盖全部可见row语义的currentness |
| ED60-P1-07 typed item/provider/mode/column/filter | Open | focused closure无`OutlinerItemId`或provider/mode registry | extensible typed World Outliner registry |
| ED60-P1-08 expansion/view state | Open | `expanded = has_children`，children始终materialized/visible | per-view expanded IDs、reveal、breadcrumb、persist/restore |
| ED60-P1-09 physical row/cell bound | Closed | 10个static controls；第11逻辑行无control；paint visible-range | 保留固定上限并补scroll overscan/cell metrics |
| ED60-P1-10 topology增量 | Open | rename/selection可sparse patch，add/remove/move/reorder仍full reflow | typed range delta、operation queue、budget、last-known-good |
| ED60-P1-11 indexed/cancelable filter | Partial | 5K单次parent index和ASCII fast path存在 | indexed typed grammar、query generation、cancel和stale rejection |
| ED60-P1-12 active filter materialization | Open | filter强制authoritative reflow并clone visible rows；sparse artifact可能被完整materialize | background projection、allocation/latency budget和paged results |
| ED60-P1-13 row renderer/columns/cell binding | Partial | bounded generic controls避免逐item callback；renderer仍只有text/hover/selection | icon、visibility、lock、active、warning、owner、pin与extension columns |
| ED60-P1-14 modifier/range selection | Open | hierarchy click只发`SelectSceneNode`；modifier未进入callback | captured modifier snapshot、Ctrl toggle、Shift anchor/range policy |
| ED60-P1-15 qualified ordered selection | Partial | ordered per-domain SelectionModel和revision存在 | per-document/provider keys、stale prune、hidden count、active/range row |
| ED60-P1-16 double-click admission | Open | 只比较NodeId和本地500 ms Instant | OS click count + window/pointer/button/position/control/session identity |
| ED60-P1-17 rename validation | Open | 只trim并拒绝empty | provider validation、lock/read-only/instance/conflict/Unicode/length policy |
| ED60-P1-18 rename draft/error/IME/lifecycle | Open | dispatch前先clear focus；失败丢draft，无IME/pending/retirement | rename session lease、draft、composition、inline error、terminal receipt |
| ED60-P1-19 drag双authority | Open | URI payload与旁路`Vec<NodeId>`并存 | single-use qualified transfer session handle |
| ED60-P1-20 drop preflight/decorator | Open | release直接commit，只有Runtime/transaction晚期self/cycle/static检查 | `ValidateDrop -> typed rejection/accept -> immutable plan` |
| ED60-P1-21 anchor与sibling order | Open | 只有parent/root，没有on/before/after | typed DropAnchor与source-authority stable order |
| ED60-P1-22 transform policy | Open | reparent command只改parent/local transform不变 | keep-world/keep-local，negative/nonuniform/singular/pivot/multi-root policy |
| ED60-P1-23 owner/instance/reference integrity | Open | 无foreign owner、instance、lock、level/layer/folder、reference/animation repair | ReparentPlanner冻结完整authoring integrity plan |
| ED60-P1-24 observability与qualification | Partial | fragment/filter/materialization/10K counters和tests已有 | operation/latency/reflow/drop/stale/rollback/retirement/100K/1M/GUI receipts |

## 6. P2：长期成熟度

| Finding | 状态 | 当前证据与剩余缺口 |
|---|---|---|
| ED60-P2-01 fake `expanded` hard cut | Open | 旧property仍存在，尚无真实view state |
| ED60-P2-02 numeric kind/control/URI bridge删除 | Open | hierarchy callback仍使用数字kind、literal control ID和URI |
| ED60-P2-03 remote/PIE/server World comparison | Partial | Edit/Play domain和active Play hierarchy query已存在；并排比较、read-only remote和cross-world bridge缺失 |
| ED60-P2-04 collections/smart folders/bookmarks | Open | 无产品owner；不得用Runtime parent模拟 |
| ED60-P2-05 multi-user/presence/edit lease/conflict | Open | 无协作session和qualified hierarchy operation identity |
| ED60-P2-06 million-item indexed/paged provider | Partial | 10K pointer/patch和固定control证明局部可扩展；1M、paged unloaded descriptor和World Partition provider缺失 |
| ED60-P2-07 redacted receipt/replay corpus | Open | 没有可导出Outliner interaction receipt与redaction policy |
| ED60-P2-08 headless query/validation API | Open | automation不能消费统一provider/plan，因为该产品尚不存在 |

## 7. 五引擎参考约束

| 参考 | 本地源码事实 | 对Zircon的硬约束 |
|---|---|---|
| Unreal Scene Outliner | item有stable ID/flags，Hierarchy负责items/children/parent，Mode负责parse/ValidateDrop/OnDrop；TreeView通过DetectDrag进入operation；pending add/move/remove按每帧预算处理并按ID恢复expansion | press不能直接变成drag；item/provider/mode/column/filter是typed边界；drop先验证；结构delta必须有预算 |
| Godot SceneTree | scene root identity变化会reset current tree；drag使用threshold/cancel；reparent围绕UndoRedo处理owner、instance、顺序、transform、reference和animation | World replacement先退休交互；reparent必须是完整authoring plan和单事务 |
| Fyrox World Viewer | provider抽象root/children/parent/name/icon/selection/mutation；filter保留祖先；per-scene expansion/breadcrumb/locate；DropAnchor区分OnTop/Side | Rust实现同样应使用provider、stable handle、view state和typed anchor |
| Bevy ECS hierarchy | ChildOf/Children由relationship合同维护；Entity由index+generation组成，stale generation拒绝；target collection不允许任意公开写 | Runtime关系authority不可被Editor副本绕过；item identity必须拒绝stale generation |
| Unity Graphics corpus | 本地仅有Converter工具：`MultiColumnTreeView`、显式filter/expanded state，以及bind时注销previous callback再注册current item | 只作为通用tree/cell证据；不能虚构闭源Unity Scene Hierarchy语义 |

共同基线是qualified identity、provider-owned hierarchy、drag detection、drop preflight、stable view state、transactional reparent和owner lifecycle retirement。目标不是复制类名，而是补齐不能删减的责任。

## 8. 目标架构与hard cut

```text
Runtime SceneGraphAuthority
  WorldSessionId / WorldGeneration
  typed relationship transaction
  hierarchy/participation/inspection delta + invalid topology diagnostics

Editor SceneDocumentSessionRegistry
  DocumentKey -> World lease / history / selection / view state

WorldOutlinerRegistry
  OutlinerProvider / OutlinerMode / OutlinerColumn / OutlinerFilter
  OutlinerItemId { provider, owner scope, local stable id, generation }
  OutlinerProjectionGeneration + typed range deltas

OutlinerViewState
  expanded IDs / active row / range anchor / scroll anchor / filter / sort / columns

HierarchyInteractionController
  Idle -> PressedCandidate -> Dragging(capture + transfer lease)
  Idle -> RenamePending(rename lease + draft + IME)
  every path -> Terminal(receipt)

ReparentPlanner
  validate owner/instance/lock/cycle/no-op
  resolve on/before/after + sibling order + transform/reference/selection policy
  immutable plan -> one Editor transaction -> receipt
```

必须硬切：

1. 删除`scene://node/{id}`加旁路IDs的双authority，不保留compat decoder。
2. 删除`expanded = has_children`假合同；Runtime不保存per-user展开状态。
3. 删除`saturating u64 -> i64` identity投影；opaque key或checked rejection二选一。
4. 删除World无关的active drag/rename；所有interaction持owner lease并在World transition前terminal。
5. 保留固定物理control上限，但删除全树reflow作为结构变化的常规唯一策略。
6. 不新增旧Hierarchy DTO别名、shim event或Editor私有folder/visibility authority。

## 9. 分层实施顺序

| Milestone | 交付内容 | 退出条件 |
|---|---|---|
| ED181-M0 | P0 interaction stop-the-line | 未形成drag的release不改World；replacement同步retire；旧event fail closed |
| ED181-M1 | qualified identity/document owner | document/world/session/item/interaction generation贯穿全链；clamp删除 |
| ED181-M2 | typed Outliner contract | provider/mode/column/filter接入Entity provider；row字段和diagnostic无损 |
| ED181-M3 | view/input/rename state | expansion/range/scroll持久化；capture/cancel；rename draft/IME/error/retirement |
| ED181-M4 | reparent planner/transaction | preflight/decorator/anchor/order/transform/owner/reference政策与receipt |
| ED181-M5 | incremental projection/query | topology range delta、frame budget、indexed cancelable filter、paged provider |
| ED181-M6 | observability/fault/scale | 100K/1M、multi-window/World churn、fault/soak和comparative receipt |

依赖不可颠倒。M0先封住误操作和跨World写入，M1建立身份后才能安全持久化view和interaction；M4必须复用现有Runtime hierarchy与Editor transaction authority；M5不能用whole-tree clone掩盖产品缺口。

## 10. 40项资格门重判

### 10.1 M0/M1：安全、身份与生命周期

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01 未越threshold的release不reparent | Fail | press即武装，release即commit |
| G02 threshold + qualified capture后才Dragging | Fail | 两者都不存在 |
| G03 cancel/focus/capture/window close单terminal receipt | Fail | hierarchy无terminal controller/receipt |
| G04 document/World/project/plugin transition退休交互 | Fail | replacement不清active drag/rename |
| G05 World A旧event不能改World B同值ID | Fail | event/intent只含裸ID |
| G06 message到receipt共同owner generation | Fail | artifact generation不等于owner identity |
| G07 全u64无碰撞/adapter checked | Partial | dispatch无损map存在，clamped property仍存在 |
| G08 terminal幂等 | Partial | `take()`阻止重复release二次提交，但无qualified terminal receipt |

### 10.2 M2/M3：模型、视图与交互

| Gate | 状态 | 当前证据 |
|---|---|---|
| G09 多provider typed items | Fail | typed Outliner contract为0 |
| G10 add/remove/move/reorder/update delta | Partial | Runtime delta有add/remove/change与reflow marker；Editor topology仍full reflow |
| G11 kind/active/children/warning/owner无损 | Partial | Runtime前三项存在，pane与warning/owner缺失 |
| G12 invalid topology typed diagnostic | Fail | fallback普通root |
| G13 expansion/reveal/breadcrumb/restore | Fail | fake expanded |
| G14 filter/sort恢复view anchors | Fail | 无OutlinerViewState |
| G15 Ctrl/Shift/hidden selection policy | Partial | SelectionModel支持extend/toggle，Hierarchy adapter不传modifier/range |
| G16 qualified double click | Fail | 同ID + 500 ms |
| G17 rename完整validation | Fail | trim/nonempty |
| G18 rename失败保draft/IME | Fail | command前clear focus |
| G19 visibility/lock/active/loaded transaction | Fail | row无这些产品列 |
| G20 recycled cell不保留旧callback | Pass | 10个generic controls按最新entity map路由，无逐item callback闭包 |

### 10.3 M4：drop与authoring integrity

| Gate | 状态 | 当前证据 |
|---|---|---|
| G21 单一qualified transfer authority | Fail | URI + Vec IDs双写 |
| G22 drag-over typed accept/reject decorator | Fail | 无drag-over validation |
| G23 self/cycle/no-op/lock/foreign/read-only preflight | Partial | Runtime晚期检查self/cycle/no-op/static；其余及preflight缺失 |
| G24 on/before/after和stable sibling order | Fail | 只有parent/root |
| G25 keep-world/local复杂transform | Fail | 只保留local字段值 |
| G26 parent-child normalization与确定顺序 | Pass | top-level root折叠、ordered source vector和Runtime stable order存在 |
| G27 owner/provenance/reference/animation/selection receipt | Fail | plan/receipt不存在 |
| G28 全plan原子rollback | Partial | parent-only command group和selection可rollback；order/transform/reference未纳入 |
| G29 undo/redo使用冻结command state | Pass | UpdateNodeCommand保存before/after，不重做drop hit-test |
| G30 large reparent预算/进度/cancel | Fail | UI callback同步无预算执行 |

### 10.4 M5/M6：规模、故障与产品资格

| Gate | 状态 | 当前证据 |
|---|---|---|
| G31 物理row/cell上限与总item无关 | Pass | static controls固定10，paint visible-range |
| G32 topology只触及affected range | Fail | add/remove/move/reorder完整reflow |
| G33 indexed/cancelable/stale-safe filter | Fail | 全量同步scan/clone |
| G34 100K loaded预算 | Fail | 最高聚焦证据5K/10K且无冻结预算 |
| G35 1M mixed paged/lazy provider | Fail | provider/page均不存在 |
| G36 multi-window/document隔离 | Partial | Edit/Play domain隔离存在；并行document/window interaction state缺失 |
| G37 gap/provider failure/teardown/OOM恢复 | Partial | fragment gap可resync；provider/teardown/OOM产品状态缺失 |
| G38 默认脱敏operation metrics | Partial | fragment/filter/materialization counters存在；operation/terminal/rejection缺失 |
| G39 真实Editor窗口产品验收 | Fail | 未形成完整交互和视觉反馈矩阵 |
| G40 同语义跨引擎benchmark | Fail | 未建立 |

## 11. 验证矩阵

| 层级 | 必须新增或修正的验证 | 当前状态 |
|---|---|---|
| Unit | qualified identity、state transition、drop anchor、rename validation、view state | 缺失；现有drag测试固化press即武装 |
| Property/fuzz | random tree move/cycle/order、stale generation、Unicode/IME、event reorder/dup | 缺失 |
| Transaction | multi-node order/transform/reference/selection原子rollback | parent-only cycle rollback已有基础 |
| Integration | World replace、project switch、provider reload、multi-document/window | fragment/reflow存在，interaction lifecycle缺失 |
| UI | threshold/capture/decorator/rejection、range、rename draft/IME、expansion/filter restore | 缺失 |
| Scale | 100K/1M memory、filter/scroll/churn、logical model与cell peak、p95/p99 | 仅5K filter、10K pointer/patch/paint |
| Fault/soak | provider failure、gap、focus/capture loss、World churn、OOM admission | gap resync局部存在，其余缺失 |
| Comparative | 冻结版本/硬件/场景/正确性下与Unreal/Godot/Fyrox对比 | 未建立 |

## 12. 审查限制与完成定义

本轮只新增review文档和索引，没有修改production Rust、ZUI或测试，也没有运行Cargo。没有启动真实Editor、注入pointer/capture/IME、执行跨World迟到事件、检查像素、运行100K/1M、fault/soak或跨引擎benchmark。253个静态test attributes不能转写为253项通过。

`review_status: current_source_refresh_complete`只表示Editor60的canonical finding已按本轮199份Zircon选择集和18份参考文件重判，不表示两条P0关闭或World Outliner production ready。只有40项门取得source-bound、current-generation、可复现receipt，并同步关闭Editor03/41/55、Runtime24/110/111等唯一owner的相关状态后，才能把`implementation_status`改为complete。Tooling按用户要求排除；本轮未查询、轮询、等待或实时跟踪协调器。
