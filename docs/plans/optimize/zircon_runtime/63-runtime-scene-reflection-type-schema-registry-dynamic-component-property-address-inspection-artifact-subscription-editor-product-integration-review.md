---
title: Runtime Scene Reflection、Type Schema Registry、Dynamic Component、Property Address、Inspection Artifact、Subscription 与 Editor Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime63
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime_interface/src/reflect
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime/src/core/framework/scene/entity_path.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/scene/world/compiled_binding
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/components/render2d
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/extension/inspector/field_editor.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_reflect
  - zircon_runtime/src/scene/tests/property_paths
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/ecs_dynamic_components_structure.rs
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 63 · Runtime Scene Reflection、Type Schema Registry、Dynamic Component、Property Address、Inspection Artifact、Subscription 与 Editor Product Integration 工程化差距

## 1. 结论

Zircon 已经拥有一套可用的反射骨架，而不是空实现：`TypeRegistry` 有全路径与歧义短路径索引，派生宏能生成字段 metadata 和 dense slot adapter，VM schema 同步会在 clone 上预检 retained payload，dynamic component batch write 只推进一次 World generation，inspection artifact 可复用不可变字段切片，subscription 有typed index、事实合并和数量/字节/年龄预算，Editor 的命令也确实通过 Runtime reflection 完成读写与 undo/redo。这些能力应保留并收敛成一个权威系统。

但当前公开 `World::register_component_type` 不是原子注册。它先修改 live `ComponentTypeRegistry`，再调用验证更严格的 `TypeRegistry::register`；若后者因重复字段等原因失败，前者不会回滚。失败后的 World 会永久留下“descriptor存在、dynamic payload可写入、reflection registration不存在”的半提交类型，重试又被前置 registry 判为重复。这是当前合法公开 API 可以制造的不可恢复运行时状态，本篇登记为唯一新增 P0。

其余差距集中在四条断裂链。第一，schema registration 没有稳定 type/field ID、version、fingerprint、provider generation、migration 与 participant transaction，schema 同步成功也不推进 World/inspection generation。第二，`ComponentPropertyPath` 同时保存 raw/component/segments，却由派生 serde 绕过构造不变量；普通 property API 取第一个点号前的组件，compiled dynamic path 则取最后一个点号前的组件，带点 plugin type path 因而有两种解释。第三，inspection 构建逐次扫描整个 registry、静默吞 adapter 错误、只缓存一个 entity，并用字符串对作为字段身份。第四，subscription 溢出会丢事实，但 `InvalidationBatch` 不携 `resync_required` 或 dropped range；Editor command 也不携 schema/world revision，无法自行弥补。

本轮登记 **1项P0、67项P1、17项P2和48项验收门禁**。目标是建立 `ReflectionCatalogTransaction + StableTypeSchema + TypedPropertyAddress + CompiledPropertyPlan + ReflectionMutationTransaction + InspectionPublication + SubscriptionCursor`，而不是继续让 registry、JSON map、property string、artifact cache 和 Editor draft 各自猜测同一个字段。本轮仅静态 review 和计划记录；没有修改 production、tests、Cargo、ABI 或参考源码，没有运行 Cargo、Editor、fuzz、100K schema/entity scale、插件 reload、跨进程或性能基准，因此不能宣称已经达到或超过 Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Public reflection and path contracts | 18 | 1,405 | 39,455 |
| Runtime registry adapters and dynamic values | 27 | 4,058 | 141,277 |
| Property binding and inspection publication | 34 | 10,488 | 392,354 |
| Builtin component declarations | 20 | 1,523 | 45,765 |
| Editor product consumers | 6 | 2,645 | 89,599 |
| Focused external tests | 19 | 4,626 | 186,494 |
| 去重合计 | **124** | **24,745** | **894,944** |

Zircon 冻结集 fingerprint 为 SHA-256 `7bba91e6466367a94c2b35c277f25006f87fafd368796a4d5c44d0b005f49e36`。算法将124个相对路径转为`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。冻结时124个文件均无 working-tree 修改。

参考集为37文件、41,133行、1,532,682 bytes，fingerprint为`c27f152182d2172f87302493bcb3b2296502948778efd085cfe0a7cb9ad280dc`：Unreal 8/10,585/419,638，Bevy 11/5,471/185,656，Fyrox 4/3,823/150,330，Godot 7/14,246/494,822，Unity Graphics 7/7,008/282,236。

### 2.2 本轮拥有与明确不拥有

- Runtime63拥有Runtime reflection catalog/schema admission、dynamic value/schema coherence、property address编译语义、reflection mutation generation、inspection字段artifact及subscription产品交接。
- Runtime60继续拥有ECS component schema/storage/archetype/query/change detection；本篇不重复“dynamic component另存HashMap”的根 storage P1，只规定reflection adapter必须消费同一个组件事实。
- Runtime61继续拥有canonical Scene persistence、unknown component round-trip和schema migration落盘；本篇拥有live catalog升级和Inspector/compiled handle失效。
- Runtime62继续拥有Hierarchy/LocalTransform/Mobility/WorldMatrix/ActiveInHierarchy protected authority；本篇只要求reflection mutation不能绕过它。
- Runtime24继续拥有World/entity/provider/generation通用identity与耗尽策略；本篇记录其在property handle、field artifact和watch token中的具体采用要求。
- Runtime54继续拥有Scene Event Mirror、跨进程cursor/backlog/reconnect；本篇的subscription finding只拥有本地`InvalidationBatch`丢失标志与schema/component watch接线，不重复累计其gap/resync P0。
- Interface02继续拥有公开DTO的wire version、预算、CAS、权限和兼容策略；本篇拥有Runtime对这些合同的实现和产品回执。
- Editor05继续拥有Inspector控件、multi-selection authoring、customization与undo UX；本篇只拥有Runtime提供给Editor的稳定schema/address/publication事实。
- 用户已要求暂停tooling优化；本篇不新增脚本、Python工具或tooling迁移里程碑。

### 2.3 当前真实链路

```text
component descriptor registration
  -> ComponentTypeRegistry::register(live)       [已修改]
  -> TypeRegistry::register
       -> duplicate field/value type validation [可失败]
  -> no rollback of first registry

VM catalog sync
  -> clone component/type registries
  -> validate every retained VM JSON payload
  -> swap four catalog sets
  -> no World generation / inspection invalidation / schema batch

Inspector selection
  -> inspection_artifact() generation
  -> one-entity fields cache
  -> iterate every RuntimeTypeRegistration
  -> adapter.contains + read_fields
  -> silently discard every adapter error
  -> string-sort and clone values
  -> Editor converts supported ReflectedValue variants
  -> string property path -> reflected command

property address
  DTO parse: first `.` owns component
  compiled dynamic: last `.` owns component
  serde: raw/component/segments can disagree

subscription overflow
  -> drop fact
  -> set process-local diagnostics.overflowed
  -> flush { generation, dirty, remaining facts }
  -> consumer receives no resync marker or dropped range
```

## 3. 当前应保留的能力

1. `TypeRegistry` 的canonical full path、short path和ambiguous short path分离是正确基础。
2. `register_vm_type`/`sync_vm_types`在clone上构造registry并验证retained payload，说明staging transaction方向已经存在。
3. `ReflectComponent`已覆盖contains/read/write/remove、dense slot、batch write和stage clone；派生组件与dynamic组件共享该入口。
4. VM dynamic schema支持递归`List<T>`与`Map<String,T>`声明，并对finite scalar/vector、entity/resource wrapper做类型校验。
5. dynamic batch field write先构造candidate再一次提交，成功只推进一次World generation。
6. reflection protected component已有Hierarchy与ActiveInHierarchy特殊adapter，证明统一adapter不等于放弃domain authority。
7. inspection artifact使用`Arc<[T]>`和generation-local delta，避免每个consumer拥有可变第二事实。
8. subscription按World/Subtree/Component/Asset建typed index，ancestor scratch复用，事实按semantic key合并且有三类预算。
9. Editor reflected command先读取before、验证editable，再通过同一Runtime facade apply/revert，已有正确transaction接入点。
10. focused tests覆盖basic schema ambiguity、remote-style read/write、dynamic VM schema、batch generation、artifact reuse/delta和bounded fact queue；这些应升级而非删除。

## 4. 参考引擎事实与 Zircon 差异

| 参考 | 代码事实 | Zircon应吸收的合同 |
|---|---|---|
| Unreal CoreUObject | `FProperty`拥有owner、flags、offset、type-specific serialize/import/export/identical/hash；`FFieldPath`与property path保存field identity、container index和类型名，而不是依靠重复split字符串。 | Type/Field必须有owner-scoped稳定identity、kind-specific operation和可验证路径段；显示字符串只是表示。 |
| Unreal PropertyEditor | `IPropertyHandle`覆盖child/container、多对象值、reset/default、pre/post change、per-object value和change event；`FPropertyChangedEvent/ChainEvent`携change kind、active member、对象索引和完整链。 | Editor mutation需要typed path、multi-object transaction、before/after通知、revision和change receipt。 |
| Bevy Reflect | `TypeRegistry`按`TypeId`、full path和歧义short path索引，registration携可扩展TypeData；`ReflectComponent`覆盖insert/apply/remove/take/copy/map entities/register component。 | adapter能力应与同一type registration原子发布，类型能力不可与metadata分离成半注册状态。 |
| Bevy ReflectPath | 路径先解析为`Access::{Field,FieldIndex,TupleIndex,ListIndex}`，再统一应用于reflect object；dynamic struct/list/map保留represented type并通过typed apply。 | 建立唯一grammar与compiled segment IR，禁止raw/component/segments三份可矛盾事实。 |
| Fyrox Reflect/Inspector | Reflect支持字段、数组、map、path resolution与custom setter；Inspector发出递归`PropertyAction`，区分Modify/Add/Remove/Revert，并以type-keyed editor definition构建/同步。 | Runtime要表达nested/container action和setter authority，Inspector不能只发top-level字符串覆盖。 |
| Godot ClassDB/Object | `PropertyInfo`区分storage/editor/read-only/restart/update-all等usage并有大量typed hint；Object提供set/get/indexed path、validate property和property-list-changed通知。 | schema metadata必须可验证且能触发全树重建或单字段刷新，不能只依赖任意World mutation失效缓存。 |
| Godot EditorInspector | 订阅`property_list_changed`重建tree，也可`update_property`局部刷新；编辑统一进入UndoRedo并恢复focus/selection/cache。 | schema generation与value generation必须分离发布，产品consumer需要明确刷新粒度和同代快照。 |
| Unity Graphics Volume | `VolumeParameter<T>`拥有override、clone、release、interp和typed value；VolumeComponent/Stack/Manager控制发现、生命周期、reset和cache validation。 | runtime-editable参数不只是JSON字段；type capability、default/override、clone/release与catalog lifecycle应成套存在。 |
| Unity Graphics DebugUI | Field由getter/setter、validation、range/step、history和runtime/editor element共同定义；数值increment显式处理overflow和precision。 | schema hint必须由Runtime校验并与写入validator绑定，不能成为只供UI猜控件的装饰metadata。 |

## 5. P0：公开类型注册可留下不可恢复半提交状态

### RSR-P0-001：`register_component_type`先提交ComponentTypeRegistry，后失败时不回滚

`zircon_runtime/src/scene/world/dynamic_components.rs:70-93`先调用`self.component_types.register(descriptor)?`，随后才调用`self.type_registry.register(...)`。前者只检查plugin prefix与duplicate type ID；后者在`type_registry.rs:380-423`额外拒绝重复字段名、空字段/value type和default type mismatch。

因此一个含重复属性名的公开`ComponentTypeDescriptor`会出现以下稳定结果：

1. `register_component_type`返回`Err(InvalidRegistration)`；
2. `component_type_descriptor(type_id)`却返回`Some`；
3. `set_dynamic_component`可按已存在descriptor写入payload；
4. `reflect_schema/reflect_read`返回`UnknownType`，Inspector也无该组件；
5. 再次注册被`ComponentTypeRegistry`拒绝为duplicate，World内没有公开修复路径。

这不是错误信息质量或最终一致性问题，而是一次失败调用永久改变live World并制造可写不可反射数据。现有`register_vm_type`已经使用clone/preflight/swap，公共descriptor注册却没有复用同一事务。必须先在staging catalog同时完成descriptor、registration、adapter、field schema、component ID与consumer admission，再以一个generation发布；失败时两套registry和component registry均保持逐字节等价。

## 6. P1 工程化差距

### 6.1 Catalog、schema admission 与 provider lifecycle

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RSR-P1-001 | VM schema sync交换registries后不推进World generation，也不标记已有entity field artifact dirty。 | 发布独立SchemaGeneration并使受影响type/entity artifact失效。 |
| RSR-P1-002 | `schema_catalog_generation`使用`saturating_add`，达到MAX后catalog继续变化但generation不变。 | 采用Runtime24统一exhaustion/epoch策略并fail closed。 |
| RSR-P1-003 | generic registration允许既非component也非resource的opaque metadata，无kind owner。 | 明确TypeRole并按role校验能力集合。 |
| RSR-P1-004 | `is_component=true`可无component adapter注册，直到产品读取才报错。 | advertised capability与adapter在同一admission transaction中校验。 |
| RSR-P1-005 | `is_resource=true`也可无resource adapter，测试把该状态固化为合法。 | metadata-only schema使用不同显式状态，不可冒充live capability。 |
| RSR-P1-006 | adapter内部`type_path`与registration full path没有一致性检查。 | 由catalog生成adapter binding key，不接受第二份caller字符串。 |
| RSR-P1-007 | `serialization=None`、`serializable=true`等组合可互相矛盾。 | 建立serialization capability矩阵并在注册时拒绝非法组合。 |
| RSR-P1-008 | 非VM registration不校验`plugin_owned/plugin_id/type_path.plugin_id/full path prefix`一致性。 | 引入ProviderId/ProviderGeneration并统一所有provider admission。 |
| RSR-P1-009 | display name、documentation、module/plugin文本无长度/字符/locale预算。 | schema admission执行bytes/items/text规范化预算。 |
| RSR-P1-010 | numeric range不校验finite、min<=max、step>0和precision边界。 | hint validator与实际write validator共用约束对象。 |
| RSR-P1-011 | enum option不校验唯一value/display、default membership和数量预算。 | enum schema生成stable variant ID并验证default/migration alias。 |
| RSR-P1-012 | editor hint可与value type完全不匹配。 | type capability决定允许的hint集合，custom hint需provider资格。 |
| RSR-P1-013 | type/field没有stable ID、schema version、layout/value fingerprint或dependency list。 | 建立StableTypeId/StableFieldId/SchemaVersion/SchemaFingerprint。 |
| RSR-P1-014 | plugin schema upsert只比对plugin id，不验证provider generation、quiescence或旧adapter仍在使用。 | unload/reload通过catalog lease、consumer ack和retirement generation。 |
| RSR-P1-015 | register/upsert/remove/clear没有统一participant transaction与typed receipt。 | `ReflectionCatalogTransaction`统一prepare/commit/publish/retire。 |

### 6.2 Value model、object address 与 property path

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RSR-P1-016 | `ReflectObjectAddress::Component`只有裸`u64` entity和String type path，无World/owner/epoch。 | 地址携WorldIdentity、EntityHandleGeneration、StableTypeId。 |
| RSR-P1-017 | address/type/path DTO派生Deserialize可绕过构造器的non-empty和一致性校验。 | custom deserialize或wire DTO->validated domain type转换。 |
| RSR-P1-018 | read/write只有top-level`field_name`，不支持nested struct/list/map/optional path。 | 使用typed segment array和container operation。 |
| RSR-P1-019 | `ComponentPropertyPath::parse`以第一个`.`切component，与要求含`.`的plugin type path冲突。 | grammar明确分隔type identity与field segments，禁止歧义。 |
| RSR-P1-020 | 同一结构保存raw/component/property_segments三份事实，serde可令三者不一致。 | 只保存canonical compiled representation，显示串按需生成。 |
| RSR-P1-021 | generic property access读component/segments，compiled dynamic access却对raw做`rsplit_once('.')`。 | 所有入口消费同一个parser/compiled IR。 |
| RSR-P1-022 | `EntityPath`同样保存raw+segments并可被serde伪造。 | entity path采用stable entity references或validated segment encoding。 |
| RSR-P1-023 | path segment只表达字符串，不能区分field、tuple、list index、map key、optional/value分支。 | 对齐Bevy/Unreal式typed access segment。 |
| RSR-P1-024 | path interner只增不减，raw string与ID映射没有retire/compact/budget。 | per-World bounded arena、reference count/epoch retirement和diagnostics。 |
| RSR-P1-025 | PathId耗尽panic，binding generation用checked add panic。 | 明确terminal exhaustion error并阻止发布新handle。 |
| RSR-P1-026 | compiled property handle只绑定root/type schema的局部generation，不携World identity。 | handle包含WorldId、root/entity generation、schema/provider generation。 |
| RSR-P1-027 | dynamic compiled handle只看ComponentTypeRegistry generation，不看完整reflection metadata generation。 | 编译依赖记录所有schema fingerprint与adapter generation。 |
| RSR-P1-028 | `ReflectedValue`递归List/Map和任意Json没有depth/items/bytes预算。 | decode/admission/write全链使用统一ValueBudget。 |
| RSR-P1-029 | `Entity(Option<u64>)`与`Resource(String)`不携类型、owner、generation或resolution状态。 | typed reference value区分null/unresolved/stale/foreign。 |
| RSR-P1-030 | value model缺f64、通用Option、tuple/array/set、typed map key、bitflags、variant payload等。 | 按TypeInfo kind实现开放的dynamic value tree。 |
| RSR-P1-031 | `ZrReflectValue`只实现有限整数/f32/String/Option<u64>/Vec/向量，Quaternion虽有variant却无通用实现。 | derive基于TypeSchema生成完整conversion capability。 |
| RSR-P1-032 | `Vec<T>`转换失败不携元素index，nested map/list错误仍只报owner field。 | error携完整PropertyAddress与segment offset。 |
| RSR-P1-033 | JSON number到f32会缩窄精度；VM路径只拒绝cast后的non-finite，不表达loss policy。 | schema声明numeric width/range与lossless/coercion policy。 |
| RSR-P1-034 | 非VM dynamic reflection经`ScenePropertyValue`，List/Map/Json和AnimationParameter不能统一读写。 | dynamic adapter直接消费typed dynamic value，不经窄中间枚举。 |

### 6.3 Dynamic component schema/value coherence

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RSR-P1-035 | ComponentTypeRegistry为空时，任何未注册dynamic type都被接受。 | 区分legacy opaque data与live registered component，默认fail closed。 |
| RSR-P1-036 | 普通`register_component_type`后的payload只按粗descriptor路径访问，不执行VM级完整shape/type校验。 | 所有live dynamic type使用同一schema validator。 |
| RSR-P1-037 | VM payload要求字段集合精确相等，没有optional/default/alias/deprecated字段。 | schema version定义required/default/alias/unknown policy。 |
| RSR-P1-038 | obsolete schema只要仍有instance就拒绝删除，没有orphan opaque保留或migration。 | catalog update带per-type migration与last-known-good rollback。 |
| RSR-P1-039 | `validate_retained_vm_payloads`同步扫描所有dynamic JSON，schema reload成本与World规模线性绑定。 | type->instance反向索引、分批preflight、deadline/cancel/progress。 |
| RSR-P1-040 | dynamic JSON与ECS presence分属HashMap和component marker，truth可分裂。 | 由Runtime60 component storage拥有唯一row，reflection只适配该row。 |
| RSR-P1-041 | `dynamic_components_for_entity`克隆全部JSON和descriptor给产品caller。 | 提供borrowed/compiled projection与字段mask。 |
| RSR-P1-042 | plugin ownership在多处用字符串prefix推断。 | 只使用catalog ProviderId/Generation与admission receipt。 |
| RSR-P1-043 | schema sync没有affected type/entity清单、old/new fingerprint或迁移统计。 | commit receipt可供inspection、compiled binding、persistence和Editor消费。 |

### 6.4 Reflection mutation 与 inspection publication

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RSR-P1-044 | 单字段reflect write没有expected revision/CAS、transaction ID、permission或committed generation。 | `ReflectionMutationTransaction`返回before/after、revision、generation和effect receipt。 |
| RSR-P1-045 | component adapter为写操作clone函数指针bundle；resource写每次解析slot并分配单元素Vec。 | 编译`PropertyPlan`并复用slot/capability，单写不伪装batch。 |
| RSR-P1-046 | component/resource adapter能力集合不对称，缺统一ensure/take/copy/map/remove/lifecycle policy。 | capability table按role明确必选/可选操作和failure semantics。 |
| RSR-P1-047 | `stage_clone`缺失返回`Ok(false)`，调用方可把不支持误当成无组件。 | 返回typed UnsupportedCapability，不静默降级。 |
| RSR-P1-048 | schema `editor_visible()`/`remote_visible()`默认排除plugin-owned type。 | filter显式表达provider scope，产品默认语义有测试。 |
| RSR-P1-049 | builtin scene导出28个component类型，reflection只注册15个；Sprite2d、Mesh2d、Collider、Joint、animation与post-process等缺统一schema。 | 建完整builtin reflection inventory并逐component声明可读/可写/derived原因。 |
| RSR-P1-050 | registered Camera/Mesh/physics字段又大量`zr_reflect(skip)`，runtime capability与真实component字段未建账。 | 每个skip有owner、reason、替代API和测试，不以默认跳过隐藏债务。 |
| RSR-P1-051 | focused field build每次遍历全部type、contains/read全部字段、clone value再字符串排序。 | 按archetype/type presence编译projection plan并支持field mask/budget。 |
| RSR-P1-052 | adapter read error被`if let Ok`静默吞掉，Inspector呈现为“字段不存在”。 | artifact携per-type diagnostic/disposition并保留last-good。 |
| RSR-P1-053 | adapter少返回声明字段时`filter_map`静默删除该字段。 | schema/value不一致使artifact publication失败或显式degraded。 |
| RSR-P1-054 | fields cache只保存一个entity，多Inspector/remote/debug consumer会互相逐出。 | generation-scoped bounded multi-entity cache或consumer-owned snapshot plan。 |
| RSR-P1-055 | field identity是`(component_type_path String, field_name String)`。 | 使用StableTypeId+StableFieldId+SchemaGeneration。 |
| RSR-P1-056 | fields artifact只有World generation，不携schema/provider generation。 | `InspectionGenerationBundle`原子绑定world/schema/provider。 |
| RSR-P1-057 | hierarchy artifact与focused fields由独立锁和独立构建时点发布。 | 同一selection snapshot引用同代topology/value/schema bundle。 |
| RSR-P1-058 | RwLock/Mutex poison被静默恢复，无diagnostic或cache quarantine。 | 记录fault、丢弃可疑cache并重建/降级。 |
| RSR-P1-059 | Editor projection丢弃Integer/List/Map/Json/Null等合法值，字段直接消失。 | Runtime提供kind/capability，Editor以unsupported editor disposition呈现而非删除。 |
| RSR-P1-060 | Editor仍手写15个builtin type path到display/property path映射。 | schema拥有canonical editor label与property address，删除第二映射表。 |
| RSR-P1-061 | Editor reflected command只存字符串type/field和before/after，apply/redo不验证schema/world revision。 | command持compiled address与expected revision，stale时冲突而非写新schema。 |

### 6.5 Subscription、失效与产品交接

| ID | 当前差距 | 所需收敛 |
|---|---|---|
| RSR-P1-062 | fact数量/字节溢出后直接丢事实，仅process-local diagnostics标记overflow。 | batch携`resync_required`、dropped count/range/reason。 |
| RSR-P1-063 | age预算超限也只mark world dirty，consumer无法知道事实序列不完整。 | cursor/epoch明确失效并要求acknowledged full snapshot。 |
| RSR-P1-064 | WatchKey的component是raw String，subtree是裸entity id。 | watch绑定WorldId、StableTypeId、entity generation和schema epoch。 |
| RSR-P1-065 | schema register/upsert/remove没有触发ComponentType watcher。 | catalog commit按affected type发布invalidations。 |
| RSR-P1-066 | token分配wrap后循环搜索，keyspace满时可无限循环；无session epoch。 | bounded allocator、epoch-qualified token与terminal exhaustion。 |
| RSR-P1-067 | watch没有lease/TTL/principal/permission/ack/cursor/resume，mutation throat同步锁session表。 | 对齐Runtime54/Interface02的session-owned subscription lifecycle和backpressure。 |

## 7. P2 完整性与维护性差距

| ID | 差距 |
|---|---|
| RSR-P2-001 | `RuntimeTypeRegistration::PartialEq`只比较adapter是否存在，不比较capability内容或generation。 |
| RSR-P2-002 | public schema大量String/Vec clone，没有interned metadata snapshot或Arc schema。 |
| RSR-P2-003 | reflected read/write error缺request/correlation ID，跨边界诊断难以串联。 |
| RSR-P2-004 | type/filter constructor与直接struct literal并存，调用约定不唯一。 |
| RSR-P2-005 | `ReflectTypeKind`声明Tuple/Enum/List/Map等kind，但当前registration admission没有kind-specific shape校验。 |
| RSR-P2-006 | `module_path`没有canonical格式或与full type path的一致性验证。 |
| RSR-P2-007 | enum documentation/display和field documentation没有localization key。 |
| RSR-P2-008 | reflected value type mismatch只返回variant名，不含schema fingerprint与actual nested location。 |
| RSR-P2-009 | inspection字段排序依赖可变display string，locale变化会改变稳定顺序。 |
| RSR-P2-010 | field delta构建每次建立两个BTreeMap并clone changed values，缺stable slot fast path。 |
| RSR-P2-011 | artifact cache equality/clone保留cache再用generation guard，语义依赖隐式约定。 |
| RSR-P2-012 | hierarchy普通World mutation也可能发布空hierarchy delta，consumer需自行辨别domain未变。 |
| RSR-P2-013 | subscription estimated bytes使用`size_of<WorldFact>`，不计String/Vec/heap payload。 |
| RSR-P2-014 | reparent fact只有new parent，无old parent、transaction ID或原因。 |
| RSR-P2-015 | source-shape测试大量依赖`include_str!().contains`，可在行为已经错误时继续通过。 |
| RSR-P2-016 | 没有10K type/100K entity、多Inspector、schema churn的基准与allocation profile。 |
| RSR-P2-017 | reflection/schema/inspection公共文档没有明确线程模型、callback重入和plugin unload保证。 |

## 8. 测试与证据缺口

现有测试证明了基础功能，但没有覆盖本篇P0。全仓搜索只找到`TypeRegistry`自身拒绝duplicate field和VM backing的validation test，没有测试`World::register_component_type`后半段失败时两套registry逐字节不变，也没有测试失败后能否重试同一type ID。

必须新增以下测试族：

1. registration failure injection：descriptor、type metadata、adapter、component ID任一步失败均零live变化；
2. schema replacement：已有instance、default/alias/migration、provider reload、old handle、old artifact、undo command同时存在；
3. property grammar round-trip：带`.`/`::`的type path、nested list/map、serde伪造raw/segments、不合法escape和Unicode边界；
4. reflection mutation：CAS冲突、multi-object atomicity、protected component、permission、undo/redo遇schema升级；
5. artifact publication：adapter error、缺字段、schema-only变更、多consumer、多selection、同代hierarchy/fields；
6. subscription overflow：消费者只看batch即可确定必须resync，并能从acknowledged snapshot恢复；
7. scale：10K types、100K entities、1K active watches、schema churn、bounded memory与deadline；
8. fault：plugin unload、poisoned cache、generation/token exhaustion、cancelled migration、out-of-budget recursive value。

当前`write_validation.rs`、`ecs_dynamic_components_structure.rs`和多个reflection structure test把源码文本/循环形状当成性能合同。它们只能作为临时结构守卫，不能替代行为、复杂度、allocation和并发证据。

## 9. 目标架构

```text
ProviderPackage / BuiltinSchemaSource
  -> ReflectionCatalogTransaction
       prepare
         - canonical type/field identity
         - role/capability matrix
         - value/hint/default validation
         - dependency + migration graph
         - provider generation + budgets
       commit
         - ComponentSchemaRegistry
         - TypeSchemaRegistry
         - AdapterTable
         - compiled field slots
       publish CatalogCommitReceipt
         {old/new epoch, affected types, migrations, retirements}

TypedPropertyAddress
  {WorldId, ObjectHandle, StableTypeId, [PropertySegment], SchemaGeneration}
  -> CompiledPropertyPlan
       {adapter generation, slot chain, validators, permissions}
  -> ReflectionMutationTransaction
       preflight / CAS / apply / protected-domain dispatch / publish
       -> MutationReceipt {before, after, world+schema generation, effects}

InspectionPublication
  <- committed topology/value/schema/provider generations
  -> bounded projection plans by archetype + field mask
  -> immutable selection artifacts + diagnostics
  -> SubscriptionCursor
       ack / overflow-resync / resume / retire
  -> Editor Inspector / remote tools / automation
```

### 9.1 必须冻结的不变量

1. 任一失败catalog operation不改变live registry、component ID、adapter、schema generation或instance eligibility。
2. 一个live type的metadata、adapter、storage role和provider generation只能原子存在或原子不存在。
3. property address只有一个canonical parser/IR；raw显示串不能改变路由。
4. 每个field handle可验证World、object、type、field、schema与provider generation。
5. reflection write遵守domain authority、permission、CAS和transaction，不提供第二条绕过路径。
6. schema-only变化能使相应compiled plan、artifact和watch明确失效。
7. inspection artifact不静默删除错误类型/字段；degraded状态和last-good均可观察。
8. subscription丢任何事实后，consumer仅凭batch就能确定必须全量resync。
9. plugin unload前所有adapter call/handle/artifact lease完成quiescence或被generation拒绝。
10. hot path访问成本与目标field/affected entity相关，不与全部registered types或World总entity数线性绑定。

## 10. 分阶段重构里程碑

### M63-0：P0 characterization 与 schema truth freeze

- 写`register_component_type`半提交RED测试和每一步failure injection；
- 枚举所有type/field registration、adapter、dynamic payload、property parser、artifact和watch caller；
- 冻结StableTypeId/StableFieldId/SchemaGeneration/ProviderGeneration schema；
- 标记source-shape测试为临时，不允许其单独作为acceptance。

### M63-1：ReflectionCatalogTransaction

- descriptor、registration、adapter、component ID在staging catalog一次验证；
- 强制role/capability、plugin identity、serialization、hint/default/range/enum预算；
- commit一次发布catalog receipt，失败零live mutation；
- hard-cut metadata-only冒充live component/resource的路径。

### M63-2：Typed value 与 property address

- 建typed property segments和唯一grammar；
- custom deserialize到validated domain address；
- 扩展dynamic value kind、typed references与nested error path；
- 引入ValueBudget和numeric coercion policy。

### M63-3：CompiledPropertyPlan 与 mutation transaction

- 编译adapter/slot/validator/permission/provider generation；
- 单字段、多字段、多对象写统一preflight/CAS/commit/receipt；
- protected component路由Runtime62 domain mutator；
- command/undo保存expected revision和inverse receipt。

### M63-4：Dynamic schema migration 与 storage收敛

- dynamic value迁入Runtime60唯一component storage；
- type->instance反向索引与bounded migration job；
- default/optional/alias/deprecated/unknown/orphan策略；
- provider reload/unload支持last-good、quiescence和retirement。

### M63-5：InspectionPublication

- schema/value/topology/provider generation bundle原子发布；
- 按archetype/type编译字段projection并支持field mask；
- adapter error与schema/value mismatch进入artifact diagnostics；
- bounded multi-entity cache、stable field ID和delta fast path。

### M63-6：Subscription 与 Editor产品迁移

- batch加入cursor、epoch、resync_required与dropped range；
- schema commit触发component/type watch；
- Editor删除手写builtin path映射，使用Runtime property address；
- unsupported editor保留可见diagnostic，不删除字段。

### M63-7：规模、故障与竞争资格

- 10K type/100K entity/schema churn/multi-consumer基准；
- plugin reload、overflow、poison、OOM、cancel、exhaustion fault suite；
- 比较compiled read/write、artifact refresh和subscription成本；
- 仅在同workload correctness与profile证据后讨论优于参考引擎。

## 11. 验收门禁

| Gate | 验收内容 |
|---|---|
| RSR-G01 | duplicate field导致registration失败后两套registry、component registry和generation逐字节不变。 |
| RSR-G02 | 任一catalog prepare participant失败均零live mutation且同type可立即重试。 |
| RSR-G03 | component/resource/metadata role与adapter capability矩阵全量验证。 |
| RSR-G04 | StableTypeId/StableFieldId跨重启、schema reorder和provider reload规则有golden。 |
| RSR-G05 | schema fingerprint覆盖kind、field、value type、default、hint、serialization和provider。 |
| RSR-G06 | plugin identity/prefix/generation不一致fail closed。 |
| RSR-G07 | range、step、precision、enum、default和文本预算均在admission验证。 |
| RSR-G08 | schema generation耗尽返回typed terminal error，不saturate/panic/wrap。 |
| RSR-G09 | catalog commit receipt列出old/new epoch与affected/add/update/remove type。 |
| RSR-G10 | schema-only metadata变更会刷新对应Inspector artifact。 |
| RSR-G11 | schema变更只失效依赖该type的compiled plan和artifact。 |
| RSR-G12 | plugin unload在adapter lease未quiesce时被阻止或defer。 |
| RSR-G13 | property path parse/serialize/display/compile round-trip唯一。 |
| RSR-G14 | dotted/qualified plugin type path与nested field无歧义。 |
| RSR-G15 | forged raw/component/segments DTO无法进入domain API。 |
| RSR-G16 | typed segment覆盖field/index/map key/tuple/optional/variant。 |
| RSR-G17 | compiled handle验证World/object/type/field/schema/provider generation。 |
| RSR-G18 | path/token/handle exhaustion无panic和无限循环。 |
| RSR-G19 | path interner有bytes/items预算、retirement与diagnostics。 |
| RSR-G20 | ReflectedValue decode/write有depth/items/bytes/time预算。 |
| RSR-G21 | numeric width/coercion/loss policy可测且默认不静默缩窄。 |
| RSR-G22 | nested conversion error返回完整property segment位置。 |
| RSR-G23 | typed entity/resource reference区分null/unresolved/stale/foreign。 |
| RSR-G24 | all live dynamic types执行同一shape/value validator。 |
| RSR-G25 | optional/default/alias/deprecated/unknown field策略有versioned migration test。 |
| RSR-G26 | retained payload migration可cancel、报告progress并回滚last-good。 |
| RSR-G27 | dynamic component storage只有一个Runtime60 authority。 |
| RSR-G28 | reflection mutation支持expected revision与CAS conflict。 |
| RSR-G29 | multi-field/multi-object mutation失败保持整批零变化。 |
| RSR-G30 | protected component reflection只能进入Runtime62 domain mutator。 |
| RSR-G31 | mutation receipt绑定world/schema generation和before/after/effects。 |
| RSR-G32 | resource/component capability failure均为typed disposition。 |
| RSR-G33 | builtin component reflection inventory覆盖或逐项解释全部scene component。 |
| RSR-G34 | 每个`zr_reflect(skip)`有owner/reason/替代API和测试。 |
| RSR-G35 | focused field build不扫描无关registry type。 |
| RSR-G36 | adapter error在artifact可见且不会伪装为字段不存在。 |
| RSR-G37 | schema声明字段缺值使publication失败/degraded，不静默filter。 |
| RSR-G38 | 多Inspector/remote/debug consumer不会互相逐出唯一cache slot。 |
| RSR-G39 | field delta使用stable ID且不因display/localization变化改身份。 |
| RSR-G40 | hierarchy、fields、schema由同一generation bundle引用。 |
| RSR-G41 | poisoned cache有diagnostic、quarantine和确定性重建。 |
| RSR-G42 | unsupported Editor value仍显示read-only diagnostic。 |
| RSR-G43 | Editor命令遇schema/world revision变化返回conflict。 |
| RSR-G44 | fact overflow/age overflow均在batch置resync_required并给出原因。 |
| RSR-G45 | full snapshot ack后cursor可恢复且不会重复/遗漏已确认事实。 |
| RSR-G46 | component/schema watcher使用qualified identity并在catalog commit触发。 |
| RSR-G47 | 10K type/100K entity/multi-consumer内存与latency满足versioned budget。 |
| RSR-G48 | Cargo/Editor/fuzz/fault/scale/profile证据绑定source fingerprint和BuildSet后才可宣称完成。 |

## 12. 实施依赖与退出条件

实施依赖顺序为：Runtime24 identity/exhaustion -> Runtime60 component schema/storage -> Runtime63 catalog/address/mutation -> Runtime61 persistence migration -> Runtime62 protected mutator -> Interface02 wire/CAS/budget -> Runtime54 cursor/resync -> Editor05 Inspector product。可以先在Runtime63完成catalog P0和内部typed address，但不能用临时字符串shim长期跨越这些边界。

Runtime63退出条件不是“Inspector能显示dynamic JSON”。必须同时满足：失败注册零变化、schema/provider generation可验证、typed address唯一、reflection write原子且有revision、artifact同代且不吞错、overflow可由batch自描述resync、Editor命令能拒绝stale schema，以及规模/故障证据达到门禁。

## 13. 审查限制与状态

- 本轮是静态E3纵切审查，不是implementation acceptance。
- 未运行Cargo、Miri、sanitizer、fuzz、Editor窗口、real plugin DLL reload、remote transport或benchmark。
- 未验证当前测试在默认/required lane的可达性；测试数量不代表资格。
- 参考源码用于提取工程合同，不表示应复制其具体对象模型、宏系统或Editor UI。
- 任何“性能优于Unreal”结论仍缺同场景、同画质、同硬件、同正确性和统计协议。

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Runtime63物理冻结与五引擎对照 | review_complete | 2026-08-20 | 124个Zircon文件、24,745行、894,944 bytes；37个参考文件、41,133行、1,532,682 bytes |
| P0/P1/P2与48项门禁 | review_complete | 2026-08-20 | 1 P0 / 67 P1 / 17 P2；source fingerprint见第2节 |
| Production重构 | pending | - | 本篇未修改production/tests/Cargo/ABI |
