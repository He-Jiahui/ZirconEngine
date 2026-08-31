---
title: Editor Level、Variant、Data Layer、Level Instance 与 World Outliner 当前源码复核
category: zircon_editor
report_id: Editor115
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor41
refreshes:
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
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
tests:
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_hierarchy_fragment.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
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

# 115 · Editor Level / Variant / Data Layer / Level Instance / World Outliner 工程化差距

## 1. 结论

当前工作树的层级树不是空壳：Runtime inspection 能生成 entity、parent、depth、display name、kind、subtree hash 和 world generation；Editor 已有 generation 防旧、稀疏 name/selection patch、祖先保留过滤、F2/双击重命名、多选拖拽 reparent、undo/redo transaction、pointer 计算和 10,000 行可见裁剪。5,000 层深度测试使用迭代构建，未发现递归栈风险。这些是新 Outliner 应复用的真实底座。

但这些底座只形成 Scene hierarchy projection，不是工程级 World Outliner。最终 pane row 只保留 `id/name/depth/selected`，runtime 的 kind/active/has-children 已丢失；没有 World/Level/Folder/Actor/Component/Descriptor/Layer/Instance typed item、source provenance、unloaded actor、folder 组织层、Data Layer membership、编辑锁/可见性、loaded/streaming、dirty/source-control/error 列、mode/column/filter registry 或展开状态持久化。模板 bridge 仍按总行数 clone retained node，topology delta 普遍要求完整 reflow，不能用 10,000 行 paint 测试宣称十万级资格。

Level Variant 是当前最明显的虚假产品面：workspace 固定显示 `Vehicle_Showcase`、`Variant_Red`、`18 overrides`、`2 conflicts`，Preview/Apply 只回写 queued 文本。生产没有 Variant asset kind、object binding、typed property address、capture registry、recorded value codec、apply/preview executor、revision preflight、rollback、cook artifact 或 runtime switch API。

Data Layer 与 Level Instance 更是产品类型缺失。`ResourceKind` 没有对应 kind，Scene entity 没有 owning level、layer membership、instance provenance。`render_layer_mask` 是渲染过滤，`active` 是 runtime activation，均不能替代 Data Layer 的 requested/effective、Unloaded/Loaded/Activated、per-user editor 状态及 server/client authority。Prefab DTO 也未闭环：`World::from_scene_asset` 不读取 `prefab_instance`，`World::to_scene_asset` 固定写 `None`，现有 load-save 能静默擦除链接、local transform 和 overrides，这是 P0 数据损失。

本轮冻结的选定 Zircon scope 为 55 files / 9,319 lines / 361,755 bytes / 70 test attributes，参考 scope 为 22 / 15,441 / 557,921 / 1 test attribute；union 为 77 / 24,760 / 919,676 / 71。Zircon fingerprint `e13d11d53f80d6346b8b4c7343477250e850ecc676a7a16689ea78d233987749`，refs `be457f551dc2269b1d834cc6b7e567da1f92ca54ebd7ecfbf8e5d6ecd378a7ee`，union `5b55d4d25648830453659d68d1ba01b6742e0753ee380817ca1b21a52ea0c489`。本报告登记 5 个 P0、70 个 P1、12 个 P2 和 M0-M11；不修改生产代码。

## 2. 证据与边界

### 2.1 当前实现核对

1. `scene/inspection` 的 hierarchy builder 使用迭代 preorder、visited 集合和稳定 subtree hash，生成不可变 artifact、children map 与 generation。
2. name-only 变化可以通过 sparse row override 传播到祖先；topology delta 有 added/changed/removed anchor，但当前 projection 对结构变化仍通常要求 full reflow。
3. retained host 有 entity/control 双向映射；fragment apply 会校验 generation、selection revision 和 anchor，不匹配时要求 resync。
4. filter 为 O(N) name 匹配并保留完整 ancestor path；F2/double-click、drag/reparent 和 mutation undo/redo 走真实 `EditorIntent`/transaction。
5. 最终 `SceneNodeData` 降级为四字段，context menu 仍是 Open/Rename/Duplicate/Delete 字符串；`context_target_path` 不是稳定 world-qualified item ID。
6. `virtual_rows` 会按 `total_row_count - authored_count` 逐行 clone；`scene_tree_control_ids` 还会扫描全部 virtual controls，结构 churn、filter、sort 都可能重建完整双向 map。
7. Level Variant ZUI 的 Set/Variant 下拉、Capture、Preview、Apply 和反馈均固定；无 document mode、query model、asset session、operation executor 或 runtime consumer。
8. `PrefabAsset` 内嵌完整 Scene，`PrefabInstanceAsset` 使用字符串 entity/property path 与 `serde_json::Value`；prefab_tools importer 是 DiagnosticOnly，Editor operations 只有 descriptor。
9. `apply_prefab_overrides`/`revert_prefab_overrides` 只处理 DTO Vec；`break_prefab_instance` 只返回 DTO。World IO 忽略并清除 `prefab_instance`，形成可静默数据丢失。
10. `ResourceKind::Prefab` 存在，但没有 VariantSet、DataLayer、WorldFragment/LevelInstance kind、catalog entry、factory、toolkit 或 cook registration。

### 2.2 不能混用的语义

1. runtime parent 是 transform/scene hierarchy，不是 actor folder、Data Layer parent 或 instance containment。
2. `render_layer_mask` 只影响 render filtering；它不表达 source asset、编辑加载、网络 authority 或 layer membership。
3. `active/active_in_hierarchy` 是实体激活传播；它不表达 editor-only visibility/lock 或 requested/effective layer state。
4. Scene `world_generation` 是 world revision/fence，不是 Data Layer、Level Instance 或 variant source revision。
5. 通用 `ResourceKind::Data` 不能作为 Data Layer 产品类型；Prefab 名称也不能自动获得 Level Instance 生命周期。
6. Editor16 的 World Partition cell/HLOD 大域、Editor03 的通用 Scene/Prefab 基线仍是邻接 owner，本报告只登记四个产品交叉缺口。

## 3. 关键差距

### 3.1 Level Variant

1. 缺少 `VariantSetAsset`、set/variant/binding/capture/function-call 稳定 ID、schema version、migration 和 artifact。
2. 缺少 world/source-safe object binding；当前不存在 loaded/unloaded label、missing binding、source revision 或 provenance map。
3. 缺少 reflection-driven capture registry；没有 typed property address、component field ID、array index、schema fingerprint 或 read-only policy。
4. recorded value 不能只存 JSON 字符串；当前没有 typed codec、custom type adapter、transform/visibility/material special case。
5. 没有 Record current value、current-vs-recorded diff、Preview reversible scope、Apply preflight、atomic/best-effort policy、rollback 或 durable receipt。
6. 没有 duplicate/move/merge set/variant/binding/property、conflict class、orphan capture、cook validation 或 runtime switching artifact。
7. 固定 `18 overrides / 2 conflicts` 和 queued 成功文字构成第二 authority，必须在真实 controller 接入前隐藏或标为 fixture。

### 3.2 Data Layer

1. 缺少 `DataLayerAsset`、`DataLayerId`、world-local `DataLayerInstance`、membership record、parent hierarchy 和 cycle validator。
2. 缺少 runtime/private、client/server filter，requested/effective state 和 Unloaded/Loaded/Activated 状态机。
3. 缺少 shared initial editor state 与 per-user loaded/visible state 的隔离，新增 entity 没有 current layer creation context。
4. 缺少 server/client authority、replication、state event、generation/stale request policy、unloaded actor descriptor 和 partition/cook registry。
5. 缺少 layer browser、membership bulk transaction、invalid member diagnostics、rename/reparent/delete/migration 以及 Outliner columns。
6. 任何把 `render_layer_mask` 或 `active` 重命名为 Data Layer 的实现都会把 rendering/runtime activation 与 authoring ownership 混成一条错误 authority。

### 3.3 Level Instance / Prefab

1. Scene entity 只有 scene-local u64 与 parent；没有 source-local stable ID、owning level、subscene、instance root 或 external package。
2. Prefab override 用字符串路径，rename/reparent/schema migration 后不可稳定解析；无 declared type、schema fingerprint、base hash 或 three-way rebase。
3. 缺少 source/instance 双向 provenance、added/removed child、component topology override、orphan override、nested ancestry 和 loop diagnostic。
4. 缺少 register/load/wait/unload/fail/stale 生命周期、dependency readiness、priority、budget、cancel、retry、shutdown drain。
5. 缺少隔离 edit session、current instance、dirty、commit/discard、source revision lock、atomic save、create/move/break/pivot/bounds transaction。
6. `World::from_scene_asset` 不消费 `prefab_instance`，`to_scene_asset` 固定 None；必须先 fail closed 或建立无损 preservation path，再允许编辑器 Save。
7. prefab_tools 的 importer、factory、runtime component、Editor operation、catalog、App provider 没有生产闭环，不能宣称 Level Instance 已完成。

### 3.4 World Outliner

1. 定义 typed `WorldOutlinerItem` union 和稳定 namespace，至少覆盖 World/Level/Folder/Actor/Component/Descriptor/DataLayer/LevelInstance/Entity。
2. 建立可插拔 hierarchy provider、mode、column、filter registry；支持 loaded entity、unloaded descriptor、source/provenance 和 plugin contribution。
3. folder 组织层与 runtime transform parent 分离；context menu、rename、delete、drag/drop 必须映射真实 operation 和 transaction。
4. filter grammar 需覆盖 type/tag/component/layer/level/instance/state，并保留 hidden selected count、match reason、query generation 和 last-known-good projection。
5. expansion、sort、group、pin、bookmark、filter preset、per-user visibility 与 workspace state 必须持久化，不能把静态 `expanded` 当作运行状态。
6. row model 使用 viewport + overscan bounded pool；hierarchy delta 支持 insert/remove/move/reorder range 与 ancestor aggregate，避免总数级 clone/reflow。
7. 需要 100k loaded items、1M descriptors 的 memory、filter、sort、scroll、paint、selection、structural churn 和 cancel 预算；现有 10k draw test 不足以证明资格。

## 4. 差距清单

### 4.1 P0：实施前必须阻断

1. **P0-01** 固定 Variant workspace 不得以 native Preview/Apply 成功状态出现；无真实 provider 时隐藏、标 fixture 或返回 unsupported。
2. **P0-02** 禁止把 `render_layer_mask`、`active`、runtime parent、`ResourceKind::Data` 或现有 hierarchy 宣称为 Data Layer/World Outliner。
3. **P0-03** 修复 Prefab/Instance Scene round-trip 数据损失；含 `prefab_instance` 的文档必须无损保存或 fail closed。
4. **P0-04** Variant Apply、Prefab override Apply、Level Instance Commit 在 stable identity、expected revision、完整 preflight、atomic rollback、durable receipt 前不得修改共享 asset/live world。
5. **P0-05** typed item/owner/editability/load/lock/visibility、真实 command 和 bounded row pool 建立前，不得以 10k paint 测试宣称工程级 Outliner。

### 4.2 P1：重构主线

1. **P1-01** 定义跨 World/session 不碰撞的 world-qualified object ID。
2. **P1-02** 为 Scene/World Fragment 对象分配 rename/reparent 稳定的 source-local ID。
3. **P1-03** 定义 `LevelInstanceId`、parent instance 和 source revision 合同。
4. **P1-04** 生成 source-object 与 instance-object 双向 provenance artifact。
5. **P1-05** 建立 typed reflected property address，不以 display path 做 authority key。
6. **P1-06** 让 address 包含 component、field、collection selector 与 schema fingerprint。
7. **P1-07** 为 VariantSet、DataLayer、WorldFragment 增加独立 ResourceKind/marker/importer。
8. **P1-08** 为新 authoring asset 定义版本化 schema、migration 和 unknown-field policy。
9. **P1-09** 所有异步 request/delta/receipt 携带 owner、expected generation、request ID。
10. **P1-10** 用 lint/test 固化 display name/path 与 stable identity 的分离。
11. **P1-11** 实现 `DataLayerAsset` 和 world-local `DataLayerInstance`。
12. **P1-12** 实现 membership record、批量 assignment 和 owner validation。
13. **P1-13** 实现 runtime/private、client/server layer filter policy。
14. **P1-14** 实现 Unloaded/Loaded/Activated requested state 机。
15. **P1-15** 计算 parent-aware effective state 并拒绝 cycle/type-invalid hierarchy。
16. **P1-16** 分离 shared initial editor state、per-user state 与 runtime state。
17. **P1-17** 将当前 layer context 接入 actor/entity creation 与 undo/redo。
18. **P1-18** 定义 server/client write authority、replication 和 stale generation policy。
19. **P1-19** 为 unloaded descriptor 保留 layer membership 与 Outliner 字段。
20. **P1-20** 产出 partition/cook registry、membership artifact、initial state 与 diagnostics。
21. **P1-21** 明确 Prefab 与 Level Instance 的共同基础及独立产品责任，禁止 type alias。
22. **P1-22** 建立版本化 World Fragment/Level Instance source asset 与 local IDs。
23. **P1-23** 实现 instance register/load/wait/unload/fail/stale 状态机。
24. **P1-24** 接入 dependency readiness、priority、budget、cancel、retry、shutdown drain。
25. **P1-25** 实现 nested ancestry、source/instance loop detection 和完整 chain diagnostic。
26. **P1-26** 以 typed address 与 source object ID 替换字符串 override key。
27. **P1-27** 实现 base/source/instance 三方 rebase 与 conflict/orphan 分类。
28. **P1-28** 实现隔离 edit session、current instance、dirty、commit、discard。
29. **P1-29** 实现 create-from-selection、move-to-instance、break、pivot、bounds transaction。
30. **P1-30** 让 Scene IO、prefab importer/runtime component 与 World lifecycle 无损闭环。
31. **P1-31** 实现 Variant Set source document、importer、artifact 和 asset editor session。
32. **P1-32** 为 set/variant/binding/capture/function-call 分配 stable ID。
33. **P1-33** 实现 loaded/unloaded 均可诊断的 object binding resolver。
34. **P1-34** 建立 reflection-driven property capture registry 与 capturability policy。
35. **P1-35** 实现 typed recorded-value codec、schema migration、custom type adapter。
36. **P1-36** 实现 Record transaction 与 current-vs-recorded comparison。
37. **P1-37** 实现可撤销 Preview scope、异常恢复、Variant 切换和多 viewport 一致性。
38. **P1-38** 实现 Apply 全量 resolve/preflight、atomic/best-effort policy 和 receipt。
39. **P1-39** 实现 set/variant/binding/capture 的 create/remove/move/duplicate/merge。
40. **P1-40** 产出 runtime cooked switching artifact，并暴露 source/artifact revision。
41. **P1-41** 定义 typed Outliner item ID 与 World/Level/Folder/Entity/Component union。
42. **P1-42** 增加 Descriptor/DataLayer/LevelInstance item 和稳定 namespace。
43. **P1-43** 建立 hierarchy provider，支持 loaded object 与 unloaded descriptor。
44. **P1-44** 建立 mode 接口，拥有 selection、context、drag/drop、rename/delete policy。
45. **P1-45** 建立 column registry，支持 cell/search/sort/action 和 plugin contribution。
46. **P1-46** 建立 type/tag/component/layer/level/instance/state filter grammar。
47. **P1-47** 将 actor folder 组织层与 runtime transform parent 分离。
48. **P1-48** 实现 expand/collapse、expand all、reveal、breadcrumb 与 workspace persistence。
49. **P1-49** 实现 pin/sort/group/only-selected/hidden/locked 和 filter preset。
50. **P1-50** 动态构造 typed context menu，并接到真实 command/transaction。
51. **P1-51** 将 retained row 容量改为 viewport + overscan bounded pool。
52. **P1-52** 按展开状态 lazy materialize children，并保留轻量 subtree aggregate。
53. **P1-53** 让 topology delta 支持 insert/remove/move/reorder range 与 ancestor patch。
54. **P1-54** 为 filter/sort 使用 query generation、cancel 和 last-known-good projection。
55. **P1-55** 建立 name/type/tag/layer 索引，避免每次击键 clone 完整 row Vec。
56. **P1-56** 将 generation/selection revision fence 扩展到全部 typed providers。
57. **P1-57** drag/drop 先校验 owner、instance、layer、folder、cycle、lock 再提交。
58. **P1-58** 大批量 selection/reparent/layer assignment 使用 chunked transaction、progress、cancel。
59. **P1-59** 维护 scroll anchor、active row、rename target、expanded path 跨 delta 稳定。
60. **P1-60** 建立 100k loaded / 1M descriptor 的 memory、query、scroll、paint、churn 预算。
61. **P1-61** Variant/DataLayer/LevelInstance 编辑接入 document dirty/save/autosave/recovery。
62. **P1-62** 跨 Scene/source asset commit 接入可恢复 multi-document transaction coordinator。
63. **P1-63** load/rebase/audit/filter/cook 接入 Editor job admission、cancel、progress、shutdown。
64. **P1-64** 定义 stable diagnostic code、owner/item/property、related asset、fix action。
65. **P1-65** 建立 source-control/dirty/error/load/visibility/lock 的真实 provider columns。
66. **P1-66** cook 拒绝 missing source、instance cycle、orphan override、invalid layer、unresolved capture。
67. **P1-67** 建立新 asset 的 roundtrip、migration、unknown-field、deterministic artifact 测试。
68. **P1-68** 建立 Variant capture/apply/rollback、DataLayer authority、LevelInstance lifecycle 矩阵。
69. **P1-69** 建立 Prefab link 无损 Scene roundtrip 及 source reload/rebase regression。
70. **P1-70** 建立多 provider、unloaded descriptor、context action 与十万级端到端 Outliner qualification。

### 4.3 P2：主线完成后扩展

1. **P2-01** Variant thumbnail、director/function-call、remote preview 与 multi-user conflict UI。
2. **P2-02** Variant composition、inheritance、parameterized variants 与 batch render/export。
3. **P2-03** Data Layer external content bundle 与跨项目可挂载 layer package。
4. **P2-04** Data Layer runtime debugging、network authority timeline 与 state heatmap。
5. **P2-05** Level Instance per-type property merge policy plugin。
6. **P2-06** Level Instance HLOD、world partition container 与 distributed cook integration。
7. **P2-07** multi-user Level Instance edit lease、review、merge 与 change-list workflow。
8. **P2-08** Outliner custom grouping、saved collections、smart folders 与 bookmarks。
9. **P2-09** remote PIE/server world comparison 与 cross-world selection bridge。
10. **P2-10** background indexed query、million-item paging 与 GPU-assisted visualization。
11. **P2-11** Variant/DataLayer/Instance Python、commandlet automation 与 headless validation。
12. **P2-12** 跨 Variant/DataLayer/Instance/PCG/Sequencer 的统一 provenance/diff browser。

## 5. 目标架构与里程碑

目标 owner 必须拆成四条产品链：`VariantSetSource -> binding/capture compiler -> immutable VariantArtifact -> preview/apply/runtime switch`；`DataLayerAsset -> membership compiler -> DataLayerRegistry -> authority-bound state snapshot`；`LevelInstanceSource -> load/edit/rebase service -> provenance/instance artifact -> World adapter`；`WorldOutlinerModel -> typed providers -> bounded projection -> truthful commands`。Scene/World 只保存稳定 ID 和引用，不保存 UI row index、display path 或 per-user expansion。

| Milestone | 交付与退出条件 |
|---|---|
| M0 | 移除固定成功文案；Prefab link 保存无损或 fail closed。 |
| M1 | identity、generation、typed address、schema/migration ADR 与测试冻结。 |
| M2 | Scene/Prefab instantiate、provenance、save/reopen 无损。 |
| M3 | Data Layer asset/membership/state/authority/cook registry 完成。 |
| M4 | Level Instance load/unload/fail/cancel/retry/shutdown 与 loop detection 完成。 |
| M5 | isolated edit、commit/discard、three-way rebase、break/pivot/bounds 完成。 |
| M6 | Variant source/binding/capture/typed value/Record/migration 完成。 |
| M7 | Preview/Apply preflight、rollback、receipt、runtime artifact 完成。 |
| M8 | typed Outliner model/provider/mode/column/filter/unloaded descriptor 完成。 |
| M9 | folder、expansion、context menu、drag/drop、layer/instance operations 接入 transaction。 |
| M10 | bounded projection、range delta、jobs/cancel、10万/100万性能、cook/diagnostics gate 完成。 |
| M11 | fixture 删除、字符串 key 迁移、文档/capability/UI/runtime 状态一致并完成 reference recheck。 |

M0-M2 必须先于任何 Apply/Commit 成功文案；M3-M7 依赖公共 identity；M8 应先复用现有 hierarchy generation/selection/transaction 基础，再在 M9-M10 扩展产品语义与规模。

## 6. 验收门

1. **G01-G06** production 能定位独立 VariantSet/DataLayer/LevelInstance 类型、asset kind、provider、revision；固定 Variant fixture 不再伪装成功；Preview/Apply 有真实 operation、atomic preflight、恢复和 roundtrip。
2. **G07-G12** Data Layer requested/effective、Unloaded/Loaded/Activated、parent/cycle、private/runtime、client/server、per-user state、descriptor 与 cook artifact 均有 table/golden 测试。
3. **G13-G19** Prefab/Instance Scene load-save-reopen 逐字段相等；rename/reparent 稳定解析；nested loop、missing dependency、cancel、failure、rebase conflict、commit/discard、create/move/break 可诊断且可 undo/recover。
4. **G20-G25** Outliner typed IDs 在多 World/PIE/unloaded/nested instance 不碰撞；mode/column/filter 可注册卸载；folder 不改 transform parent；context action 只显示真实 executor；filtered selection 明确隐藏项。
5. **G26-G30** topology range patch、bounded row pool、100k/1M memory/latency、query cancel/last-known-good、owner/request/source generation fence 通过。
6. **G31-G32** Windows 首选 lane 的 dynamic/migration/cook/crash-recovery/performance 矩阵通过；文档、manifest、菜单、UI、runtime artifact 对完成状态一致，无 fixture 伪装。

## 7. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码和物理范围 fingerprint 复核；没有修改 Runtime、Editor、plugin、interface 或 tests，没有执行 Cargo 动态测试。选定 scope 的缺失路径为 0，报告索引/coverage 每项应恰为 1，`git diff --check` 作为收尾门。实施前必须重算 77-file manifest 并复核任何在途的 scene-inspection/template 入口；跨模块 contract、ResourceKind 或 operator workflow 改变时，仍需执行模块文档维护与 hard-cutover 审查。
