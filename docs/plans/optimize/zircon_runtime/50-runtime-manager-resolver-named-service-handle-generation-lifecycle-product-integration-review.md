---
title: Runtime Manager Resolver、Named Service、Handle、Generation、Lifecycle 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime50
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/handle/service_identity.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/scene/module
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_editor/src/ui/retained_host/app/asset_runtime_access.rs
  - zircon_editor/src/ui/retained_host/viewport/render_framework_access.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/module.rs
tests:
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
  - zircon_runtime/src/foundation/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02/2026-08-15-runtime-module-service-lifecycle-current-architecture-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/Subsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/SubsystemCollection.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Subsystems/SubsystemCollection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/EngineSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/WorldSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Subsystems/LocalPlayerSubsystem.h
  - dev/bevy/crates/bevy_ecs/src/system/system_param.rs
  - dev/bevy/crates/bevy_app/src/sub_app.rs
  - dev/godot/core/config/engine.h
  - dev/godot/core/config/engine.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ContextContainer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 50 · Runtime Manager Resolver、Named Service、Handle、Generation、Lifecycle 与 Product Integration 工程化差距

## 1. 结论

`zircon_runtime::core::manager` 不是空壳。当前五个核心文件提供弱引用 `ManagerResolver`、canonical manager name、typed `ManagerServiceHandle<T>`、注册包装 `RegisteredManagerService<T>` 和按 generation 复核的解析路径；Foundation 测试也证明旧 manager handle 在模块卸载并重新激活后会被拒绝。37个 focused 文件反查出该API已进入Runtime、Editor、App和首方插件，`ManagerServiceHandle`有64处/21文件引用，`resolve_manager_service`有80处/34文件引用，不能把问题降格为未使用helper。

但它还不是工程级 subsystem/service kernel。当前同时存在“按具体类型注册裸manager”和“把trait object包装成另一项service”两套模型；AI、Network、Physics、Sound、Animation和Scene等模块会为同一能力注册具体实现与接口包装两个节点。`ManagerDescriptor`只声明字符串名称、启动策略、依赖与factory，不声明contract type、scope、provider、ABI、线程亲和、访问策略或卸载策略；`RegisteredManagerService<T>`只是`Arc<T>`包装，类型错误要到`Any` downcast时才暴露。

更关键的是，底层 `ServiceHandle<T>::enter()` 已能在调用期间增加in-flight计数并在guard析构时归还，但production manager路径没有使用它。`ManagerServiceHandle<T>::resolve()`校验identity/generation后克隆内部裸`Arc<T>`，随即丢弃admission guard；模块关闭 admission 后，已解析的manager仍能继续调用，卸载等待也看不到这些调用。对原生插件trait object，这会把活跃vtable调用与DLL卸载解耦。该硬阻塞继续由Runtime01、Interface01、Plugins01和App01拥有，本篇不重复登记P0。

所谓索引句柄也没有形成O(1)查找。`ManagerServiceHandle`携带公开的`index/generation/service`，解析时却仍以完整`RegistryName`进入全局`Mutex<HashMap<...>>`，`index`只在取出entry后用于校验。graphics resource streamer有20处重复`asset_manager()`解析，dynamic session在frame/event/host request路径反复解析Input，render loop和Editor viewport也在使用点持续hash、锁、clone和downcast。该性能主问题已由`PERF-MVP-628`拥有，本篇只补齐manager contract、guard接线、scope、错误与产品资格差距。

因此Runtime50登记 **0项新增P0、56项P1和14项P2**。目标不是复制Unreal UObject或Godot singleton，而是建立编译后的typed manager directory、显式scope/owner/provider、不可伪造的generation handle、调用期lease、常数时间slot访问、结构化availability与产品级reload/unload证据。最终性能是否优于Unreal必须由同场景、同画质、同硬件、同失败条件的统计基准证明，不能由接口数量或静态审查推断。

本轮只做静态review，没有修改production、tests、Cargo或reference source；没有运行Cargo、动态库卸载、Editor、WGPU、soak或benchmark。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 结论 |
|---|---:|---|
| `core/manager` | 5 / 435 / 14,235 / 3 | 全部逐文件读取；目录clean |
| descriptor、registry、handle与state | 9个focused文件 | 读取注册identity、generation、admission、in-flight、lazy factory、解析和错误合同 |
| Runtime产品caller | 15个focused文件 | 读取Foundation/Asset/Input/Scene/Graphics/Platform/Animation/Navigation与dynamic session/runtime loop |
| Editor与首方插件caller | 8个focused文件 | 读取retained host、viewport及AI/Net/Physics/Sound模块注册 |
| focused fingerprint集合 | 37 / 7,351 / 269,212 / 29 | SHA-256 `7edf502c02b69e06a8761840cc7bb6f5b9400f91271052a732ac1ae35cac7849` |

focused fingerprint按相对路径小写排序，将每项编码为 `path + NUL + per-file SHA-256`，以LF连接后再次计算SHA-256。它只标识本次读取集合，不是artifact、ABI或release identity。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`。

冻结时`core/runtime/error.rs`、`handle/activation.rs`、`handle/resolution.rs`与`dynamic_api/session/state.rs`已有其他会话/用户改动；本报告按当前working tree内容读取并纳入fingerprint，不覆盖、暂存或回退这些改动。因生命周期关键文件不是clean baseline，`source_recheck_required`保持true。

### 2.2 当前双轨模型

```text
模块描述符
  +-> raw concrete manager: Arc<DefaultXManager>
  |     -> CoreHandle::resolve_manager::<DefaultXManager>()
  |
  +-> interface wrapper: RegisteredManagerService<dyn XManager>
        -> ManagerServiceHandle<dyn XManager>
        -> resolve_registered_manager()
        -> clone inner Arc<dyn XManager>

handle内容：index + generation + RegistryName
实际查找：RegistryName -> global Mutex<HashMap> -> entry -> 再校验index/generation
```

AI、Network、Physics、Sound、Animation和Scene等模块用第二个service包装第一个具体manager，形成重复factory、依赖边和生命周期节点。EditorManager、VM等仍直接使用raw concrete model。两套模型没有统一的contract key、scope和调用lease，因此不能通过继续扩充宏列表收敛。

### 2.3 当前可保留底座

- `ManagerResolver`只持有`CoreWeak`，避免resolver反向延长Core生命周期；
- handle在捕获时记录service identity，在每次解析时复核index/generation，可拒绝明显stale handle；
- module deactivation已经具备close admission、等待registered in-flight、cleanup与generation invalidation的基本顺序；
- `ServiceHandle<T>`/`ServiceCallGuard<T>`已经实现调用期计数，可作为manager path收敛输入；
- `RegistryName`在构造时解析namespace/name/kind，避免调用方到处手写字符串切分；
- typed accessor让产品调用方不需要直接持有Core或任意查询registry。

这些能力证明重构可以硬切到统一service kernel，不需要保留双轨兼容层；但当前production manager调用尚未消费最关键的guard。

## 3. 关键代码事实

### 3.1 generation只保护解析瞬间

`manager_service_handle<T>`先读取注册identity；`resolve_manager_service`随后按名称找到entry、复核index/generation，再把`RegisteredManagerService<T>`中的`Arc<T>`clone给调用方。完成解析后，handle、identity与admission状态不再参与调用。调用方可长期保存该`Arc<T>`，模块deactivation的in-flight计数不会增加，generation失效也无法撤回已克隆对象。

相邻的`ServiceHandle<T>::enter()`则会先升级Core、验证identity、进入service admission并返回`ServiceCallGuard<T>`，guard drop时减少in-flight。反查production后，原始`ServiceHandle<T>`与`ServiceCallGuard<T>`没有产品caller。这不是“底层没有能力”，而是manager facade绕开了正确能力。

### 3.2 descriptor不描述manager contract

`ManagerDescriptor`只有name、startup、dependencies和factory。名字可以定位entry，却不能回答：预期接口类型是什么、作用域属于engine/world/session/project/player/view/device中的哪一级、由哪个module/artifact/ABI提供、是否允许替换、在哪个线程/阶段调用、是否可跨reload、谁有访问权、关闭时怎样drain。service object最终存为`Any`，contract错误直到downcast才被发现。

`RegisteredManagerService<T>`又只是一个`Arc<T>`字段，没有把上述信息补回。结果是descriptor graph描述“启动顺序”，却不等于可验证的service contract graph。

### 3.3 名为index的字段没有索引能力

service index来自process-global `AtomicU32`，但registry物理结构仍是`Mutex<HashMap<RegistryName, ServiceEntry>>`。每次resolve都要hash完整名称、获取全局锁、定位entry、校验identity、downcast并clone Arc。`ManagerServiceHandle.index`并未直接定位dense slot，且三个字段为public，外部代码可以构造互相矛盾的identity组合；最终虽然大多会被校验拒绝，但这不是不可伪造handle。

`RegistryName::from_parts`还会对无效组合panic；builtin canonical name使用Pascal形式，插件使用`ai.runtime.*`等小写namespace。稳定ID、显示名、日志名与持久schema名没有分域。

### 3.4 产品热路径已经依赖resolver

- graphics `ResourceStreamer`有20处`self.asset_manager()`调用，每次都经`ProjectAssetManagerAccess::resolve()`；
- dynamic runtime loop在frame submit/present以及render binding生命周期中解析RenderFramework；
- dynamic session保存Input handle，在frame、event和host request路径按使用点解析；
- Editor viewport保存RenderFramework handle并在任务/绘制路径解析；
- Runtime、Editor和AI/Net/Physics/Sound等模块总计形成64处handle引用和80处resolve引用。

按使用点解析有利于发现stale generation，应保留该语义；实现应变成slot generation check + call lease，而不是要求调用方缓存裸Arc。当前路径的hash、global lock、downcast与Arc clone也没有延迟分布、contention、retry或scale benchmark。

### 3.5 失败与shutdown没有产品闭环

resolver错误会在Graphics/RenderFramework等调用方被格式化为字符串，丢失service identity、scope、provider、状态、retry与remediation。诊断收集器常用`and_then`做best-effort解析，manager缺失时相关状态静默消失。dynamic session shutdown使用`if let Ok`调用manager并继续推进watcher/module状态，无法区分“已完成”“manager不可达”“调用失败但仍关闭”。

这种容错可用于析构期尽力清理，但产品控制面必须同时留下结构化terminal receipt，否则成功状态会掩盖未drain资源或未执行backend shutdown。

## 4. 与参考引擎的可迁移差异

| 参考 | 已核对能力 | Zircon应吸收的合同 | 不照搬项 |
|---|---|---|---|
| Unreal | Subsystem显式绑定Engine/Editor/GameInstance/World/LocalPlayer lifetime；collection负责Initialize/Deinitialize、dependency、dynamic module populate/depopulate与typed single/array lookup | scope owner、依赖初始化、模块增删、集合查询、逆序销毁和调用期有效性必须显式 | 不复制UObject、反射宏、全局engine singleton或其线程成本 |
| Bevy | `Res/ResMut`在scheduler初始化时登记access，冲突检测以ComponentId/slot完成，并保留change tick；SubApp有独立World/Schedule | typed access declaration、阶段/冲突模型、dense typed slot与sub-world scope | 不把Rust TypeId/monomorphization当跨DLL稳定ABI，也不强制所有manager变ECS resource |
| Godot | Engine singleton记录name、pointer、class name、user-created/editor-only，拒绝重复并支持查询、列举和对称删除 | 可发现catalog、重复provider拒绝、origin/policy与对称remove | 不复制裸pointer singleton或无generation全局表 |
| Fyrox | PluginRegistrationContext限制注册期能力；PluginContext显式借用scene/resource/UI/graphics/time；trait区分register/init/on_loaded/on_deinit与graphics context事件 | registration/use/deinit阶段能力分离，使用期传入受限borrow/context | 不复制过宽context，也不让一个context成为新的万能service locator |
| Unity Graphics | ContextContainer以typed slot array完成Create/Get/GetOrCreate、重复检测、active index和Dispose复用；VolumeManager显式拥有stack与更新入口 | render局部context可用dense slot、active set和批量reset，不必进入全局registry | 仅作为Graphics package内局部容器参考，不外推为完整engine manager architecture |

共同底线是service必须同时具备scope、owner、typed contract、生命周期阶段和受控访问。Zircon可以用更紧凑的数据布局、更少虚调用和编译后的slot计划争取性能优势，但不能省略这些语义。

## 5. 与既有 canonical owner 的边界

| 事实 | Canonical owner | Runtime50只拥有 |
|---|---|---|
| module admission、in-flight drain、逆依赖shutdown、失败恢复 | Runtime01、App01 | manager facade接入call lease及产品receipt要求 |
| scheduler phase、任务取消与线程亲和 | Runtime02 | descriptor暴露manager access/affinity声明 |
| qualified identity、generation、owner epoch、index exhaustion | Runtime24 | manager handle不可伪造与scope投影 |
| profile/capability/provider组合 | Runtime42 | manager directory消费最终composition plan |
| module/service descriptor、typed binding、context/factory/snapshot | Runtime46 | manager contract成为统一service descriptor的一种投影 |
| native ABI、vtable、DLL unload | Interface01、Plugins01 | trait manager调用持有module code lease |
| process-global index、HashMap/global mutex/condvar热路径 | PERF-MVP-628 | manager caller规模、O(1) slot目标与资格门 |

因此本篇不新增P0。实现时必须更新父owner状态和共享manifest，不能把相同事实分别修成多个互不兼容的registry。

## 6. P1差距清单（56项）

1. 同一Core同时暴露raw concrete manager与wrapped trait manager两套解析模型，没有单一service kernel。
2. interface wrapper不是`ManagerDescriptor`声明的contract，而是额外注册的一项普通service。
3. 捕获`ManagerServiceHandle<T>`时只确认名称存在，不验证该entry能提供`T`。
4. service对象以`Any`保存，错误contract要到use-time downcast才失败。
5. descriptor没有跨crate/跨DLL稳定的contract key与版本。
6. handle的`index/generation/service`全部public，可由外部构造矛盾identity。
7. handle不携带runtime instance、BuildSet、module owner或provider generation provenance。
8. manager只有runtime-wide全局scope，没有engine/world/session/project/player/view/device作用域。
9. 泛型`manager_service_handle<T>(name)`允许任意名称与任意T组合，typed accessor只是约定。
10. `ManagerServiceResolver`因泛型方法不可object-safe，也不表达availability/access policy。
11. 多个模块为一个能力同时注册DefaultManager与interface wrapper，重复factory、依赖与生命周期节点。
12. registry没有唯一provider选择、多实现集合、priority或query语义。
13. required、optional、disabled、blocked、degraded状态没有统一typed availability结果。
14. provider artifact、ABI/schema version、signature与source provenance不进入manager identity。
15. canonical manager name散落在模块代码和宏中，没有从compiled composition生成。
16. builtin Pascal命名与plugin小写namespace并存，稳定namespace规则未冻结。
17. feature gate会让manager accessor/contract从编译表面消失，不能证明powerset schema稳定。
18. 没有不可变compiled manager directory；运行期仍依赖可变字符串registry。
19. `ManagerServiceHandle::resolve`绕过已存在的`ServiceHandle::enter`。
20. 解析得到的裸`Arc<T>`可跨generation失效与deactivation继续存活。
21. manager方法调用不计入registered service in-flight drain。
22. 原生插件trait object可在library unload边界后仍持有vtable/code引用。
23. raw `CoreHandle::resolve_manager<T>`同样返回无call guard的裸Arc。
24. `ServiceHandle<T>`/`ServiceCallGuard<T>`的production caller为0，正确底座没有进入产品。
25. generation只在resolve瞬间校验，不保护完整调用区间。
26. 多次manager调用或跨await操作没有phase lease、deadline和一致generation快照。
27. resolve消费handle并由调用方自行clone，API没有区分短调用borrow与长期subscription。
28. 没有reload/rebind政策，调用方不知道旧handle应失败、重取还是自动跟随provider。
29. 没有provider replacement、availability变化或generation变化订阅。
30. 多manager依赖无法按同一composition generation原子批量解析。
31. handle捕获只检查生命周期状态，不证明contract、provider和factory readiness。
32. lazy factory可由任意解析调用线程触发，没有专属初始化阶段或executor。
33. descriptor没有main/render/task线程亲和及允许调用phase。
34. manager调用没有deadline、cancel token、readiness fence或completion receipt。
35. 错误缺少结构化reason、retryability、remediation和last-good信息。
36. handle index不参与直接寻址，名称HashMap仍是实际authority。
37. steady-state resolve经过全局`Mutex<HashMap>`，不同manager互相竞争。
38. 完整`RegistryName`继续参与hash/比较/错误clone，句柄没有紧凑slot key。
39. wrapper路径增加一次Any downcast和额外Arc层，却没有增加contract语义。
40. Graphics、render loop、dynamic session和viewport热路径反复resolve同一manager。
41. 没有phase-local、generation-bound slot borrow来合并同帧重复解析。
42. service index为process-global单调Atomic且不复用，隔离与耗尽由Runtime24/PERF-MVP-628继续拥有。
43. registry等待使用共享condvar/状态锁，独立service可能产生无关唤醒；由PERF-MVP-628继续拥有。
44. 没有resolve hash、lock wait、factory wait、downcast、Arc clone、stale reject指标。
45. 没有manager数量、线程数、reload频率与尾延迟的scale benchmark。
46. best-effort diagnostics在manager缺失时静默省略字段，无法区分Unavailable与collector失败。
47. shutdown路径忽略部分manager resolve/call错误后仍推进状态，缺少最终degraded/failure receipt。
48. Graphics/RenderFramework等调用方把`CoreError`字符串化，丢失typed root cause。
49. render/drop清理吞掉backend失败，无法证明manager资源已在卸载前释放。
50. viewport异步任务只捕获handle，lazy factory、thread affinity与call lease没有随任务建模。
51. 没有typed discovery/list/status catalog供Editor、Hub、diagnostics展示真实provider状态。
52. manager解析没有principal、capability、sandbox或least-privilege access policy。
53. scheduler不知道manager read/write/exclusive access，无法做冲突检测与并行规划。
54. core manager测试主要验证名称、源码形状和weak Core lifetime，不足以覆盖真实contract。
55. 缺少wrong-T、public mutation、cross-Core、admission close、resolved-Arc unload与guard drain测试。
56. 缺少原生插件热卸载、feature powerset、多scope、多world、soak和产品热路径性能资格。

## 7. P2差距清单（14项）

1. `ManagerResolver`实际是typed service locator，名称没有说明scope、lease或availability语义。
2. `ManagerServiceResolver` trait与free/Core方法重复，没有形成稳定扩展点。
3. accessor命名不一致，如`resource_handle`、`input_actions_handle`和其他`*_manager_handle`并存。
4. `PlatformManager` canonical name实际映射PreferenceStorage，领域名称与contract不匹配。
5. builtin/plugin canonical name在日志中的大小写与分隔风格不一致，影响诊断可扫读性。
6. typed accessor由宏手工枚举，新增manager易漏catalog、测试或feature组合。
7. `PhantomData<fn() -> T>`的variance/send/sync意图没有合同说明。
8. handle Debug输出不含contract、provider、scope和runtime identity。
9. Core weak升级失败返回`ServiceUnavailable("CoreRuntime")`，与runtime unavailable语义混用。
10. resolve消费handle导致大量显式clone，掩盖短借用与长期持有的差别。
11. canonical name是`&str`常量，不是构建期验证、intern或生成的稳定key。
12. 错误显示字符串不含contract/provider/scope/retry元数据。
13. `include_str!`源码形状测试容易被无行为变化的重排击碎，也可能被注释伪绿。
14. stale测试用`generation + 1`，没有near-max、wrap、exhaustion和slot reuse fixture。

## 8. 目标架构

### 8.1 单一 compiled service directory

```text
CompositionPlan / BuildSet
  -> ServiceContractDescriptor
       stable contract key + version
       scope + owner + provider artifact/ABI
       startup/dependency + affinity/access
       reload/unload + availability policy
  -> validated dense ServiceSlot[]
       generation + admission + in_flight
       state + provider lease + typed projection

ManagerHandle<T>
  private { runtime_id, scope_id, slot, generation, contract }
  -> enter(call_context)
  -> ManagerCallGuard<'a, T>
  -> method call
  -> guard drop / in_flight--
```

manager不再是另一套registry；它是统一service descriptor上的typed、scoped调用投影。raw concrete实现只能作为factory私有对象，不能与public contract再注册成两个独立service。兼容re-export、旧resolver或双写registry不应保留。

### 8.2 scope与访问模型

- engine/runtime、project/session、world、local player、view/render device必须有不同scope identity和销毁顺序；
- accessor由compiled directory生成，contract key与slot在激活事务中冻结；
- scheduler任务声明read/write/exclusive、affinity、deadline与capability，进入调用时验证；
- optional manager返回结构化Unavailable/Disabled/Blocked/Degraded，required manager在admission前失败；
- dynamic provider replacement发布新generation，旧generation只允许drain，不自动跳到新实现。

### 8.3 性能方向

- steady-state路径只做runtime/scope/generation检查、dense slot读取和轻量call lease；
- 同一phase允许生成受生命周期约束的batch borrow，避免20次重复hash/lock/downcast；
- metadata、错误字符串与discovery catalog走冷路径，不能污染manager调用热结构；
- per-slot admission/waiter避免全局condvar无关唤醒；
- benchmark必须分别报告median/p95/p99、contention、reload、memory和错误路径，不只测单线程平均值。

## 9. 重构里程碑

### M0 · Contract与调用图冻结

- 枚举全部raw/wrapped manager、scope、provider、feature gate和production caller；
- 给现有descriptor生成contract mismatch、duplicate provider、missing provider和hot-path基线；
- 冻结Runtime01/24/42/46、Interface01、Plugins01、App01与PERF-MVP-628的父owner边界。

### M1 · Unified Descriptor与Dense Directory

- 扩展统一service descriptor，加入contract/scope/provider/ABI/affinity/access/unload policy；
- composition阶段验证唯一provider、依赖、scope引用和feature powerset；
- 用runtime-owned dense slot directory替代manager steady-state RegistryName HashMap查找。

### M2 · Guarded Typed Access硬切

- `ManagerServiceHandle<T>`字段私有化并加入runtime/scope/contract identity；
- 所有manager调用通过`enter()`返回call guard，完整调用区间计入in-flight；
- 删除raw public resolve和wrapped双注册，不保留shim、re-export或双写。

### M3 · Product Caller与Lifecycle收敛

- 迁移Graphics、dynamic session/runtime loop、Editor viewport与首方插件；
- 引入phase-local batch borrow、结构化availability和terminal shutdown receipt；
- 原生provider必须持有module code lease，先drain call guard再析构对象和卸载library。

### M4 · Qualification与竞争基线

- 完成wrong-contract、stale、cross-runtime、多scope、reload、panic/fault、shutdown race测试；
- 完成feature powerset、manager scale、contention、soak与native unload认证；
- 用同硬件、同场景、同语义基准与参考实现对比，未达正确性门前不得优化掉合同。

## 10. 验收门（36项）

| Gate | 验收内容 |
|---|---|
| G01 | 所有manager contract来自同一compiled service descriptor，不存在raw/wrapped双注册 |
| G02 | descriptor显式记录stable contract key/version、scope、owner、provider、ABI与BuildSet |
| G03 | duplicate、missing、wrong-version和scope-invalid provider在activation前fail-close |
| G04 | feature powerset不会改变持久contract identity，禁用项返回typed Disabled/Unavailable |
| G05 | handle字段私有且无法跨runtime、scope、contract或generation伪造 |
| G06 | stale handle在slot reuse、reload、world/session销毁后稳定拒绝 |
| G07 | 每次manager调用都持有call guard并准确增加/减少in-flight |
| G08 | admission关闭后不再进入新调用，已进入调用可drain到明确deadline |
| G09 | 原生trait object销毁和DLL unload发生在所有call guard归还之后 |
| G10 | panic、cancel、timeout与early return都不会泄漏in-flight或provider lease |
| G11 | lazy factory只在声明的初始化executor/phase运行，不能由任意caller触发 |
| G12 | affinity错误、scheduler access冲突和exclusive manager并发调用可检测 |
| G13 | required/optional/degraded/blocked状态有结构化reason、retry与remediation |
| G14 | provider replacement发布新generation，旧调用只drain且不会隐式跳转 |
| G15 | 多manager batch resolve绑定同一composition/scope generation并原子失败 |
| G16 | engine/project/session/world/player/view/device scope均有owner与逆序teardown测试 |
| G17 | steady-state manager lookup不hash RegistryName、不获取全局HashMap mutex |
| G18 | handle slot直接寻址为有界O(1)，index exhaustion/reuse策略可证明 |
| G19 | phase-local borrow在生命周期结束后不可使用，不能逃逸为裸长期Arc |
| G20 | discovery/status catalog报告contract、provider、scope、generation和真实availability |
| G21 | principal/capability策略在返回guard前执行并留下audit evidence |
| G22 | CoreError跨Runtime/Graphics/Editor边界保持typed，不以字符串作为authority |
| G23 | diagnostics manager缺失显示Unavailable/Failed，不静默省略 |
| G24 | shutdown每个manager步骤产生terminal receipt，best-effort失败可见且不伪绿 |
| G25 | Graphics resource streamer同phase重复access合并，仍保留generation校验 |
| G26 | dynamic frame/event/host request在正确scope与thread phase解析Input/Render manager |
| G27 | Editor viewport任务验证scope、generation、deadline与cancel，不在错误线程建manager |
| G28 | AI/Net/Physics/Sound/Animation/Scene只发布一个public contract service |
| G29 | wrong-T、wrong-name、mutated identity、cross-Core与near-max generation测试通过 |
| G30 | admission-close与已解析对象竞态测试证明无调用越过module unload |
| G31 | native plugin热装/热卸/失败恢复测试在真实动态库上通过 |
| G32 | 1/64/1K/64K manager和1/8/64线程报告median、p95、p99与lock/contention |
| G33 | resolve、call lease、factory wait、stale reject和shutdown drain有低开销指标 |
| G34 | feature powerset、multi-world、多session、reload soak无泄漏、死锁或旧代访问 |
| G35 | 同场景同硬件基准同时满足正确性、失败、CPU、memory与tail-latency门 |
| G36 | `git diff --check`、frontmatter路径、链接、索引、coverage、计数和fingerprint复核通过 |

## 11. 状态与限制

| 项目 | 状态 | 证据 / 限制 |
|---|---|---|
| current source静态审查 | review_complete | 37文件、7,351行、269,212 bytes、29项test；focused fingerprint如第2节 |
| production reachability反查 | review_complete | handle 64处/21文件；resolve 80处/34文件；Graphics streamer 20处asset manager access |
| 参考引擎对照 | review_complete | Unreal 6、Bevy 2、Godot 2、Fyrox 2、Unity Graphics 2个本地源码文件 |
| P0/P1/P2 | review_complete | 0 / 56 / 14；P0继续由既有canonical owner负责 |
| Production重构 | pending | 本篇没有修改源码、测试、Cargo、ABI或产品行为 |
| 动态与性能验证 | not_run | 未运行Cargo、DLL unload、Editor、WGPU、soak、benchmark；不能形成性能领先声明 |
