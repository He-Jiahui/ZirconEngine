---
title: Editor Scene Object Creation、Placement Palette、Factory、Asset Drag/Drop、Template、Favorites、Preview、Transform、Transaction、Plugin 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor65
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/ui/workbench/state
  - zircon_editor/src/ui/retained_host/app/asset_drag_payload
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/src/ui/retained_host/menu_pointer
  - zircon_editor/assets/ui/editor/asset_browser.zui
tests:
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs
  - zircon_editor/src/tests/editor_event/runtime/registry.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_browser.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/asset_metadata_and_fields.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/64-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
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
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/65-editor-scene-object-creation-placement-palette-factory-asset-drag-drop-template-favorites-preview-transform-transaction-plugin-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Object Creation、Placement Palette、Factory、Asset Drag/Drop、Template、Favorites、Preview、Transform、Transaction、Plugin 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前确实能创建场景节点：菜单把七种固定`NodeKind`送入`EditorIntent::CreateNode`，`CreateNodeCommand`调用Runtime `spawn_node`，记录一个`NodeRecord`并支持撤销/重做；Asset Browser的Quick Import还能导入模型后调用`spawn_mesh_node`。资产侧也已经有typed `UiDragPayloadKind::Asset`，视口有pointer路由，extension store能登记command、asset importer、scene mode和viewport overlay。这些都是应保留的局部底座。

但这不是工程级Scene Object Creation与Placement。产品权威仍是九项闭集`NodeKind`，默认名称、默认组件、菜单项、action id和binding在多处穷举复制；创建请求只携`kind`或`model/material`，没有目标document/world/parent、placement transform、factory、template、asset readiness、batch、preview、命名、owner或typed rejection。菜单创建条件为`Always`，视口完全不消费资产drag payload，Quick Import把资产摄取与场景实例化绑成一个动作。所谓“Drop into scene”只存在showcase overlay，没有生产placement session。

Unreal参考实现不是“多几个按钮”，而是`PlaceableItem + PlacementCategory + PlacementFactory + AssetPlacementInfo + PlacementOptions + PlacementSubsystem + ActorPositioning + AssetDragDropOp`的闭环：factory发现与校验、preview、批处理、目标level、surface/grid/vertex snap、bounds/collision offset、事务、recent/favorite和plugin注册属于同一合同。Godot和Fyrox也都至少将可构造类型、资产drop、预览、位置计算、父级选择与Undo串成实际产品路径。Bevy的新Scene/Template实现进一步证明多实体构造、依赖解析、typed error和失败清理必须在底层合同中存在。Unity Graphics只作为渲染专用type catalog/Undo参考，不把它误当通用场景放置系统。

Editor08、44、50、55、57、59、60、61、63、64及Runtime99i至99k已拥有command、template/default、extension、drag payload、asset、viewport、hierarchy、document、transaction、component和World父问题，本报告不重复抬高其P0。本轮新增 **0项P0、18项P1、6项P2**，并登记 **48个全部Fail的资格门**。目标架构是Runtime唯一权威的`SceneObjectTypeDescriptor + SceneObjectFactoryRegistrySnapshot + SceneConstructionPlan + SceneConstructionReceipt`，以及Editor持有的`PlaceableCatalogSnapshot + PlacementSession + PlacementTransformSolver + PlaceSceneObjectCommand + PlacementPreferenceStore`。Runtime负责真实World、类型事实、构造预检/提交/复验；Editor负责document、selection、interaction、preview编排与transaction。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、asset-to-viewport drop、preview/cancel/commit、save/reopen、plugin reload、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称功能、表现或性能达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Runtime creation | **3 / 321 / 12,595 / 0** | `NodeKind`、`NodeRecord`、default/spawn/bootstrap | `3be0b48f239f4549672ace3c05d3522ea3952b2000de0020d59fba888d7f608e` |
| Zircon Editor command/catalog | **12 / 3,113 / 113,659 / 10** | intent、command/journal、fixed command/menu/id/binding、extension batch | `51ac6de5c2464e555157e7cb8bec8bedbd63ed5af1fd3117b855ed5f06cd8ab7` |
| Zircon Editor product/drop | **12 / 1,528 / 81,177 / 0** | asset payload、pointer lifecycle、reference consumer、Quick Import、viewport、ZUI | `ea0fc8f62b2a93fb9b33d81266eeaefd2480b91a1794326f47936a79f3aaa409` |
| Zircon focused tests | **8 / 2,323 / 77,909 / 51** | create/undo/journal/import、menu registry和drag source/reference-field tests | `21b51c958f5dae09653811f5011ebf057ca1626bb6272a50a08ad3235872dfc0` |
| Unreal selected set | **10 / 5,642 / 206,673 / 0** | placement subsystem/factory/category/palette/drag/positioning/transaction | `efd360e3b03ece477487a83182bc20d98d9d7598b6ec98afb423d86e0ae90a96` |
| Godot selected set | **3 / 13,690 / 514,193 / 0** | searchable creation、favorite/history、scene/asset drop、3D preview与Undo | `b9cfeaffe6cc5587ecb14d6ec9e50258310af7d43f804c7f949d93d2bf5a7a2e` |
| Bevy selected set | **5 / 4,762 / 166,026 / 51** | reflect construction、Scene/Template、dependency、bundle write、failure cleanup | `73d530f614b8c4fa76ead32f80865eca45153a0beb997860234291ccef45c11e` |
| Fyrox selected set | **4 / 3,048 / 103,912 / 0** | constructor menu、asset preview/drop、pick/plane/grid、reversible subgraph | `a5cae301b84d140b88277c6e6be4542293443cac78b065b3456fa2582debe83f` |
| Unity Graphics selected set | **5 / 2,270 / 90,722 / 9** | Volume type display、filter/category、add/remove/order/reset、Undo与runtime collection tests | `63019e522aafa6cd33167615fa919853f117e1bd9f49b9cd5340b1fa82fd27b0` |

fingerprint按规范化相对路径与逐文件SHA-256基于本轮working-tree内容计算，只证明所列源码被读取；它不是ABI、artifact、动态测试或性能receipt。主仓与Unreal镜像基线为`bee4c707b714738346b49bba15c59468b8bd9b39`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 在途修改隔离

共享checkout存在大量其他Session修改。本轮冻结的focused production文件在写报告前没有dirty项；相邻的showcase、Inspector component row、scene tree row与workbench reference builder存在非本轮修改，但未作为本报告placement实现证据，也未被覆盖或回退。coordinator Session为`optimize-editor65-object-placement-review-r1-20260822`，baseline epoch为339；报告与三个共享索引均取得精确lease且candidate plan没有open failure。

MVP `00-current-source-baseline-recovery`仍处于`in_progress`，F4不能绕过F0验收。本报告只做静态审查，没有用Cargo结果包装review结论。

### 2.3 范围与非范围

本报告覆盖“用户如何发现一种可放置对象，并从菜单、palette、资产或template开始，在Scene/Viewport中预览、定位、提交、撤销、重做与恢复”的完整产品链。UI Asset Editor自己的widget palette和Asset Creation Template属于各自document/asset域，不是Scene Object Placement的实现。

以下父域只引用、不重复建账：command registry归Editor08；prefab/archetype/default归Editor44；provider生命周期归Editor50；portable drag/clipboard payload归Editor55；Asset Browser归Editor57；viewport input/picking归Editor59；parent/drop destination归Editor60；Scene Document归Editor61；transaction/journal归Editor63；component graph归Editor64；Runtime ECS/World/hierarchy归Runtime99i至99k。

## 3. 当前实现拓扑与可保留基础

### 3.1 Runtime创建入口是真实primitive，但请求模型过窄

`World::spawn_node(kind)`分配`NodeId`、调用`default_node_record`并更新generation/facts；`spawn_mesh_node(model, material)`先生成`Mesh`节点，再替换Name与MeshRenderer。`World::new()`还硬编码Camera、DirectionalLight和Cube。它证明World拥有最终实体创建权，这一点应保留。

但`default_node_record`和`default_name`都对`NodeKind`穷举。Camera、Light、Cube/Mesh的默认transform/component写死在bootstrap中。接口无法表达目标World/level、parent/order、预览或正式实例、factory/template identity、asset dependency、name policy、placement transform、plugin owner、batch或失败清理；因此它只能是内层primitive，不能继续作为Editor公开创建合同。

### 3.2 Editor路径是“一次点击、一个枚举、一个记录”

当前调用链为：

`fixed menu item -> MenuAction::CreateNode(NodeKind) -> EditorIntent::CreateNode(NodeKind) -> EditorCommand::CreateNode -> World::spawn_node -> select one NodeId`。

`CreateNodeCommand`第一次apply保存一个`NodeRecord`，undo移除一个实体，redo重新插入该record。Mesh import只是同一命令中的第二种私有intent。journal schema `zircon.editor.scene.create_node` version 1只序列化intent与单record。

这条链对当前九项demo类型可用，且现有tests覆盖create、selection、undo/redo和journal payload。它不具备多实体template、nested instance、subobject/component graph、依赖资产、partial failure、preview promotion、factory migration或provider generation的守恒能力。

### 3.3 固定菜单不是可放置对象目录

`defaults.rs`登记Cube、Camera和五类Light，category与path均固定在`Selection`，when clause为`Always`；`Empty`和`Mesh`虽能解析，却不在菜单中。retained `menu_items_for_layout`又手写相同七项。`node_kind_id/from_id`、`menu_action_id/from_id`和`menu_item_binding`继续复制穷举映射。

因此新增一个场景对象类型至少要改Runtime enum/default、command registry、retained menu、id编码和binding，多处遗漏可以编译后才暴露。更重要的是产品没有category/search/filter/favorite/recent/thumbnail/help，也没有按当前Scene、插件、资源和feature profile过滤的placeable snapshot。

### 3.4 Extension store没有Scene Object Factory贡献类型

`ContributionBatch`已经支持views、drawers、menu items、commands、asset importers/types、scene modes、viewport overlays、operation factories等贡献，这是可扩展底座。但没有scene object type descriptor、placeable item、placement category、construction factory、placement filter或template provider。

插件最多登记一个通用command并自行旁路World；host无法把它纳入统一预检、preview、transaction、recent/favorite、unload revoke和产品capability。通用operation factory也不能替代typed placement factory，因为它没有对象/资产/目标World语义。

### 3.5 Asset drag payload存在，但Scene没有consumer

Asset Browser在pointer down时构造`UiDragPayloadKind::Asset`并保存为`active_asset_drag_payload`；引用字段showcase能按`AssetFieldDropped`等action消费它。相关tests覆盖arm、clear和reference-field drop。

但pointer up会直接清空payload，viewport `pointer_event`只转发Down/Move/Up/Scroll/Cancel到selection/gizmo/camera，`pointer_dispatch`也没有drag-over/drop或payload参数。全仓`active_asset_drag_payload`生产consumer只落在showcase reference fields。`workbench_drag_overlay.zui`的“Drop into scene”是未接产品事件的原型。因此typed payload的存在不能证明asset-to-scene placement。

### 3.6 Quick Import错误耦合资产摄取与World修改

`assets/workspace.rs`的Quick Import先stage/import模型、解析默认material，然后立即调用`import_mesh_asset`，最终在当前World以默认transform生成Mesh节点并改变selection。用户无法选择“只导入”、目标document/parent/position、批处理、预览或失败政策。

工程上Import Asset与Place Asset必须是两个可组合但独立的operation。导入成功、放置失败时资产仍应有明确终态；放置已导入资产不应重新走摄取；后台reimport更不能隐式生成场景对象。

### 3.7 邻近palette/template不能冒充Scene Placement

仓库存在Command Palette、UI Asset Editor widget palette和Asset Creation Template。前者发现command，后两者分别编辑UI document或创建asset。它们都不返回`SceneConstructionPlan`，不持有World destination，不做viewport placement，也没有Scene object factory lifecycle。

产品命名和文档必须保持边界，避免用“已经有palette/template”掩盖Scene Placement缺失。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：主参考是完整放置协议，不是Actor枚举

`FAssetPlacementInfo`携asset data、name override、preferred level、finalized transform、factory override、stable item guid与settings object；`FPlacementOptions`携instancing grid、batch preference与preview标记。`UPlacementSubsystem`能选择/注册factory，按factory分组并执行`BeginPlacement -> PrePlaceAsset -> PlaceAsset -> PostPlaceAsset -> EndPlacement`，同时返回每项element handle。

`UActorFactory`提供display name、priority、actor class、quick-menu、surface orientation、placement extent与spawn offset，并以`CanCreateActorFrom`做资产准入。`FActorPositioning`执行world trace、忽略preview actor、camera-front fallback、grid/vertex/surface snapping、normal orientation、extent/collision与factory offset。`FAssetDragDropOp`携单/多asset、path、factory、thumbnail/decorator和刷新后的registry tags，使factory identity不会在drag中丢失。

`IPlacementModeModule`提供category、placeable item、register/unregister/filter、sorted snapshot与recent event。内建Favorites、Recent、Basic、Lights、Visual Effects、Volumes和All；`SPlacementModeTools`提供search、category、favorite、thumbnail、自定义drag handler。点击创建用`FScopedTransaction`把create和move合并为一次操作并加入recent。

Zircon应借鉴这些合同关系，不照搬UObject/Actor层级。尤其不能把Unreal Editor里的factory归属机械复制成Runtime依赖Editor：Zircon固定架构要求Runtime拥有可构造事实与World提交，Editor只拥有authoring交互和presentation。

### 4.2 Godot：动态类型发现、可用性过滤与资源分派

`CreateDialog`从`ClassDB`与script global classes填充类型，过滤feature profile、abstract/virtual、错误inheritance、unexposed、blacklist、disabled addon和无法加载的script。搜索排序结合exact match、preferred type、favorite与recent，历史和收藏按base type持久化，收藏还支持拖动重排。`instantiate_selected`统一走native/custom/script实例化并应用object defaults。

`SceneTreeDock`创建节点时选择parent、验证名称、设置owner、绑定当前scene history、同步selection/live debug；PackedScene批量实例化先完成load/instantiate/cycle validation，任一失败会清理已创建实例，再统一登记Undo。资源drop会区分property assignment、PackedScene、AudioStream与script attach/instantiate。

`Node3DEditorViewport`对PackedScene、Mesh、Audio创建preview，对Material/Texture应用材质preview；drop admission检查资源类型和循环依赖，支持多文件，按Shift/Alt选择parent，计算parent-local transform，并将实际创建放入单次Undo action。Zircon至少要达到这种typed admission、preview cleanup、destination和transaction完整度。

### 4.3 Fyrox：较小实现也没有跳过构造器、预览和命令守恒

Fyrox Create菜单从`serialization_context.node_constructors`的variants动态生成并按group分类；UI scene和game scene使用不同constructor container。创建后用camera `placement_position`，把`AddNodeCommand`与`MoveNodeCommand`组成一个command group。

资产拖到GameScene时，drag-over加载Model并实例化preview subgraph；位置优先用pick且过滤preview自身，空白处退回与平面相交，再经过grid snap。drop时先把preview从Scene取成reserved subgraph，随后用`AddModelCommand`与selection command提交，保证只有command修改正式Scene。World tree drop还能把asset交给data provider并指定parent。

Fyrox仍没有Unreal同等级factory/category/recent/favorite合同，所以它是“Zircon当前缺失的最低产品链”证据，不是最终上限。

### 4.4 Bevy：运行时构造的多实体、模板、依赖与失败回滚

Bevy本身不提供通用Scene Placement UI，不能拿来证明palette/drag体验。但`ReflectFromWorld`允许类型使用World构造，`ReflectBundle`提供insert/apply/apply-or-insert/remove/take；新的`Scene`/`ResolvedScene`将依赖注册、template、bundle、related scene与cached patch分开解析。

`ResolvedSceneRoot::spawn`先生成实体，apply失败会despawn；`ApplySceneError`区分template、cached scene和related scene错误。tests覆盖cached patch顺序、hierarchy、scene component、spawn中queue，以及`drop_is_called_for_uninserted_components`和`despawn_on_failed_spawn`。这直接反证Zircon“保存一个NodeRecord即可覆盖所有构造”的假设。

### 4.5 Unity Graphics：只取渲染专用目录与Undo语义

`VolumeComponentListEditor`从当前render pipeline可显示类型构造category，支持search、visibility、add/remove/reset/reorder、copy/paste、help URL和version-control editability。新增component先创建对象、登记`Undo.RegisterCreatedObjectUndo`、作为subasset写入profile；remove在序列化引用更新后才`Undo.DestroyObjectImmediate`，明确保护redo顺序。

这组代码适用于渲染Volume override的类型目录与结构编辑，不是GameObject/scene placement实现。Zircon只应吸收type catalog、capability、Undo顺序和测试纪律，不应从Unity Graphics推导通用factory或viewport行为。

## 5. 差异矩阵

| 能力 | Zircon current source | Unreal / 其他参考 | 结论 |
|---|---|---|---|
| 类型身份 | 闭集`NodeKind`，多处字符串映射 | stable item/type/class/factory identity与动态注册 | 必须硬切为stable key + generation |
| 目录 | 七项固定菜单，无Scene palette | category/search/filter/favorite/recent/thumbnail | 产品发现面缺失 |
| Factory | `default_node_record` match | factory validate/pre/post/batch/offset | 构造协议缺失 |
| 请求 | `kind`或`model/material` | asset、factory、level、transform、settings、preview | 上下文严重丢失 |
| 结果 | 一个`NodeId`/`NodeRecord` | element handles、多实例、typed error/receipt | 无法守恒复杂图 |
| 目标 | 隐式当前World/root | document/level/parent/order/data ownership | destination无身份 |
| 变换 | hard-coded default transform | ray/surface/grid/vertex/bounds/collision/camera fallback | 放置求解缺失 |
| Preview | 无产品状态 | transient preview、sanitize、exclude、cleanup/promotion | 交互生命周期缺失 |
| Asset drop | payload只到showcase引用字段 | asset/type admission -> preview -> transaction | Scene consumer缺失 |
| Template/prefab | Scene创建请求不可表达 | PackedScene/Scene/Template/archetype graph | 不能生成复杂对象 |
| Transaction | 单record create command | multi-object action、cleanup、exact graph retention | 数据守恒不足 |
| Plugin | generic command/operation contribution | factory/item/category register/revoke | 无统一生命周期 |
| Batch | 单kind/单mesh | multi-asset、per-factory grouping、batch option | 性能与错误模型缺失 |
| Tests | basic create/undo与drag source | failure cleanup、hierarchy/template、runtime collections | placement关键场景全空 |

## 6. 新增发现

### 6.1 P1：架构与产品闭环

#### ED65-P1-01：闭集`NodeKind`被误用为可创建类型权威

新增场景对象必须修改Runtime enum、default record/name和Editor多处穷举。它无法承载插件、script class、prefab、asset-derived type、schema generation或deprecated/missing type。应引入稳定`SceneObjectTypeKey`和Runtime发布的immutable descriptor snapshot；`NodeKind`只可作为迁移期builtin标签，最终不得继续控制产品目录。

#### ED65-P1-02：宽`NodeRecord`与hard-coded defaults不能表达构造图

`NodeRecord`以大量optional builtin字段表示对象，bootstrap match写死默认transform/component。这无法表达多实体、nested prefab、动态/plugin component、subobject、external dependency和construction provenance。factory应构造detached entity batch/scene fragment，再由World原子提交。

#### ED65-P1-03：菜单、action id与binding存在多份可漂移类型映射

七项菜单在command defaults和retained layout重复；NodeKind/action/binding又有多份match。必须从同代`PlaceableCatalogSnapshot`生成menu/palette/action binding，并以link validator保证descriptor、command、icon、localization和factory可达。

#### ED65-P1-04：没有Scene Placeable Catalog与查询合同

产品没有category、search token、filter、favorite、recent、thumbnail、help、deprecation、project feature或current target capability。需要Runtime type/factory事实与Editor presentation的generation-qualified目录，查询结果必须携descriptor/factory generation，过期项Fail closed。

#### ED65-P1-05：没有统一Scene Object Factory注册、选择和撤销协议

`default_node_record`和special mesh branch是两条私有路径，extension store没有factory贡献。需要typed factory descriptor、asset-class admission、priority/selection、prepare/construct/finalize/abort、batch capability和owner generation；provider unload前必须quiesce preview/session并撤销目录项。

#### ED65-P1-06：创建请求与结果没有typed上下文和拒绝原因

请求只携`kind`或`model/material`，结果靠selection推断新NodeId。需要`SceneConstructionRequest`显式携document/world revision、destination、item/factory/template key、asset refs、transform candidate、preview/batch/options和expected generations；返回typed rejection、warnings、created roots、exact batch与receipt。

#### ED65-P1-07：创建command无条件可用，未经过统一admission

`WhenClause::Always`只在play mode下由较后层拒绝。read-only asset、无项目、无Scene document、closing/reloading、插件disabled、资源未ready、错误parent或World generation过期都没有菜单/drag-over同源preflight。所有入口必须调用同一`PlacementAdmissionService`，UI disabled reason与commit复验共享reason code。

#### ED65-P1-08：destination没有document/world/parent/order/owner身份

菜单创建隐式落到当前World默认root，Quick Import也是如此。请求必须明确SceneDocumentSession、WorldGeneration、parent qualified address、before/after insertion、folder/layer/partition/data owner与selection policy；Runtime在commit前复验destination仍有效。

#### ED65-P1-09：没有工程级Placement Transform Solver

当前创建只用hard-coded transform。需要将viewport ray、runtime pick receipt、surface normal、grid/angle/vertex snap、bounds/pivot、collision/placement extent、factory offset、camera fallback、parent-local conversion和multi-item layout形成确定性输入/输出；Editor负责交互策略，Runtime负责空间事实与最终有效性。

#### ED65-P1-10：没有preview session的创建、更新、取消与提升语义

当前没有ghost/transient ownership、preview object exclusion、asset load pending、pointer capture、Escape/cancel、viewport leave、document replacement、plugin unload和host shutdown cleanup。必须建立显式`PlacementSession`状态机，并保证preview对象不进入save、history、runtime systems、selection authority或普通picking。

#### ED65-P1-11：资产drag payload没有Scene/Viewport drop consumer

payload在pointer up被清空，viewport route不携drag-over/drop。需要在host input层建立drag session lease，按asset type选择factory，异步加载preview，连续更新transform，并只在drop terminal event提交；unknown/rejected target必须保留可诊断状态而非静默清空。

#### ED65-P1-12：Quick Import把资产导入与场景实例化错误耦合

Import与Place必须拆成两个operation。Quick Import可提供“导入后开始放置”复合工作流，但应具有两个receipt、独立rollback/partial success、可取消placement和明确目标document；reimport与background import不得隐式修改World。

#### ED65-P1-13：template/archetype/prefab/script class无法进入统一创建路径

创建intent无法表达construction template、prefab asset、script class、class defaults、instance override或nested dependency。需要同一factory协议消费builtin type、asset、template和plugin provider，输出exact construction graph；default/override传播仍由Editor44拥有。

#### ED65-P1-14：单`NodeRecord`命令不能守恒复杂创建与失败清理

未来factory可能创建根、children、components、subobjects和external references。command必须持有Runtime返回的exact detached batch、dependency/remap receipt与selection delta；prepare或commit任一阶段失败必须清理中间对象，undo/redo不能重新调用可能已升级的factory来猜测结果。

#### ED65-P1-15：journal v1缺少factory/template/schema与迁移身份

当前payload只有intent和record。需要记录stable object/factory/template keys及version、provider generation、qualified destination、asset dependency versions、construction receipt、entity remap与codec version；replay必须先做compatibility/migration，不能把旧`NodeId`或枚举直接应用到新World。

#### ED65-P1-16：插件不能贡献并安全撤销placeable item/category/factory

generic menu/command不足以纳入统一目录和lifecycle。需要owner-qualified registration ticket、immutable snapshot generation、capability declaration、factory callback fault boundary、in-flight lease、revoke barrier与missing provider恢复策略；插件卸载不得留下可点击的stale item或无法取消的preview。

#### ED65-P1-17：没有batch/multi-asset placement与可扩展性能合同

Unreal按factory分组，Godot支持多文件，Zircon只处理单kind/单mesh。需要batch preflight、per-item disposition、atomic/all-or-partial policy、stable layout、bounded async load、progress/cancel、factory grouping和一次transaction；不得为每项重建catalog、reflection与World snapshot。

#### ED65-P1-18：测试只证明primitive，不覆盖placement产品状态机

现有51个focused test attribute覆盖create/undo、registry和drag source/reference fields，却没有asset drag-over viewport、factory selection、preview update/cancel/cleanup、destination replacement、multi-asset、partial failure、plugin unload、save exclusion、journal migration或100k item catalog。必须先补RED矩阵再实施，防止再次以showcase或静态字符串测试冒充产品闭环。

### 6.2 P2：质量、可维护性与资格证据

#### ED65-P2-01：名称冲突和默认命名没有独立策略

当前name按`NodeKind`写死。应由descriptor提供base display name，destination naming service负责casing、unique suffix、locale-independent stable identity与batch numbering，并把最终名称写入receipt。

#### ED65-P2-02：icon、localization、help、tooltip与accessibility没有link合同

placeable item需引用可验证的localization key、icon/thumbnail source、documentation/help、search aliases与screen-reader label。缺失资源必须在catalog compile/link阶段给出diagnostic，而不是运行时显示空白或硬编码英文。

#### ED65-P2-03：favorite/recent与placement settings缺少版本化持久化

需要per-project/per-user作用域、stable type key、最大容量、去重、排序、missing provider保留/清理和schema migration。recent只能在成功commit后更新，preview或失败操作不得污染历史。

#### ED65-P2-04：缺少结构化诊断与可观测receipt

应记录admission/factory/load/preview/transform/commit各阶段耗时、reason code、item/factory generation与created count，但不得泄露资产内容或把telemetry当正确性依据。错误面要能定位是资源、factory、destination、transaction还是plugin fault。

#### ED65-P2-05：目录与preview热路径没有缓存、预算和benchmark基线

必须定义10k/100k placeable item查询、1/100/1k batch、60/120Hz preview update的CPU、allocation、latency与memory预算；catalog索引、thumbnail请求和transform solver不得每帧全量clone或排序。达到Unreal功能后才能做同场景同硬件比较。

#### ED65-P2-06：modifier与placement policy没有统一设置模型

parent选择、surface orientation、snap、collision offset、align-to-normal、batch spacing和preview material应来自版本化policy/settings snapshot。菜单点击、drag/drop和palette放置必须消费同一设置，快捷修饰键只产生临时override，不可形成另一套隐式规则。

## 7. 目标架构与职责边界

### 7.1 Runtime唯一权威

建议Runtime提供：

- `SceneObjectTypeKey { namespace, name, schema_version }`：稳定类型身份，不含Editor展示状态。
- `SceneObjectTypeDescriptor`：builtin/plugin/script/template类别、required capabilities、supported asset types、factory key、default naming seed和construction schema。
- `SceneObjectFactoryRegistrySnapshot { generation, descriptors }`：immutable、owner-qualified、可撤销，拒绝重复/悬空factory。
- `SceneConstructionDestination`：World/session generation、parent qualified address、insertion、ownership/layer/partition事实。
- `SceneConstructionRequest`与`SceneConstructionPlan`：preflight固定factory、dependencies、exact prospective roots、warnings、resource leases和expected generations。
- `SceneConstructionReceipt`：created root/entity set、exact detached batch handle、asset/ref remap、final transform/name、factory/schema generations与terminal disposition。
- `SceneObjectFactory`：`preflight/prepare/commit/abort`，callback有fault boundary；prepare只能产生隔离对象，commit由World原子完成并复验。

Runtime不得依赖Editor UI、favorites或viewport widget；Editor不得复制Runtime可构造类型/组件事实，也不得直接循环低层`spawn_node`拼装复杂对象。

### 7.2 Editor authoring与产品层

建议Editor提供：

- `PlaceableCatalogSnapshot`：由Runtime descriptor、Asset Catalog和extension presentation link编译出的只读视图，携generation与diagnostic。
- `PlacementQueryModel`：category/search/filter/favorite/recent/all、stable sorting、paged/virtualized result。
- `PlacementSession`：`Idle -> Admitting -> PreviewLoading -> Previewing -> CommitPending -> Committed|Cancelled|Rejected|Faulted`，每一终态都释放capture/resource/preview lease。
- `PlacementTransformSolver`：消费viewport/pick/settings/factory extent，输出确定性candidate及解释信息；commit时由Runtime复验。
- `PlaceSceneObjectCommand`：绑定Scene Document transaction，持有plan/receipt/exact batch和selection delta，undo/redo不重跑factory。
- `PlacementPreferenceStore`：stable-key favorite/recent/settings，provider缺失时保持可诊断tombstone。
- `PlacementProductSurface`：palette、menu、asset drag/drop、keyboard/controller accessibility都进入同一admission/session/command。

### 7.3 事务时序

1. UI从同代catalog选择item或由asset payload解析候选factory。
2. Editor创建qualified placement session并请求Runtime preflight。
3. Runtime返回plan或typed rejection；Editor异步加载隔离preview。
4. pointer/keyboard更新candidate transform，preview对象始终带transient owner并排除普通World系统。
5. drop/click触发document transaction；Runtime复验plan/destination/generations并原子commit。
6. Editor依据receipt更新selection、history、dirty、recent和UI；任一失败执行abort并清理preview。
7. undo将exact committed batch detach，redo按receipt恢复；plugin reload或schema变化不重新解释旧factory逻辑。

## 8. 分阶段重构计划

### ED65-M0：能力真实性与RED基线

- 将未接线“Drop into scene”明确标记Prototype/Unavailable；Quick Import文案不再暗示通用drop。
- 增加当前缺失的viewport asset drop、preview cancel/cleanup、destination generation和factory failure RED tests。
- 冻结现有`NodeKind`/menu/id/binding调用点与focused fingerprint，阻止继续新增穷举分支。

### ED65-M1：Runtime identity、descriptor与factory registry

- 建立stable type/factory key、descriptor、owner-qualified immutable snapshot和typed registration error。
- builtin九类通过同一registry发布；禁止Editor直接以`NodeKind`生成产品目录。
- 加入duplicate、missing factory、plugin revoke、generation stale、capability与asset type admission tests。

### ED65-M2：Construction plan、隔离graph与receipt

- factory先生成detached batch/scene fragment，World preflight/commit原子化。
- 支持多实体、components/subobjects、asset dependencies、typed warnings/errors和失败清理。
- 用Bevy式failure cleanup与Godot式batch prevalidation建立fault injection tests。

### ED65-M3：Placement session与transform solver

- 建立drag/click/keyboard统一state machine、capture、preview ownership与terminal cleanup。
- 接入runtime picking、surface/grid/angle snap、bounds/collision/factory offset、camera fallback和parent-local transform。
- 对document/world replacement、viewport leave、Escape、focus loss、host shutdown进行强制cancel。

### ED65-M4：Asset、template、prefab与script class统一入口

- 拆分Import receipt与Place receipt；资产drop按type选择factory。
- 将builtin、mesh/material/audio、prefab/template/script/plugin type统一到同一construction request。
- 与Editor44/55/57协作保证defaults、portable payload、reimport和missing asset的owner边界。

### ED65-M5：Document transaction、journal与undo/redo守恒

- `PlaceSceneObjectCommand`绑定qualified document/world/object generations。
- history保存exact detached batch/receipt，journal记录stable factory/template/schema并提供migration。
- 覆盖savepoint、dirty、save/reopen、crash recovery、cross-document rejection和provider upgrade。

### ED65-M6：插件生命周期与产品palette

- extension contribution增加placeable category/item/factory/filter/presentation link并实现revoke barrier。
- 实现search/category/favorite/recent/thumbnail/help、virtualized result和disabled reason。
- menu、palette、asset drag与快捷操作全部从同一catalog/admission生成。

### ED65-M7：规模、故障、表现与性能资格

- 运行10k/100k目录、1k batch、连续preview、asset load delay、factory panic、plugin reload、document switch与memory pressure矩阵。
- 在同硬件、同资产、同视口/碰撞/snap语义下比较Unreal/Fyrox/Godot；报告p50/p95/p99、allocation、CPU/GPU与memory。
- 只有48门全部Pass、父P0关闭且performance receipt可复现后，才允许声称工程级完成或性能优于Unreal。

## 9. 资格门

| Gate | 验收条件 | 当前 | 所需证据 |
|---|---|---|---|
| ED65-G01 | builtin/plugin/script/template对象具有stable type key | Fail | registry unit + migration tests |
| ED65-G02 | factory具有stable key、version与owner generation | Fail | descriptor snapshot tests |
| ED65-G03 | catalog是immutable generation-qualified snapshot | Fail | stale snapshot rejection tests |
| ED65-G04 | 重复type/factory/category/item注册Fail closed | Fail | negative registration matrix |
| ED65-G05 | descriptor与factory/icon/help/localization link可验证 | Fail | catalog linker tests |
| ED65-G06 | provider撤销后新查询不再返回stale item | Fail | revoke publication tests |
| ED65-G07 | missing/deprecated provider有明确tombstone策略 | Fail | unload/save/reopen tests |
| ED65-G08 | Editor不再维护`NodeKind`产品目录副本 | Fail | source guard + callsite audit |
| ED65-G09 | 所有入口共享typed placement admission | Fail | menu/palette/drop parity tests |
| ED65-G10 | request携document/world/destination generations | Fail | request schema tests |
| ED65-G11 | request携factory/template/asset/options身份 | Fail | serialization round-trip tests |
| ED65-G12 | Runtime在commit前复验全部expected generations | Fail | stale race tests |
| ED65-G13 | factory支持preflight/prepare/commit/abort | Fail | lifecycle contract tests |
| ED65-G14 | factory callback fault被隔离并有typed result | Fail | panic/fault injection tests |
| ED65-G15 | partial prepare失败清理全部中间对象 | Fail | allocation/drop counter tests |
| ED65-G16 | commit返回exact construction receipt | Fail | entity/component/reference audit |
| ED65-G17 | target parent/order/owner/layer明确且可复验 | Fail | destination matrix tests |
| ED65-G18 | read-only/closing/reloading/play状态同源拒绝 | Fail | capability parity tests |
| ED65-G19 | asset type到factory选择确定且可解释 | Fail | priority/ambiguity tests |
| ED65-G20 | multi-asset按factory分组并保留per-item disposition | Fail | heterogeneous batch tests |
| ED65-G21 | viewport drag-over实际消费typed asset payload | Fail | retained host integration test |
| ED65-G22 | pointer up/drop/cancel有唯一terminal disposition | Fail | input state-machine tests |
| ED65-G23 | preview对象有transient owner且不进入save/history | Fail | save/history exclusion tests |
| ED65-G24 | preview对象从普通pick/render/system路径正确隔离 | Fail | pick/system visibility tests |
| ED65-G25 | viewport leave/Escape/focus loss清理preview | Fail | cancel lifecycle tests |
| ED65-G26 | document/world replacement强制终止旧session | Fail | generation replacement tests |
| ED65-G27 | plugin unload等待或取消in-flight placement | Fail | quiesce/revoke race tests |
| ED65-G28 | asset load pending可取消且资源lease有界 | Fail | delayed loader tests |
| ED65-G29 | ray/surface/grid/angle/vertex snap有确定性结果 | Fail | numeric golden tests |
| ED65-G30 | bounds/pivot/collision/factory offset进入求解 | Fail | geometry placement tests |
| ED65-G31 | 空白viewport有camera/plane fallback | Fail | perspective/ortho tests |
| ED65-G32 | parent-local transform转换无漂移 | Fail | hierarchy transform tests |
| ED65-G33 | create/move/select/dirty是一次document transaction | Fail | transaction integration test |
| ED65-G34 | undo保存exact多实体batch而非单`NodeRecord` | Fail | graph conservation tests |
| ED65-G35 | redo不重跑已变化或已卸载factory | Fail | provider upgrade/unload tests |
| ED65-G36 | journal记录stable factory/template/schema identity | Fail | journal schema tests |
| ED65-G37 | journal migration失败给出typed terminal error | Fail | old/new codec tests |
| ED65-G38 | cross-document/world replay被拒绝 | Fail | qualified identity tests |
| ED65-G39 | save/reopen保留object graph、asset refs与ownership | Fail | persistence round-trip tests |
| ED65-G40 | crash/fault后没有preview或half-created对象泄漏 | Fail | recovery/fault tests |
| ED65-G41 | Import与Place拥有独立receipt和partial-success语义 | Fail | workflow integration tests |
| ED65-G42 | prefab/template/script/builtin共享一个factory协议 | Fail | construction parity matrix |
| ED65-G43 | favorite/recent只用stable key并可迁移 | Fail | preference migration tests |
| ED65-G44 | recent只在成功commit后更新 | Fail | reject/cancel history tests |
| ED65-G45 | 10k/100k catalog查询分页/虚拟化且预算内 | Fail | CPU/allocation profile |
| ED65-G46 | 1k batch无per-item全量snapshot/sort/clone | Fail | batch allocation profile |
| ED65-G47 | 60/120Hz preview更新满足p95 latency预算 | Fail | product trace + profile |
| ED65-G48 | 同语义跨引擎功能/表现/性能receipt可复现 | Fail | Unreal/Fyrox/Godot benchmark |

## 10. 测试与验证矩阵

### 10.1 Unit / property

- stable type/factory key parse、version、collision、owner generation和snapshot ordering；
- descriptor/factory link、asset type admission、priority ambiguity和typed rejection；
- transform solver的ray、plane、surface normal、snap、bounds、collision、parent-local数值性质；
- naming、favorite/recent migration、batch layout与journal codec。

### 10.2 Integration / product

- menu、palette、asset browser drag、hierarchy target和viewport drop共享同一plan/receipt；
- builtin、mesh、audio、prefab、script class和plugin factory的preview/commit/undo/redo/save/reopen；
- multi-document replacement、read-only、play mode、selection改变、parent删除与World generation race；
- plugin enable/disable/reload、factory panic、asset load fail、partial batch与host shutdown。

### 10.3 Fault / scale / performance

- 每个lifecycle hook注入fail/panic/cancel，核对对象、asset lease、capture、preview、history与dirty守恒；
- 10k/100k catalog，1/100/1k mixed batch，长路径/大metadata与thumbnail delay；
- 60/120Hz pointer preview下CPU、allocation、latency、memory和render currentness；
- release build、固定硬件、预热与至少31次重复，报告p50/p95/p99和置信区间。

### 10.4 本轮未运行

本轮未运行任何Cargo命令，未包含`--locked`命令；未做workspace-wide或crate-local动态验证，因为这是review-only且MVP F0仍未完成。本轮也未触碰shared API或workspace wiring，未新增/删除兼容路径；只增加报告与索引。静态验证在报告落盘后执行link、计数、frontmatter path、占位标记、trailing whitespace与`git diff --check`。

## 11. Owner路由与非重复计数

| 主题 | 唯一父owner | Editor65约束 |
|---|---|---|
| command/keymap/menu/palette基础 | Editor08 | Scene placeable只消费目录，不另造command registry |
| prefab/archetype/default/override | Editor44 | factory引用其artifact，不复制传播算法 |
| extension mount/revoke/quiesce | Editor50 | 新贡献类型服从同一owner generation与barrier |
| portable drag/clipboard/remap | Editor55 | asset payload扩展复用canonical payload identity |
| Asset Browser/import/reimport | Editor57 | Import与Place通过receipt组合，不夺取asset authority |
| viewport input/picking/capture | Editor59 | placement session消费qualified input/pick receipt |
| hierarchy destination/reparent | Editor60 | parent/order选择走同一qualified item/address |
| Scene Document lifecycle | Editor61 | placement绑定document session与transition lease |
| transaction/history/journal | Editor63 | `PlaceSceneObjectCommand`进入canonical document history |
| component structural graph | Editor64 | factory构造组件图，Editor65不另造component registry |
| ECS/World/hierarchy | Runtime99i-99k | Runtime负责真实构造、提交、transform与ownership复验 |

父报告现有P0继续阻断实施，尤其document/world identity、transaction qualified object、drag payload数据守恒、viewport cancel/capture和Runtime entity/component exact batch。Editor65新增计数保持0项P0，避免同一根因被多个报告重复宣称。

## 12. 最终判定

当前Zircon的Scene Object Creation只能判定为“固定builtin primitive已可点击并可单记录撤销”，不能判定为工程级对象创建，也不能判定资产已支持拖入Scene。最危险的临时实现不是某个默认数值，而是把`NodeKind + default_node_record + fixed menu + one NodeRecord command`当成公开架构；继续沿该路径增加类型会放大多权威、插件不可扩展、复杂图数据丢失与产品入口分叉。

正确顺序是先关闭父级identity/transaction/document/viewport P0，再建立Runtime factory/plan/receipt，随后实现Editor placement session、asset/template统一入口、产品palette和规模资格。未完成48门与同语义profile前，任何“达到或超过Unreal”的功能或性能结论都没有证据基础。
