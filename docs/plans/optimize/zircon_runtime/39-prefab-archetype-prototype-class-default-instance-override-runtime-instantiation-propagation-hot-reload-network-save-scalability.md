---
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/scene/asset.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/assets/scene/management.rs
  - zircon_runtime/src/asset/assets/project_document/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_imported_asset.rs
  - zircon_runtime/src/asset/project/manager/importer_access.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/registration_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/categories.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/content.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/content.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/entity/dynamic_entity.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn
  - zircon_runtime/src/scene/dynamic_scene/scene_asset
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_plugins/prefab_tools
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/woc/zircon-project.toml
  - examples/woc/assets/scenes/bootstrap.scene.toml
tests:
  - zircon_runtime/src/asset/tests/assets/authoring.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/asset/tests/assets/importer/registry_priority.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/level_apply.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
  - zircon_runtime/src/tests/plugin_extensions/plugin_workspace_shape.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectArchetype.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Internal/UObject/UObjectArchetypeHelper.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/ClassDefaultObjectTest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/InheritableComponentHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/InheritableComponentHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ComponentInstanceDataCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ComponentInstanceDataCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverrideAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverrideAsset.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition/WorldPartitionPropertyOverride.cpp
  - dev/godot/scene/property_utils.h
  - dev/godot/scene/property_utils.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/tests/scene/test_packed_scene.cpp
  - dev/Fyrox/fyrox-core/src/variable.rs
  - dev/Fyrox/fyrox-core/src/reflect/inherit.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 39 · Prefab / Archetype / Prototype / Class Default / Instance Override Runtime 工程化差距

## 1. 结论

Zircon当前没有可用于工程级产品的Prefab运行时。仓内已经有`PrefabAsset`、`PrefabInstanceAsset`、Prefab cache payload、一个内建`.prefab.toml`解析器、`prefab_tools`插件清单，以及相当扎实的Dynamic Scene预检/提交事务；但这些部分没有组成`source -> validated graph -> compiled artifact -> runtime instance -> stable provenance -> update/rebase -> unload`闭环。生产代码中`load_prefab_asset`和`ImportedAsset::Prefab`只抵达类型化加载与缓存，没有实例化消费者；Vampire与WOC也没有一份真实Prefab资产或端到端使用证据。

当前实现还存在三条互相矛盾的表面能力。内建导入器以默认priority 0成功解析`.prefab.toml`，`prefab_tools`又以相同suffix和priority注册`DiagnosticOnlyAssetImporter`并报告backend未安装；registry会把同matcher同priority视为fatal duplicate，而`ProjectAssetManager::active_importer_registry`的合并路径却用`let _ = register_arc(...)`丢弃错误。与此同时插件描述写着“Prefab component, importer, and instancing services”，实际只注册一个反射component descriptor和诊断导入器，没有instancing service或runtime system。这不是可扩展架构，而是authority、capability和行为互相不一致。

Dynamic Scene值得保留：它已有source-to-target `EntityRemap`、schema/component/world/change-tick generation检查、隔离preflight World、bounded staging以及无失败publication。这些能力适合成为Prefab实例化的底层事务原语，但现在只把Scene再次spawn进目标World。`EntityRemap`是一次性返回值，没有source asset/revision、instance ID、source-object identity、instance entity set或反向索引；Scene asset reload只监听`SceneAsset`，提交新spawn，不回收旧实例，也不做base/local/current三方合并。因此它不能被算作Prefab实例化或热更新传播完成。

本报告不复制Editor44的authoring/default/Inspector所有权，也不新增P0。Editor44登记的5个P0仍是硬前置：尤其`World::from_scene_asset`忽略`prefab_instance`、`to_scene_asset`固定写`None`会在合法World roundtrip中静默丢失实例关系。Runtime39只拥有runtime compiler/artifact、instantiation transaction、provenance、update/rebase、lifecycle、streaming/network/save集成与规模资格。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试属性 / ignored | 证据强度与结论 |
|---|---:|---:|---|
| Runtime Prefab、asset与Dynamic Scene | 60 / 12,827 / 463,752 | 38 / 0 | E3逐DTO、导入器、registry、module wiring、World IO、spawn/reload transaction |
| `prefab_tools` package | 15 / 908 / 33,207 | 11 / 0 | E3逐manifest、runtime/editor/dist registration和helper；无instancer |
| focused tests | 8 / 2,843 / 106,354 | 50 / 0 | E3覆盖DTO/cache/registry/Dynamic Scene结构；无Prefab E2E |
| product与父计划控制面 | 18 / 8,490 / 678,992 | 5 / 0 | E2核对Vampire/WOC与唯一owner；产品资产零Prefab |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | 26 / 17,253 / 649,378 | 37 / 0 | E2/E3按archetype、packed scene、inheritance、resolved patch和override stack路由 |
| selected combined scope | 127 / 42,321 / 1,931,683 | 141 / 0 | 工作树fingerprint `63617f8adc10077fb354239974d3805232a422835e6aee98be1ad250d87d07a7` |

指纹按127个selected path排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。它只证明本轮静态证据集合；实施前必须重算，不能用测试属性数量替代产品行为或性能证明。

### 2.2 检查方法

本轮逐链检查authoring DTO、project document、artifact cache、importer selection、plugin aggregation、runtime module装配、typed load、World IO、Dynamic Scene compile/preflight/commit、asset reload、网络/Session搜索、第一方产品资产与参考实现。对生产域做`PrefabAsset|PrefabInstanceAsset|prefab_instance|.prefab.toml|prefab_tools`消费者搜索，并分别在network、Dynamic Scene Session和World目录检查provenance/save集成。

### 2.3 动态证据边界

本轮是review-only，没有修改Runtime、Editor、Interface、Plugin、App生产代码或测试，也没有重跑已知无法抵达本专题的全量测试。此前Editor lib编译已被239个既有错误和122个warning阻断；Prefab运行时本身又没有E2E入口，所以静态审查不能声称import、instantiate、propagate、network、save或performance通过。

## 3. 当前可保留的真实基础

1. Asset pipeline已有typed kind、importer descriptor、capability report、artifact cache与project generation，可承接Prefab source compiler和artifact publication。
2. `PrefabAsset`内嵌`SceneAsset`并能报告直接依赖，至少提供了待迁移legacy source envelope。
3. Scene project document与artifact cache payload能保存`prefab_instance`，证明序列化层存在无损扩展点。
4. Asset importer registry已有availability rank、priority、longest full suffix、duplicate matcher检测和COW published generation，选择语义可被收敛而无需重写。
5. Dynamic Scene能在spawn前校验schema catalog、component registry、World generation和change tick，避免stale plan直接提交。
6. Dynamic Scene通过隔离preflight World物化component/resource rows，再以紧凑commit artifact发布，适合作为Prefab实例化事务底座。
7. `EntityRemap`为source entity到target entity提供确定映射，可扩展为stable source-object到instance-object map，而不是另建第二套裸handle remapper。
8. reload queue已有revision supersede、bounded pending/result/apply bytes、time budget、cancellation与stale rejection，可复用为artifact generation安装队列。
9. Plugin catalog、target mode、maturity与capability status已能表达Partial/Unavailable，具备fail-closed的承载面。
10. Editor44已经冻结默认值层、typed override、传播与命令语义；Runtime39应消费该contract，不在Runtime私建第二套authoring格式。

## 4. 当前代码事实与断路

| 链路 | 当前事实 | 工程后果 |
|---|---|---|
| Source schema | `PrefabAsset = uri + name + embedded Scene + Vec<String>` | 无source revision、stable graph ID、typed parameter schema或migration |
| Instance record | asset ref + local transform + path/path/JSON overrides | rename/reparent/type migration后不可证明命中与兼容 |
| Builtin import | 泛型`toml::from_str`后直接产出`ImportedAsset::Prefab` | 无graph、cycle、dependency、override、budget或target validation |
| Plugin import | 同suffix/priority的diagnostic-only importer | 与builtin重复matcher，package行为随装配路径失真 |
| Error path | 一条active registry合并用`let _`忽略注册结果 | capability可能显示已选，实际importer被静默丢弃 |
| Runtime load | `load_prefab_asset`只有定义和generic `ImportedAsset`路由 | 无instantiate、instance handle、despawn、query或reload consumer |
| Component descriptor | `prefab_tools.Component.PrefabInstance`只有asset_ref和JSON字段 | descriptor不是storage、system、resolver或lifecycle行为 |
| World load | `from_scene_asset`不消费`prefab_instance` | source link在进入live World时消失 |
| World save | `to_scene_asset`固定`prefab_instance: None` | 正常保存静默擦除Prefab关系和overrides |
| Dynamic spawn | 每次分配新target entity并返回临时remap | 无stable instance identity、reuse、replace或old entity retirement |
| Scene reload | 监听`SceneAsset`并再次spawn | 不监听Prefab source，不定位既有实例，不做rebase/merge |
| Network/save | net与Dynamic Scene Session对Prefab/provenance搜索为0 | 复制与存档无法区分source值、local override和runtime transient |
| Product proof | Vampire/WOC无`.prefab.toml`资产 | 没有真实项目证明cook、load、spawn、update、save/reopen |

这条链的风险不是“功能少”，而是成功表象不可信：source可能被builtin解析成功，plugin capability可能被选中，component descriptor可能出现在catalog，但最终没有实体产生；若Scene经过World保存，已有instance metadata还会被写成`None`。

## 5. 参考实现差异与适用边界

| 参考 | 可验证机制 | Zircon必须吸收 | 不应机械复制 |
|---|---|---|---|
| Unreal | archetype/CDO查找与cache invalidation；construction rerun前缓存instance property/reference；Level Instance override policy | stable prototype authority、reinstance期间provenance、old/new reference map、分阶段apply | 不必复制UObject宏系统或全部Editor-only路径 |
| Godot | `SceneState`保存node/property/connection/subscene/editable instance；`PackedScene::instantiate`恢复嵌套实例和路径引用 | 编译图包含拓扑、属性、连接、嵌套依赖与实例状态；失败清理半成品 | NodePath可用于展示/序列化兼容，不能继续充当Zircon稳定身份 |
| Fyrox | ModelResource实例化写入inheritance data和original handle；Graph维护instance ID map并resolve/remap；逐字段modified位控制继承 | source-object map、实例ID、reference remap、clean/modified字段传播 | 不把每个运行时字段都包装成高开销动态容器 |
| Bevy | ScenePatch先resolve依赖，ResolvedScene缓存patch并支持spawn/apply；失败时despawn中间entity | compile/resolve与apply分离、依赖ready gate、原子失败清理、可组合patch | 不能把实验API成熟度或ECS细节当作完整Prefab authoring答案 |
| Unity Graphics | VolumeParameter显式`overrideState`，VolumeManager分层求default并用flat parameter list重置 | runtime紧凑override bitset、预解析default stack与增量reset思想 | 仅是渲染参数domain，不证明通用对象拓扑、Prefab或传播系统 |

目标不是选一个引擎照抄，而是组合可验证合同：Unreal的prototype/reinstance身份、Godot的完整scene state、Fyrox的modified inheritance、Bevy的resolve/apply事务，以及Unity Graphics在高频参数路径上的扁平化状态。Zircon最终实现必须通过自身network/save/streaming/plugin边界和同场景基准。

## 6. 唯一 Owner、父子 Finding 与目标合同

Editor44继续拥有5个P0及authoring source、default layer、typed override、apply/revert/reset/break和传播UX；Editor41拥有Scene/Level Instance无损持久化；Runtime04拥有通用asset/import/cache；Runtime05拥有World/ECS lifecycle；Runtime24拥有稳定handle/generation；Runtime08E拥有复制底座；Plugin01拥有package/capability/ABI。Runtime39只拥有Prefab运行时compiler、artifact、instance registry、transaction、provenance、artifact update、streaming/network/save adapters和资格门。

建议唯一runtime owner为`PrefabRuntimeService`，内部拆为`PrefabCompiler`、`PrefabArtifactStore`、`PrefabInstanceRegistry`、`PrefabInstantiationTransaction`、`PrefabUpdateCoordinator`和domain adapters。核心不可变产物建议为：

```text
ResolvedPrefabArtifactV1
  source_id + source_revision + source_digest
  schema/catalog/plugin/build_set fingerprints
  stable object/component graph + creation order
  compact typed component/resource payloads
  internal/external reference relocation table
  exposed parameter schema + default layer digest
  nested prefab dependency DAG
  estimated entity/component/bytes/cost budgets

PrefabInstanceSnapshotV1
  instance_id + owner/world/level + generation
  artifact identity + installed revision
  source-object -> instance-entity map
  resolved override digest + provenance mode
  lifecycle + pending update + network/save epochs
```

运行时frame hot path只消费resolved artifact和紧凑override delta，不解析TOML/JSON/path、不遍历authoring继承链。Editor与cook负责把source和typed override编译成artifact；runtime只在generation admission后实例化或原子替换。

## 7. P1：Authority、Capability、Import、Validation 与 Compiler

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-001 | builtin与plugin同时声称`.prefab.toml` | 选择唯一owner；其他入口删除或显式adapter，不允许双轨 |
| PRF-P1-002 | 两者priority同为0 | 同matcher冲突在catalog/admission阶段返回完整owner与priority诊断 |
| PRF-P1-003 | active registry忽略register错误 | 所有merge失败进入fatal load report，禁止`let _`吞错 |
| PRF-P1-004 | plugin description宣称instancing但无服务 | capability由实际factory/system/health证明，缺一即Unavailable |
| PRF-P1-005 | Partial状态仍发布importer/component表面 | descriptor区分Declared/Registered/Operational/Qualified状态 |
| PRF-P1-006 | import只做泛型TOML反序列化 | version/schema/header、unknown field、limit、diagnostic span与migration先行 |
| PRF-P1-007 | 无稳定source identity/revision | source ID、revision、digest和expected-before纳入每次compile |
| PRF-P1-008 | 无graph validation | 检查root、parent、cycle、duplicate ID、component owner与deterministic order |
| PRF-P1-009 | 无nested dependency validation | 构建Prefab DAG，报告完整cycle chain、missing与version incompatibility |
| PRF-P1-010 | 无target/provider admission | target mode、component codec、plugin capability与platform在compile前fail-close |
| PRF-P1-011 | 无deterministic compiler | 相同source/dependency/toolchain/target得到相同artifact bytes、diagnostics和digest |
| PRF-P1-012 | 无DDC/LKG publication | compile失败保留同源last-good，标记stale并生成rollback/upgrade receipt |

## 8. P1：Artifact、Stable Identity、Dependency 与 Nested Prefab

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-013 | cache只镜像authoring DTO | 发布独立`ResolvedPrefabArtifact`，禁止runtime重解析source Scene |
| PRF-P1-014 | entity path承担identity | object/component/property使用stable namespaced ID，path仅为display |
| PRF-P1-015 | `EntityRemap`只存裸source entity | artifact object ID映射generation-qualified runtime entity handle |
| PRF-P1-016 | 无instance stable ID | registry分配owner+slot+generation并检测stale/cross-world引用 |
| PRF-P1-017 | 无source-to-instance反向索引 | 按source/revision/world/level维护bounded、可分区、可恢复索引 |
| PRF-P1-018 | 无instance-to-entity ownership | 每实例保存完整entity/component/resource ownership set和root |
| PRF-P1-019 | 无reference relocation table | internal、external、soft、asset与entity refs在commit前全部解析或分类 |
| PRF-P1-020 | 无nested instance map | 每层保留source/instance namespace、parent instance和local mapping |
| PRF-P1-021 | 无exposed parameter schema | 参数有stable ID、type、unit、constraint、default、alias和codec version |
| PRF-P1-022 | 无component/topology operation | artifact表达add/remove/reparent/reorder/component replace与policy |
| PRF-P1-023 | 无unknown provider保留策略 | opaque payload无损保存、不可执行并阻断实例化，安装兼容provider后恢复 |
| PRF-P1-024 | 无artifact compatibility key | 校验engine ABI、schema catalog、plugin set、target、build set和endianness |

## 9. P1：Instantiation Transaction、Remap、Lifecycle 与 Failure Atomicity

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-025 | 没有公开instantiate request | request含artifact、owner/world/level、transform、parameters、policy与budget |
| PRF-P1-026 | Dynamic Scene直接分配新entity | reserve instance/entity identities后再compile writes，失败统一释放 |
| PRF-P1-027 | remap不持久 | commit时把mapping原子装入Instance Registry并随generation查询 |
| PRF-P1-028 | 无deferred construction phases | Allocate -> Construct -> ResolveRefs -> Validate -> Activate -> Publish |
| PRF-P1-029 | resource write混入通用Scene | Prefab声明resource ownership/share policy，禁止实例隐式覆盖World singleton |
| PRF-P1-030 | 无root/local/world transform合同 | nested transform组合、parent attach和keep-world policy确定且可测试 |
| PRF-P1-031 | 无duplicate/idempotency policy | request token支持dedupe、retry、cancel与terminal receipt |
| PRF-P1-032 | 无instance lifecycle | Requested/Resolving/Staging/Active/Updating/Retiring/Failed/Removed |
| PRF-P1-033 | 无despawn instance API | quiesce systems -> detach refs -> remove owned entities -> release leases -> tombstone |
| PRF-P1-034 | 无partial failure cleanup证明 | 每个阶段fault injection后World、registry、resources和counters回到原状态 |
| PRF-P1-035 | commit只检查World总体generation | 同时检查artifact、source、override、owner、level和network/save generation |
| PRF-P1-036 | 无instance completion event | typed success/failure/cancel/stale receipt含mapping、cost、diagnostics与correlation ID |

## 10. P1：Defaults、Override Resolution、Propagation、Rebase 与 Hot Reload

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-037 | override是path/path/JSON | runtime只接收typed、stable-addressed、versioned override operations |
| PRF-P1-038 | 无base value/revision | operation携expected base hash/revision并能分类stale/conflict |
| PRF-P1-039 | 无default layer digest | artifact记录native/script/class/prefab/variant解析结果与source fingerprint |
| PRF-P1-040 | 无effective override compiler | authoring operations编译为sorted compact delta、bitset和relocation |
| PRF-P1-041 | 无clean/modified状态 | 每字段至少区分Inherited/LocalOverride/RuntimeTransient/Conflict/Orphan |
| PRF-P1-042 | 无source update listener | Prefab artifact publication产生typed change set并查询受影响实例索引 |
| PRF-P1-043 | Scene reload误作热更新 | 建立Prefab-specific update coordinator，不复用“再次spawn”语义 |
| PRF-P1-044 | 无三方rebase | old base/new base/local delta输出clean/kept/conflict/orphan/type mismatch |
| PRF-P1-045 | 无topology rebase | object/component add-remove-reparent按stable ID和显式policy合并 |
| PRF-P1-046 | 无live instance replace transaction | stage新generation、迁移runtime state、CAS publish、retire old generation |
| PRF-P1-047 | 无runtime transient保留策略 | physics/script/AI/network状态按component adapter决定copy/reset/recreate/reject |
| PRF-P1-048 | 无rollback/LKG | update任一步失败保留旧active instance与artifact，记录degraded/stale状态 |

## 11. P1：World、Streaming、Network、Save、Script 与 Product Integration

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-049 | World IO丢`prefab_instance` | 消费Editor41无损codec gate；未通过时任何Prefab save/load fail-close |
| PRF-P1-050 | 无Level/partition owner | instance归属World/Level/Cell，stream out自动retire并保存必要状态 |
| PRF-P1-051 | 无unloaded consumer更新 | manifest记录required artifact generation，load前重编译或阻断 |
| PRF-P1-052 | 无cross-level reference policy | hard/soft/streaming refs有lease、unresolved状态、timeout与teardown顺序 |
| PRF-P1-053 | network目录零Prefab语义 | spawn复制source/artifact/instance identity与authoritative parameter digest |
| PRF-P1-054 | 无client artifact admission | digest/plugin/schema不匹配时拒绝spawn并请求兼容artifact或断开 |
| PRF-P1-055 | 无replicated override delta | authority、visibility、ordering、reliability、late join与rollback明确 |
| PRF-P1-056 | save/session零provenance | Save记录instance/source revision、typed local delta、runtime state和migration |
| PRF-P1-057 | 无restore ordering | resolve artifacts -> reserve IDs -> instantiate -> apply save delta -> activate |
| PRF-P1-058 | script只能绕过或裸spawn | script提交bounded typed request，不能构造raw Prefab DTO或直接改registry |
| PRF-P1-059 | products没有Prefab资产 | Vampire与WOC各建立真实source、nested instance、override、save/reopen案例 |
| PRF-P1-060 | server/headless未限定 | server可无render组件实例化玩法graph，client/editor使用同artifact contract |

## 12. P1：Performance、Budget、Observability、Tests 与 Qualification

| Finding | 当前差距 | 目标合同 |
|---|---|---|
| PRF-P1-061 | source DTO内嵌完整Scene | cook后使用紧凑SoA/component batches与共享immutable pages |
| PRF-P1-062 | 每次spawn重做reflection解析 | artifact预解析codec/storage slots，按catalog generation缓存plan |
| PRF-P1-063 | remap为BTreeMap临时对象 | 规模路径使用bounded contiguous mapping并保留debug可读投影 |
| PRF-P1-064 | 无实例数量/bytes预算 | per-world/level/owner/source设count、entity、component、memory和time limits |
| PRF-P1-065 | update fan-out无预算 | change set分批、priority、deadline、cancel、backpressure与stale coalescing |
| PRF-P1-066 | 无artifact resident policy | refcount/lease/LRU/pin、streaming预取、high-water和eviction telemetry |
| PRF-P1-067 | 无实例health snapshot | counts、lifecycle、age、generation、pending update、stale/conflict与last failure |
| PRF-P1-068 | 无跨阶段trace | compile/load/instantiate/update/despawn共享source/instance/correlation IDs |
| PRF-P1-069 | tests只证明DTO/descriptor | 增加contract、migration、fault、network/save、streaming和product E2E层 |
| PRF-P1-070 | 无规模benchmark | 10万实例、深嵌套、万override、source storm和stream churn基准 |
| PRF-P1-071 | 无跨平台确定性 | Windows/Linux相同输入产生相同artifact、mapping order、receipt和digest |
| PRF-P1-072 | “优于Unreal”无证据 | 同内容、同硬件、同质量测CPU/内存/load/update/frame spike并保存基线 |

## 13. P2：完整性与长期演进

| Finding | 后续要求 |
|---|---|
| PRF-P2-001 | 支持variant/derived Prefab链及可视化source DAG |
| PRF-P2-002 | 支持parameter collection、preset与批量instance parameter binding |
| PRF-P2-003 | 支持实例池化但保持generation、reset和ownership正确性 |
| PRF-P2-004 | 支持按组件选择runtime state migration adapter与版本协商 |
| PRF-P2-005 | 支持大型Prefab按subtree/cluster流式实例化和独立retire |
| PRF-P2-006 | 支持network预测spawn、确认映射和rejection rollback |
| PRF-P2-007 | 支持跨项目/package导入的source/object identity remap receipt |
| PRF-P2-008 | 支持source revision时间线、runtime provenance查询和调试overlay |
| PRF-P2-009 | 支持content-addressed duplicate artifact去重与共享只读payload |
| PRF-P2-010 | 支持插件提供custom component relocation、migration和rebase policy |
| PRF-P2-011 | 支持world partition HLOD/instancing系统消费Prefab cluster metadata |
| PRF-P2-012 | 支持server shard间instance ownership transfer与handoff receipt |
| PRF-P2-013 | 支持mod sandbox对可实例化类型、资源和预算的admission policy |
| PRF-P2-014 | 支持machine-readable dependency/fan-out/conflict/performance报告 |
| PRF-P2-015 | 支持旧path/JSON source的离线诊断、批量迁移和只读quarantine |
| PRF-P2-016 | 建立Unreal/Godot/Fyrox/Bevy可比Prefab corpus与长期回归看板 |

## 14. Hard Cutover 与禁止保留的双轨

1. `.prefab.toml`只能有一个authoritative importer；builtin与plugin二选一，禁止靠priority或注册顺序长期共存。
2. 所有importer merge必须传播错误；删除忽略`register_arc`结果的路径，duplicate matcher不得退化成warning。
3. `prefab_tools`在instancer factory、health和E2E资格存在前不得宣称instancing service；component descriptor不能作为行为完成证据。
4. legacy `PrefabAsset`和path/JSON override只能作为迁移输入；新artifact和新save不得继续写该authority。
5. `World::to_scene_asset`的`prefab_instance: None`父P0未解决前，Prefab实例Scene保存必须明确拒绝，不能静默降级。
6. Dynamic Scene继续作为事务原语，但Scene reload不得被命名或宣传为Prefab hot reload。
7. 新Instance Registry上线后，裸`EntityRemap`只作为事务结果视图；长期查询必须使用instance ID和generation。
8. update必须replace/rebase既有instance generation；禁止“再次spawn并留下旧实体”的兼容模式。
9. network/save/script只能走typed Prefab runtime facade，禁止直接构造authoring DTO或依赖display path。
10. 真实产品案例、fault gates和同场景基准未通过前，catalog maturity不得升级为Stable或宣称性能领先。

## 15. 重构里程碑

### M0 · Truth Freeze 与父P0封口

冻结唯一owner；修复/阻断World roundtrip数据损失；duplicate importer fatal可见；plugin capability按实际行为降级；建立真实Prefab最小corpus。

### M1 · Stable Schema 与 Compiler

定义source/object/component/property identity、revision、typed operation、nested DAG、migration和deterministic compiler；legacy进入只读quarantine。

### M2 · Resolved Artifact 与 Cache

发布带compatibility key、relocation、budget和dependency manifest的artifact；建立DDC/LKG、atomic publication和cross-platform digest。

### M3 · Instance Registry 与 Transaction

实现generation-qualified instance ID、persistent mapping、ownership、deferred construction、failure cleanup、despawn和terminal receipt。

### M4 · Override、Rebase 与 Update

编译compact delta；实现clean/modified/conflict/orphan分类、topology merge、runtime state adapter、CAS swap、rollback与source fan-out。

### M5 · World、Streaming 与 Product Host

接入Level/partition lifecycle、unloaded manifest、cross-level refs、server/headless与Vampire/WOC真实产品路径。

### M6 · Network、Save 与 Script

完成authority/digest admission、replicated spawn/update、late join、save/restore migration和bounded script facade。

### M7 · Scale、Observability 与 Failure

完成预算、cache/eviction、health snapshot、trace、fault matrix、source storm、stream churn和10万实例benchmark。

### M8 · Hard Cutover 与 Competitive Qualification

删除builtin/plugin/legacy双轨和伪能力；通过跨平台、产品E2E及与参考引擎同场景基准后再提升maturity与性能声明。

依赖顺序为M0 -> M1 -> M2 -> M3 -> M4 -> M5 -> M6 -> M7 -> M8。M0前不得开放保存，M2前不得做runtime实例化，M3前不得接network/save，M4前不得声称hot reload，M7前不得调整Stable maturity。

## 16. 验收门（40项）

1. **PRF-GATE-01** catalog装配后`.prefab.toml`恰有一个authoritative importer，owner、priority、availability可查询。
2. **PRF-GATE-02** 人工注册同suffix同priority importer时runtime startup fatal，并包含双方ID与matcher。
3. **PRF-GATE-03** importer registry任何merge错误都进入load report，源码无忽略结果路径。
4. **PRF-GATE-04** 缺instancer factory/system/health时Prefab capability为Unavailable且产品入口不可执行。
5. **PRF-GATE-05** malformed、unknown version、oversized、duplicate ID、graph cycle和missing dependency均在import/compile阶段失败。
6. **PRF-GATE-06** 相同source/dependencies/toolchain/target重复compile的artifact bytes、diagnostic order和digest一致。
7. **PRF-GATE-07** compile失败保留同源last-good artifact并发布stale/rollback receipt，不覆盖active generation。
8. **PRF-GATE-08** rename/reparent source object后override和instance mapping仍由stable ID命中，display path更新。
9. **PRF-GATE-09** nested Prefab cycle返回完整有序cycle chain，且import/load/cook/runtime都拒绝。
10. **PRF-GATE-10** unknown component codec payload无损保留为opaque，禁止实例化；安装兼容provider后可恢复。
11. **PRF-GATE-11** artifact ABI/schema/catalog/plugin/build-set任一不匹配均在World mutation前失败。
12. **PRF-GATE-12** instantiate成功后instance ID、artifact revision、root、完整mapping与ownership set可查询。
13. **PRF-GATE-13** instantiate每个阶段注入失败后World entity/component/resource、registry和lease计数逐项不变。
14. **PRF-GATE-14** internal/external/soft/asset/entity reference relocation全部命中；ambiguity或missing按policy明确失败。
15. **PRF-GATE-15** nested transform、parent attach和keep-world组合有golden matrix且无一帧错误姿态发布。
16. **PRF-GATE-16** duplicate request token幂等，retry不重复生成instance，cancel有唯一terminal receipt。
17. **PRF-GATE-17** despawn按逆序释放owned entities/resources/refs，stale instance handle永远不能命中新实例。
18. **PRF-GATE-18** 包含non-None Prefab instance的Scene load-save-reopen逐字段无损；旧codec不能保留时拒绝保存。
19. **PRF-GATE-19** typed override的type/unit/constraint/schema不兼容产生诊断，不进入raw JSON best effort。
20. **PRF-GATE-20** source clean field变化自动更新instance，local override保持effective value并更新base provenance。
21. **PRF-GATE-21** old base/new base/local delta同时变化产生稳定conflict artifact，重复运行digest一致。
22. **PRF-GATE-22** source object/component删除将受影响operation分类为orphan，不误命中同path新对象。
23. **PRF-GATE-23** topology add/remove/reparent和nested source变化按policy rebase，失败保持旧active generation。
24. **PRF-GATE-24** live update以CAS发布新generation并retire旧entity set，不累加重复实体或残留引用。
25. **PRF-GATE-25** physics/script/AI/network transient state按adapter迁移；无adapter时明确reset或reject。
26. **PRF-GATE-26** source storm合并stale revisions，队列满足bytes/time/cancel latency预算且无半发布。
27. **PRF-GATE-27** stream out/in恢复instance identity、artifact revision、override和save state，过期artifact先升级或阻断。
28. **PRF-GATE-28** cross-level hard/soft ref在目标卸载、重载和超时下遵守lease与diagnostic合同。
29. **PRF-GATE-29** server复制spawn含authoritative artifact digest；不兼容client在生成实体前拒绝。
30. **PRF-GATE-30** late join获得同一network epoch的instance map、override delta和active lifecycle snapshot。
31. **PRF-GATE-31** SaveGame restore按artifact resolve、ID reserve、instantiate、delta、activate顺序完成，任一步可回滚。
32. **PRF-GATE-32** script只能提交typed bounded request，越权source、owner、budget或raw payload被拒绝并审计。
33. **PRF-GATE-33** server/headless不创建render-only payload但保持玩法component、identity、network/save语义一致。
34. **PRF-GATE-34** Vampire与WOC各通过source创建、nested instantiate、override、update、save/reopen、cook和运行时加载。
35. **PRF-GATE-35** 10万实例稳定态frame hot path不解析TOML/JSON/path、不遍历authoring chain且无新增全局锁。
36. **PRF-GATE-36** artifact/instance/update内存高水位、cache hit、fan-out、stale/conflict和最长阶段可查询并可导出。
37. **PRF-GATE-37** Windows/Linux相同corpus产生相同artifact digest、mapping order、conflict分类和receipt。
38. **PRF-GATE-38** process crash或provider unload发生在compile/stage/commit任一点时可恢复到完整旧或新generation。
39. **PRF-GATE-39** legacy path/JSON资产迁移生成backup、loss report、identity remap和可重复验证的receipt。
40. **PRF-GATE-40** 与Unreal/Godot/Fyrox/Bevy同内容、同硬件、同质量基准保存CPU、内存、load/update spike；只有实测达标才允许“优于”声明。

## 17. 状态与产出记录

- 审查结论：**Runtime Prefab产品未形成**；DTO、cache、plugin descriptor和Dynamic Scene spawn不能相加为完成度。
- 新增P0：**0**；消费Editor44的5个父P0，尤其Scene World roundtrip数据损失。
- 本篇新增：**72项P1、16项P2、40个验收门**。
- 保留：asset pipeline、importer registry选择模型、artifact cache、Dynamic Scene generation/preflight/commit、reload queue预算原语。
- 收敛：唯一Prefab owner、resolved artifact、Instance Registry、typed override/rebase、atomic update、network/save/streaming adapters。
- 删除或隔离：builtin/plugin重复importer、吞错merge、无后端instancing声明、path/JSON写入authority、再次spawn式伪热更新。
- 动态验证：本轮未执行；实施阶段按M0-M8依赖顺序补contract、migration、fault、product、cross-platform与competitive benchmark证据。
