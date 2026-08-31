---
related_code:
  - zircon_runtime/src/core/runtime/descriptors/service_object.rs
  - zircon_runtime/src/core/runtime/descriptors/service_factory.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/handle/service_identity.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/resource/data.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/framework/state/registry.rs
  - zircon_runtime/src/core/framework/script/call_frame.rs
  - zircon_runtime/src/scene/ecs/component/registry.rs
  - zircon_runtime/src/scene/ecs/component/table_column.rs
  - zircon_runtime/src/scene/ecs/events/store.rs
  - zircon_runtime/src/scene/ecs/messages/store.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/ecs/resource_store/store.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/registration.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registration.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/runtime_provider/registration.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_editor/src/ui/layouts/views/view_projection/projection_composition.rs
  - zircon_editor/src/ui/retained_host/primitives.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/registry.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime_interface/src/reflect/type_path.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
tests:
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/validation_cache.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/registry.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/versioned_json.rs
  - zircon_runtime/src/script/vm/reflection/tests/schema_invariants.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/diagnostics.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/lifecycle.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/21-zr-language-parser-type-system-semir-bytecode-package-loader-vm-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_tooling/21-unsafe-rust-ffi-native-memory-thread-affinity-panic-unload-safety-governance-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
  - docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md
  - docs/plans/optimize/zircon_tooling/33-reference-engine-source-corpus-snapshot-provenance-citation-applicability-comparison-currentness-review.md
  - docs/plans/optimize/zircon_tooling/34-global-state-scope-singleton-service-locator-static-registry-cache-initialization-reset-multi-instance-isolation-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/Templates/Casts.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/Class.h
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_reflect/src/type_path.rs
  - dev/bevy/crates/bevy_ecs/src/component/info.rs
  - dev/Fyrox/fyrox-core/src/reflect/mod.rs
  - dev/godot/core/object/class_db.h
  - dev/godot/core/variant/variant.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 36 · Type Erasure、Dynamic Dispatch、Any/Downcast、Trait Object、Reflection Type Identity、VTable Generation 与性能审查

## 1. 结论

Zircon 的问题不是“用了 `Any` 或 `dyn Trait`”。ECS table column 已把 `TypeId`、layout、write/take/drop callback编译进列描述符，unsafe callback前验证类型，热查询不让 `Any`持有列主体；asset importer registry会拒绝重复ID和同优先级matcher，发布COW generation并能按plugin撤销；Core service handle、plugin bridge和render executor registry也已经有generation或compiled-pipeline validation。类型擦除、虚调用和字符串展示名在这些局部边界是合理工具，不能机械改成enum、泛型单体化或手写函数表。

真正的断点是“擦除前的类型事实”没有稳定地穿过注册、编译、调用、重载、持久化与诊断。Core service先按字符串kind/name解析，再在consumer侧 `Arc::downcast`，descriptor与handle没有预期类型合同；PluginInterface只有字符串 `INTERFACE_ID`，错误类型被压成`NotEnabled`；reflection registry以derive生成的`module_path!()`字符串作为公开身份，没有稳定TypeKey、schema version/hash、alias或migration；Editor projection cache只用`composition_id`和generation的K类型命中，完全未校验metadata的M类型；VM backend registry静默覆盖同名family，把所有resolve错误都当“不是我”；render pass pipeline虽缓存“ID存在”验证，每一pass执行仍按字符串ID做BTreeMap查找再虚调用。

这些缺口会同时伤害正确性、热重载、内容兼容和性能：模块重命名可改变反射身份；相同字符串ID可绑定不同Rust contract；旧函数表没有per-type owner generation时无法证明卸载安全；热路径只验证ID却不把call target绑定进compiled artifact；downcast失败又常被`None`、`false`或通用NotEnabled吞掉。要达到工程级引擎，必须建立双层身份：进程内`LocalTypeId/TypeSlot`用于快速分派，跨进程/持久化/plugin/ABI使用稳定`TypeContractId + SchemaVersion/Hash`；注册阶段完成兼容与owner校验，编译阶段把字符串解析成generation-bound slot/function plan，运行阶段只执行已验证计划。

本篇登记 **0 项 P0、48 项 P1、12 项 P2 和40个验收门**。没有新增P0，因为当前静态证据能证明类型合同缺失、错误分类丢失和可避免的热路径查找，却未独立证明shipping BuildSet已经发生错误DLL调用、不可恢复内容破坏或同条件性能灾难。Tooling21继续拥有raw FFI/unsafe memory与DLL soundness，Tooling27拥有通用版本窗口和migration，Tooling32拥有全仓hot-path cost，Runtime04/05/21分别拥有resource、ECS与Zr VM具体语义，Tooling35拥有owner/lease/cycle。本篇只拥有：

`ErasureInventory -> StableTypeIdentity -> TypeRegistry -> RegistrationValidation -> DowncastBoundary -> DispatchPlan -> Owner/Generation -> Unload/SchemaMigration -> DispatchCost -> TypeErasureQualificationReceipt`

## 2. 审查边界、口径与限制

### 2.1 词法账本只用于发现候选

| Candidate signal | 命中 | production-like保守文件 | 解释 |
|---|---:|---:|---|
| `Any` | 225行 / 85文件 | 62文件 | 包含panic payload、enum variant、测试与合理owner-local slot，不是225个缺陷 |
| `TypeId` | 551行 / 122文件 | 92文件 | process-local identity通常正确；只有越过持久化、DLL或schema边界才必须升级 |
| `downcast` | 140行 / 66文件 | 48文件 | 既包含已由TypeId先验证的invariant guard，也包含错误被吞掉的late check |
| trait-object/shared dyn | 1,147行 / 478文件 | 380文件 | `Box/Arc/Rc/Weak/& dyn`是多态候选，不代表虚调用一定在hot path |
| Any/TypeId/downcast domain | Runtime 473行/98文件；Editor 316/82；Plugins 89/37；Runtime Interface 16/6；App 4/1；Hub 4/1 | - | 只固定重点域，不按命中数排序severity |
| textual `vtable/VTable` | 0 | 0 | Rust trait object仍有编译器vtable；0只说明没有显式命名的统一函数表合同 |
| source evidence currentness | 计算后写入本篇状态记录 | - | plan/index不进入自引用指纹，实施前仍需重取Cargo-resolved BuildSet |

### 2.2 TypeIdentity必须按边界分层

| Identity layer | 合法用途 | 禁止承担的职责 |
|---|---|---|
| `LocalTypeId` | 同进程、同binary generation中的Rust类型匹配 | 持久化、网络、跨DLL、跨build或跨语言身份 |
| `TypeSlot` | registry freeze后紧凑索引、array dispatch、compiled plan | 单独作为存档身份或跨owner capability |
| `TypeContractId` | 稳定类型/接口/服务/RPC身份，显式命名空间 | 用display name、module path或随机插入序替代 |
| `SchemaVersion/Hash` | field/method/payload/layout语义兼容决策 | 只比较字符串ID或全局generation |
| `OwnerGeneration` | plugin/DLL/project/world注册代与retirement | 仅靠Arc存活或registry全局计数证明可调用 |
| `DisplayPath` | UI、日志、调试与人类可读短名 | 作为唯一权威identity、权限或migration key |

### 2.3 Evidence限制

1. 本轮逐项读取关键registration、lookup、downcast、generation、execute与代表性测试；没有运行微基准、plugin reload、反射schema migration或双Build兼容矩阵。
2. 当前工作树有其他会话的Runtime/Editor源码修改，报告记录审查时物理内容与HEAD，不把dirty内容误称为提交基线；所有finding实施前必须source recheck。
3. 已知Editor、Hub、WOC、plugin metadata动态验证阻断未变化，本篇不重复触发；这些阻断不用于证明或否定类型擦除finding。
4. Rust `TypeId`只保证当前程序中的类型身份，不能被本篇推断为稳定序列化ID；反过来，字符串或UUID稳定也不自动证明schema兼容。
5. 本篇不要求消灭所有虚调用。低频控制面可以保留trait object；每帧/每entity/每pass路径必须以同workload profile证明成本，优先在compile/freeze阶段绑定slot。

## 3. 必须保留的工程基础

### 3.1 ECS table column把擦除限制在已验证callback边界

`TableColumnLayout`记录`TypeId`、Layout、type name及write/take/drop函数；调用者先比对component TypeId并满足alignment/capacity不变量，hot query不把`Any`作为列owner。这里的函数表是合理的数据导向多态，应扩展诊断与generation，而不是回退成每元素downcast。

### 3.2 Asset importer registry已有freeze、冲突和owner撤销

importer是`Arc<dyn AssetImporterHandler>`，但注册前规范化extension/suffix、拒绝重复ID和同优先级matcher，COW generation保证reader看到完整索引，还能按plugin ID移除全部slot。它证明trait object registry可以工程化；后续只需补稳定output contract和generation receipt。

### 3.3 Core service handle已有generation-bound admission

service entry和handle校验index/generation/kind，call guard阻止closing generation的新调用并跟踪in-flight。缺口是类型合同仍在最后downcast才发现，不应删除现有admission、dependency stack和generation逻辑。

### 3.4 Plugin bridge已有固定slot和奇偶generation发布

bridge table freeze后按interface ID映射`InterfaceSlot`，entry将provider与generation原子发布，disabled代拒绝pin，WeakBridge也按generation刷新。应在此上增加interface schema和typed mismatch，不应退回每次遍历exports。

### 3.5 Reflection短名歧义与catalog generation已被处理

TypeRegistry拒绝重复完整path、维护唯一短名与ambiguous set，VM plugin type还有plugin-owned、prefix和field value type验证；schema catalog每次变化递增generation。这些是正确基础，但仍缺稳定TypeKey、per-type schema和迁移。

### 3.6 Render executor registry至少有compiled validation cache

registry generation变化会让compiled pipeline重新检查所有executor ID，validation cache避免同代重复全量扫描；executor还声明serial/parallel-safe recording policy。目标是把验证结果升级成绑定call target的DispatchPlan，而不是删除validation generation。

### 3.7 Any可在私有同步slot中作为invariant guard

Core `StateRegistry`按`TypeId`存`Box<dyn Any>`，typed API同一处插入/获取；script call frame的runtime context只在同步borrow scope内downcast。只要不越过持久化、异步、DLL或公开plugin边界，这类owner-local擦除可以保留，并补debug assertion与类型名诊断。

### 3.8 versioned C ABI table是另一套正确多态手段

`ZrRuntimeApiV7`和Host API以abi version/size和冻结function table发布，不依赖Rust trait object跨DLL。它是Tooling21的正向基线；Tooling36只要求table引用的component/schema/interface再携带稳定contract，不重写raw ABI owner。

## 4. 已确认的结构断点

### 4.1 Core service在consumer端才发现类型不一致

`ServiceObject`只是`Arc<dyn Any + Send + Sync>`；`RegistryName`固定module/kind/service字符串，`ServiceEntry`记录index/generation/factory/instance，却没有expected TypeContract。`resolve_driver<T>`等先完成字符串解析和实例化，最后`Arc::downcast::<T>`，失败只返回`ServiceDowncast(name)`。错误registration可能一直到首个特定consumer才暴露，错误也没有expected/actual stable type或provider owner。

### 4.2 Resource与ECS同时存在local和durable identity但未统一分层

ResourceManager按`ResourceKind`后downcast payload，Runtime04已经拥有该专项缺口；ECS Rust component用TypeId是合理local identity，dynamic plugin component却用字符串。当前没有共同TypeContractId将reflection、resource、dynamic component、serialization与plugin generation连接起来，导致同一逻辑类型可能有多套互不证明兼容的key。

### 4.3 MessageStore的擦除字段通过特殊Clone/Eq被整体忽略

MessageStore按TypeId存`Box<dyn Any>`和advance function，active channel优化是正向基础；但`Clone`返回全新default，`PartialEq`无条件true，World clone又直接调用它。runtime-only message投影可能是有意设计，却没有声明`WorldProjectionPolicy`、discard reason或测试receipt，擦除字段因此可以在clone/equality中静默消失。

### 4.4 Reflection公开身份随Rust模块路径漂移

derive宏用`concat!(module_path!(), "::", Type)`生成full type path，`ReflectTypePath::new`只验证trim后非空。crate/module重命名、re-export或hard cutover可改变存档、RPC和editor引用的类型身份。registration没有stable UUID/key、schema version/hash、alias、rename或migration table。

### 4.5 Reflection注册只对VM子集执行严格owner校验

VM dynamic type会验证plugin-owned、plugin ID一致、prefix与declared field type；通用`register`主要检查component/resource flags、field name和value shape。普通plugin/native registration缺同等级namespace、owner generation和contract collision准入，短名ambiguity只解决lookup，不解决schema冲突。

### 4.6 Reflection函数表没有per-type retirement合同

`ReflectComponent`/`ReflectResource`携带Rust函数指针，静态内建类型完全合理；但registration自身没有owner generation、retired state或operation lease。未来只要可卸载native provider贡献同类function table，全局catalog generation不足以证明某个旧call target仍可执行。

### 4.7 PluginInterface字符串ID不能证明Rust trait contract相同

`PluginInterface`只声明`INTERFACE_ID`；export把`Arc<T>`再次擦除到`Arc<dyn Any>`。同ID若绑定不同Rust类型，provider downcast失败被映射为`BridgeError::NotEnabled`，与正常disabled/reload窗口不可区分。freeze阶段也没有method signature hash、major/minor或collision source诊断。

### 4.8 VM backend registry静默覆盖并吞掉operational error

`register_family`直接`BTreeMap::insert`并只返回name，没有duplicate error、owner token、generation或unregister。`resolve`无明确prefix时遍历所有family，只保留第一个Ok，把包括编译/配置/IO失败在内的Err都当“not mine”，最终返回UnknownBackend；`contains`甚至调用resolve并可能构造backend或触发副作用。

### 4.9 Render pass执行仍在每pass做字符串树查找

compiled pipeline validation只证明同代registry里存在executor ID；`execute`仍以`context.executor_id`查BTreeMap再虚调用，`supports_parallel_recording`也重复查找。pipeline没有保存`ExecutorSlot + registry generation + Arc<dyn Executor>`或专用fn pointer，编译收益没有穿过执行边界。

### 4.10 Render executor替换语义未成为显式construction transaction

`register_executor`静默替换并返回previous，批量`register_explicit_executors`忽略返回值。即使上层extension registry拥有plugin owner，局部registry仍允许同ID不同recording policy/call target按插入顺序择胜。重复应在freeze前变成typed collision或显式override policy，并进入compiled artifact fingerprint。

### 4.11 Editor composition cache没有把metadata类型纳入key

thread-local cache以字符串`composition_id`索引，命中只downcast/比较generation的K并检查row sharing；泛型metadata M不参与校验。相同ID与K若由不同M调用，会直接返回旧Model，随后`ModelRc::metadata::<M>()`只得到None。None又无法区分“metadata不存在”和“缓存绑定了错误类型”。

### 4.12 RPC descriptor、schema validator和handler可被无声替换

RPC payload schema已有`schema_id`与reflection request，这是基础；但manager的三个HashMap都按字符串直接insert，register函数始终Ok或无返回，descriptor与handler不是原子typed registration，也没有owner generation。相同RPC ID、schema字符串或validator可在运行中被另一个provider无声改写。

### 4.13 Runtime provider/collector registration只有字符串ID和trait object

graphics provider registration记录debug name、provider ID、priority和Arc trait object；runtime prepare collector记录collector ID和call target。具体上层registry可能另有owner/排序规则，但registration本身不携输入输出contract、capability schema、owner generation或compiled slot，无法单独成为可验证插件边界。

### 4.14 Downcast失败在多个domain被压成缺失或布尔值

service返回name-only error，bridge返回NotEnabled，Editor metadata返回None，observer/event callback常返回false，job coalescing也以false表示类型不匹配。错误类型、stale generation、缺值和正常不接受被合并，导致诊断无法判断registration corruption、caller bug或合法分支。

## 5. 目标架构

### 5.1 TypeContractId与LocalTypeSlot双层身份

每个可持久化、跨plugin、跨DLL、RPC或公开reflection类型必须声明`TypeContractId(namespace,name,major)`、`SchemaVersion`、`SchemaHash`、aliases与owner。registry freeze分配紧凑`TypeSlot`并可缓存local `TypeId`；运行时数组索引走slot，存档/网络/manifest写stable contract，二者由同代registry snapshot映射。

### 5.2 TypedRegistration与RegistrationValidation

统一registration envelope至少包含contract、display path、local type witness、capabilities、function table/method schema、owner generation、override policy、retirement hook和source provenance。重复stable ID、local TypeId冲突、schema不兼容、错误namespace或owner必须在publish前失败，不允许在首次consumer downcast时才发现。

### 5.3 DowncastBoundary只作为最后不变量检查

公开lookup先比较stable contract、slot generation和capability；内部downcast保留为debug/release-safe invariant guard。失败返回`TypeMismatch { contract, expected_local, actual_local, owner, generation, callsite }`，不得折叠成NotFound、None或false。release构建仍必须fail closed，不把panic作为主要合同。

### 5.4 CompiledDispatchPlan

字符串path/ID只允许在authoring、load、registration或compile阶段解析。计划中保存generation-bound slot、call target、recording/execution policy、schema hash和owner lease；执行路径按array/compact vector访问并直接调用。registry变更先产生新snapshot，旧plan按声明完成或失效，不在每pass重新做树查找。

### 5.5 Unload与SchemaMigration

owner revoke顺序为close admission、等待in-flight、retire registrations、失效compiled plans、迁移/拒绝durable values、释放function table、最后卸载DLL。type rename通过alias/migration graph解析，schema compatibility输出typed decision，不用全局catalog generation替代per-type迁移。

### 5.6 TypeErasureQualificationReceipt

receipt绑定BuildSet、registry snapshot、contract catalog hash、owner generations、dispatch plan hash、scenario和性能样本；报告registration collision、downcast failure、stale slot、dynamic dispatch count、lookup count及migration结果。只有同代静态和动态证据都通过，才能声明type-safe、reload-safe或hot-path-qualified。

## 6. P1 重构项

### TE-P1-001 · 建立ErasureInventory单一真源

从Cargo-resolved AST与ABI schema生成Any/trait object/function table/variant/opaque handle inventory，标注owner、边界、频率、identity与downcast policy。

### TE-P1-002 · 定义TypeIdentity taxonomy

区分LocalTypeId、TypeSlot、TypeContractId、SchemaId、InterfaceId与DisplayPath，禁止一类ID越权承担另一层职责。

### TE-P1-003 · 建立StableTypeContract catalog

为跨持久化/plugin/DLL/RPC/reflection类型分配命名空间稳定ID、major、schema hash、aliases和canonical owner。

### TE-P1-004 · 绑定BuildSet与registry snapshot

TypeCatalog必须记录source revision、feature/profile、target ABI和生成器版本；不能从mutable process registry推断build身份。

### TE-P1-005 · 定义TypedRegistration envelope

统一携带contract、local witness、capabilities、function table、owner generation、override policy和provenance。

### TE-P1-006 · 将冲突检测前移到publish

重复stable ID、same-ID/different schema、same-local/different contract和namespace违规必须事务失败，不允许last insert wins。

### TE-P1-007 · 建立per-type generation与retirement

除全局catalog generation外，每个slot记录registration generation、active/retiring/retired和in-flight count。

### TE-P1-008 · 定义TypeErasureQualificationReceipt

把catalog、owner、compiled plan、collision/mismatch census和性能证据绑定到同一产品场景。

### TE-P1-009 · 给Core service descriptor加入ServiceContractId

factory注册时声明产出contract与local witness，dependencies可要求capability/schema范围，而不只要求RegistryName。

### TE-P1-010 · 在service registration时验证factory产出

startup/preflight实例化或typed factory construction必须证明对象匹配descriptor，consumer downcast只作不变量检查。

### TE-P1-011 · 让ServiceHandle携带expected contract

handle identity包含ServiceContractId/major与registration generation；错误类型在resolve handle时fail closed。

### TE-P1-012 · 完整化service mismatch诊断

`ServiceDowncast(name)`升级为expected/actual contract、Rust type name、provider owner、generation和resolution path。

### TE-P1-013 · 收敛ResourceKind与payload TypeContract

由Runtime04实现kind到stable payload contract映射，lease/get不只验证宽泛kind；旧API硬切而非长期双轨。

### TE-P1-014 · 分离ECS local component与durable component身份

Rust TypeId只映射local slot；dynamic/persisted component使用stable contract、schema版本、owner代和明确storage layout。

### TE-P1-015 · 给dynamic component registration增加schema hash

同字符串type ID但字段/layout/behavior不同必须拒绝或迁移，不能按首次插入或字符串相等视为同类型。

### TE-P1-016 · 声明World type-erased projection policy

MessageStore/Event/Observer/Resource等字段的Clone/Eq/serialize行为进入显式projection manifest，discard必须有reason与receipt。

### TE-P1-017 · 保留并标准化ECS erased column function table

将现有TypeId/layout/callback正例抽成conformance，补owner generation、debug name和stale layout拒绝。

### TE-P1-018 · 让event/message mismatch成为typed invariant failure

通道TypeId与payload不一致不得静默false/None；内部可debug assert，产品构建仍记录corruption并隔离通道。

### TE-P1-019 · 给reflection建立稳定TypeKey

公开reflection registration不再把module path当唯一身份；module/type path成为display/alias，stable key用于引用与migration。

### TE-P1-020 · 给reflection schema增加version与hash

field name/type/cardinality/default/flags、component/resource adapters和method capability共同进入canonical schema hash。

### TE-P1-021 · 建立type alias、rename与migration graph

支持crate/module/type/field rename和upgrade/downgrade decision，禁止仅靠旧字符串永久兼容分支。

### TE-P1-022 · 强化ReflectTypePath规范化

验证namespace、separator、segment、reserved prefix和case policy；trim非空不足以成为canonical identity。

### TE-P1-023 · 统一所有plugin reflection registration准入

把VM子集已有plugin owner/prefix/field检查推广到native/script/general plugin贡献，并绑定PluginModuleId generation。

### TE-P1-024 · 为reflection function table加owner lease

所有可卸载provider的construct/clone/get/set/remove函数在调用期间固定正确generation，retirement等待在途归零。

### TE-P1-025 · 将field lookup编译为ReflectAccessPlan

authoring path解析成TypeSlot/FieldSlot/schema generation；高频inspector/serialization不重复分割和树查字符串。

### TE-P1-026 · 让ReflectedValue携带声明type contract

Json/list/map等通用载体必须与expected schema绑定，不能只凭runtime variant shape宣称已验证业务类型。

### TE-P1-027 · 定义InterfaceContract

PluginInterface增加stable ID、major/minor、method schema/hash、capabilities和owner generation；字符串常量只作canonical textual form。

### TE-P1-028 · 在bridge freeze时验证interface collision

same ID/different schema、same owner duplicate、override和optional version negotiation必须输出typed decision。

### TE-P1-029 · 分离Bridge absent、disabled、stale与type mismatch

错误枚举保留expected/actual contract和slot generation，不能把downcast corruption映射为NotEnabled。

### TE-P1-030 · 让WeakBridge绑定contract与schema generation

cache命中除provider generation外还验证interface contract/schema；reload后不同major不能复用旧typed guard。

### TE-P1-031 · 重构VM backend registration

`register_family`返回typed token/result，重复name/selector拒绝，registration携owner、generation、selectors和capability。

### TE-P1-032 · 为VM backend增加revoke与quiescence

plugin unload先关闭selector admission、等待backend operation，再删除family；旧Arc不能跨generation无界调用。

### TE-P1-033 · 区分Backend probe与operational failure

family返回`NotMine/Resolved/Failed`，编译、IO、配置错误不得被遍历吞成UnknownBackend。

### TE-P1-034 · 冻结VM selector map

registration完成后构建selector到family slot的immutable snapshot，显式prefix O(1)/O(log n)解析，不每次clone遍历全部family。

### TE-P1-035 · 让contains成为无副作用metadata查询

presence只查询snapshot capability，不构造backend、不做IO、不改变cache或吞掉故障。

### TE-P1-036 · 将render executor绑定进CompiledRenderPipeline

compile产物保存ExecutorSlot/call target/recording policy/registry generation，执行不再按字符串BTreeMap查找。

### TE-P1-037 · 显式化render executor override policy

builtin、feature、plugin contribution的冲突在construction transaction中决定；批量注册不得忽略previous。

### TE-P1-038 · 给render executor加入contract hash

executor ID还需声明pass-data/resource/context schema与recording policy hash，compiled pipeline验证完整contract。

### TE-P1-039 · 绑定executor owner generation与in-flight lease

plugin executor retirement先失效新pipeline并等待recording/execution完成，旧compiled plan不得调用卸载provider。

### TE-P1-040 · 重构Editor composition typed key

cache key至少包含composition stable ID、`TypeId<K>`、`TypeId<M>`、window/project/session owner和generation contract。

### TE-P1-041 · 给Model metadata提供typed wrapper

`ModelMetadata<M>`在construction保持M身份，读取区分Absent与TypeMismatch，并记录producer composition。

### TE-P1-042 · 清除UI metadata mismatch的None语义

错误类型必须使cache miss重建或返回typed failure，不能把旧M当作“无metadata”继续渲染。

### TE-P1-043 · 将job coalescing key与payload type绑定

由Editor09实现`JobKey<T>`/contract，same textual key不同task type不能以false静默绕过coalescing策略。

### TE-P1-044 · 将script runtime context变成typed capability set

由Runtime21把`&dyn Any` context升级为声明capability/contract的borrowed call context，保持同步scope且禁止异步逃逸。

### TE-P1-045 · 事务化RPC registration

descriptor、payload schema、validator和handler按RpcContractId一次发布，重复/override/缺件返回typed error。

### TE-P1-046 · 绑定RPC与graphics provider owner/schema

RPC handler、validator、runtime provider和collector都携PluginModuleId generation、input/output contract与revoke token。

### TE-P1-047 · 建立dispatch cost基线

统计每frame字符串lookup、BTreeMap/HashMap lookup、downcast、virtual call、slot call与plan invalidation；与Tooling32同workload联动。

### TE-P1-048 · 以QualificationReceipt作为产品准入

所有shipping dynamic registry必须证明collision fail-close、stale reject、unload quiescence、schema migration与热路径预算。

## 7. P2 完善项

### TE-P2-001 · 增加erased slot调试名称

slot显示stable contract、local type、owner和generation；display name不得反向成为权威key。

### TE-P2-002 · 建立TypeCatalog可视化

展示contract、schema、alias、capability、provider与consumer图，支持从finding跳到source。

### TE-P2-003 · 增加downcast候选lint

标记public/plugin/async/persisted/hot-path downcast，owner-local invariant guard可用结构化waiver保留。

### TE-P2-004 · 增加trait object hot-path采样

按callsite聚合virtual call count和CPU时间，不按trait object数量判性能债务。

### TE-P2-005 · 提供interned display path

对高频日志/lookup避免重复String分配，但intern pool必须有scope、owner和budget，归Tooling25共同验收。

### TE-P2-006 · 增加schema collision fuzz

随机ID、alias、版本、field和owner顺序验证确定性、冲突拒绝与transaction rollback。

### TE-P2-007 · 增加dispatch plan property test

随机register/revoke/reload/slot reuse确保旧plan永不调用新owner或retired function table。

### TE-P2-008 · 发布类型擦除工程手册

说明何时用enum、generic、trait object、Any、function table、stable ID与slot，并绑定Tooling28 currentness。

### TE-P2-009 · 增加registry snapshot导出

开发构建可导出脱敏catalog、generation、schema hash和collision history，禁止序列化raw TypeId或pointer。

### TE-P2-010 · 建立reference currentness复核

通过Tooling33跟踪Unreal cast/class、Bevy reflect/ECS、Fyrox UUID、Godot ClassDB/Variant与Unity RenderGraph漂移。

### TE-P2-011 · 细分mismatch指标基数

按contract/owner/callsite聚合并限流，避免把高基数type path或用户payload直接写入telemetry。

### TE-P2-012 · 建立type-erasure debt趋势

统计unknown boundary、string-only contract、late downcast、silent replacement和hot lookup；数量不是质量KPI，只看关闭趋势与receipt。

## 8. 参考引擎差异与适用性

### 8.1 Unreal

Unreal `Cast`通过对象的`UClass`、继承关系和`EClassCastFlags`判断，`UClass/UStruct`同时保存size、alignment、constructor、property link与热重载相关函数。适用结论是让类型metadata和call target成为同一注册合同，并在重载时显式更新/失效；Zircon不需要复制UObject、全局GC或宏生成体系，也不能用Unreal规模替代自身benchmark。

### 8.2 Bevy

Bevy TypeRegistry同时维护TypeId registration、full type path索引、短名歧义和可扩展TypeData；ECS component metadata把storage/layout与类型注册绑定。它证明process-local TypeId与人类path可以并存，但Bevy的TypeId同样不是跨build持久ID。Zircon应吸收typed registration、dependency registration和ambiguity处理，再为自身plugin/serialization增加stable contract。

### 8.3 Fyrox

Fyrox `TypeInfo`同时包含type name、derived TypeId和显式`type_uuid`，文档明确UUID用于trait object serialization中关联实际类型与序列化表示；downcast仍用local TypeId。这个“稳定UUID + local TypeId”双层模型直接适用于Zircon reflection，但Zircon还需schema version/hash、owner generation和migration，不能只增加UUID字段就结束。

### 8.4 Godot

Godot ClassDB的ClassInfo包含inheritance、method map、property metadata、creation与extension信息，并计算API hash；Variant使用显式Type枚举、conversion表与结构化CallError。Zircon可吸收“动态调用前有完整metadata、调用错误不折叠、API可hash”的原则，不应引入新的全局ClassDB service locator或把全部Rust类型压成单一Variant。

### 8.5 Unity Graphics

Unity RenderGraph以`AddRasterRenderPass<PassData>`/`AddComputePass<PassData>`创建typed pass data，pass保存typed render delegate，NativePassCompiler再按compiled pass id调度、合并render pass并处理resource lifetime。Zircon应让compiled pipeline绑定executor与policy，消除每pass字符串registry lookup；不需要照搬C# delegate/object pool，也不能据此假设绑定后一定快于Unreal，仍需同场景profile。

## 9. 实施顺序

### M0 · Inventory与identity冻结

- 用Cargo-resolved AST重取ErasureInventory，给每个boundary分类；
- 冻结新增string-only public contract、silent replacement和hot-path late downcast；
- 建立TypeContractId/SchemaId/OwnerGeneration基础schema。

### M1 · Registration与diagnostic基础

- 实现TypedRegistration、collision validation、per-type generation和receipt；
- 保留ECS table column、asset importer、service admission和bridge slot正例；
- 统一TypeMismatch错误，不再压成None/false/NotEnabled。

### M2 · Service、reflection与bridge硬切

- service descriptor/handle接ServiceContractId；
- reflection接stable TypeKey/version/hash/alias/migration；
- bridge接InterfaceContract并在freeze验证collision。

### M3 · Provider与Editor cache硬切

- 重构VM backend、RPC、graphics provider/collector owner与revoke；
- composition cache使用K/M typed key；
- message/world projection policy与job/script context回到domain owner实施。

### M4 · Compiled dispatch与热路径

- render pipeline绑定ExecutorSlot/call target/policy；
- reflection field path和VM selector编译为slot plan；
- 以Tooling32统一workload比较lookup/downcast/virtual/slot成本。

### M5 · Reload、migration与动态验证

- 覆盖plugin/DLL reload、schema rename/version、stale plan、collision和wrong-type injection；
- 验证close-admission、in-flight、retire、unload顺序；
- 生成BuildSet-bound TypeErasureQualificationReceipt。

### M6 · Required gate与文档

- registration/schema/dispatch/unload/performance gate进入required CI；
- waiver含owner、reason、expiry、source fingerprint与替代证据；
- G01-G40完成后才能升级implementation状态。

## 10. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | Cargo-resolved shipping/editor/tool BuildSet中的Any、trait object、function table、variant与opaque handle进入inventory |
| G02 | 每个erasure boundary声明local/stable identity、owner、generation、schema、frequency与downcast policy |
| G03 | public/plugin/persisted/RPC/DLL类型不再只依赖TypeId、module path或display string |
| G04 | TypeContractId命名空间唯一，重复/冲突registration在publish前fail closed |
| G05 | TypeSlot只在同registry snapshot有效，跨generation使用返回typed stale |
| G06 | schema hash覆盖字段/方法/载体语义，canonical生成跨进程确定 |
| G07 | alias/rename/migration支持升级与拒绝，旧path不成为永久隐式fallback |
| G08 | TypedRegistration绑定BuildSet、source、owner generation与override policy |
| G09 | downcast只作已验证slot后的invariant guard，不作为公开类型发现机制 |
| G10 | mismatch错误区分absent、disabled、stale、schema incompatible和wrong local type |
| G11 | Core service factory产出在registration/preflight验证，错误类型不等首个consumer发现 |
| G12 | ServiceHandle携contract与generation，resolution错误含expected/actual/owner/path |
| G13 | ResourceKind映射stable payload contract，错误payload不能只靠late downcast拒绝 |
| G14 | ECS Rust component使用local slot，dynamic/persisted component有stable contract/schema/owner |
| G15 | ECS erased column callback只在TypeId/layout/generation校验后调用 |
| G16 | World clone/equality/serialize对message/event/observer/resource擦除字段有显式projection policy |
| G17 | MessageStore丢弃runtime queues产生可验证reason，不以always-true Eq掩盖语义 |
| G18 | reflection type identity不因普通module/crate重命名无迁移地改变 |
| G19 | reflection registration有stable key、schema version/hash、alias和migration |
| G20 | 所有plugin reflection贡献执行namespace、owner、prefix、collision与generation验证 |
| G21 | reflection function table调用期间固定owner generation，retirement等待in-flight归零 |
| G22 | inspector/serialization高频field path编译为generation-bound access plan |
| G23 | ReflectedValue与声明schema绑定，shape相同不冒充同业务类型 |
| G24 | PluginInterface contract包含method schema/version，same ID/different trait在freeze失败 |
| G25 | bridge错误不把type mismatch压成NotEnabled，诊断能定位provider generation |
| G26 | WeakBridge cache同时验证provider和interface schema generation |
| G27 | VM backend duplicate family/selector返回typed error且有owner-scoped revoke |
| G28 | backend probe区分NotMine与Failed，operational error不被UnknownBackend吞掉 |
| G29 | `contains`不构造backend、不做IO且不改变状态 |
| G30 | VM selector解析使用frozen map，不在每次resolve clone遍历全部family |
| G31 | compiled render pipeline保存executor slot/call target/policy/registry generation |
| G32 | 每pass执行不再按字符串做BTreeMap lookup，registry变化显式失效旧plan |
| G33 | executor duplicate/override在construction transaction确定且进入plan fingerprint |
| G34 | plugin executor/provider revoke阻止新调用、等待在途并失效compiled plan后才卸载 |
| G35 | Editor composition key包含K/M类型与owner scope，同ID不同M不能复用旧model |
| G36 | Model metadata读取区分Absent与TypeMismatch，错误cache绑定可恢复并可诊断 |
| G37 | RPC descriptor/schema/validator/handler同事务同owner发布，重复不静默覆盖 |
| G38 | fault injection覆盖collision、wrong type、stale slot、schema rename、reload与old plan invocation |
| G39 | 同BuildSet/workload测量字符串lookup、map lookup、downcast、virtual与slot dispatch，性能声明绑定统计置信度 |
| G40 | `git diff --check`、frontmatter路径、finding ID/severity、source fingerprint、索引/coverage/总账计数通过 |

## 11. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Any/TypeId/downcast/trait-object lexical inventory | review_complete | 2026-08-16 | Any 225/85；TypeId 551/122；downcast 140/66；trait-object/shared dyn 1,147/478，均只作candidate |
| representative registry/dispatch review | review_complete | 2026-08-16 | service/resource/ECS/reflection/bridge/VM/render/importer/RPC/Editor composition与ABI table |
| source/reference evidence fingerprint | review_complete | 2026-08-16 | HEAD `25e09a23...d404`；60个source/test/reference输入、1,071,391 bytes；SHA-256 `34f97bde267e476c88737a1278e104b93e7682bfe8e2998f02ad2062b03860bf` |
| five-engine comparison | review_complete | 2026-08-16 | Unreal UClass/Cast、Bevy Reflect/ECS、Fyrox Type UUID、Godot ClassDB/Variant、Unity RenderGraph |
| TypeContract/Registration/DispatchPlan/Receipt architecture | design_complete | 2026-08-16 | 本篇第5节；未实现schema、registry、migration、slot plan或receipt |
| production refactor与动态reload/performance tests | pending | - | 本篇只review，不修改production/tests/Cargo/workflow |

当前结论仍是`review_complete / implementation_pending`。在M0-M6和G01-G40完成前，Zircon不能把“Rust编译通过”“TypeId相等”“字符串ID存在”“downcast通常成功”“registry有generation”或“用了trait object所以可扩展”当成稳定类型身份、schema兼容、plugin可卸载或热路径已优化的工程证明。
