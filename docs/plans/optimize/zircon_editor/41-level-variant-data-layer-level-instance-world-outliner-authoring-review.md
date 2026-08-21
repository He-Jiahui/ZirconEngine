---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_level_variant_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_plugins/prefab_tools
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/40-procedural-content-generation-rule-graph-biome-world-generation-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/DataLayer/DataLayerAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/DataLayer/DataLayerInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldPartition/DataLayer/DataLayerManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstanceInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstanceSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManagerContent/Source/VariantManagerContent/Public/LevelVariantSets.h
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManagerContent/Source/VariantManagerContent/Public/VariantObjectBinding.h
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManagerContent/Source/VariantManagerContent/Public/PropertyValue.h
  - dev/UnrealEngine/Engine/Plugins/Enterprise/VariantManager/Source/VariantManager/Public/VariantManager.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerTreeItem.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerHierarchy.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerMode.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerColumn.h
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/Fyrox/editor/src/world/mod.rs
  - dev/Fyrox/editor/src/world/graph.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/BatchLayers.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/RenderingLayerUtils.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 41 · Level Variant / Data Layer / Level Instance / World Outliner Authoring 工程化差距

## 1. 结论

Zircon当前只有可工作的Entity Hierarchy基础，没有工程级World Outliner、Data Layer、Level Instance或Level Variant产品。四者不能再由一个层级面板、`render_layer_mask`、`active`布尔值或`PrefabInstanceAsset`名称替代：它们分别拥有不同的资产、身份、状态机、运行时authority、编辑会话、事务和cook责任。

Hierarchy并非纯mock。Runtime会生成稳定entity row、parent/depth、subtree hash与world generation；Editor有generation保护、稀疏name/selection patch、过滤祖先保留、F2/双击重命名、多选拖拽reparent、undo/redo事务和10,000行可见区裁剪。5,000层深树过滤没有递归栈风险。这些基础必须保留，不能重写成静态树控件。

但当前投影只认识`id/name/depth/selected`。它没有Level/Scene归属、actor folder、Data Layer成员关系、instance/source provenance、editor visibility/lock、loaded/streaming、dirty/source-control/error列、actor/component/descriptor item种类、mode/column/filter扩展点或展开状态。绘制按可见区裁剪，但模板桥仍按总行数clone retained node，结构变化通常要求全树reflow；因此它尚不能作为十万级World Outliner资格证明。

Level Variant页面是明确的P0虚假能力面。页面固定展示`Vehicle_Showcase`、`Variant_Red`、`18 overrides`和`2 conflicts`；Preview/Apply只把固定`queued`文字写回输出行。仓内没有Variant asset kind、binding/property capture模型、typed property address、record/apply executor、transaction、source revision、conflict preflight、rollback或cook/runtime消费链。

Data Layer和Level Instance则连产品类型都不存在。`ResourceKind`没有对应kind，Scene entity没有authoring layer membership、owning level或instance provenance，Editor intent也没有layer/level/instance操作。`render_layer_mask`是渲染过滤掩码，`active`是运行时激活状态，二者都不能承担Data Layer的编辑可见性、编辑加载状态、运行时Unloaded/Loaded/Activated、层级有效状态和client/server authority。

Prefab DTO不能被当作Level Instance完成度。`PrefabInstanceAsset`以字符串`entity_path/property_path`和无类型JSON保存override；runtime插件的`.prefab.toml` importer明确是`DiagnosticOnlyAssetImporter`，Editor五个operation只有descriptor，helper只清空或返回override Vec。更严重的是`World::from_scene_asset`不消费`prefab_instance`，`World::to_scene_asset`固定写`None`，一次Scene加载保存即可擦除链接与override。

目标架构应建立四个独立owner：版本化`VariantSetAsset`与typed capture/apply服务；`DataLayerAsset/DataLayerInstance`与运行时state authority；`LevelInstanceSource/LevelInstanceRecord/InstanceEditSession`与provenance/rebase系统；可插拔`WorldOutlinerModel`及typed item/mode/column/filter registry。Scene/World只保存或引用这些owner的稳定ID，Editor transaction、background job、cook artifact和runtime authority通过明确接口协作。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Runtime、Editor、plugin、interface生产代码或tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes / ignored / 在途 | 审查方式 |
|---|---:|---:|---|
| Level Variant false surface | 10 / 13,025 / 726,046 | 1 / 0 / 1 | E3逐ZUI、binding、navigation、preview action、feedback与preview设计入口 |
| Hierarchy与World Outliner基础 | 115 / 11,942 / 415,917 | 86 / 0 / 1 | E3逐Runtime inspection/artifact、message/delta、retained projection、pointer、paint、filter、rename、drag/reparent与focused tests |
| Scene、Prefab与Level Instance边界 | 26 / 4,372 / 176,712 | 12 / 0 / 0 | E3逐ResourceKind、Scene DTO/document/artifact/World IO及完整prefab_tools package |
| Unreal参考 | 60 / 28,078 / 1,028,423 | 0 / 0 / 0 | E2/E3逐Data Layer authority、Level Instance lifecycle、Variant capture/apply和Scene Outliner extension model |
| Godot参考 | 6 / 11,652 / 397,730 | 0 / 0 / 0 | E2/E3逐SceneTreeDock、SceneTreeEditor、PackedScene owner/instance/editable children |
| Fyrox参考 | 6 / 2,952 / 106,903 | 1 / 0 / 0 | E2/E3逐WorldViewer provider、search、breadcrumb、drop validation和undo command边界 |
| Bevy参考 | 6 / 2,823 / 113,414 | 1 / 0 / 0 | E2/E3逐ScenePatch、dependency resolve、queued spawn、apply与failure lifecycle |
| Unity Graphics参考 | 3 / 239 / 9,640 | 0 / 0 / 0 | E2确认Batch/Rendering Layer是render scheduling/filtering语义，不是authoring Data Layer |
| selected combined scope | 232 / 75,083 / 2,974,785 | 101 / 0 / 2 | 当前工作树fingerprint `d710ebd9f2fd9a51c30ca6dd3f9d1424a51874f6c62e7b4df1394709e9ae4b71` |

指纹算法为：对232个选择路径按PowerShell `Sort-Object`排序，逐文件计算小写SHA-256，形成`forward/slash/path|file_sha256`行，以单个LF连接且末尾不追加LF，再对UTF-8无BOM payload计算SHA-256。选择规则包括完整Runtime scene inspection、Editor hierarchy pointer、hierarchy callback、Outliner context-menu、hierarchy renderer与完整`zircon_plugins/prefab_tools`目录，表中其余为显式文件；缺失路径0，重复路径0。

读取时2个在途文件为`zircon_editor/src/ui/host/scene_inspection_publication.rs`和`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs`，均非本报告产生。实施前必须重导232文件manifest、重算指纹并复核两处终态。

101个test attributes主要覆盖Hierarchy snapshot/delta/filter/pointer/paint、Prefab DTO helper及参考实现局部测试。它们不覆盖Variant资产roundtrip/capture/apply、Data Layer authority、Level Instance加载编辑提交、Prefab链接Scene roundtrip、Outliner typed item/column/filter或十万对象结构churn，因此不能替代本报告验收门。

### 2.2 名称命中、真实类型与错误等价

1. `LevelVariant`生产命中只落在Workbench ZUI、route/navigation、fixed feedback与preview design。
2. `DataLayer`与`LevelInstance`没有Zircon生产asset、component、manager、subsystem或Editor controller类型。
3. `WorldGeneration`是Scene world revision，不是world generation、Data Layer或Level Instance owner。
4. `ResourceKind::Data`是通用data资源，不等于Data Layer产品类型。
5. `ResourceKind::Prefab`存在，但没有Level Instance resource kind或稳定world-fragment identity。
6. `render_layer_mask`只进入render visibility/filter语义，不能决定资产归属、编辑加载或网络authority。
7. `active`与`active_in_hierarchy`是运行时node激活传播，不是editor-only visibility/lock或layer load state。
8. `parent`只表达runtime transform/scene hierarchy，不能表达actor folder、Data Layer parent或Level Instance source containment。
9. `PrefabInstanceAsset`是未接入World生命周期的DTO，不能仅凭字段存在宣称Prefab或Level Instance可用。
10. Level Streaming Workbench中的固定cell文本已由Editor16覆盖；本报告不重复把静态cell UI当作Data Layer backend。

### 2.3 Level Variant页面的实际执行路径

1. 页面初始隐藏，由extension workspace host显示。
2. 左栏固定列出`Vehicle_Showcase`、`Variant_Red`和`Override_Material`。
3. Variant/Overrides/Diff tab只切换control选中状态，没有document mode或query model。
4. 中央四行固定为CarBody材质、Wheel可见性、Light强度和Door transform conflict。
5. 输出固定为`18 overrides 2 conflicts`，没有artifact、runtime snapshot或diagnostic来源。
6. Variant下拉固定为Red/Blue/Day/Night四个字符串。
7. Set下拉固定为Vehicle Showcase/City Block/Environment/Gameplay四个字符串。
8. Capture字段保存整段`Capture: Selected Props`文字，没有selection/property capture payload。
9. template binding只注册Click、Change和Submit route。
10. navigation spec只做control选择和workspace显示。
11. Preview route重写到`.preview.invoke`后只返回`Preview queued Variant_Red 18 overrides`。
12. Apply route重写到`.apply.invoke`后只返回`Apply queued 18 overrides 2 conflicts`。
13. row selection也只返回固定CarBody或Door文字。
14. 仓内没有Variant importer、asset editor、operation factory、executor、transaction command或runtime switch API。
15. 没有object binding identity、weak/soft reference、unloaded label或missing binding诊断。
16. 没有typed property path segment、array index、component field ID、schema/type fingerprint或recorded value encoding。
17. 没有record current value、compare currentness、apply setter、function caller、transform/visibility/material specialization。
18. 没有duplicate/merge/move set/variant/binding/property的authoring model。
19. 没有source revision、captured revision、conflict class、preflight、atomic apply或rollback。
20. 当前主Editor把设计fixture描述为native extension workspace，构成错误能力声明。

### 2.4 Hierarchy已具备的真实基础

1. Runtime row具有entity、parent、depth、display name、kind、subtree hash、focus、active与has-children。
2. hierarchy构建采用迭代preorder，visited集合会隔离cycle，不依赖递归调用栈。
3. subtree hash由稳定FNV组合产生，可定位子树内容变化。
4. immutable inspection artifact建立row index、children map与child aggregate。
5. name-only变化可通过sparse row override更新，并向祖先传播hash。
6. general topology delta带added/changed/removed anchor与generation。
7. Editor publication按`WorldStructure`生成消息并保留generation连续性。
8. retained projection建立entity到control和control到entity的双向稳定路由。
9. fragment apply在generation、selection revision或anchor不匹配时拒绝稀疏patch并要求resync。
10. selection delta与selection snapshot分开处理，避免每次选择都扫描完整hierarchy。
11. case-insensitive name filter为O(N)，匹配后保留完整ancestor path。
12. 5,000 flat与5,000 deep测试覆盖过滤的规模和非递归性质。
13. F2和double click进入inline rename，最终发布真实`EditorIntent::RenameNode`。
14. drag source保留authoritative multi-selection，即使屏幕正显示filtered projection。
15. drop可发布`SetParents`到node或root，批量操作先收敛为top-level selection。
16. create/delete/rename/reparent/transform/undo/redo均通过Editor transaction scope。
17. play mode会阻止编辑世界mutation，delete还保护最后一个camera invariant。
18. native renderer只绘制clipped viewport可见行，10,000行测试限制draw range。
19. pointer route按scroll offset和row pitch直接求index，不逐行hit-test。
20. 这些能力应成为新Outliner的底层adapter，而不是被静态fixture替换。

### 2.5 Hierarchy不能等同World Outliner的原因

1. 最终`SceneNodeData`只有`id/name/depth/selected`四个字段。
2. Runtime row的`kind/active/has_children`在最终pane payload中被丢弃。
3. 没有World、Level、Scene、Folder、Actor、Component、Actor Descriptor或Instance typed item union。
4. 没有stable item ID namespace，裸entity u64无法跨world/session/source instance消歧。
5. 没有owner scene/level、source asset、instance root或source entity provenance。
6. 没有actor folder；runtime parent和编辑组织层级被迫重合。
7. 没有Data Layer membership、有效layer state或layer操作入口。
8. 没有editor-only visibility、lock、selectability、loaded state或per-user state。
9. 没有streaming status、unloaded actor descriptor、HLOD/cell或remote world representation。
10. 没有dirty、unsaved、source control、warning/error、mobility或type columns。
11. 没有column registry、mode registry、hierarchy provider或tree-item factory。
12. 过滤只匹配display name，没有type/tag/component/layer/scene/level/property filter grammar。
13. 没有sort、group、pin、bookmark、collection、hide temporary或only selected mode。
14. 没有node expand/collapse操作；ZUI中的`expanded`只是authored静态属性。
15. 全仓Hierarchy expand/collapse命中仅为drawer开合，不是tree node状态。
16. 没有展开状态的workspace持久化、filter前后恢复或instance边界策略。
17. 右键SceneNode菜单固定为Open/Rename/Duplicate/Delete，但只投影字符串。
18. 没有context menu item selection到Editor command的dispatch；`context_target_path`只有展示测试。
19. menu path基于显示文字生成`workbench://scene/...`，不是world-qualified stable item ID。
20. paint虽裁剪可见行，pane仍持有全部row model。
21. 模板所谓`virtual_rows`会为`total_row_count - authored_count`逐行clone真实UiTreeNode。
22. `scene_tree_control_ids`扫描并排序所有virtual control，完整reflow会重建双向BTreeMap。
23. topology delta当前统一标记`hierarchy_reflow_required: true`，insert/remove/reorder不能做局部range patch。
24. filter创建新的完整Vec并强制authoritative full reflow。
25. 没有10万/100万item的model memory、structural churn、sort/filter、selection和interaction预算。

### 2.6 Data Layer缺失的工程责任

1. 没有`DataLayerAsset`、`DataLayerId`、`DataLayerInstance`或membership record。
2. 没有runtime/editor/private layer类型，也没有client/server load filter。
3. 没有Unloaded、Loaded-but-hidden、Activated三个运行时状态。
4. 没有requested state与effective state分离。
5. 没有parent/child layer状态传播、循环与类型兼容校验。
6. 没有authoring membership add/remove validation或bulk assignment transaction。
7. 没有editor visible、initially loaded、user-loaded和effective loaded的分离。
8. 没有per-user editor state存储，不能避免把个人加载选择污染共享Scene资产。
9. 没有actor creation context，新增entity无法自动进入当前Data Layer集合。
10. 没有runtime authority：server-only/client-only/replicated state均未定义。
11. 没有state change event、generation、stale request rejection或observer snapshot。
12. 没有World Partition/cell/HLOD/cook manifest关联；该部分继续受Editor16约束。
13. 没有unloaded actor descriptor可解析membership，因此关掉cell后Outliner无法保持组织视图。
14. 没有layer deletion、rename、reparent、asset replacement或membership migration策略。
15. 没有cross-level/cross-instance membership限制。
16. 没有layer color、error、loaded、visible等Outliner column/provider。
17. 没有layer browser、filter、actor count、referencer和invalid member诊断。
18. 没有commandlet/cook验证阻止missing layer、cycle或authority-invalid content进入包。

### 2.7 Level Instance与Prefab边界

1. `SceneEntityAsset`只有单一scene-local u64 identity与parent，没有source-local stable entity ID。
2. Scene没有owning level、subscene、instance root或external actor package字段。
3. `PrefabAsset`直接内嵌完整`SceneAsset`，exposed property仍是字符串Vec。
4. `PrefabInstanceAsset`只有prefab reference、local transform和override Vec。
5. override以字符串entity path和property path定位，rename/reparent/schema migration后不稳定。
6. override value是无类型`serde_json::Value`，没有declared type、schema fingerprint或custom codec。
7. 没有source revision、base value hash、override revision或three-way rebase状态。
8. 没有source entity到instance entity的provenance map。
9. 没有added/removed child、component topology override或orphan override模型。
10. 没有nested instance ancestry、loop detection或parent/child edit约束。
11. 没有load request ID、async status、dependency readiness、cancel、failure与retry。
12. 没有instance bounds、pivot、streaming behavior、runtime behavior或filter。
13. 没有独立edit world/session、current edited instance、dirty tracking、commit/discard。
14. 没有commit preflight、source lock、concurrent source revision检测或atomic save。
15. 没有create from selection、move actors to instance或break instance的真实world mutation。
16. prefab runtime插件只注册component metadata与diagnostic-only importer，capability状态也是Partial。
17. prefab Editor只注册五个operation descriptor，没有对应executor registration。
18. `apply_prefab_overrides`只返回dedupe后的Vec并清空instance Vec，不修改source prefab。
19. `revert_prefab_overrides`只清空Vec，不从source重新实例化或恢复world state。
20. `break_prefab_instance`只返回DTO，不把world subtree转换成ordinary owned entities。
21. validation只核对source布尔值与字符串路径非空，不解析object/property地址。
22. `World::from_scene_asset`不读取`prefab_instance`。
23. `World::to_scene_asset`固定输出`prefab_instance: None`。
24. 现有Scene load/save可静默擦除prefab link、local transform和override，是P0数据损失。
25. Level Instance不能只是给Prefab换名；它还拥有world fragment加载、instance edit、streaming、ownership与cook责任。

### 2.8 参考源码给出的职责边界

1. Unreal Data Layer把asset定义、world instance、runtime state manager、editor subsystem、hierarchy与columns拆开。
2. Unreal区分runtime/private、client/server filter、requested/effective state和editor per-user load state。
3. Unreal Level Instance用稳定ID/Guid、world asset、load/unload、ancestry、loop check和edit/commit/discard形成闭环。
4. Level Instance subsystem集中管理当前edit instance、dirty、bounds、create/break与property override。
5. Variant Manager不是简单JSON diff；它拥有set/variant/object binding/captured property/function caller与typed property resolver。
6. capture会识别精确property path、array index及transform/visibility/material特殊语义。
7. Variant authoring支持create/remove/move/duplicate/merge、selection、record/apply和thumbnail/director调用。
8. Scene Outliner把tree item、hierarchy、mode、column和filter拆为接口，支持多类item与独立folder hierarchy。
9. Godot SceneTree区分node parent、owner、packed-scene instance、editable children与placeholder，并维护资源路径变更。
10. Fyrox WorldViewer通过data provider适配不同scene类型，提供search、breadcrumb、expand/collapse、drop validation和undo command。
11. Bevy ScenePatch把依赖注册、resolve、queued spawn、apply与removed/failure lifecycle显式化，可作为Level Instance加载状态参考。
12. Unity Graphics的BatchLayer/RenderingLayer只处理渲染调度和mask转换，反证Zircon不能复用`render_layer_mask`完成Data Layer。
13. 参考源码用于确定责任与不变量，不要求复制UObject、Slate、Godot owner或Bevy ECS的具体类型布局。

## 3. 目标架构

### 3.1 所有权矩阵

| Owner | 权威数据 | 不得拥有 |
|---|---|---|
| `VariantSetAsset` | set/variant/binding/capture、recorded value、source revision | live World entity、Outliner transient selection |
| `VariantApplyService` | resolve/preflight/apply/rollback receipt | asset编辑历史、硬编码UI row |
| `DataLayerAsset` | 可复用layer定义、runtime/private/load filter | 某个World的requested/effective state |
| `DataLayerWorldState` | world instance hierarchy、membership、editor/runtime state、authority | render layer mask、folder organization |
| `LevelInstanceSource` | 可实例化world fragment及stable source object IDs | instance-local override、edit-session transient state |
| `LevelInstanceRecord` | source ref、instance ID、transform、behavior、override set | source asset mutation |
| `LevelInstanceSubsystem` | load lifecycle、provenance、ancestry、edit session、commit/discard | generic Scene serialization policy |
| `WorldOutlinerModel` | typed item projection、mode/filter/sort/column、expansion与per-user state | Runtime World authority或资产payload |
| `Scene/World` | runtime entity/component状态与owner references | Variant Editor UI、Data Layer个人状态、静态fixture计数 |

### 3.2 稳定身份与地址

1. 定义world-qualified `WorldObjectId { world_session, entity_generation }`，禁止跨session复用裸u64。
2. 定义asset-qualified `SourceObjectId { source_asset, stable_local_id }`，rename/reparent不改变身份。
3. 定义`LevelInstanceId`并记录parent instance、source revision与instance generation。
4. provenance map必须双向回答source object到instance object及instance object到source object。
5. 定义`ReflectedPropertyAddress`，包含component/type stable ID、field stable ID、collection element selector与schema fingerprint。
6. display path只用于UI，不得作为override、capture或context action的权威key。
7. Outliner item ID使用typed namespace，明确World/Level/Folder/Entity/Component/Descriptor/DataLayer/Instance。
8. 所有异步request、delta、apply、commit与cook receipt携带owner ID、source generation和request generation。

### 3.3 Data Layer产品

1. `DataLayerAsset`版本化保存stable ID、display metadata、runtime/private类型、load filter和migration version。
2. `DataLayerWorldState`保存layer instance hierarchy、many-to-many membership和world generation。
3. editor state拆成shared initial state与per-user visible/loaded/expanded state。
4. runtime state机至少支持Unloaded、Loaded与Activated，并分别暴露requested/effective state。
5. authority policy明确standalone、server、owning client与replicated observer的写权限。
6. parent/child传播、cycle、跨world、cross-instance和private/runtime兼容必须由domain validator拥有。
7. create context允许新增entity自动加入当前layer集合，同时生成可撤销transaction。
8. streaming/partition只消费layer state和membership artifact，不在Editor widget里直接装卸world。
9. unloaded descriptor保留stable object ID、bounds、layer、folder、level与instance provenance供Outliner使用。
10. cook输出layer registry、membership partition、initial state、authority policy与dependency digest。

### 3.4 Level Instance产品

1. `LevelInstanceSource`应引用版本化World Fragment/Scene资产，并保证stable local object IDs。
2. `LevelInstanceRecord`保存source、instance ID、parent instance、transform、runtime behavior和override artifact reference。
3. `LevelInstanceLoadRequest`拥有request ID、dependency set、budget、cancel token、priority与target generation。
4. subsystem状态机覆盖Registered、WaitingDependencies、Loading、Loaded、Unloading、Failed和Stale。
5. instantiate在单一事务中分配entity、建立hierarchy、provenance和ownership；失败必须回滚。
6. nested ancestry在注册和source变更时做loop detection，错误携带完整asset/instance chain。
7. override以source object ID和typed property address定位，并记录base/source/instance三方值或hash。
8. source reload产生rebase plan：clean、applied、conflict、orphan、type-mismatch、missing-object。
9. edit session在隔离edit world或明确staging layer工作，禁止直接破坏live instance authority。
10. commit执行source lock、revision preflight、transaction、atomic save、rebase all instances和rollback receipt。
11. discard恢复实例与selection/context，不能仅清空override Vec。
12. create-from-selection、move-to、break与pivot修改均是可撤销且可恢复的多资产operation。

### 3.5 Level Variant产品

1. 建立独立`ResourceKind::VariantSet`、source document、artifact importer和asset editor session。
2. asset由set、variant、binding、capture和optional function call组成，每层均有stable ID。
3. binding使用world/source/asset-safe object reference，并保留unloaded display metadata。
4. capture registry按reflected type列出可捕获property，支持transform、visibility、material和collection element。
5. recorded value使用typed codec与schema version，不以通用JSON字符串作为唯一格式。
6. Record读取当前值并写asset transaction；Apply不修改asset历史。
7. Apply先resolve全部binding/address，再分类missing/type mismatch/read-only/conflict。
8. atomic mode要求全部preflight成功后一次提交；best-effort mode必须显式选择并返回逐项receipt。
9. Preview使用可撤销preview scope，关闭/切换/失败时恢复原值。
10. runtime switching消费cooked immutable artifact，不能依赖Editor selection或UI control。
11. duplicate/move/merge必须保留或重建stable identity并检测重复binding/property。
12. Diff对比recorded/current/source schema，显示精确object/property和fix action。

### 3.6 World Outliner产品

1. `WorldOutlinerItem`是typed union，而不是把所有条目降为Scene node。
2. `WorldOutlinerHierarchyProvider`按mode按需创建root/children/parent，支持loaded entity和unloaded descriptor。
3. `WorldOutlinerMode`拥有selection sync、folder、context action、drag/drop、rename/delete与filter政策。
4. `WorldOutlinerColumn`提供header、cell projection、search text、sort key和write action。
5. `WorldOutlinerFilterRegistry`支持type/tag/component/layer/level/instance/state与plugin filter。
6. folder是editor organization record，不篡改runtime transform parent。
7. expansion、column layout、sort、filter preset、pin和per-user visibility进入workspace state。
8. model只materialize展开且接近viewport的row window；非可见subtree保留轻量aggregate/index。
9. topology delta表达insert/remove/move/reorder ranges与ancestor aggregates，避免统一full reflow。
10. retained template row使用bounded reusable pool，容量由viewport + overscan决定，不由world item总数决定。
11. context menu根据typed item、mode、capability和selection动态构造，并映射真实Editor operation。
12. filter结果保留ancestor、match reason、hidden count和query generation；取消旧query结果。
13. selection独立于过滤投影，批量操作必须说明隐含selected item和跨owner限制。
14. Outliner显示Data Layer、Level Instance、dirty/source-control/error等真实provider状态。

### 3.7 事务、作业、诊断与cook

1. 单World mutation进入现有transaction engine；跨资产操作使用可恢复multi-document transaction coordinator。
2. load、rebase、capture scan、large filter/sort、cook与reference audit接入Editor09 job预算/取消/关闭排空。
3. operation payload只传stable ID、expected revision和policy，不传UI row index或display text。
4.所有apply/commit返回durable receipt：before/after revision、affected IDs、diagnostics和rollback status。
5. diagnostic拥有stable code、owner/item/property、related asset、severity、source revision和fix action。
6. cook gate拒绝missing source、instance cycle、orphan override、invalid layer hierarchy和unresolved Variant capture。
7. runtime artifact只含运行所需状态，不携带Editor selection、expanded row或per-user load状态。
8. telemetry区分model materialization、visible paint、filter/sort、topology delta、instance load/rebase和variant apply成本。

## 4. 差距清单

### 4.1 P0：实施前必须先阻断

1. **P0-01** 在真实Variant asset/controller/executor接入前，主Editor不得把固定`Vehicle_Showcase`、18 overrides和2 conflicts呈现为可Preview/Apply的native产品；入口必须隐藏、标为明确fixture或返回unsupported。
2. **P0-02** 禁止把`render_layer_mask`、`active/active_in_hierarchy`、runtime parent、`ResourceKind::Data`或现有Hierarchy面板标记为Data Layer/World Outliner完成度；先建立独立产品类型与authority状态。
3. **P0-03** 修复Scene roundtrip数据损失：在World能够实例化、保留并重存Prefab/instance provenance之前，包含`prefab_instance`的Scene不得经当前load-save链静默写成`None`；必须fail closed或提供无损preservation path。
4. **P0-04** Variant apply、Prefab override apply和Level Instance commit在拥有stable object/property identity、expected revision、完整preflight、transaction、atomic rollback和durable receipt前不得修改共享资产或live authoritative World。
5. **P0-05** 在typed item/owner/editability/load/lock/visibility语义、真实context command和bounded row pool建立前，不得把当前`id/name/depth/selected`树宣称为工程级World Outliner或用10,000行paint测试替代十万级完整资格门。

### 4.2 P1：主线重构

#### 4.2.1 公共身份、资产与反射地址

1. **P1-01** 定义world-qualified object ID和generation，移除跨World边界裸u64歧义。
2. **P1-02** 为Scene/World Fragment对象定义rename/reparent稳定的source-local ID。
3. **P1-03** 定义`LevelInstanceId`、parent instance与source revision合同。
4. **P1-04** 建立source-object/instance-object双向provenance artifact。
5. **P1-05** 建立typed reflected property address，不以display path作为authority key。
6. **P1-06** property address包含type、component、field、collection selector与schema fingerprint。
7. **P1-07** 为Variant Set、Data Layer和World Fragment增加独立ResourceKind/marker/importer路由。
8. **P1-08** 所有新authoring asset使用版本化source schema、migration和unknown-field policy。
9. **P1-09** 所有异步request/delta/receipt携带owner、expected generation和request ID。
10. **P1-10** 定义display name/path与stable identity分离的API和lint/test合同。

#### 4.2.2 Data Layer domain

11. **P1-11** 实现`DataLayerAsset`和world-local `DataLayerInstance`。
12. **P1-12** 实现runtime/private与client/server load filter政策。
13. **P1-13** 实现Unloaded/Loaded/Activated requested state机。
14. **P1-14** 计算parent-aware effective state并检测cycle/类型不兼容。
15. **P1-15** 实现many-to-many entity membership与bulk transaction。
16. **P1-16** 分离shared initial editor state、per-user state与runtime state。
17. **P1-17** 实现actor/entity creation Data Layer context与undo/redo。
18. **P1-18** 建立server/client write authority、replication与stale generation policy。
19. **P1-19** 为unloaded descriptor保留layer membership及Outliner projection字段。
20. **P1-20** 输出partition/cook registry、membership artifact、initial state与diagnostics。

#### 4.2.3 Level Instance与Prefab收敛

21. **P1-21** 明确Prefab与Level Instance共同基础及各自独立产品责任，禁止简单type alias。
22. **P1-22** 建立版本化World Fragment/Level Instance source asset与stable local object IDs。
23. **P1-23** 实现instance register/load/wait/unload/fail/stale状态机。
24. **P1-24** 接入dependency readiness、priority、budget、cancel、retry与shutdown drain。
25. **P1-25** 实现nested ancestry、source/instance loop detection和完整chain diagnostic。
26. **P1-26** 以typed address和source object ID替换字符串override key。
27. **P1-27** 实现base/source/instance三方rebase与conflict/orphan分类。
28. **P1-28** 实现隔离edit session、current instance、dirty、commit与discard。
29. **P1-29** 实现create-from-selection、move-to-instance、break、pivot和bounds事务。
30. **P1-30** 让Scene load/save、prefab importer/runtime component和World实例生命周期无损闭环。

#### 4.2.4 Level Variant asset与执行

31. **P1-31** 实现Variant Set source document、importer、artifact和asset editor session。
32. **P1-32** 为set/variant/binding/capture/function call分配stable ID。
33. **P1-33** 实现loaded/unloaded都可诊断的object binding resolver。
34. **P1-34** 建立reflection-driven property capture registry与可捕获性policy。
35. **P1-35** 实现typed recorded-value codec、schema migration与custom type adapter。
36. **P1-36** 实现Record transaction及current-vs-recorded comparison。
37. **P1-37** 实现Preview scope、自动恢复、失败回滚和多viewport一致性。
38. **P1-38** 实现Apply全量resolve/preflight、atomic/best-effort policy和receipt。
39. **P1-39** 实现create/remove/move/duplicate/merge set、variant、binding和capture。
40. **P1-40** 产出runtime cooked switching artifact与source/artifact revision可见性。

#### 4.2.5 World Outliner模型与扩展

41. **P1-41** 定义typed Outliner item ID与World/Level/Folder/Entity/Component/Descriptor/Layer/Instance item。
42. **P1-42** 建立hierarchy provider，支持loaded object和unloaded descriptor。
43. **P1-43** 建立mode接口，拥有selection、context action、drag/drop、rename/delete和folder policy。
44. **P1-44** 建立column registry，支持cell/search/sort/action和plugin contribution。
45. **P1-45** 建立filter registry与type/tag/component/layer/level/instance/state query grammar。
46. **P1-46** 分离actor folder组织层级与runtime transform parent。
47. **P1-47** 实现expand/collapse、expand all、collapse all、reveal、breadcrumb和workspace持久化。
48. **P1-48** 实现pin、sort、group、only-selected、hidden/locked和filter preset。
49. **P1-49** 动态构造typed context menu并接到真实Editor operation/transaction。
50. **P1-50** 保留filter外selection并显式显示hidden selected count与跨owner操作限制。

#### 4.2.6 Outliner规模、delta与交互

51. **P1-51** 把retained row容量改成viewport + overscan有界pool，不按总item数clone node。
52. **P1-52** hierarchy provider按展开状态lazy materialize children和轻量subtree aggregate。
53. **P1-53** topology delta支持insert/remove/move/reorder range及ancestor aggregate patch。
54. **P1-54** filter/sort使用query generation、cancel与last-known-good projection。
55. **P1-55** 建立name/type/tag/layer等可索引字段，避免每次击键clone完整row Vec。
56. **P1-56** 保留现有generation/selection revision防护并扩展到typed item providers。
57. **P1-57** drag/drop先做owner、instance、layer、folder、cycle和lock validation再提交。
58. **P1-58** 大批量selection/reparent/layer assignment使用chunked transaction和进度/取消政策。
59. **P1-59** 维护滚动anchor、active row、rename target和expanded path跨delta稳定。
60. **P1-60** 建立10万/100万item memory、filter、sort、scroll、paint和churn性能预算。

#### 4.2.7 Editor会话、diagnostics、cook与测试

61. **P1-61** Variant/Data Layer/Level Instance编辑接入document dirty/save/autosave/recovery。
62. **P1-62** 跨Scene/source资产commit接入可恢复multi-document transaction coordinator。
63. **P1-63** load/rebase/audit/filter/cook接入Editor job admission、cancel、progress和shutdown。
64. **P1-64** 定义stable diagnostic code、owner/item/property定位、related asset与fix action。
65. **P1-65** 建立source control/dirty/error/load/visibility column的真实provider。
66. **P1-66** cook拒绝missing source、instance cycle、orphan override、invalid layer和unresolved capture。
67. **P1-67** 建立所有新asset的roundtrip、migration、unknown-field和deterministic artifact测试。
68. **P1-68** 建立Variant capture/apply/rollback、Data Layer authority和Level Instance lifecycle矩阵。
69. **P1-69** 建立Prefab link无损Scene roundtrip及source reload/rebase regression测试。
70. **P1-70** 建立Outliner多provider、unloaded descriptor、context action与十万级端到端资格门。

### 4.3 P2：主线完成后扩展

1. **P2-01** Variant thumbnail、director/function call、remote preview和multi-user conflict UI。
2. **P2-02** Variant composition、inheritance、parameterized variants和batch render/export。
3. **P2-03** Data Layer external content bundle与跨项目可挂载layer package。
4. **P2-04** Data Layer runtime debugging、network authority timeline和state heatmap。
5. **P2-05** Level Instance property override policy plugin与per-type merge adapter。
6. **P2-06** Level Instance HLOD、world partition container和distributed cook integration。
7. **P2-07** multi-user Level Instance edit lease、review、merge和change-list workflow。
8. **P2-08** Outliner custom grouping、saved collections、smart folders和bookmark sets。
9. **P2-09** Outliner remote runtime/PIE/server world comparison与cross-world selection bridge。
10. **P2-10** Outliner background indexed query、million-item paging和GPU-assisted visualization。
11. **P2-11** Variant/Data Layer/Instance Python/commandlet automation与headless validation API。
12. **P2-12** 跨Variant、Data Layer、Instance、PCG和Sequencer的统一provenance/diff浏览器。

## 5. 里程碑

| Milestone | 交付内容 | 退出条件 |
|---|---|---|
| M0 | 产品真实性与数据损失止血 | Variant固定成功入口不可误导；Prefab link load-save fail closed或无损 |
| M1 | stable object/source/instance/property identity ADR与公共合同 | identity、generation、address、migration测试通过 |
| M2 | Scene/Prefab无损闭环 | importer、World instantiate、provenance、save/reopen不丢链接与override |
| M3 | Data Layer source/world/runtime authority | asset、membership、state hierarchy、per-user state和authority测试通过 |
| M4 | Level Instance load subsystem | dependency、async load/unload、loop、cancel、rollback和provenance通过 |
| M5 | Level Instance edit/rebase | edit/commit/discard、source revision、three-way conflict和break通过 |
| M6 | Variant asset/capture | set/variant/binding/capture、typed value、record和migration通过 |
| M7 | Variant preview/apply/runtime artifact | resolve/preflight/rollback/receipt及cooked switching通过 |
| M8 | typed World Outliner model | item/hierarchy/mode/column/filter/provider和unloaded descriptor通过 |
| M9 | Outliner interaction与transaction | expansion、context menu、folder、drag/drop、layer/instance action通过 |
| M10 | scale、jobs、cook与diagnostics | bounded pool、range delta、10万/100万预算、cancel和cook gate通过 |
| M11 | migration、兼容清理与文档收敛 | fixture删除、旧字符串key迁移、全矩阵与reference recheck通过 |

M0-M2必须先于任何“Apply/Commit成功”文案；M3-M7可在公共identity合同稳定后分域并行；M8先复用现有Hierarchy delta/selection/transaction基础，再在M9-M10扩展语义和规模。Editor16继续拥有World Partition cell/HLOD大域，Editor03继续拥有通用Scene/Prefab/selection基线，本报告只增加四个交叉产品的专属owner。

## 6. 验收门

1. **G01** production搜索可定位独立Variant Set、Data Layer和Level Instance类型、asset kind、owner与公开合同。
2. **G02** 固定`Vehicle_Showcase/18 overrides/2 conflicts`不再作为真实项目默认成功状态。
3. **G03** Variant Preview/Apply按钮必须触达真实operation、revision、transaction和receipt。
4. **G04** 任一binding/property无法resolve时atomic apply不产生部分World mutation。
5. **G05** Preview关闭、切换Variant、undo、play transition和异常均恢复原值。
6. **G06** Variant source roundtrip/migration保留stable IDs、typed value和unknown fields。
7. **G07** Data Layer requested/effective及Unloaded/Loaded/Activated状态机有完整table test。
8. **G08** parent propagation、cycle、private/runtime和client/server非法组合fail closed。
9. **G09** per-user loaded/visible state不写入共享Scene source或污染其他用户。
10. **G10** runtime layer state写入遵守server/client authority并拒绝stale generation。
11. **G11** unloaded descriptor仍可在Outliner显示level/folder/layer/instance/provenance。
12. **G12** cook artifact包含layer registry/membership/initial state并拒绝missing/cycle。
13. **G13** 含`prefab_instance`的Scene load-save-reopen逐字段相等，不再写成`None`。
14. **G14** source entity rename/reparent后instance override仍由stable ID解析。
15. **G15** nested Level Instance loop在注册/加载/cook阶段均返回完整chain diagnostic。
16. **G16** dependency missing、cancel、load failure和unload failure不泄漏entity、ownership或provenance。
17. **G17** source reload将override分类为clean/applied/conflict/orphan/type mismatch并可重演。
18. **G18** edit commit执行revision preflight、atomic save、rollback；discard恢复world和context。
19. **G19** create/move/break/pivot操作可undo/redo、save/reopen并通过crash recovery fixture。
20. **G20** Outliner typed item ID在多World、PIE、unloaded descriptor和nested instance间不碰撞。
21. **G21** mode/column/filter/provider可由first-party plugin注册并在卸载时安全撤销。
22. **G22** actor folder变更不修改runtime transform parent，reparent语义由drop target明确决定。
23. **G23** expand/collapse、reveal、breadcrumb、filter前后恢复和workspace persistence通过。
24. **G24** SceneNode右键菜单所有可见action均触达真实command；无executor项不显示。
25. **G25** filtered projection下批量操作保留完整authoritative selection并提示隐藏项。
26. **G26** topology insert/remove/move/reorder局部更新，不对单一变化强制全树reflow。
27. **G27** retained row node数量受viewport + overscan上限约束，与10万item总量无关。
28. **G28** 10万loaded items下scroll/paint/input维持帧预算，filter/sort不阻塞UI线程超预算。
29. **G29** 100万descriptor下memory、query latency、cancel和last-known-good projection达到书面预算。
30. **G30** 所有异步结果校验owner/request/source generation，旧结果不得覆盖新World或document。
31. **G31** dynamic测试、migration golden、cook commandlet、crash recovery与性能qualification在Windows首选lane通过。
32. **G32** 文档、capability manifest、菜单、UI和runtime artifact对四个产品的完成状态一致，无fixture伪装成功。

## 7. 本轮验证与限制

本轮完成静态源码、测试inventory、参考源码与物理范围fingerprint复核，没有修改production Runtime、Editor、plugin、interface代码或tests，也没有执行实现修正。

上一轮`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断。本轮没有重复无法到达本产品行为的相同动态lane；101个test attributes仅作为静态覆盖inventory，不能声称动态通过。

实施时必须先重算232文件fingerprint并复核2个在途入口。任何milestone若改变Scene/Prefab公共合同、Runtime/Editor跨模块接口、ResourceKind或operator workflow，还必须同步执行模块文档维护和hard-cutover审查，禁止长期保留字符串override key、静态Variant route或双重Outliner authority。
