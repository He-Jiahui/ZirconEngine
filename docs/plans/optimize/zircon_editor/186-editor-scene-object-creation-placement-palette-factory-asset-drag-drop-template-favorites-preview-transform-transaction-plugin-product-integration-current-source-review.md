---
title: Editor Scene Object Creation、Placement Palette、Factory、Asset Drag/Drop、Template、Favorites、Preview、Transform、Transaction、Plugin 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor186
review_date: 2026-08-27
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/extension/store
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/assets/ui/editor/asset_browser.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_drag_overlay.zui
tests:
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_metadata_and_fields.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/176-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-current-source-review.md
  - docs/plans/optimize/zircon_editor/178-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_editor/180-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/182-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/185-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/EditorFramework/Public/Subsystems/PlacementSubsystem.h
  - dev/UnrealEngine/Engine/Source/Editor/EditorFramework/Private/Subsystems/PlacementSubsystem.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PlacementMode/Public/IPlacementModeModule.h
  - dev/UnrealEngine/Engine/Source/Editor/PlacementMode/Private/PlacementModeModule.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PlacementMode/Private/SPlacementModeTools.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/ActorFactories/ActorFactory.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Factories/ActorFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Editor/ActorPositioning.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Editor/ActorPositioning.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/DragAndDrop/AssetDragDropOp.h
  - dev/godot/editor/gui/create_dialog.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/editor/src/menu/create.rs
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/Fyrox/editor/src/world/mod.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/from_world.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/bundle.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/lib.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentListEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/Volumes/VolumeCollectionTests.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Object Creation、Placement 与 Factory 当前源码复核

## 1. 结论

Editor65识别的根问题仍然成立：Zircon当前只有“固定菜单选择一种`NodeKind`，在当前World创建一个`NodeRecord`”的primitive，没有工程级Scene Object Placement产品。仓库中仍不存在`PlaceableCatalog`、`SceneObjectFactory`、`SceneConstructionPlan`、`PlacementSession`、`PlacementTransformSolver`或`PlaceSceneObjectCommand`等对应协议；资产drag payload也没有Scene/Viewport消费者。

当前源码并非毫无进展。Runtime的DynamicScene spawn已经形成隔离编译、preflight、publish和`EntityRemap`基础，`DetachedEntityBatch`能精确detach/restore子树；Editor主Scene mutation进入Document history和world route校验，create失败补偿、journal codec registry、extension immutable snapshot/revoke、viewport grid/angle/local-global transform primitive都比Editor65基线更完整。这些能力应被复用，不能另造平行机制。

但这些底座尚未连成统一的类型目录、工厂生命周期、目标地址、预览会话、资产投放、模板构造和精确提交receipt。当前canonical状态为：**0项新增P0；18项P1中9 Open / 9 Partial / 0 Closed；6项P2中5 Open / 1 Partial / 0 Closed；48门中30 Fail / 18 Partial / 0 Pass**。Editor65继续作为唯一finding owner，Editor186只刷新当前状态，不重复增加finding数量。

本轮只做静态review与重构计划，不修改生产代码，不运行Cargo、Editor、GUI、fault/scale/profile或跨引擎benchmark。Tooling按用户要求排除；未查询、轮询、等待或实时跟踪协调器。现有证据不足以声明功能、表现或性能达到或优于Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree是本轮证据源

主仓HEAD在冻结时为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。共享checkout包含其他Session的在途修改，且focused Runtime/Editor源码中也有dirty或untracked内容；因此本报告以**读取时磁盘上的current working tree**为事实源，而不是假设HEAD等于当前实现。未覆盖、回退或整理这些他人修改。

MVP `00-current-source-baseline-recovery`仍为`in_progress`，F4不能绕过F0-F3资格门。本报告可用于后续RED基线和分层重构，但不是高级实现授权，也不是动态验收receipt。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Runtime creation/construction | **7 / 2,220 / 87,624 / 1** | `NodeKind`、`NodeRecord`、bootstrap、detached batch、DynamicScene spawn transaction | `9cd58bf7c692f48925888f6bad9ee6fa08614af6fb351b34baa368f06cd43f91` |
| Zircon Editor command/catalog | **16 / 4,629 / 161,637 / 10** | create intent/command、admission、journal codec、fixed command/id/binding、extension store | `ddbc092d46f83373a46ac17e4a84905ab80e336e6b3722c3fcfbd7590685868b` |
| Zircon Editor product/drop/viewport | **21 / 4,097 / 179,214 / 5** | asset drag payload/consumer、Quick Import、viewport input/session/settings/math、ZUI | `b12aea63a105f2eb166f1db1638daf8607438b9df5d03a72f6cfc19ccb3f8bee` |
| Zircon focused tests | **12 / 5,383 / 188,393 / 115** | create/undo/journal/import、DynamicScene、save/reopen、drag source/reference field | `0412d134ebee2eee4fef6fa2ffd6367e098bdd1d3603d0c0265478520c329731` |
| Unreal selected set | **10 / 5,642 / 206,673 / 0** | placement subsystem、factory、category/palette、asset drag、positioning | `fcfb21a189c29d027745acffc998e9193f68db9ed96196f7d7695a1726d78ff5` |
| Godot selected set | **3 / 13,690 / 514,193 / 0** | searchable creation、favorite/recent、PackedScene、3D preview/drop/Undo | `b89dcdde18e1f518435ee80525c77883f7514c2023413ebfcf63834b7429ffc8` |
| Fyrox selected set | **4 / 3,048 / 103,912 / 0** | constructor menu、preview/drop、pick/plane/grid、reversible subgraph | `3043c9d7c4708f5e51c0c754aa554dd9ca59b5a98524eb5b4c0cd0b862f30b22` |
| Bevy selected set | **5 / 4,762 / 166,026 / 49** | reflect construction、Scene/DynamicScene、entity remap与failure cleanup | `fcf3718bd091faa32d1d5bd6bdbf0076ef8dafc37c0ee77536e94ac5abea4f2a` |
| Unity Graphics selected set | **5 / 2,270 / 90,722 / 7** | Volume type catalog、add/remove/order/reset、Undo与runtime collection | `7a0272e9ac8ed0d32dc7a6122ba04c22ab1315c6f9b4a5bc31d32fbfe1849df8` |

fingerprint计算方法为：规范化相对路径，对每个文件计算SHA-256，形成`path::file_hash`，按路径排序并以当前环境换行连接，再对整体计算SHA-256。它只证明选择集的current-source内容，不代表ABI、构建产物、动态行为或性能。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor186只审查“发现可放置对象 -> admission -> preview/transform -> commit/undo/replay”的产品闭环。Command基础由Editor178拥有，portable payload由Editor176拥有，viewport input/pick由Editor180拥有，parent/destination由Editor181拥有，Scene Document由Editor182拥有，transaction/journal由Editor184拥有，component graph由Editor185拥有，Runtime ECS/World/hierarchy由Runtime99i至99k拥有。本报告只定义placement对这些父合同的消费要求，不复制其finding。

## 3. 当前实现拓扑

### 3.1 Runtime仍是闭集primitive

`NodeKind`仍是九种variant的闭集。`NodeRecord`继续以大量optional builtin component表示对象；`World::spawn_node(kind)`调用hard-coded `default_node_record`，`spawn_mesh_node`先创建Mesh再替换Name/MeshRenderer，`World::new`硬编码Camera、DirectionalLight和Cube。这条路径可作为最终World mutation内层primitive，但不能表达插件/script/template identity、多实体图、目标parent/order/owner、preview owner、资源lease或失败清理。

DynamicScene的spawn transaction已经能先编译entity/component/resource writes和remap，preflight publication，再统一publish并返回`EntityRemap`。`DetachedEntityBatch`能精确保留并恢复子树。这两项是未来`SceneConstructionPlan`和exact receipt的重要材料，但前者当前是whole DynamicScene导入/克隆路径，后者只被Editor delete undo使用；create仍没有消费它们。

### 3.2 Editor create仍是一条单记录链

当前调用模型仍是`fixed command -> CreateNodeIntent::{Node, Mesh} -> CreateNodeCommand -> World::spawn_* -> one NodeRecord -> one selection`。首次apply保存一个record，undo移除一个entity，redo插回旧record；这避免redo重新生成单节点默认值，却无法守恒root+children+dynamic components+subobjects+references构成的复杂图。

主Scene create已进入Document history，`EditWorldRoute`能绑定gateway identity并拒绝stale world，Play状态也拒绝mutating edit。gateway post-callback或selection失败时有补偿测试。这些是正确的事务底座，但请求本身仍没有document/world generation、parent/order/owner/layer、factory/template、asset版本、placement options或typed rejection上下文；UI create command的`WhenClause`仍是`Always`。

### 3.3 可见菜单收敛了一份，目录仍不存在

生产可见标签现在集中在`defaults.rs`的Cube、Camera和五类Light，Editor65所述另一份retained visible label list已不再是当前生产证据，因此映射漂移项从Open降为Partial。但`menu_item_binding`、`node_kind_id/from_id`和`menu_action_id/from_id`仍各自编码类型映射，新增对象类型仍需跨层修改。

仓库没有Scene placeable catalog、category/query/filter/favorite/recent/thumbnail/help、stable item key或catalog generation。固定command列表不是catalog，Command Palette也只能发现command，不能回答某个Scene target是否可构造某个对象。

### 3.4 Extension Store有生命周期底座，没有placement family

当前`ContributionSnapshot`已是immutable generation snapshot，ticket携source/capability，`ContributionBatch`能做原子验证；19类family覆盖view、menu、asset importer/type、scene mode、overlay、command和operation factory等，并有通用revoke框架。

但没有placeable category/item、scene object factory、placement filter或presentation link family。插件只能贡献通用command/operation并自行访问World，host无法把它纳入统一factory selection、preview、transaction、favorite/recent、in-flight quiesce和missing-provider恢复。该项是可复用框架Partial，不是placement产品完成。

### 3.5 Asset drag只到引用字段，Quick Import仍耦合

Asset Browser能建立`UiDragPayloadKind::Asset`及metadata，retained host维护`active_asset_drag_payload`，reference field consumer会消费并在terminal/invalid路径清理。测试覆盖arm、clear和reference drop。

Scene/Viewport没有drag-over/drop consumer；`ViewportInput`只处理pointer move/press/release、scroll、resize，`ViewportDragSession`只覆盖selection/orbit/pan/handle。`workbench_drag_overlay.zui`中的“Drop into scene”仍是prototype展示，不是产品事件。

`complete_model_import`在刷新Asset Manager并解析model/default material后立即调用`import_mesh_asset`，在当前World创建Mesh并改变selection。Import与Place仍没有独立request、receipt、目标document、partial-success和cancel语义，reimport与纯摄取也无法与场景修改可靠分离。

### 3.6 Viewport数学可复用，但没有placement session

Viewport已有Local/Global、GridMode、`SceneViewportSnapSteps`、axis projection、rotation delta、`maybe_snap`和pick/cancel primitive；现有`cancel_interaction`会终止handle/camera/selection交互。这些足以作为transform solver的部分输入。

当前没有placement-owned preview实体、transient exclusion、异步load lease、drag/click统一状态机、candidate transform receipt、surface/vertex/bounds/collision/factory offset求解、drop promotion或document/world/plugin replacement cleanup。普通interaction cancel不能证明不存在的placement session可以正确取消。

## 4. Canonical finding状态账本

### 4.1 P1架构与产品闭环

| Finding | 当前 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED65-P1-01 闭集`NodeKind`被误用为可创建类型权威 | Open | 九variant enum仍控制bootstrap和Editor创建 | stable `SceneObjectTypeKey`、descriptor、missing/deprecated type与迁移；Editor不复制类型事实 |
| ED65-P1-02 宽`NodeRecord`与hard-coded defaults不能表达构造图 | Partial | DynamicScene transaction和detached batch已能承载图/隔离提交，create未使用 | factory输出detached graph/plan，World原子提交并返回exact receipt |
| ED65-P1-03 菜单、action id与binding多份映射漂移 | Partial | visible labels已集中，但kind/action/id/binding仍多份match | 由同代catalog生成menu/action binding并做link validation/source guard |
| ED65-P1-04 没有Scene Placeable Catalog与查询合同 | Open | category/search/filter/favorite/recent及generation均不存在 | immutable generation-qualified catalog、paged query、disabled reason与tombstone |
| ED65-P1-05 没有统一Scene Object Factory注册、选择和撤销协议 | Open | 只有bootstrap match和generic operation factory | stable factory key/version/owner，preflight/prepare/commit/abort与deterministic selection |
| ED65-P1-06 创建请求/结果没有typed上下文和拒绝原因 | Partial | transaction/route错误已typed，create intent/result仍窄 | qualified request、typed admission/factory/commit rejection与structured receipt |
| ED65-P1-07 create command无条件可用，未统一admission | Partial | Play、active Scene document和stale world已有guard；UI仍`Always` | UI与runtime共享同一capability/admission snapshot，覆盖read-only/closing/reload/resource/provider |
| ED65-P1-08 destination缺document/world/parent/order/owner身份 | Partial | Document history和world route已存在，create request仍隐式current/root | `SceneConstructionDestination`携session/world generation和qualified parent/order/owner/layer |
| ED65-P1-09 没有工程级Placement Transform Solver | Partial | grid/angle/local-global/pick数学可复用 | 独立确定性solver整合ray/surface/vertex/bounds/pivot/collision/offset/fallback |
| ED65-P1-10 没有preview session创建、更新、取消和提升 | Open | 没有placement preview owner或状态机 | 明确session状态、capture/resource lease、transient exclusion及唯一terminal disposition |
| ED65-P1-11 asset drag payload没有Scene/Viewport consumer | Open | payload只被reference field消费 | host drag lease -> asset admission -> preview -> viewport drop transaction |
| ED65-P1-12 Quick Import错误耦合导入与场景实例化 | Open | model import完成后立即创建当前World Mesh | Import和Place双operation/双receipt，允许只导入、导入后放置和partial success |
| ED65-P1-13 template/archetype/prefab/script class不能统一创建 | Open | intent只有Node或model/material | builtin、asset、template、prefab、script和plugin都进入统一factory/construction plan |
| ED65-P1-14 单`NodeRecord`命令不能守恒复杂创建和失败清理 | Partial | detached batch/DynamicScene已存在；create仍保存单record | command持有exact batch、remap、dependency和selection delta，失败清除全部中间态 |
| ED65-P1-15 journal v1缺factory/template/schema和迁移身份 | Partial | codec registry有type/schema和typed decode error；create payload仍为intent+record | stable factory/template/schema/provider/destination identity及显式migration terminal result |
| ED65-P1-16 插件不能贡献并安全撤销placement能力 | Partial | Store已有ticket/snapshot/revoke框架，无placeable families | 加入category/item/factory/filter link及in-flight lease、quiesce、fault boundary |
| ED65-P1-17 没有batch/multi-asset placement与性能合同 | Open | create/import均单项 | grouped factory preflight、per-item disposition、atomic policy、progress/cancel和有界资源 |
| ED65-P1-18 测试不覆盖placement产品状态机 | Open | 115项focused tests仍没有viewport asset drop、preview、factory/plugin/template/batch | 先建立RED integration/fault/scale矩阵，再分层实现 |

### 4.2 P2质量与资格证据

| Finding | 当前 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED65-P2-01 默认命名与冲突策略缺失 | Open | 名称仍由`NodeKind` hard-code | destination naming service、stable seed、batch numbering和receipt final name |
| ED65-P2-02 icon/localization/help/tooltip/accessibility缺link合同 | Open | 没有placeable descriptor/linker | catalog compile阶段验证全部presentation引用与fallback |
| ED65-P2-03 favorite/recent/settings缺版本化持久化 | Open | 产品能力不存在 | stable-key、scope、容量、去重、missing provider和schema migration |
| ED65-P2-04 缺少结构化诊断与可观测receipt | Open | 只有局部command/status/error | 每阶段reason/timing/generation/count，不泄露资产内容 |
| ED65-P2-05 catalog/preview无缓存预算与benchmark | Open | 没有目标hot path和同语义profile | 10k/100k catalog、1k batch、60/120Hz preview预算和可复现profile |
| ED65-P2-06 modifier与placement policy没有统一设置模型 | Partial | snap steps/GridMode已有immutable authority | 建立versioned policy snapshot，所有menu/palette/drop入口共享，modifier只产生temporary override |

## 5. 五套参考实现的差异

### 5.1 Unreal：完整协议是主参考

`UPlacementSubsystem`接收单/多`FAssetPlacementInfo`和`FPlacementOptions`，选择并按factory分组，执行`BeginPlacement -> PrePlaceAsset -> PlaceAsset -> PostPlaceAsset -> EndPlacement`并返回element handles。`UActorFactory`提供资产准入、priority/class/extent/surface/offset；`FActorPositioning`处理trace、preview ignore、camera fallback、grid/vertex/surface、normal、extent与collision。

`IPlacementModeModule`对category/item提供register/unregister/filter和Favorites/RecentlyPlaced，`SPlacementModeTools`实现search/category/favorite/thumbnail/drag handler，`FAssetDragDropOp`保留单/多asset与factory metadata。Zircon应学习协议分层和生命周期，不复制UObject层级，也不能把Runtime构造事实放进Editor。

### 5.2 Godot：动态类型与资源实例化完整进入Undo

`CreateDialog`联合native/script/custom type，过滤abstract、feature profile、disabled addon与不可加载脚本，提供search/favorite/recent/help。`SceneTreeDock`对PackedScene批量load/instantiate/cycle validation，设置parent/name/owner并把完整创建纳入当前scene history；失败会清理已创建实例。3D viewport拥有preview、ray/grid/vertex/snap/cancel和parent-local transform。Zircon当前缺少的是这些能力之间的统一产品链，而非一个额外菜单。

### 5.3 Fyrox：较小实现也保留preview与subgraph守恒

Fyrox Create菜单来自constructor registry/variants/group。asset drag-over会实例化preview subgraph，pick时排除preview，空白处退回plane，再做grid snap；drop把preview取成reserved subgraph，用add/select command group提交。它没有Unreal完整catalog合同，但足以证明单`NodeRecord`与无preview drop低于工程基线。

### 5.4 Bevy：用作Runtime构造图下限

Bevy的FromWorld/ReflectBundle、Scene/DynamicScene/ResolvedScene、type registry和entity remap证明模板构造必须处理多实体、组件、相关scene和typed error；失败spawn会despawn，未insert component也有drop测试。Bevy不提供Zircon所需Editor placement UX，因此只用于Runtime construction/failure cleanup对照。

### 5.5 Unity Graphics：只借鉴typed catalog与Undo顺序

Volume编辑器按render pipeline构建可显示type/category，支持search、add/remove/reset/reorder/help和Undo，runtime profile维持组件collection语义。它适合校验typed component/catalog/Undo纪律，不是Scene Object Factory或Viewport Placement参考，不能越界推导通用产品完成度。

## 6. 目标架构

### 6.1 Runtime唯一构造权威

- `SceneObjectTypeKey`和`SceneObjectFactoryKey`：稳定namespace/name/version identity，支持missing/deprecated/migration。
- `SceneObjectDescriptor`与`FactoryRegistrySnapshot`：owner-qualified immutable generation，声明asset/type/capability、factory、naming seed和construction schema。
- `SceneConstructionDestination`：document session之外的Runtime world generation、qualified parent/order/owner/layer/partition事实。
- `SceneConstructionRequest -> Plan -> Receipt`：preflight固定factory/dependency/expected generation，prepare只产生隔离graph，commit由World复验并原子发布。
- `SceneConstructionReceipt`：exact roots/entities/components/references/remap/final transform/name/provider generations和terminal disposition；可转为`DetachedEntityBatch`供undo/redo守恒。
- `SceneObjectFactory`：`preflight/prepare/commit/abort`，每个callback有fault boundary、resource lease与幂等cleanup。

Runtime不得依赖palette/favorite/viewport widget；Editor不得用`NodeKind`、JSON或多次`spawn_node`复制可构造事实。

### 6.2 Editor authoring与产品层

- `PlaceableCatalogSnapshot`：链接Runtime descriptor、Asset Catalog和extension presentation，generation-qualified、可分页、可诊断。
- `PlacementAdmission`：menu、palette、asset drag和shortcut共享，输出typed reject或固定factory/request。
- `PlacementSession`：`Idle -> Admitting -> PreviewLoading -> Previewing -> CommitPending -> Committed|Cancelled|Rejected|Faulted`，每个终态释放capture、preview和resource lease。
- `PlacementTransformSolver`：消费qualified pick、viewport/camera、policy、factory extent，输出确定性candidate与解释；Runtime在commit时复验。
- `PlaceSceneObjectCommand`：绑定Document history，保存plan/receipt/exact batch与selection delta，redo不重新执行已变化/卸载factory。
- `PlacementPreferenceStore`和产品surface：stable-key favorite/recent/settings；menu/palette/drop/accessibility只投影同一catalog/session。

### 6.3 Commit时序

1. UI或asset payload从同代catalog解析item/factory。
2. Editor固定document/world/destination generations并请求Runtime preflight。
3. Runtime返回plan或typed rejection；Editor加载placement-owned transient preview。
4. 输入更新由solver生成candidate，preview不进入save/history/普通pick与非预览systems。
5. drop/click启动一次document transaction；Runtime复验全部generation并原子commit。
6. receipt驱动selection、history、dirty、recent和diagnostic；任何失败走abort并清理。
7. undo detach exact batch，redo恢复receipt内容，不重跑factory或重新解释模板。

## 7. 分阶段重构计划

### Editor186-M0：能力真实性与RED基线

冻结当前`NodeKind`、菜单/id/binding和Quick Import调用点；把prototype “Drop into scene”明确标成Unavailable；先补viewport asset drop、preview cancel/cleanup、destination replacement、factory failure和save exclusion RED tests。依赖Editor176/180/182/184的当前合同，不修改其owner。

### Editor186-M1：Runtime identity与factory registry

建立stable type/factory key、descriptor、owner generation、immutable snapshot和typed registration error；builtin类型也走同一registry。禁止新增Editor端`NodeKind`产品穷举，并测试duplicate、missing factory、stale generation、revoke和capability admission。

### Editor186-M2：Construction plan、隔离graph与receipt

把DynamicScene compile/preflight/publish和`DetachedEntityBatch`收敛为可复用construction transaction材料；factory生成detached graph，World原子提交多实体/component/reference并返回exact receipt。对每个阶段做fail/panic/cancel cleanup测试。

### Editor186-M3：Placement session与transform solver

建立click/drag/keyboard统一session、capture、preview owner和唯一终态；整合qualified pick、surface/grid/angle/vertex、bounds/pivot/collision/factory offset、camera fallback及parent-local transform。document/world replacement、focus loss、viewport leave、Escape和shutdown必须取消。

### Editor186-M4：Asset、template、prefab、script统一入口

拆开Import和Place receipt；asset drop按type和priority选择factory。Builtin、mesh/audio/material应用、template/prefab/script/plugin都输出同一种construction plan；default/override传播仍由其canonical owner提供artifact。

### Editor186-M5：Document transaction、journal与恢复

`PlaceSceneObjectCommand`绑定qualified document/world/destination；history保存exact batch，journal保存stable factory/template/schema/provider和migration identity。覆盖undo/redo、savepoint、save/reopen、crash recovery、cross-document rejection及provider升级/缺失。

### Editor186-M6：插件生命周期与产品catalog

在Extension Store增加placeable category/item/factory/filter/presentation link，复用ticket/snapshot/revoke并补in-flight lease、quiesce/fence和callback fault isolation。实现search/category/favorite/recent/thumbnail/help、disabled reason和virtualized results；所有入口只消费同一snapshot。

### Editor186-M7：规模、故障、表现和性能资格

运行10k/100k目录、1k mixed batch、60/120Hz preview、delayed asset load、factory panic、plugin reload、document switch和memory pressure矩阵。在相同资产、视口、碰撞、snap、release build和硬件上与参考引擎比较p50/p95/p99、allocation、CPU/GPU和memory；48门全部Pass之前不得宣称工程级完成或优于Unreal。

## 8. 资格门

| Gate | 验收条件 | 当前 | 当前证据 / 缺口 |
|---|---|---|---|
| ED65-G01 | builtin/plugin/script/template有stable type key | Fail | 仍由闭集`NodeKind`控制 |
| ED65-G02 | factory有stable key、version与owner generation | Fail | placement factory不存在 |
| ED65-G03 | catalog是immutable generation-qualified snapshot | Fail | Extension snapshot不能替代不存在的placeable catalog |
| ED65-G04 | 重复type/factory/category/item注册Fail closed | Fail | 无对应registration family |
| ED65-G05 | descriptor与factory/icon/help/localization link可验证 | Fail | 无descriptor/linker |
| ED65-G06 | provider撤销后新查询无stale item | Fail | 无placement query/revoke产品链 |
| ED65-G07 | missing/deprecated provider有tombstone策略 | Fail | 无stable item identity |
| ED65-G08 | Editor不维护`NodeKind`产品目录副本 | Fail | defaults/id/binding仍穷举 |
| ED65-G09 | 所有入口共享typed placement admission | Fail | menu create与Quick Import各自直达create |
| ED65-G10 | request携document/world/destination generations | Partial | history有Document、route有world identity；request仍未携带 |
| ED65-G11 | request携factory/template/asset/options identity | Fail | intent只有kind或model/material |
| ED65-G12 | Runtime commit前复验expected generations | Partial | route stale reject与DynamicScene preflight可复用，create无plan复验 |
| ED65-G13 | factory支持preflight/prepare/commit/abort | Fail | factory协议不存在 |
| ED65-G14 | factory callback fault隔离且typed | Fail | generic Store基础不能覆盖不存在的placement callbacks |
| ED65-G15 | partial prepare失败清理全部中间对象 | Partial | DynamicScene隔离编译/发布和command compensation存在，未覆盖factory graph |
| ED65-G16 | commit返回exact construction receipt | Partial | `EntityRemap`和单record保留存在，无统一graph receipt |
| ED65-G17 | target parent/order/owner/layer明确可复验 | Partial | hierarchy/parent primitive存在，create destination仍隐式root |
| ED65-G18 | read-only/closing/reloading/play同源拒绝 | Partial | Play和active Scene guard存在，其余状态/入口未统一 |
| ED65-G19 | asset type到factory选择确定可解释 | Fail | Quick Import硬编码model路径 |
| ED65-G20 | multi-asset按factory分组并保留per-item disposition | Fail | 仅单项create/import |
| ED65-G21 | viewport drag-over消费typed asset payload | Fail | consumer为0 |
| ED65-G22 | pointer up/drop/cancel有唯一terminal disposition | Fail | placement state machine不存在 |
| ED65-G23 | preview有transient owner且不进save/history | Fail | placement preview不存在 |
| ED65-G24 | preview从普通pick/render/system正确隔离 | Fail | 无preview身份/过滤 |
| ED65-G25 | leave/Escape/focus loss清理preview | Partial | generic viewport cancel存在，不覆盖placement资源 |
| ED65-G26 | document/world replacement终止旧session | Partial | world route generation可拒旧操作，无placement session退休 |
| ED65-G27 | plugin unload等待或取消in-flight placement | Fail | 无factory/session lease |
| ED65-G28 | pending asset load可取消且lease有界 | Fail | 无placement load生命周期 |
| ED65-G29 | ray/surface/grid/angle/vertex snap确定 | Partial | grid/angle/axis/pick primitive存在，surface/vertex统一solver缺失 |
| ED65-G30 | bounds/pivot/collision/factory offset进入求解 | Fail | 无placement geometry contract |
| ED65-G31 | 空白viewport有camera/plane fallback | Fail | 无placement fallback路径 |
| ED65-G32 | parent-local transform无漂移 | Partial | local/global和hierarchy数学可复用，未形成placement golden matrix |
| ED65-G33 | create/move/select/dirty为一次document transaction | Partial | 单node create/select已进history，缺move/destination/receipt完整原子性 |
| ED65-G34 | undo保存exact多实体batch而非单record | Partial | detached batch存在并服务delete，create仍单record |
| ED65-G35 | redo不重跑变化/卸载factory | Partial | 单record redo不重跑bootstrap，尚无factory/graph/provider语义 |
| ED65-G36 | journal记录stable factory/template/schema identity | Partial | codec有command/schema，payload缺placement identity |
| ED65-G37 | journal migration失败给typed terminal error | Partial | decode/unregistered error typed，无placement migration |
| ED65-G38 | cross-document/world replay被拒绝 | Partial | Document history和stale world route存在，journal destination未qualified |
| ED65-G39 | save/reopen保留graph、asset refs与ownership | Partial | 基础Scene mesh binding round-trip存在，复杂construction未覆盖 |
| ED65-G40 | fault后无preview或half-created对象泄漏 | Partial | create compensation和DynamicScene preflight存在，无session/factory fault矩阵 |
| ED65-G41 | Import与Place有独立receipt/partial success | Fail | Quick Import仍直接实例化 |
| ED65-G42 | prefab/template/script/builtin共享factory协议 | Fail | 统一协议不存在 |
| ED65-G43 | favorite/recent只用stable key并可迁移 | Fail | 产品能力不存在 |
| ED65-G44 | recent只在成功commit后更新 | Fail | recent与commit receipt均不存在 |
| ED65-G45 | 10k/100k catalog分页/虚拟化且预算内 | Fail | catalog不存在 |
| ED65-G46 | 1k batch无逐项全量snapshot/sort/clone | Fail | batch路径不存在 |
| ED65-G47 | 60/120Hz preview满足p95预算 | Fail | preview路径和预算不存在 |
| ED65-G48 | 同语义跨引擎功能/表现/性能receipt可复现 | Fail | 未运行动态或同硬件benchmark |

## 9. 验证与当前源码守卫

本轮静态守卫确认：目标架构核心名称`PlaceableCatalog`、`SceneObjectFactory`、`SceneConstruction`、`PlacementSession`、`PlacementTransform`、`PlaceSceneObject`在当前仓库均为0；Scene/Viewport对`UiDragPayloadKind::Asset`的placement consumer为0；create command仍只保留一个`NodeRecord`；Quick Import仍把asset import完成与Scene Mesh创建串联。上述“0”只针对当前选择集和仓库文本，不代表未来分支或未落盘代码。

后续实现的最低验证矩阵应包含：stable key/registry/link/migration unit tests；menu/palette/asset drag一致性；builtin/template/prefab/script/plugin construction parity；preview update/cancel/save/pick exclusion；multi-document/world replacement；plugin unload/factory panic/asset delay/partial batch；exact undo/redo/save/reopen/crash recovery；10k/100k catalog、1k batch和60/120Hz preview profile。

本轮没有运行Cargo或GUI，因此没有宣称build green、行为green或性能green。报告落盘只执行frontmatter path、finding/gate计数、索引唯一性、fingerprint currentness和`git diff --check`等静态校验。

## 10. 最终判定

当前Zircon的create primitive和周边基础可以保留，但不能继续以增加`NodeKind`分支、复制菜单映射、直接`spawn_node`、把prototype drop文案接到单record command等方式扩展。这会把插件、模板、预览、undo、journal和性能问题继续固化在错误边界上。

正确收敛顺序是：**Runtime stable type/factory identity与construction transaction -> exact graph receipt -> Editor qualified placement session/solver -> asset/template统一入口 -> document transaction/journal -> plugin catalog lifecycle -> scale/fault/performance资格**。在该链完成并让48门全部Pass前，本功能应保持“工程化重构待实施”，不能标记为完整Scene Object Placement。
