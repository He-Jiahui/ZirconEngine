---
title: Editor Archetype、Class Defaults、Instance Override、Property Propagation 与 Reset-to-Default 当前源码复审
category: zircon_editor
report_id: Editor221
review_date: 2026-08-29
baseline_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
verification_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
canonical_owner: Editor44
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/118-editor-archetype-class-defaults-instance-override-property-propagation-reset-current-source-review.md
  - docs/plans/optimize/zircon_editor/165-editor-archetype-class-defaults-instance-override-property-propagation-reset-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/208-editor-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-current-source-review.md
  - docs/plans/optimize/zircon_editor/218-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
  - docs/plans/optimize/zircon_editor/219-editor-scene-snapshot-world-diff-merge-restore-conflict-resolution-current-source-review.md
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/artifact/cache_payload
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime_interface/src/reflect
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/extension
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_plugins/prefab_tools
  - zircon_plugins/editor_support/src/lib.rs
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor221 · Archetype / Class Defaults / Instance Override / Property Propagation 当前源码复审

## 1. 结论

Zircon 当前仍没有工程级 class default、Prefab source、instance override、property propagation 或 Reset-to-Default 产品。对 `zircon_editor`、`zircon_runtime`、`zircon_runtime_interface` 与 `zircon_plugins` 的 **17,235 个 tracked + 2,384 个 untracked = 19,619 个** Rust/TOML/ZUI/Zr 物理文件复核后，`DefaultValueAuthority`、`EffectivePropertyValue`、`DefaultSourceIdentity`、`PrefabSourceDocument`、`PrefabPropagation`、`PrefabInstanceRecord`、`ApplyToSource` 与 `RevertToParent` 均为 **0 命中**；`ResetToDefault` 的 16 个 tracked 命中仍全部属于 dock/workspace layout reset，untracked 为 0。

Editor165 记录的三项安全进展仍成立。Scene formal codec 与 World project IO 会将 `PrefabInstanceAsset` 保存在 `zircon.prefab.instance` 中，保存端对损坏 retained JSON fail closed；`prefab_tools` 没有注册 Create/Open/Apply/Revert/Break 写操作、菜单、factory 或 executor；当前也没有绕过 source revision 与 transaction 的 Prefab production 写 helper。因此 P0-01、P0-02、P0-04 保持 Closed。

近期新增的通用 `DocumentSourceWriteAuthority` 提供 normalized project path 写租约、expected-bytes CAS、atomic write 与 publication outcome，这是真实的 document save 底座，但它是单文件字节发布 authority：没有 Prefab source revision、stable property identity、多文档 transaction、instance rebase、propagation 或 domain receipt，不能据此关闭任何 Prefab mutation finding。

产品差距依旧明显。`PrefabAsset` 仍只有内嵌 Scene 与字符串 exposed properties；override 仍是 `entity_path + property_path + serde_json::Value`。主 Editor 的 233 行 Prefab Workspace 固定显示 `PF_Chest`、`Chest_04`、18 children、6 overrides、2 warnings，并把 Apply/Validate 映射为固定 queued 文本；相反，Prefab plugin 明确只开放只读 surface，并用 `DiagnosticOnlyAssetImporter` 报告 backend 未安装。主 UI 与插件的产品承诺互相矛盾，P0-03、P0-05 保持 Open。

Editor44/118/165 的 canonical finding 数与 ID 不变。本轮状态仍为：**P0：2 Open / 3 Closed；P1：59 Open / 11 Partial；P2：12 Open；Gates：28 Fail / 2 Partial / 2 Pass**。没有 source resolver、transactional reset/apply/revert/break、propagation/rebase、resolved cook artifact 或 100k-instance benchmark，不能宣称达到或优于 Unreal、Godot、Fyrox、Bevy 或 Unity Graphics，更没有证据支持性能优于 Unreal。

## 2. 审查范围、统计与 currentness

统计读取共享 working tree 的当前物理内容并包含未跟踪文件。行数按物理文本记录计；tests 只统计 Rust `#[test]` / `#[tokio::test]`。fingerprint 对 repository-relative path 与每文件 SHA-256 的有序清单再次计算 SHA-256。选择集用于证明本轮读取面，不等价于全仓性能资格。

| 范围 | 文件 / 行 / 非空行 / bytes / tests | fingerprint |
|---|---:|---|
| Editor editing / extension / Inspector / Material / Prefab surfaces | **118 / 21,015 / 19,230 / 737,023 / 108** | `8653761b4ea5e6b74bd0e900d73471a7cefb6ca4a60decb26c032faefd223253` |
| Runtime asset / persistence / importer | **101 / 23,430 / 21,511 / 834,601 / 176** | `a088c981326723894d540f63ab74f2c6d21c9e4210fd97d4e31b42e477925a35` |
| Runtime Scene / reflection / ECS archetype / interface | **66 / 8,730 / 7,975 / 300,196 / 31** | `ef3b1b3093688f7a173927af17ec4b05baca803332cf66ed481a047aaa7215f4` |
| Prefab plugin product boundary | **17 / 1,244 / 1,137 / 45,973 / 10** | `56fbecab5722d588657f321ccf980ecabe5149b66c1106086450d0c67786f740` |
| Zircon selected union | **302 / 54,419 / 49,853 / 1,917,793 / 325** | `c439430b0c4b508fb6159b3ef3eb2664b488a3ed1d0aa3186807ce34d28a1452` |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **43 / 29,307 / 25,491 / 1,079,792 / 41** | `2af9073db5f00945682e36751d042d753de3b832fd5f957296a0b2a3126b1239` |
| All selected | **345 / 83,726 / 75,344 / 2,997,585 / 366** | Zircon 与 reference fingerprint 分列冻结 |

- baseline/verification HEAD 为 `f660cfa9f3f84bff0903e4564ff1af4d065aee73`，commit 时间为 2026-08-29T02:25:25+08:00。选择集内存在大量 shared dirty 与 untracked 文件；本报告以当前物理内容为准，没有覆盖或回退这些修改。
- Editor 集合覆盖 editing/transaction、extension/toolkit/save、Inspector snapshot、Material projection、Prefab ZUI 与 fixed feedback。Runtime 集合覆盖 Prefab/Scene DTO、formal/cache/import/load、World project IO、reflection 与 ECS storage archetype。
- reference 集合是 frontmatter 的精确 43 文件。Unreal 承担 object/class default 与 LevelInstance 主基线；Godot、Fyrox、Bevy、Unity Graphics 分别补充 property revert、inheritance、resolved scene layering 与局部 override/Undo。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；Tooling 暂不纳入。本轮只写 review、索引与覆盖记录。

## 3. 当前源码事实与语义边界

### 3.1 Scene roundtrip 已无损保留 metadata，但没有 instance resolution

1. `World::from_scene_asset` 将 `PrefabInstanceAsset` 写入 reserved dynamic component `zircon.prefab.instance`，`World::to_scene_asset` 通过 `prefab_instance_for_record` 恢复。
2. formal Scene codec 与 World tests 覆盖 prefab reference、local transform 和 raw overrides；runtime extension 安装后仍保持 Scene 精确相等。
3. retained component 存在但 JSON 无法反序列化时，`prefab_instance_for_record` 返回 `SceneProjectError::SceneAsset`，保存失败而不是静默丢弃 metadata。
4. 这条链没有加载 source Prefab、生成 source subtree、解析 effective property、执行 topology override 或传播 source revision。

### 3.2 Prefab schema 仍是不可演进的字符串路径 + JSON

1. `PrefabAsset` 仅包含 URI、name、内嵌 `SceneAsset` 和 `Vec<String> exposed_properties`，没有 format/schema version、source revision、dependency digest、compiled artifact identity。
2. `PrefabInstanceAsset` 仅包含 `AssetReference`、local transform 与 override vector；`PrefabPropertyOverrideAsset` 仍使用两个 String path 和 `serde_json::Value`。
3. rename/reparent、component replacement、field rename、collection reorder、plugin schema migration 都没有 stable remap key；value 没有 declared type、unit、constraint、base hash 或 expected-before。
4. schema 没有 child/component add/remove/reparent/reorder、reference remap、nested ancestry、loop、orphan、conflict 或 break materialization record。

### 3.3 Import/load 是 DTO 管道，不是 Prefab runtime

1. builtin `zircon.builtin.toml.prefab` 能将 `.prefab.toml` 解析为 `PrefabAsset`，generic `AssetKind::Prefab` dispatch 也能 load DTO。
2. production 中没有 domain-specific instantiate、source graph resolver、effective override application、reverse dependency index 或 propagation consumer。
3. plugin 又以相同 priority 0 注册同一 `.prefab.toml` matcher，但 handler 是 `DiagnosticOnlyAssetImporter`；registry 会拒绝同 priority duplicate matcher，产品装配没有单一 owner 决策。
4. Scene preservation identity 为 `zircon.prefab.instance`，plugin component identity 为 `prefab_tools.Component.PrefabInstance`，两者无 adapter、migration 或 ownership contract。

### 3.4 Reflection 是 typed schema 底座，不是 default authority

1. `ReflectFieldInfo` 有 typed `ReflectedValue default_value`、numeric range、enum options 与 type validation，可作为 native-default provider 的输入。
2. `ReflectObjectAddress` 只区分 `Component { entity: u64, type_path: String }` 与 Resource；field name 仍是 String，不是跨 rename/reopen/cook 的 authoring stable property identity。
3. generic Inspector property snapshot 仍只有 field id、name、label、string value、value kind、editable 与 field editor，没有 effective/default/origin/local override/mixed/modified/reset target/source revision。
4. Material Editor 的 `default_value`、`override_value`、`is_overridden` 是局部 domain projection，不能冒充全引擎 precedence、provenance、reset 或 propagation authority。

### 3.5 通用 transaction/save 底座没有接入 Prefab domain

1. `DocumentSourceWriteAuthority` 可按 resolved project path 串行化 writer，并在 current bytes 与 expected bytes 相等时发布 replacement。
2. 它能区分 durable-best-effort、published-not-durable、not-published 与 source-changed；还会拒绝项目根外路径和只读 source。
3. 该 authority 没有 Prefab source revision/schema/plugin digest，也不知道 instance、override、dependent document、selection/history 或 domain receipt。
4. 当前 Prefab 没有 typed command/factory/executor，因此通用 transaction、journal、save token 与 source-write CAS 都没有形成 create/apply/revert/reset/break 闭环。

### 3.6 Prefab plugin 封口写操作，但 contract 仍漂移

1. Editor plugin 只注册 view/drawer/Inspector customization，测试明确断言五个 writable operation 与对应菜单不存在。
2. `plugins://prefab_tools/editor/authoring.zui` 与 `plugins://prefab_tools/editor/prefab_instance.zui` 没有对应物理资源。
3. runtime capability 标为 Partial，importer 固定诊断 `prefab importer backend is not installed`；README 也明确 Create/Open/Apply/Revert/Break 尚未安装。
4. component descriptor 将 `prefab` 标为 serializable，却将 `overrides: json` 标为 non-serializable，与 formal Scene DTO 的可持久化 override 语义冲突。

### 3.7 主 Editor Prefab Workspace 继续伪造产品状态

1. 当前 ZUI 仍为 233 行、19 个 routes，固定 `PF_Chest`、`Chest_04`、18 children、6 overrides、2 warnings 和四个 prefab option。
2. Apply/Validate route 经模板桥变成 `.invoke`，callback 直接输出 `Prefab override apply queued`、`Prefab validation queued` 和固定计数。
3. route 链没有 provider、active document、asset revision、job、transaction、dirty/save 或 runtime receipt；queued 只证明 callback 被命中。
4. plugin 已诚实禁用写操作，而主 Editor 仍展示 Apply/Validate，两个 product surface 对同一能力给出相反承诺。

### 3.8 ECS archetype 必须继续只拥有 storage layout

1. `scene/ecs/archetype` 管理 component signature、table、column、row、locator 与 change tick，目标是存储局部性和 query 执行。
2. 它没有 object archetype、class default object、source parent、override bit、default origin 或 reset policy。
3. 将 ECS archetype 扩展为 authoring inheritance authority 会混合 runtime storage generation 与长期 source identity，必须明确禁止。

### 3.9 当前测试只证明保留、封口与局部 CAS

1. Scene tests 证明 metadata roundtrip 与 extension 安装后保留；plugin tests 证明 writable operation/menu 不存在、字符串 override 去重与基础诊断。
2. source-write tests 证明单路径 lease、byte CAS、只读/根外拒绝和 publication outcome，但没有 Prefab domain command。
3. 没有 source instantiate、clean/overridden propagation、rename/reparent migration、nested topology、apply/revert/reset/break transaction、crash recovery 或 cook/runtime install test。
4. 没有 100k instances/source storm、bounded cache、cancel、memory、cross-platform deterministic 或可比性能数据。

## 4. 参考引擎差异

| 能力 | Zircon 当前 | 参考源码 | 必须收敛的边界 |
|---|---|---|---|
| Object/class default | reflection field 可选 default，无来源/层级 | Unreal 区分 CDO、object archetype、subobject template 与 authoritative class | 独立 default provider/layer identity，禁止借用 ECS archetype |
| Component inheritance | 无 source component record/override ownership | Unreal InheritableComponentHandler 与 ComponentInstanceDataCache 保存 template/instance state | stable source object/component/property identity 与 instance data cache |
| Reset policy | generic Inspector 无 reset/origin | Unreal PropertyEditorArchetypePolicy/SResetToDefault；Godot property revert；Fyrox typed inherit/revert | immediate-parent/explicit-layer policy 与 transaction command |
| Scene/Prefab source | metadata 无损但不 instantiate | Godot PackedScene/SceneState 保存 owner/instance/inheritance；Bevy ScenePatch 先 resolve 后 apply | versioned source、dependency resolution、resolved instance artifact |
| Override evidence | string path + JSON，仅 last-wins | Unreal LevelInstance GUID mapping/diff/policy；Fyrox modified/parent availability | typed address、base/source/instance hash、provenance、conflict/orphan |
| Local parameter override | Material 局部 projection | Unity Graphics VolumeParameter `overrideState` + SerializedObject/Undo | 可复用局部 adapter 经验，但不能替代通用 Prefab authority |
| Runtime execution | 无 resolved Prefab artifact | Bevy ResolvedScene 解析 dependency/cached patch 后按层 apply | cook-time resolve、schema/generation install、frame hot path 零 JSON/path |

## 5. 差距清单

### 5.1 P0：2 Open / 3 Closed

1. **P0-01 · Closed** Scene/World 已无损保留 `prefab_instance`，损坏 retained payload 在 save projection 时 fail closed。
2. **P0-02 · Closed** `prefab_tools` 在无 backend/factory/executor 时不注册 Create/Open/Apply/Revert/Break operation/menu。
3. **P0-03 · Open** 主 Editor 固定 Prefab Workspace 仍以真实资产外观伪造 Apply/Validate queued 状态。
4. **P0-04 · Closed** 旧 DTO 写 helper 已删除，当前没有绕过 stable identity/revision/preflight/rollback 的 Prefab source/live World production 写路径。
5. **P0-05 · Open** UI、view 与 plugin 描述仍把 DTO、metadata preservation 和固定 projection 表达为 Prefab authoring/instancing 产品。

### 5.2 P1：59 Open / 11 Partial

1. **P1-01 · Open** 定义 Native/Script/Class/Prefab/Variant/Instance/Transient default layer identity。
2. **P1-02 · Partial** 已有 AssetReference UUID/locator 与 Scene entity ID；缺 source/instance/object/component/property stable identity。
3. **P1-03 · Open** 定义 source revision、schema fingerprint、plugin catalog digest。
4. **P1-04 · Partial** 已有 Component/Resource address 与 reflection adapter；Prefab 仍用 raw entity/string path，缺 collection selector/migration。
5. **P1-05 · Partial** reflection 有 typed value/range/enum validation；Prefab value 仍是无 declared type 的 JSON。
6. **P1-06 · Open** 定义 default origin、effective value、local override、modified state API。
7. **P1-07 · Open** 定义 parent/source/instance provenance map。
8. **P1-08 · Open** 定义 owner/generation/request/receipt 传播。
9. **P1-09 · Open** 将 display path 与 authority identity 分离。
10. **P1-10 · Open** 为 legacy string/JSON override 建立只读迁移边界。
11. **P1-11 · Open** 实现 `DefaultValueAuthority` provider registry。
12. **P1-12 · Partial** reflection default 可作为 native provider 输入；尚无 provider、origin、generation 或 invalidation。
13. **P1-13 · Open** 接入 Script/Class compiled default artifact（Editor208 owner）。
14. **P1-14 · Open** 实现 Prefab/Archetype source default provider。
15. **P1-15 · Open** 实现 Variant layer provider（Editor218 owner）。
16. **P1-16 · Open** 实现 Instance/Session transient provider 与 priority policy。
17. **P1-17 · Open** 建立 effective resolution cache 与 dependency invalidation。
18. **P1-18 · Partial** plugin 可报告 missing source、空 path、重复 path；缺 missing field/type/cycle/orphan/conflict artifact。
19. **P1-19 · Open** 为 provider 定义 thread/read consistency contract。
20. **P1-20 · Open** 为 layer change 输出 generation-qualified snapshot。
21. **P1-21 · Open** 建立 versioned Prefab source asset 与 migration。
22. **P1-22 · Open** 建立 source object/component/property stable records。
23. **P1-23 · Open** 建立 typed topology operations。
24. **P1-24 · Open** 建立 source/instance provenance 与 reverse dependency index。
25. **P1-25 · Open** 实现 nested ancestry、loop detection、orphan classification。
26. **P1-26 · Open** 将 string path override 迁移为 typed address + source object ID。
27. **P1-27 · Open** 记录 base/source/instance value hash 与 expected-before。
28. **P1-28 · Open** 实现 source reload、three-way rebase、conflict/orphan/type mismatch。
29. **P1-29 · Partial** generic asset manager 可 import/load `PrefabAsset`；无 instance activation/wait/fail/stale/cancel/retry lifecycle。
30. **P1-30 · Partial** formal Scene、World 与 cache carrier 无损保留 metadata；autosave/archive/cook/migration/source validity 未闭合。
31. **P1-31 · Open** 实现 property effective/origin/mixed/modified Inspector snapshot。
32. **P1-32 · Open** 实现 immediate-parent 与 explicit-layer Reset policy。
33. **P1-33 · Open** reset 只删除目标 layer override，不影响 sibling property。
34. **P1-34 · Open** 实现 Apply-to-Source 的 Prefab revision/CAS preflight；通用 byte CAS 尚未接 domain revision。
35. **P1-35 · Open** 实现 Revert-to-Parent 的 live World/Inspector/dirty 更新。
36. **P1-36 · Open** 实现 Create/Apply/Revert/Reset/Break typed commands。
37. **P1-37 · Open** 将 commands 接入 Editor02 transaction/history/savepoint。
38. **P1-38 · Open** 实现 atomic rollback、selection restore、typed receipt。
39. **P1-39 · Open** 将 Prefab source save 接入 atomic multi-document transaction；当前 source authority 仅串行化单路径。
40. **P1-40 · Open** source change 只访问 reverse dependency index 命中的 instances。
41. **P1-41 · Open** clean instance 自动传播 source change 并更新 generation。
42. **P1-42 · Open** overridden instance 保留 effective local value 与 provenance。
43. **P1-43 · Open** simultaneous source/instance changes 生成 stable conflict artifact。
44. **P1-44 · Open** loaded/unloaded/partitioned instance 使用相同 rebase policy。
45. **P1-45 · Open** added/removed child/component/reference 有明确 propagation policy。
46. **P1-46 · Open** break 物化完整 subtree、component、reference、ownership。
47. **P1-47 · Open** break 后不再依赖 source asset，且可 undo/reopen。
48. **P1-48 · Open** plugin codec/provider 无法解析时 opaque 保留并阻止编辑。
49. **P1-49 · Partial** writable plugin operation/menu 已禁用；仍有缺失 ZUI、DiagnosticOnly importer、duplicate matcher、双 identity 与描述漂移。
50. **P1-50 · Open** Prefab Workbench 改为 provider-bound document/toolkit。
51. **P1-51 · Open** Outliner/Inspector 显示 origin、modified、orphan、conflict、source revision。
52. **P1-52 · Open** multi-selection 逐 target resolve default，mixed value 不可误写。
53. **P1-53 · Open** jobs 接入 load/rebase/propagation/cook/cancel/shutdown。
54. **P1-54 · Open** 统一 stable diagnostic code、affected property、related asset、fix action。
55. **P1-55 · Open** cook 产出 resolved runtime default/override artifact 与 provenance。
56. **P1-56 · Open** runtime frame 不解析 JSON/path、不遍历 authoring inheritance chain。
57. **P1-57 · Open** runtime install 校验 artifact schema/plugin/generation。
58. **P1-58 · Open** hot reload/source replacement 保持 effective value 与 instance identity。
59. **P1-59 · Open** default cache 具有 bounded memory、invalidation 和 stale rejection。
60. **P1-60 · Open** 记录 resolution、propagation、rebase、save、cook telemetry。
61. **P1-61 · Open** 增加 default precedence golden matrix。
62. **P1-62 · Partial** 已有 formal/World/runtime-extension metadata roundtrip；缺 Prefab source/instance migration golden。
63. **P1-63 · Open** 增加 stable rename/reparent/property migration tests。
64. **P1-64 · Open** 增加 reset/apply/revert/break transaction undo/redo tests。
65. **P1-65 · Open** 增加 source propagation/rebase/conflict/orphan matrix。
66. **P1-66 · Open** 增加 nested topology/reference remap/loop tests。
67. **P1-67 · Partial** 已测试 writable factory/action 不存在及 DiagnosticOnly package contract；缺 unknown codec/factory failure/unload/opaque preservation。
68. **P1-68 · Open** 增加 fault-injected save/rollback/crash recovery tests。
69. **P1-69 · Open** 增加 100k instances/source storm/cancel/memory performance tests。
70. **P1-70 · Partial** 旧危险 DTO 写 helper 已删除；固定 Workbench、字符串 key、缺失资源和双 identity 尚未硬切。

### 5.3 P2：12 Open

1. **P2-01 · Open** class default visualizer、archetype graph 与 source navigation。
2. **P2-02 · Open** parameterized prefab、variant composition 与 inheritance templates。
3. **P2-03 · Open** per-type merge policy/plugin marketplace。
4. **P2-04 · Open** multi-user default/override edit lease 与 review。
5. **P2-05 · Open** remote/unloaded instance live preview 与 lazy propagation。
6. **P2-06 · Open** HLOD/partition-aware resolved default artifact。
7. **P2-07 · Open** script hot-reload schema diff 与 automated migration。
8. **P2-08 · Open** batch reset/apply、commandlet 与 headless validation。
9. **P2-09 · Open** provenance/diff browser 跨 Prefab/Variant/LevelInstance/Snapshot。
10. **P2-10 · Open** content-addressed default cache、dedup 与 remote build。
11. **P2-11 · Open** collaborative conflict auto-resolution、policy simulation 与 audit。
12. **P2-12 · Open** 以同数据完整度、同事务和同 runtime artifact 建立超过参考引擎的 benchmark。

## 6. 目标架构与 ownership

~~~mermaid
flowchart LR
    Native["Native reflection defaults"] --> Authority["DefaultValueAuthority"]
    Script["Script/Class compiled defaults"] --> Authority
    Source["Versioned Prefab source"] --> Authority
    Variant["Variant layer"] --> Authority
    Instance["Instance/Session overrides"] --> Authority
    Authority --> Snapshot["Effective value + origin + generation"]
    Snapshot --> Inspector["Inspector / Outliner"]
    Source --> Resolver["Instance resolver + provenance"]
    Resolver --> Plan["Propagation / rebase plan"]
    Plan --> Preflight["Revision CAS + typed validation"]
    Preflight --> Tx["One Editor transaction"]
    Tx --> Receipt["Apply / reset / break receipt"]
    Resolver --> Cook["Resolved runtime artifact"]
    Cook --> Runtime["Generation-qualified install"]
~~~

Runtime Reflection 只拥有 typed schema、value codec 与 native default input；Asset/Prefab domain 拥有 versioned source、stable source records、dependency index 与 resolver；Editor Default domain 拥有 precedence、origin、reset policy 与 command construction；Editor transaction engine 是唯一 authoring mutation authority；Editor219 提供 semantic diff/conflict artifact；cook 输出已解析 artifact，runtime 不读取 authoring JSON/path。ECS archetype 继续只拥有 component storage layout。

## 7. 依赖顺序与里程碑

| Milestone | 当前 | 退出条件 |
|---|---|---|
| M0 | Partial | Scene fail closed、插件写入口封口；主 Editor 固定 Prefab UI/queued feedback 仍须删除或 capability-disable。 |
| M1 | Not met | stable identity、typed address、schema/migration、layer precedence ADR 冻结。 |
| M2 | Not met | DefaultValueAuthority、effective snapshot、origin/modified/reset policy 完成。 |
| M3 | Not met | versioned Prefab source、typed topology/property、provenance、loop validation 完成。 |
| M4 | Partial | formal/World metadata roundtrip 已完成；autosave/archive/cook/migration 未完成。 |
| M5 | Not met | create/apply/revert/reset/break transaction、CAS、rollback、receipt 完成。 |
| M6 | Not met | reverse index、propagation、rebase/conflict/orphan、loaded/unloaded policy 完成。 |
| M7 | Not met | Inspector/Prefab toolkit、Outliner columns、multi-selection、accessibility 完成。 |
| M8 | Not met | resolved runtime artifact、generation install、hot reload、debug provenance 完成。 |
| M9 | Partial | writable plugin admission 已禁用；provider/codec/factory/resources/identity compatibility 未闭合。 |
| M10 | Not met | 100k instance、source storm、partition、fault、cross-platform qualification 完成。 |
| M11 | Not met | legacy JSON/path/fixed fixture 硬切，32 门及文档/manifest/CI 闭合。 |

## 8. 32 个验收门

| Gate | 状态 | 当前证据 / 缺口 |
|---|---|---|
| G01 non-None Prefab formal/World roundtrip | Pass | prefab reference、transform、overrides 精确相等。 |
| G02 extension preservation + malformed fail closed | Pass | runtime extension 后保留；损坏 retained JSON 返回保存错误。 |
| G03 plugin backend admission truth | Partial | writable operation/menu 不存在；缺失 ZUI、DiagnosticOnly importer、identity 漂移仍在。 |
| G04 fixed UI truth | Fail | Apply/Validate 继续输出固定 queued 文本。 |
| G05 stable IDs | Fail | 无 source object/component/property identity。 |
| G06 typed codec/schema/migration | Fail | String path + JSON，无 version/migration。 |
| G07 layer precedence golden | Fail | 无统一 layer model/provider。 |
| G08 origin/modified/mixed snapshot | Fail | generic Inspector 无相关字段。 |
| G09 immediate-parent reset | Fail | 无 reset command/policy。 |
| G10 explicit-layer reset | Fail | 无 layer target。 |
| G11 source revision/CAS | Fail | generic byte CAS 不是 Prefab revision/CAS。 |
| G12 transaction rollback/receipt | Fail | 通用底座未接 Prefab domain。 |
| G13 stable rename/reparent | Fail | string path 会漂移。 |
| G14 typed topology operation | Fail | 无 add/remove/reparent/reorder override。 |
| G15 provenance/nested loop | Fail | 无 ancestry/provenance/loop owner。 |
| G16 orphan/conflict classification | Fail | 只有 missing/empty/duplicate string diagnostic。 |
| G17 three-way rebase | Fail | 无 base/source/instance evidence。 |
| G18 deterministic change artifact | Fail | 无 propagation plan/receipt。 |
| G19 apply-to-source live behavior | Fail | 无 production operation。 |
| G20 revert/reset live behavior | Fail | 无 production operation。 |
| G21 break/reference remap | Fail | 无 subtree materialization/remap。 |
| G22 undo/dirty/selection | Fail | 无 Prefab transaction integration。 |
| G23 crash recovery | Fail | 无 domain journal/recovery workflow。 |
| G24 persistence across all paths | Partial | formal/World/cache carrier 已保留；autosave/archive/cook/migration 未证实。 |
| G25 opaque plugin codec | Fail | unknown schema 不能 opaque retain/edit-block。 |
| G26 cook artifact fingerprint | Fail | 无 resolved Prefab artifact。 |
| G27 runtime generation/hot reload | Fail | 无 install receipt/generation fence。 |
| G28 unloaded/partitioned parity | Fail | 无 reverse index 或 unloaded policy。 |
| G29 runtime hot path | Fail | 无法证明零 JSON/path 与 bounded work。 |
| G30 telemetry/diagnostics | Fail | 无 resolution/propagation/rebase/cook telemetry。 |
| G31 scale/cross-platform qualification | Fail | 无 100k/source storm/fault/cross-platform evidence。 |
| G32 product E2E/UI/docs/benchmark | Fail | 固定 UI 与 manifest 仍过度表达，E2E 和可比 benchmark 为零。 |

## 9. 重构顺序

1. 完成 M0 产品诚实性：删除、disabled 或真实 capability-bind 主 Editor Prefab Apply/Validate 与固定成功文本；修复 plugin 缺失资源、duplicate importer owner 与双 component identity。
2. 冻结 source/instance/object/component/property stable IDs、typed address/value、source revision、schema/plugin digest、legacy migration 与 layer precedence。
3. 建立只读 authority：provider registry、effective/origin/modified snapshot、provenance、dependency index、load lifecycle 与 deterministic diagnostics；此阶段仍不开放 source mutation。
4. 建立 versioned Prefab source 与 resolver：typed property/topology operation、nested ancestry、loop/orphan/conflict、resolved instance artifact。
5. 只通过 Editor transaction 开放 typed reset/apply/revert/break：revision CAS、多文档 preflight、rollback、selection/dirty/history 与 receipt。
6. 接入 propagation/rebase：reverse index 命中、clean auto-update、override preservation、three-way conflict、loaded/unloaded/partitioned 一致性。
7. 完成 cook/runtime：resolved artifact、schema/plugin/generation install、hot reload、bounded cache；frame hot path 不得解析 authoring JSON/path。
8. 最后完成 plugin/UX/fault/scale qualification，并硬切旧字符串 key、固定 Workbench、duplicate importer 与所有旁路。

## 10. 本轮验证与限制

本轮只做静态源码、测试 inventory、物理内容 fingerprint 和本地参考源码复核。没有运行 Cargo、Editor、save/reopen、Prefab instantiate、propagation、reset/apply/revert/break、cook/runtime install、fault/scale/soak、跨平台或跨引擎动态 benchmark。选择集存在大量共享 dirty 文件，因此后续实施前必须重新冻结 source revision 与 fingerprint。

Editor218 负责 Variant/Data Layer/Level Instance/Outliner，Editor219 负责 semantic snapshot/diff/conflict，Editor208 负责 Script/Class compiled schema，Editor05 负责 generic Inspector，Runtime99i 负责 ECS storage archetype，Runtime99j 负责 World/Scene IO。本报告只拥有统一 default authority、Prefab source/instance override、propagation/reset 产品收敛，不建立竞争 owner。整体工程 review 继续进行中。
