---
title: Editor Archetype、Class Defaults、Instance Override、Property Propagation 与 Reset-to-Default 当前源码复核
category: zircon_editor
report_id: Editor118
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor44
refreshes:
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime_interface/src/reflect
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_plugins/prefab_tools
  - zircon_plugins/editor_support/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectArchetype.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Internal/UObject/UObjectArchetypeHelper.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/ClassDefaultObjectTest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/InheritableComponentHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/InheritableComponentHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ComponentInstanceDataCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ComponentInstanceDataCache.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyEditorArchetypePolicy.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyEditorArchetypePolicy.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SResetToDefaultMenu.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/UserInterface/PropertyEditor/SResetToDefaultPropertyEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverrideAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverrideAsset.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverridePolicy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverridePolicy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition/WorldPartitionPropertyOverride.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/Private/K2Node_GetClassDefaults.cpp
  - dev/godot/scene/property_utils.h
  - dev/godot/scene/property_utils.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/editor/inspector/editor_inspector.h
  - dev/godot/editor/inspector/editor_inspector.cpp
  - dev/godot/editor/scene/packed_scene_editor_plugin.h
  - dev/godot/editor/scene/packed_scene_editor_plugin.cpp
  - dev/godot/tests/scene/test_packed_scene.cpp
  - dev/Fyrox/fyrox-core/src/variable.rs
  - dev/Fyrox/fyrox-core/src/reflect/inherit.rs
  - dev/Fyrox/fyrox-ui/src/inspector/editors/inherit.rs
  - dev/Fyrox/editor/src/scene/property.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_reflect/src/std_traits.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 118 · Editor Archetype / Class Defaults / Instance Override / Property Propagation 工程化差距

## 1. 结论

当前 Zircon 没有统一的对象默认值、class default、Prefab 原型、实例 override、来源传播或 Reset-to-Default 产品。相近名称来自四个不同层：reflection field 的可选 `default_value`、Prefab DTO 的字符串路径 + JSON override、ECS archetype 的 component signature table、Inspector/Material 的局部 projection。ECS archetype 解决 storage locality，不是对象原型或继承 authority；reflection default 也没有来源、层级、版本或 reset policy。

Prefab 链目前不是“功能少”，而是没有安全写入条件：`PrefabAsset` 内嵌 Scene，`PrefabInstanceAsset` 以 `entity_path/property_path/serde_json::Value` 表达差量，没有 stable source object/component/property identity、source revision、typed base value、schema migration、topology override、orphan/conflict 状态或传播回执。source rename/reparent/component replacement 后路径会失配；World IO 又不读取 `prefab_instance` 并在保存时固定写 `None`，合法 load-save 会静默擦除 link、local transform 和 overrides。

`prefab_tools` 的 apply/revert/break 只是 DTO helper：apply 去重后清空 vector，revert 清空 vector，break 返回 transform/override；没有修改 source、恢复 live effective value、物化 subtree、remap reference 或 transaction。插件 importer 明确 DiagnosticOnly，Editor 只有 descriptor、无 factory/executor，Prefab Workbench 固定 `PF_Chest`、`Chest_04`、18 children、6 overrides、2 warnings 并伪造 queued 成功。接入这些入口会把不完整能力变成数据破坏。

目标应建立单一 `DefaultValueAuthority`：Native Schema、Script/Class Default、Prefab/Archetype Source、Variant、Instance Override、Session/Runtime Transient 作为可观察层；versioned source + stable property address 承载差量；propagation service 分类 clean/overridden/conflict/orphan/missing/type-incompatible；Editor 以 transaction 执行 reset/apply-to-source/revert/break；cook 只发布 resolved runtime artifact 和 provenance。

本轮 Zircon scope 为 80 files / 12,850 lines / 11,690 non-empty / 463,627 bytes / 59 test attributes；参考 scope 为 43 / 29,347 / 25,491 / 1,079,792 / 46；union 为 123 / 42,197 / 37,181 / 1,543,419 / 105。Zircon fingerprint `6c97b643a007efcfeb4662c6982da29042f990ad868d93ba286328ff28955476`，refs `d54b0b801cebda11d275678f1fba0e1f1eadef54058810871d1f41f9b2e2e8db`，union `e44dafc09675c28358c79cbb28b332ff408dbc3205a465991ee332f6f85878ac`。本报告登记 5 个 P0、70 个 P1、12 个 P2 与 M0-M11；不修改生产代码。

## 2. 证据与参考差异

### 2.1 当前代码事实

1. reflection `ReflectFieldInfo` 只有可选 `default_value`；registry 只做类型合法性，不记录 default origin、source revision、override state 或 reset action。
2. generic Inspector row 只保存 id/label/type/string value/editable；snapshot 不携带 default、origin、mixed、local override、expected-before 或 target revision。
3. `PrefabAsset` 只有 URI、name、内嵌 Scene、字符串 exposed properties；`PrefabInstanceAsset` 只有 source reference、local transform、override vector。
4. override key 是 entity/property string path，value 是无类型 JSON；无 schema fingerprint、base hash、source revision、typed codec、topology/reference operation。
5. `World::from_scene_asset` 不消费 entity `prefab_instance`；`World::to_scene_asset` 固定写 `None`。Scene roundtrip 存在 P0 数据损失。
6. `prefab_tools` importer 是 DiagnosticOnly，Editor 五个 operation 只有 descriptor；helper 不接 World、Document、Asset transaction 或 source save。
7. `apply_prefab_overrides`、`revert_prefab_overrides`、`break_prefab_instance` 不改变 live scene/source，tests 只证明 DTO/vector 行为。
8. ECS `archetype` 目录维护 signature、table、row、locator、change tick，是运行时存储布局，不能承担 CDO/Prefab/instance default。
9. Material Editor 有局部 `default_value/override_value/is_overridden` projection，是 domain adapter，不是通用 default resolver。
10. Prefab Workbench 与 plugin 没有 provider/factory/catalog/transaction 连接；固定 feedback 不反映真实 artifact 或 revision。

### 2.2 参考引擎吸收边界

1. Unreal 区分 CDO、object archetype、subobject template、InheritableComponentHandler、ComponentInstanceDataCache、PropertyEditorArchetypePolicy 和 LevelInstancePropertyOverridePolicy。
2. Unreal reset 由 property policy 判断 default 来源、差异和可重置性；不是从一个 JSON 值直接覆盖。
3. Godot 按 native class default、script exported default、scene inheritance/instance stack 解析 property origin，PackedScene 保留 owner/instance 状态。
4. Fyrox 的 modified bit、parent availability、typed Revert action 展示了最小但完整的 inherited-value 语义。
5. Bevy ReflectDefault 与 resolved ScenePatch 证明 typed default/spawn layering；不提供 Zircon 所需的 Editor propagation/conflict UX。
6. Unity Volume 参数与 reset stack 证明局部 SerializedObject/Undo 语义，不能被解释为通用 Prefab authority。

## 3. 差距清单

### 3.1 P0：实施前必须阻断

1. **P0-01** 含 `prefab_instance` 的 Scene 在无损 roundtrip 或 fail-closed 保护前不得允许普通 Save。
2. **P0-02** Prefab plugin 无 backend/factory/executor 时不得暴露可执行 Apply/Revert/Break/Validate 入口。
3. **P0-03** 固定 Prefab Workbench 不得以真实 asset/default/override 状态伪造 queued 或成功反馈。
4. **P0-04** 在 stable identity、typed codec、source revision、preflight、atomic rollback、receipt 前不得写 source/live World。
5. **P0-05** 不得把 ECS archetype、reflection default、render layer 或局部 Material override 宣称为 class default/instance propagation 产品。

### 3.2 P1：70 项重构主线

1. **P1-01** 定义 Native/Script/Class/Prefab/Variant/Instance/Transient default layer identity。
2. **P1-02** 定义 source/project/scene/instance/object/component/property stable IDs。
3. **P1-03** 定义 source revision、schema fingerprint、plugin catalog digest。
4. **P1-04** 定义 typed property address、collection selector、field migration。
5. **P1-05** 定义 value codec、unit/constraint、unknown-field/opaque policy。
6. **P1-06** 定义 default origin、effective value、local override、modified state API。
7. **P1-07** 定义 parent/source/instance provenance map。
8. **P1-08** 定义 owner/generation/request/receipt 传播。
9. **P1-09** 将 display path 与 authority identity 分离。
10. **P1-10** 为 legacy string/JSON override 建立只读迁移边界。
11. **P1-11** 实现 `DefaultValueAuthority` provider registry。
12. **P1-12** 实现 native/reflection default provider 与 typed validation。
13. **P1-13** 接入 Script/Class compiled default artifact（Editor31 owner）。
14. **P1-14** 实现 Prefab/Archetype source default provider。
15. **P1-15** 实现 Variant layer provider（Editor41 owner）。
16. **P1-16** 实现 Instance/Session transient provider 与 priority policy。
17. **P1-17** 建立 effective resolution cache 与 dependency invalidation。
18. **P1-18** 建立 missing/type-incompatible/cycle diagnostics。
19. **P1-19** 为 provider 定义 thread/read consistency contract。
20. **P1-20** 为 layer change 输出 generation-qualified snapshot。
21. **P1-21** 建立 versioned Prefab source asset 与 migration。
22. **P1-22** 建立 source object/component/property stable records。
23. **P1-23** 建立 typed topology operations（child/component add/remove/reparent/reorder）。
24. **P1-24** 建立 source/instance provenance 与 reverse dependency index。
25. **P1-25** 实现 nested ancestry、loop detection、orphan classification。
26. **P1-26** 将 string path override 迁移为 typed address + source object ID。
27. **P1-27** 记录 base/source/instance value hash 与 expected-before。
28. **P1-28** 实现 source reload、three-way rebase、conflict/orphan/type mismatch。
29. **P1-29** 实现 load/register/wait/fail/stale/cancel/retry 生命周期。
30. **P1-30** 让 Scene/cache/autosave/archive/cook IO 无损保存 Prefab instance。
31. **P1-31** 实现 property effective/origin/mixed/modified Inspector snapshot。
32. **P1-32** 实现 immediate-parent 与 explicit-layer Reset policy。
33. **P1-33** reset 只删除目标 layer override，不影响 sibling property。
34. **P1-34** 实现 Apply-to-Source 的 revision/CAS preflight。
35. **P1-35** 实现 Revert-to-Parent 的 live World/Inspector/dirty 更新。
36. **P1-36** 实现 Create/Apply/Revert/Reset/Break typed commands。
37. **P1-37** 将 commands 接入 Editor02 transaction/history/savepoint。
38. **P1-38** 实现 atomic rollback、selection restore、receipt。
39. **P1-39** 将 prefab source save 接入 atomic multi-document transaction。
40. **P1-40** source change 只访问 reverse dependency index 命中的 instances。
41. **P1-41** clean instance 自动传播 source change 并更新 generation。
42. **P1-42** overridden instance 保留 effective local value 与 provenance。
43. **P1-43** simultaneous source/instance changes 生成 stable conflict artifact。
44. **P1-44** loaded/unloaded/partitioned instance 使用相同 rebase policy。
45. **P1-45** added/removed child/component/reference 有明确 propagation policy。
46. **P1-46** break 物化完整 subtree、component、reference、ownership。
47. **P1-47** break 后不再依赖 source asset，且可 undo/reopen。
48. **P1-48** plugin codec/provider 无法解析时 opaque 保留并阻止编辑。
49. **P1-49** plugin factory/executor/catalog/capability 完整后才可启用菜单。
50. **P1-50** Prefab Workbench 改为 provider-bound document/toolkit。
51. **P1-51** Outliner/Inspector 显示 origin、modified、orphan、conflict、source revision。
52. **P1-52** multi-selection 逐 target resolve default，混合值不可误写。
53. **P1-53** Editor jobs 接入 load/rebase/propagation/cook/cancel/shutdown。
54. **P1-54** 统一 stable diagnostic code、affected property、related asset、fix action。
55. **P1-55** cook 产出 resolved runtime default/override artifact 与 provenance。
56. **P1-56** runtime frame 不解析 JSON/path、不遍历 authoring inheritance chain。
57. **P1-57** runtime install 校验 artifact schema/plugin/generation。
58. **P1-58** hot reload/source replacement 保持 effective value 与 instance identity。
59. **P1-59** default cache 具有 bounded memory、invalidation 和 stale rejection。
60. **P1-60** 记录 resolution、propagation、rebase、save、cook 性能 telemetry。
61. **P1-61** 增加 default precedence golden matrix。
62. **P1-62** 增加 Prefab source/instance roundtrip 与 migration golden。
63. **P1-63** 增加 stable rename/reparent/property migration tests。
64. **P1-64** 增加 reset/apply/revert/break transaction undo/redo tests。
65. **P1-65** 增加 source propagation/rebase/conflict/orphan matrix。
66. **P1-66** 增加 nested topology/reference remap/loop tests。
67. **P1-67** 增加 plugin unknown codec/factory failure/unload tests。
68. **P1-68** 增加 fault-injected save/rollback/crash recovery tests。
69. **P1-69** 增加 100k instances/source storm/cancel/memory performance tests。
70. **P1-70** 删除固定 Workbench、DTO helper authority 和旧字符串 key，完成端到端资格门。

### 3.3 P2：主线完成后扩展

1. **P2-01** class default visualizer、archetype graph 与 source navigation。
2. **P2-02** parameterized prefab、variant composition 与 inheritance templates。
3. **P2-03** per-type merge policy/plugin marketplace。
4. **P2-04** multi-user default/override edit lease 与 review。
5. **P2-05** remote/unloaded instance live preview 与 lazy propagation。
6. **P2-06** HLOD/partition-aware resolved default artifact。
7. **P2-07** script hot-reload schema diff 与 automated migration。
8. **P2-08** batch reset/apply、commandlet 与 headless validation。
9. **P2-09** provenance/diff browser 跨 Prefab/Variant/LevelInstance/Snapshot。
10. **P2-10** content-addressed default cache、dedup 与 remote build。
11. **P2-11** collaborative conflict auto-resolution、policy simulation 与 audit。
12. **P2-12** 以同数据完整度、同事务和同 runtime artifact 条件建立超过参考引擎的 propagation benchmark。

## 4. 目标架构与里程碑

```text
Default Providers -> EffectiveValueSnapshot -> Inspector/Runtime projection
PrefabSource -> Typed Source Artifact -> Instance Resolver -> Propagation Plan
Propagation Plan -> revision/CAS preflight -> one Editor transaction -> Receipt
```

Runtime Reflection 提供 typed schema/codec；Prefab/LevelInstance owner 提供 source/provenance/load；Editor Default domain 解析层级和 reset；Editor Transaction 提供 atomic apply；Editor42 提供 semantic conflict artifact；Asset/Jobs/Diagnostics 提供 persistence、cancel、cook、receipt。ECS archetype 只留在 storage owner。

| Milestone | 退出条件 |
|---|---|
| M0 | Scene roundtrip fail-closed；静态 Prefab UI/helper 不再可执行或伪造成功。 |
| M1 | stable identity、typed address、schema/migration、layer precedence ADR 冻结。 |
| M2 | DefaultValueAuthority、effective snapshot、origin/modified/reset policy 完成。 |
| M3 | versioned Prefab source、typed topology/property、provenance、loop validation 完成。 |
| M4 | Scene/cache/autosave/cook 无损 roundtrip 与 migration 完成。 |
| M5 | create/apply/revert/reset/break transaction、CAS、rollback、receipt 完成。 |
| M6 | reverse index、source propagation、rebase/conflict/orphan、loaded/unloaded policy 完成。 |
| M7 | Inspector/Prefab toolkit、Outliner columns、multi-selection、accessibility 完成。 |
| M8 | resolved runtime artifact、generation install、hot reload、debug provenance 完成。 |
| M9 | plugin provider/codec/factory/capability 与 compatibility suite 完成。 |
| M10 | 100k instance、source storm、partition、fault、cross-platform deterministic/performance 完成。 |
| M11 | legacy JSON/path/helper/fixture 硬切，32 门及文档/manifest/CI 闭合。 |

## 5. 验收门

1. **G01-G06** non-None Prefab Scene roundtrip 无损或 fail closed；插件无 backend 时入口 unavailable；fixture 不伪造成功；identity/codec/schema/migration 通过。
2. **G07-G12** 六层 default precedence、origin/modified/mixed/reset、source revision/CAS、transaction rollback 通过 golden。
3. **G13-G18** source/instance stable rename/reparent、typed topology、provenance、nested loop、orphan/conflict/rebase 通过。
4. **G19-G24** apply/revert/reset/break 对 live World、dirty、history、selection、reference remap、crash recovery 一致。
5. **G25-G30** plugin opaque codec、cook artifact fingerprint、runtime generation/hot reload、loaded/unloaded、partition policy 通过。
6. **G31-G32** 100k/source storm/取消/内存/跨平台 benchmark，以及 UI/docs/manifest/telemetry 与真实 provider 状态一致。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码与物理范围 fingerprint 复核；没有修改 Runtime、Editor、Interface、Plugin 或 tests，也没有运行 Cargo、Prefab roundtrip、default resolution、propagation、fault 或性能动态验证。frontmatter 路径需在实施前重新展开；P0/P1/P2=5/70/12、M0-M11、32 门和三处索引唯一链接是文档收尾门。Editor41 负责 Scene/Level Instance 无损存储，Editor05 负责 generic Inspector，Editor31 负责 Script/Class schema，Editor42 负责 semantic snapshot/merge；本报告不建立竞争 owner，整体 review 仍保持进行中。
