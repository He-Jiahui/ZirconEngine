---
title: Runtime Scene Reflection、Type Schema Registry、Dynamic Component、Property Address、Inspection Artifact、Subscription 与 Editor Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime111
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime_interface/src/reflect
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/scene/world/compiled_binding
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/components
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/extension/inspector/field_editor.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
tests:
  - zircon_runtime/src/scene/reflect/derived/tests.rs
  - zircon_runtime/src/scene/world/compiled_binding/tests.rs
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/inspection/tests/sparse_artifact.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/src/scene/tests/ecs_reflect
  - zircon_runtime/src/scene/tests/property_paths
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/ecs_dynamic_components_structure.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63/2026-08-21-allocation-free-single-resource-reflection-write.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/UnrealType.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PropertyAccessUtil.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/FieldPath.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/PropertyPathName.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyHandle.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyPath.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyPath.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyChangeListener.cpp
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_reflect/src/type_path.rs
  - dev/bevy/crates/bevy_reflect/src/path/mod.rs
  - dev/bevy/crates/bevy_reflect/src/path/access.rs
  - dev/bevy/crates/bevy_reflect/src/path/parse.rs
  - dev/bevy/crates/bevy_reflect/src/path/error.rs
  - dev/bevy/crates/bevy_reflect/src/structs.rs
  - dev/bevy/crates/bevy_reflect/src/list.rs
  - dev/bevy/crates/bevy_reflect/src/map.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/component.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/resource.rs
  - dev/Fyrox/fyrox-core/src/reflect/mod.rs
  - dev/Fyrox/fyrox-core/src/reflect/field.rs
  - dev/Fyrox/fyrox-ui/src/inspector/mod.rs
  - dev/Fyrox/fyrox-ui/src/inspector/editors/mod.rs
  - dev/godot/core/object/property_info.h
  - dev/godot/core/object/class_db.h
  - dev/godot/core/object/class_db.cpp
  - dev/godot/core/object/object.h
  - dev/godot/core/object/object.cpp
  - dev/godot/editor/inspector/editor_inspector.h
  - dev/godot/editor/inspector/editor_inspector.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeStack.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugUI.Fields.cs
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime111 · Scene Reflection / Type Schema / Inspection 当前源码工程化差距复核

## 1. 结论

Runtime63 的核心结论在当前源码中仍成立。Zircon 已经有可保留的反射骨架：full/short type path 与歧义索引、派生 adapter、VM catalog staging、dynamic batch write、不可变 inspection artifact、typed watch index、事实合并和 Editor undo 接入点都是真实实现。问题不在于“完全没有功能”，而在于这些能力尚未收敛为一个原子 catalog、唯一 typed property address、带 revision 的 mutation transaction，以及可证明同代的 inspection/subscription publication。

最高风险没有变化。`World::register_component_type` 仍先修改 live `ComponentTypeRegistry`，再调用验证更严格的 `TypeRegistry::register`；duplicate field 等后置失败不会回滚前置修改。失败调用因此仍能永久留下“descriptor 存在、dynamic payload 可写、reflection schema 不存在且无法重试”的半注册类型。**RSR-P0-001 保持 Open。** 当前测试仍没有覆盖失败后两套 registry、generation、instance eligibility 逐字节不变和立即重试。

从 Runtime63 baseline 到当前源码，唯一能改变 finding 状态的语义增量是 resource 单字段写新增 `write_field_by_slot`，删除了单元素 `Vec` 分配；100,000 writes/sample 的 ignored release benchmark也记录了 exact allocation 从 100K 到 0 的候选结果。但调用仍在每次写入时扫描 field name 得到 slot，component adapter仍clone函数指针bundle，`CompiledPropertyPlan`、catalog/provider generation、CAS和多字段事务均不存在，managed P50/P95 validation仍 pending。因此 **RSR-P1-045 只能从 Open 变为 Partial**，不能记为 Closed。

当前总账为：**P0 1 Open；P1 66 Open、1 Partial、0 Closed；P2 17 Open、0 Partial、0 Closed；48 项 RSR gate 全部 Fail。** 本文不新增重复 finding，Runtime63 的 RSR 编号继续作为唯一 owner。目标架构保持为 `ReflectionCatalogTransaction + StableTypeSchema + TypedPropertyAddress + CompiledPropertyPlan + ReflectionMutationTransaction + InspectionPublication + SubscriptionCursor`。

本轮只做 review 与文档维护，没有修改 production、tests、Cargo、ABI 或 `dev/` 参考源码；也没有运行 Cargo、Editor、真实 plugin reload、fuzz、fault、scale、profile 或跨引擎 benchmark。因此本文不宣称性能或表现达到、更不宣称超过 Unreal；它记录的是建立这种资格前仍缺失的正确性、生命周期和测量合同。

## 2. 审查边界、currentness 与 ownership

### 2.1 Canonical owner 与去重

| 领域 | Canonical owner | Runtime111 的作用 | 不重复登记 |
|---|---|---|---|
| Reflection catalog/schema/address/inspection/subscription | Runtime63 | 当前源码逐项刷新 1/67/17 findings 与 48 gates | RSR-P0/P1/P2、RSR-G 编号 |
| ECS storage/schema/query | Runtime60 / Runtime108 | 验证 dynamic JSON 双事实与 adapter storage 交接 | ECS kernel finding |
| World/persistence/schema migration | Runtime61 / Runtime109 | 验证 generation、unknown type 与 migration 交接 | persistence finding |
| Protected scene mutation | Runtime62 / Runtime110 | 要求 reflection write 进入同一 domain authority | hierarchy/derived finding |
| Stable identity/exhaustion | Runtime24 | 采用 World/type/field/provider/token generation | 通用 identity finding |
| Event mirror/cross-process cursor | Runtime54 | 本文只拥有本地 batch 自描述 resync 与 schema watch | remote broker finding |
| Public DTO/wire/CAS/budget | Interface02 | 本文验证 Runtime 实现和产品回执 | wire ABI finding |
| Inspector control/multi-selection/customization | Editor05 | 本文只拥有 Runtime 稳定事实与 command revision | Editor UX finding |

固定 package 形态仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package。反射 catalog、schema、address、mutation、artifact 与 subscription truth 必须归 runtime；Editor只消费 immutable publication并提交authoring intent，不能保留第二套手写type/field路由。用户已明确暂停tooling优化，本篇不登记tooling、脚本或Python迁移内容。

### 2.2 当前源码物理冻结

算法：repo-relative path转`/`并小写排序去重；逐文件计算lowercase SHA-256；以`path<TAB>hash`按LF连接且末尾无LF；再对UTF-8 manifest计算SHA-256。

| 冻结组 | 文件 | 行 | 非空行 | bytes | test attrs | ignored | unsafe 行 | Fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Public contracts | 18 | 1,405 | 1,245 | 39,455 | 0 | 0 | 0 | `020f0b59eab94c65754065d6b38a6bfc6aea3f33f4390070710aa67583f66911` |
| Catalog / adapters / storage | 27 | 4,064 | 3,721 | 141,643 | 6 | 0 | 0 | `88abd1189fd73fb053000d402c2076ca5777c56372e50eac1e8f74826f3388d5` |
| Property / inspection production | 30 | 8,389 | 7,867 | 316,941 | 4 | 0 | 0 | `79a760f02934d08a908a4769660dbfeac50aab9b1ea9745c1707275e22679198` |
| Builtin declarations | 20 | 1,523 | 1,405 | 45,765 | 0 | 0 | 0 | `65281480b2b45e8616e3e6ec7e7cacf08ac278b60db71ea733789f667cb5516c` |
| Editor consumers | 6 | 2,270 | 2,087 | 79,349 | 3 | 0 | 0 | `0ad5861c7ac218351adf26b062b4bd33fe352264db39eab8e05407c087812071` |
| Focused tests | 25 | 7,912 | 7,252 | 302,285 | 189 | 1 | 0 | `da2639fc5021cb50999a0fae8e80955b6fe1617f5f56042caabc7abd364bd3e9` |
| 去重 production | **101** | **17,651** | **16,325** | **623,153** | **13** | **0** | **0** | `8ffc5a1a660e2c19b2d3635726c25e1eb1dc38e1fa1b10d7e835245db25240c0` |
| 去重 focused set | **126** | **25,563** | **23,577** | **925,438** | **202** | **1** | **0** | `247a609cc8a999e4f278f0492fa9db0ccff032b2ddb8df385a0858f90e2d34d4` |

五套参考冻结共37文件、41,133行、35,620非空行、1,532,682 bytes，按同一算法 fingerprint 为 `5cc304dca687b7bbd1678e9911f7f0e14dec5e0a894bdf3a117b7e1a3f2e4952`：Unreal 8/10,585/419,638，Bevy 11/5,471/185,656，Fyrox 4/3,823/150,330，Godot 7/14,246/494,822，Unity Graphics 7/7,008/282,236。该current refresh显式使用上述lowercase path + TAB算法，不能与旧报告采用不同manifest规范的fingerprint直接比较。

冻结对应HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。冻结时共享工作区有247个status entries；focused set仅有`world_reflection.rs`和`ecs_reflect/foundation.rs`两个working-tree格式变化。本文绑定实际working-copy内容，不归因、不回退，也不把并行未集成内容当成accepted baseline；实施前必须重算fingerprint。

相对 Runtime63 baseline `bea1acf91b909525ab1759e2c800858b0eda6528`，focused set只有5个文件变化、158 additions、4 deletions。`scene_inspection_publication.rs`与当前两个working-tree变化是格式调整；语义增量集中于resource single-slot write、route assertion和ignored release benchmark，没有建立catalog transaction、typed address、mutation receipt或publication generation bundle。

### 2.3 复核方法

1. 逐文件读取15个公开reflect DTO、world-sync invalidation/watch与`EntityPath`，检查serde admission、identity、budget和revision。
2. 逐文件读取catalog、component/resource adapter、dynamic JSON、VM schema sync和dynamic storage，复现registration顺序与migration路径。
3. 展开property access、compiled binding、inspection artifact/cache/subscription，核对parser、generation、allocation、error与overflow语义。
4. 沿Editor projection、field update、selection publication与undo command真实产品链检查unsupported value、stale schema和同代publication。
5. 读取25个focused test文件，并对Runtime63 baseline后的5个相关变化区分语义实现、格式、source-shape assertion和未完成benchmark。
6. 逐套复核37个参考文件；Unreal作为系统规模主参考，Bevy/Fyrox校验Rust反射与Inspector结构，Godot校验property list/update/undo语义，Unity Graphics只校验typed parameter/lifecycle/runtime debug field合同。

## 3. 当前真实链路

```text
public component descriptor registration
  -> ComponentTypeRegistry::register(live)       [live state already changed]
  -> TypeRegistry::register                      [stricter validation may fail]
  -> no rollback / no retry path                 [RSR-P0-001]

VM catalog sync
  -> clone component/type registries
  -> synchronously validate all retained VM JSON
  -> swap registry sets
  -> no catalog receipt / World generation / affected artifact invalidation

property request
  -> DTO: String type path + top-level field_name
  -> generic path parser splits first '.'
  -> compiled dynamic path splits last '.'
  -> raw/component/segments may disagree after serde
  -> adapter clone or per-write slot scan
  -> no World/provider/schema-qualified compiled plan or CAS

Inspector selection
  -> hierarchy artifact and one-entity fields cache built independently
  -> scan every RuntimeTypeRegistration
  -> contains/read every candidate and clone values
  -> swallow adapter errors and missing declared values
  -> string-sort by mutable display labels
  -> Editor drops unsupported variants or uses hardcoded builtin path aliases

subscription overflow
  -> discard fact
  -> mark process-local diagnostics only
  -> flush {world generation, dirty, remaining facts}
  -> consumer cannot observe gap, dropped range, cursor invalidation or resync requirement
```

## 4. P0 当前证据

### RSR-P0-001：公开component type注册仍可留下不可恢复半提交状态

状态：**Open**。

- `World::register_component_type`仍先调用`component_types.register(descriptor)`，随后才调用`type_registry.register(registration)`。
- 前者接受的descriptor可在后者因duplicate field、空value type或default mismatch失败；失败路径没有staging copy、rollback或catalog transaction。
- 前置成功使`component_type_descriptor(type_id)`可见，也允许dynamic payload按该descriptor写入；反射schema/read却返回unknown type。
- 同type重试被前置registry判duplicate，公开API没有修复或撤销半注册状态。
- VM sync虽然已经在clone上preflight retained payload并swap，但公共descriptor注册没有复用该方向。
- focused tests仍没有“失败后两套registry、component registry、generation和instance eligibility逐字节不变，并可立即重试”的行为测试。

关闭条件：所有descriptor、type registration、adapter、component ID、provider identity、field schema与consumer admission先在staging catalog完成；任一participant失败时live state零变化；成功只发布一个catalog generation与typed receipt。

## 5. Runtime63 P1 状态逐项刷新

状态计数：**Open 66；Partial 1（045）；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RSR-P1-001 | Open | VM schema sync交换registries后仍不推进World generation，也不列affected type/entity或使现有field artifact失效。 |
| RSR-P1-002 | Open | `schema_catalog_generation`与component schema generation仍用`saturating_add`，MAX后catalog可变而generation不变。 |
| RSR-P1-003 | Open | generic registration仍允许既非component也非resource的opaque metadata，无显式TypeRole owner。 |
| RSR-P1-004 | Open | `is_component=true`仍可无component adapter注册，能力在产品读取时才失败。 |
| RSR-P1-005 | Open | `is_resource=true`仍可无resource adapter，metadata-only状态继续冒充live capability。 |
| RSR-P1-006 | Open | adapter内部`type_path`与registration full path仍没有单一binding key或一致性admission。 |
| RSR-P1-007 | Open | serialization enum与serializable flags仍能表达矛盾组合。 |
| RSR-P1-008 | Open | 非VM registration仍不统一验证plugin/provider identity、generation和type path ownership。 |
| RSR-P1-009 | Open | display/documentation/module/plugin文本仍无bytes、字符和locale预算。 |
| RSR-P1-010 | Open | numeric range仍不验证finite、min<=max、step>0和precision边界。 |
| RSR-P1-011 | Open | enum option仍不验证唯一value/display、default membership和数量预算。 |
| RSR-P1-012 | Open | editor hint仍可与value type不匹配，也没有custom hint provider资格。 |
| RSR-P1-013 | Open | type/field仍无StableTypeId、StableFieldId、schema version/fingerprint或dependency list。 |
| RSR-P1-014 | Open | plugin schema upsert仍只围绕字符串plugin id，缺provider generation、lease、quiescence和retirement。 |
| RSR-P1-015 | Open | register/upsert/remove/clear仍无统一participant transaction和typed receipt。 |
| RSR-P1-016 | Open | component address仍只有裸`u64` entity与String type path，无World/owner/entity generation。 |
| RSR-P1-017 | Open | address/type/path DTO仍derive Deserialize，可绕过constructor non-empty和一致性检查。 |
| RSR-P1-018 | Open | read/write仍只有top-level `field_name`，不支持nested struct/list/map/optional path。 |
| RSR-P1-019 | Open | `ComponentPropertyPath::parse`仍按首个`.`切component，与含`.`的qualified provider type path冲突。 |
| RSR-P1-020 | Open | path仍保存raw/component/property_segments三份事实，serde可制造不一致。 |
| RSR-P1-021 | Open | generic access消费component/segments，compiled dynamic access仍对raw做`rsplit_once('.')`。 |
| RSR-P1-022 | Open | `EntityPath`仍保存raw+segments并derive Deserialize，缺validated canonical encoding。 |
| RSR-P1-023 | Open | segment仍主要是字符串，不能统一表达field/tuple/list index/map key/optional/variant。 |
| RSR-P1-024 | Open | path与component-field interner仍只增不退，无per-World bytes/items budget、retire或compact。 |
| RSR-P1-025 | Open | PathId和binding generation耗尽仍以`checked_add(...).expect` panic。 |
| RSR-P1-026 | Open | compiled handle只有root/entity局部generation，不携World identity与provider/schema generation bundle。 |
| RSR-P1-027 | Open | dynamic compiled handle仍只看ComponentTypeRegistry generation，不看完整reflection catalog/adapter generation。 |
| RSR-P1-028 | Open | `ReflectedValue`递归List/Map/Json仍无depth/items/bytes/time预算。 |
| RSR-P1-029 | Open | `Entity(Option<u64>)`与`Resource(String)`仍无type、owner、generation和resolution disposition。 |
| RSR-P1-030 | Open | value tree仍缺f64、通用Option、tuple/array/set、typed map key、bitflags与variant payload。 |
| RSR-P1-031 | Open | `ZrReflectValue`仍只覆盖有限scalar/vector/Vec/Option<u64>，没有schema驱动的完整conversion capability。 |
| RSR-P1-032 | Open | nested conversion failure仍缺完整PropertyAddress、元素index与segment offset。 |
| RSR-P1-033 | Open | JSON number到f32仍可缩窄，只有finite检查，没有lossless/coercion policy。 |
| RSR-P1-034 | Open | 非VM dynamic reflection仍经过较窄`ScenePropertyValue`，List/Map/Json/AnimationParameter不能统一读写。 |
| RSR-P1-035 | Open | ComponentTypeRegistry为空时仍接受任意未注册dynamic type，live path未默认fail closed。 |
| RSR-P1-036 | Open | 普通component descriptor payload仍没有执行VM级完整shape/type/default验证。 |
| RSR-P1-037 | Open | VM payload仍要求精确字段集合，无optional/default/alias/deprecated/unknown策略。 |
| RSR-P1-038 | Open | obsolete schema有实例时仍只能拒绝删除，无orphan opaque保留、migration或last-good rollback。 |
| RSR-P1-039 | Open | retained VM payload validation仍同步扫描全部dynamic JSON，无反向索引、deadline/cancel/progress。 |
| RSR-P1-040 | Open | dynamic JSON HashMap与ECS component presence仍是两份可分裂truth。 |
| RSR-P1-041 | Open | `dynamic_components_for_entity`仍向caller克隆全部JSON和descriptor，无borrowed projection/field mask。 |
| RSR-P1-042 | Open | plugin ownership仍在多处依赖字符串prefix推断，而非ProviderId/Generation receipt。 |
| RSR-P1-043 | Open | schema sync仍无affected type/entity、old/new fingerprint、migration/retirement统计。 |
| RSR-P1-044 | Open | 单字段write仍无expected revision/CAS、transaction ID、permission与committed generation receipt。 |
| RSR-P1-045 | Partial | resource单写已用`write_field_by_slot`去掉单元素Vec，但每次仍扫描field name找slot；component adapter仍clone，CompiledPropertyPlan/CAS/managed latency均缺。 |
| RSR-P1-046 | Open | component/resource adapter能力仍不对称，ensure/take/copy/map/remove/lifecycle policy没有统一capability table。 |
| RSR-P1-047 | Open | `stage_clone`等缺能力时仍可能返回`Ok(false)`，Unsupported与Absent语义混同。 |
| RSR-P1-048 | Open | `editor_visible()`/`remote_visible()`默认仍排除plugin-owned type，provider scope filter语义未产品化。 |
| RSR-P1-049 | Open | builtin reflection仍仅15个registration；Sprite2d、Mesh2d、Collider、Joint、animation player和post-process等无统一inventory。 |
| RSR-P1-050 | Open | Camera/MeshRenderer/RigidBody/LocalTransform等仍有大量`zr_reflect(skip)`，无owner/reason/替代API账本。 |
| RSR-P1-051 | Open | focused field build仍遍历全部registration，contains/read/clone后按字符串排序，无archetype projection plan。 |
| RSR-P1-052 | Open | adapter read error仍由`if let Ok`吞掉，Inspector把失败伪装成不存在。 |
| RSR-P1-053 | Open | adapter少返回声明字段时仍`filter_map`静默删除，无degraded artifact。 |
| RSR-P1-054 | Open | fields cache仍只保留一个entity，多Inspector/remote/debug consumer可互相逐出。 |
| RSR-P1-055 | Open | field identity仍是String type path + String field name，无stable ID/schema generation。 |
| RSR-P1-056 | Open | fields artifact仍只有World generation，不携schema/provider generations。 |
| RSR-P1-057 | Open | hierarchy与focused fields仍经独立cache/构建时点发布，selection revision不等于同代schema/value bundle。 |
| RSR-P1-058 | Open | RwLock/Mutex poison仍以`into_inner`静默恢复，无fault diagnostic、quarantine或确定性重建。 |
| RSR-P1-059 | Open | viewport projection仍丢弃Null/Integer/List/Map/Json；部分workbench可写Integer并不能补齐统一unsupported disposition。 |
| RSR-P1-060 | Open | Editor projection仍手写约15个builtin type path和field alias，维护第二套路由truth。 |
| RSR-P1-061 | Open | reflected command仍只存node/string type/string field/before/after，apply/revert不验证world/schema revision。 |
| RSR-P1-062 | Open | fact count/byte overflow仍直接丢事实，只设置process-local diagnostics，batch无`resync_required`或dropped range。 |
| RSR-P1-063 | Open | age overflow仍只mark dirty，consumer无法判断cursor事实序列已不完整。 |
| RSR-P1-064 | Open | WatchKey component仍是raw String、subtree仍是裸entity id，无World/type/entity generation。 |
| RSR-P1-065 | Open | schema register/upsert/remove仍不触发ComponentType watcher或affected-type invalidation。 |
| RSR-P1-066 | Open | token allocator仍wrap后循环搜索，keyspace满可无限循环且token无session epoch。 |
| RSR-P1-067 | Open | watch仍无lease/TTL/principal/permission/ack/cursor/resume，mutation throat同步锁session表。 |

## 6. Runtime63 P2 状态逐项刷新

状态计数：**Open 17；Partial 0；Closed 0**。

| ID | 状态 | 当前源码复核 |
|---|---|---|
| RSR-P2-001 | Open | `RuntimeTypeRegistration::PartialEq`仍只比较metadata和adapter存在性，不比较capability内容或generation。 |
| RSR-P2-002 | Open | public schema仍大量String/Vec clone，没有interned metadata snapshot或Arc schema。 |
| RSR-P2-003 | Open | reflected read/write error仍缺request/correlation ID。 |
| RSR-P2-004 | Open | validated constructor与直接struct literal/derive serde仍并存，admission约定不唯一。 |
| RSR-P2-005 | Open | TypeKind声明Tuple/Enum/List/Map等kind，但registration admission仍缺kind-specific shape验证。 |
| RSR-P2-006 | Open | `module_path`仍无canonical格式或与full type path一致性校验。 |
| RSR-P2-007 | Open | enum/field documentation与display metadata仍无localization key。 |
| RSR-P2-008 | Open | reflected mismatch仍主要返回variant名，缺schema fingerprint与nested actual location。 |
| RSR-P2-009 | Open | inspection排序仍依赖可变display string，locale变化会改变稳定顺序。 |
| RSR-P2-010 | Open | field delta仍建立两个BTreeMap并clone changed values，无stable slot fast path。 |
| RSR-P2-011 | Open | artifact cache equality/clone与generation guard仍依赖隐式约定。 |
| RSR-P2-012 | Open | 普通World mutation仍可能发布空hierarchy delta，consumer自行辨别domain未变。 |
| RSR-P2-013 | Open | subscription byte estimate仍基于`size_of<WorldFact>`，不计String/Vec/heap payload。 |
| RSR-P2-014 | Open | reparent fact仍只有new parent，无old parent、transaction ID或reason。 |
| RSR-P2-015 | Open | tests仍大量使用`include_str!().contains`锁源码形状，而非行为、generation或复杂度。 |
| RSR-P2-016 | Open | 虽有100K sparse artifact characterization，仍缺10K types + 100K entities + multi-Inspector + schema churn完整allocation/latency矩阵。 |
| RSR-P2-017 | Open | 公共reflection/schema/inspection文档仍未冻结线程模型、callback重入和plugin unload保证。 |

## 7. 五套参考实现对照

| 参考 | 当前源码复核到的工程做法 | Zircon 当前差距 | 采用边界 |
|---|---|---|---|
| Unreal CoreUObject / PropertyEditor | `FProperty`统一owner、flags、offset、serialize/import/export/identical/hash；field/property path保存field identity、type和container index；PropertyHandle覆盖child/container、多对象值、default/reset与change通知。 | Zircon是String type path + top-level field，缺owner-scoped identity、container path、multi-object transaction、revision与complete change chain。 | 作为system-scale主参考；不复制UObject、宏和Editor widget体系。 |
| Bevy Reflect / ECS reflect | TypeRegistry按TypeId/full path/ambiguous short name索引并携可扩展TypeData；ReflectComponent把insert/apply/remove/take/copy/map/register绑定到同一registration；ParsedPath统一typed access与错误offset。 | Zircon metadata与adapter可半注册；parser分裂；dynamic value/container capability不完整。 | 采用Rust type-data、typed path与operation table思想；不把Bevy API形状视为最终产品合同。 |
| Fyrox Reflect / Inspector | Reflect支持field/array/map/path resolution/custom setter和recursive enumeration；Field metadata含read-only/range/step/precision；Inspector以type-keyed editor definition构建/同步并发Modify/Add/Remove action。 | Zircon只支持顶层字符串overwrite，hint与write validator分离，Editor hardcode type path且unsupported value消失。 | 用于Rust reflection/Inspector分责校验；不恢复面向对象Node树。 |
| Godot ClassDB / Object / EditorInspector | PropertyInfo统一type/name/class/hint/usage；Object支持indexed set/get与property-list-changed；Inspector区分full tree rebuild、单property refresh、多property undo并维护focus/cache。 | Zircon schema/value generation没有独立publication；watch不响应schema commit；command没有stale conflict或multi-field transaction。 | 采用property-list lifecycle与刷新粒度；不复制SceneTree singleton。 |
| Unity Graphics Volume / DebugUI | VolumeParameter<T>成套拥有override、interp、clone、release和typed value；Stack/Manager维护flat parameter cache、reset、validation与lifecycle；DebugUI Field把getter/setter/validation/range/step/history绑定。 | Zircon metadata只是UI提示，缺default/override/lifecycle/compiled accessor；范围校验不约束真实write。 | 仅作为typed runtime parameter、lifecycle和debug field合同参考；不让Graphics package拥有Scene reflection catalog。 |

共同结论不是叠加五套API，而是冻结五个结构事实：type metadata与能力原子发布；property path只有一个typed IR；write是带revision的transaction；artifact绑定可比较generation；任何事实丢失都由batch自描述resync。

## 8. 目标架构与不变量

```text
ProviderPackage / BuiltinSchemaSource
  -> ReflectionCatalogTransaction
       prepare: identity + role/capability + value/hint/default + dependency/migration + budget
       commit: ComponentSchemaRegistry + TypeSchemaRegistry + AdapterTable + compiled slots
       publish: CatalogCommitReceipt {epoch, affected types, migration, retirement}

TypedPropertyAddress
  {WorldId, ObjectHandle, StableTypeId, [PropertySegment], SchemaGeneration}
  -> CompiledPropertyPlan
       {adapter/provider generation, slot chain, validators, permission}
  -> ReflectionMutationTransaction
       preflight / CAS / protected-domain dispatch / atomic apply / publication
       -> MutationReceipt {before, after, world+schema generation, effects}

InspectionPublication
  <- committed topology/value/schema/provider generation bundle
  -> bounded archetype projection + field mask + explicit diagnostics
  -> immutable selection artifacts
  -> SubscriptionCursor {ack, overflow-resync, resume, retire}
  -> Editor Inspector / remote consumers / automation
```

必须冻结的不变量：

1. 任一失败catalog operation不改变live registry、component ID、adapter、generation或instance eligibility。
2. 一个live type的metadata、adapter、storage role和provider generation只能原子存在或原子不存在。
3. property address只有一个canonical parser/IR，显示字符串不能改变路由。
4. 每个compiled plan验证World、object、type、field、schema和provider generation。
5. reflection write遵守domain authority、permission、CAS和transaction，不提供第二条绕过路径。
6. schema-only变化明确失效依赖它的plan、artifact和watch，不用无关World mutation代替。
7. artifact不静默删除adapter/schema错误；degraded和last-good均可观察。
8. subscription丢任何事实后，consumer只看batch即可确定resync、gap和恢复位置。
9. provider unload前adapter call/plan/artifact lease完成quiescence或被generation拒绝。
10. hot path成本与目标field/affected entity相关，不与全部registered types或World总entity数线性绑定。

## 9. 重构里程碑

| Milestone | 必做内容 | 退出条件 |
|---|---|---|
| M63-0 Characterization | 建半注册P0 RED测试和failure injection；枚举全部registration/address/artifact/watch入口；冻结stable identity；删除source-shape acceptance资格 | P0稳定复现，所有入口有owner与行为测试 |
| M63-1 Catalog transaction | descriptor/registration/adapter/component ID staging；role/provider/serialization/hint/default预算；一次commit/receipt；hard-cut metadata伪live路径 | RSR-G01-G12通过，失败零live mutation且可立即重试 |
| M63-2 Typed value/address | 唯一typed segment grammar；custom wire decode；完整dynamic kind/reference/error path；ValueBudget与numeric coercion | RSR-G13-G27通过，无歧义split或伪造DTO |
| M63-3 Compiled plan/mutation | 编译slot/validator/permission/provider generation；single/multi-field/object CAS事务；protected mutator；inverse receipt | RSR-G28-G32与Editor stale conflict通过 |
| M63-4 Dynamic migration/storage | 收敛Runtime60唯一row；type->instance索引；versioned defaults/aliases/orphans；bounded migration与provider retirement | schema churn可cancel/rollback，storage truth不分裂 |
| M63-5 Inspection publication | 同代generation bundle；archetype projection/field mask；diagnostic artifact；multi-entity cache与stable delta slot | RSR-G33-G41通过，不扫描无关type、不吞错 |
| M63-6 Subscription/Editor | cursor/epoch/resync/dropped range；schema watch；删除Editor手写mapping；unsupported value显式read-only | RSR-G42-G46通过，真实Editor链消费runtime truth |
| M63-7 Qualification | 10K type/100K entity/multi-consumer/schema churn；reload/overflow/poison/OOM/cancel/exhaustion；双平台profile与同语义参考对照 | RSR-G47-G48通过且证据绑定fingerprint/BuildSet |

实现顺序不能调换。M63-0/M63-1必须先关闭半注册P0，再做compiled slot、cache或Editor控件优化；否则只是让不可信catalog更快地扩散到更多consumer。

## 10. 48 项门禁当前状态

| Gate组 | 当前状态 | 失败原因摘要 |
|---|---|---|
| RSR-G01-G12 Catalog / provider | **12 Fail** | 半注册P0仍在；无stable type/field identity、atomic participant commit、typed receipt、terminal exhaustion和provider quiescence。 |
| RSR-G13-G27 Address / value / dynamic storage | **15 Fail** | parser/serde事实分裂；typed segments、budget、full value kind/reference、versioned migration与唯一storage authority均未完成。 |
| RSR-G28-G41 Mutation / artifact | **14 Fail** | 无CAS/multi-write transaction/receipt；builtin inventory与skip账本不全；artifact全registry扫描、吞错、单entity cache且generation分裂。 |
| RSR-G42-G48 Product / subscription / qualification | **7 Fail** | unsupported value消失、command不拒绝stale、overflow不自描述、watch identity不合格，完整规模/故障/profile证据缺失。 |

Runtime63中RSR-G01-G48的逐项定义继续有效。本轮没有证据把任何gate标为Partial或Pass。resource单写child record只证明一个allocation候选优化，不满足G17 compiled generation、G28 CAS、G31 receipt、G47 versioned latency/memory budget或G48 managed acceptance。

## 11. 禁止的临时修补

- 禁止只在`register_component_type`失败时删一张map，却不回滚component ID、adapter、generation、watch与instance eligibility。
- 禁止保留first-dot/last-dot两套split并用更多字符串转义修补qualified type path。
- 禁止把StableTypeId定义成当前type path哈希，却没有namespace、schema version、provider和collision policy。
- 禁止继续把dynamic JSON HashMap与ECS marker双写，再增加reconciliation pass掩盖truth分裂。
- 禁止把single-field `Vec`删除宣称为PropertyPlan完成；slot扫描、adapter clone、generation和validator仍必须编译。
- 禁止让Inspector把adapter error、missing declared field或unsupported value继续显示为“不存在”。
- 禁止给one-entity cache增加更多全局slot替代consumer scope、budget和generation retirement。
- 禁止用process-local overflow counter代替consumer可见的gap/resync/cursor协议。
- 禁止以source-shape test、ignored benchmark、API存在或compile-only证明工程化完成。
- 禁止为旧字符串address、raw mutation或双catalog路径保留长期compat facade；迁移后必须hard cutover。

## 12. 当前状态与下一执行切片

- review状态：current-source refresh complete；implementation状态：pending。
- canonical总账：P0=1 Open；P1=66 Open/1 Partial/0 Closed；P2=17 Open/0 Partial/0 Closed；RSR gates=48 Fail。
- 本轮只新增review与索引记录，没有修改production、tests、Cargo、ABI或reference source。
- MVP 00 baseline仍为in progress，F0及后续阶段尚未解锁；本轮docs-only review没有运行Cargo、Editor或动态产品验证。
- Runtime63 child implementation只完成resource single-write allocation candidate；combined managed P50/P95 validation仍pending，不能宣称latency达标。
- 首个实现切片固定为M63-0/M63-1：先建立半注册RED矩阵并以`ReflectionCatalogTransaction`关闭RSR-P0-001；在此之前不得优先扩展Inspector控件、remote watch或局部cache workaround。
