---
related_code:
  - zircon_runtime/src/engine_module
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/contexts
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/registration
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/builtin/runtime_modules/assembly/profile_selection.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly/target_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report/report.rs
  - zircon_runtime/src/script/vm/module
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/groups/resolution.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins
tests:
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/profile_modules.rs
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/tests
plan_sources:
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-17-module-descriptor-regeneration.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/godot/modules/register_module_types.h
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension.h
  - dev/godot/main/main.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 46 · Engine Module / Service Contract / Context / Factory / Descriptor Snapshot / Composition / Lifecycle 工程化差距

## 1. 结论

`zircon_runtime::engine_module`不是应该删除的过渡目录。仓库路线图和接口收敛规则已经明确把`IModule -> EngineModule`、`IService -> EngineService`、`IDriver/IManager/IPlugin -> EngineDriver/EngineManager/EnginePlugin`映射到这里；Core也已拥有真实的descriptor注册、模块和服务依赖校验、五态生命周期、四阶段`ModuleLifecycle`、service generation、call admission、反向卸载图和devtools snapshot。这些基础说明问题不是“没有模块系统”，而是声明门面、composition编译产物和Core运行态之间仍有多份可漂移真值。

当前`EngineModule`同时暴露`module_name()`、`module_description()`和任意可执行的`descriptor()`。名称和说明因此各有两份authority；`AssetModule`已经出现真实说明漂移：trait返回“Project asset pipeline, import workers, and resource indexing”，descriptor却返回“Asynchronous asset I/O and CPU-side decoding”。名称暂时一致只是实现约定，不是类型或编译门保证。更严重的是，Runtime profile选择阶段确实缓存了一次descriptor并用于依赖闭包与排序，却在`RuntimeModuleLoadReport`中只返回`Arc<dyn EngineModule>`；App随后重新运行一遍builtin group resolution，再用Runtime返回的module替换group entry，最后`try_finish()`再次调用`descriptor()`。现有“descriptor只求值一次”测试只证明单个Runtime sorter或单个`ResolvedPluginGroup`内部不重复，不能证明Runtime→App→Core端到端只物化一次。

`EngineService`也不是已经生效的metadata contract。三个contract constructor和四个trait都进入prelude，但除本目录测试外没有生产消费者；Core注册、依赖图、devtools和resolution均直接读取原始descriptor。contract复制owner、registry name、kind、startup mode和dependencies，owner与name、kind与name还可彼此矛盾。它既不是descriptor的权威view，也不是compiled graph的输入或输出，因此目前只是公开的平行元数据表面。重构方向应保留接口族，但让`EngineService`由经过验证的compiled service declaration实现，而不是继续维护未消费的clone wrapper。

工厂和上下文还有一项新的正确性阻断。Immediate service factory通常在`activate_module_with_graph()`的`catch_unwind`内执行；模块已运行后的Lazy service factory则在`resolve_existing_service_inner()`中直接调用。若lazy factory panic，函数会在`result.is_err() && claimed_initialization`复位之前展开，service可永久停在`Initializing`并保留`initialization_owner`，其他线程可能持续等待，同时panic越过engine API边界。这个问题不能用“Rust内部插件可信”解释，因为`factory()`与`plugin_factory()`是公开prelude能力，且Core已经对module lifecycle panic作了typed隔离。

本报告新增 **1项P0、48项P1、12项P2和36个资格门**。Runtime01继续拥有完整激活、停机、反向卸载和service teardown父问题；Runtime42拥有Profile/target/provider/extension composition选择；Runtime07与Plugins01拥有VM/native package、ABI、安全和hot reload；App01拥有产品host与shutdown；Runtime45拥有preference backend语义。本文只拥有`EngineModule/EngineService`声明门面、一次物化、compiled contract、factory/context authenticity和Runtime→App→Core snapshot连续性，不重复累计既有P0。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 证据等级 | 本轮检查重点 |
|---|---:|---|---|
| `engine_module`门面与测试 | 8 / 509 / 14,323 | E3 | trait、contract、context helper、name helper、factory wrapper、prelude surface |
| Core descriptor/context/lifecycle消费者 | 24 / 3,402 / 122,927 | E3 | 注册校验、图冻结、factory调用、panic边界、generation、devtools projection |
| Runtime/App composition消费者与测试 | 11 / 2,373 / 81,344 | E3 | Profile cache、load report、group二次解析、snapshot配对、bootstrap factory patch |
| 生产`EngineModule` owner | 26 / 2,358 / 85,016 | E3 | 逐实现核对名称/说明/descriptor、动态配置捕获与owner分布 |
| VM plugin context消费者 | 4 / 1,593 / 54,941 | E3 | Core PluginContext与VM package host context混用、roots派生与变更 |
| 父报告与开放failure | 5 / 1,780 / 164,038 | E2 | 唯一owner、旧结论currentness、不得重复计数的父问题 |
| Unreal、Bevy、Godot、Fyrox参考 | 16 / 15,310 / 563,961 | E2/E3 | module identity、phase/policy、readiness、query、reload/unload、context与状态恢复 |
| selected combined scope | 93 / 26,955 / 1,073,201 | E2/E3 | 工作树fingerprint `a9f0c987934bc148cfc7e2abd2ef26e9a6afbf362ed7d9d62f9a32e2d8daac83` |

指纹按93个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path<TAB>hash`和LF连接、无末尾LF后取总SHA-256。统计冻结的是2026-08-19当前工作树，基线提交为`25e09a23178000f2e783ce2143cf70a8b118d404`。本轮只修改review文档和索引，不修改Rust、测试、Cargo、ABI或资产。

### 2.2 检查方法

按`module owner -> trait identity -> descriptor materialization -> Runtime profile dependency closure -> target/plugin append -> sort cache -> load report -> App builtin group re-resolution -> replace/add -> ResolvedPluginGroup -> EngineEntry query/bootstrap -> host factory patch -> Core registration validation -> FrozenModuleGraph -> eager/lazy factory -> lifecycle context -> devtools snapshot -> unload/reload`顺序逐段阅读，并反向搜索全部生产消费者。

每段核对identity、single evaluation、pairing invariant、binding、provenance、generation、error/panic、thread/wait、capability、path authority、type erasure、lifecycle、diagnostics、allocation和qualification。结构审计脚本给出的当前分类是：`engine_module`为stable facade；`zircon_editor`当前module owner形态为converged；`zircon_runtime`与`zircon_app`因生产热点仍是needs-refactor。这里的converged只表示owner/descriptor骨架存在，不表示本文契约已完成。

Unity Graphics没有模块/插件宿主的完整可比owner，本轮不为了凑齐引擎名称强行引用。它继续只在RHI、render package和graphics utility报告中作为参考。

### 2.3 动态证据边界

1. 本轮未运行Cargo；lazy factory panic、并发waiter、descriptor call count和reclamation均是待实施的RED gate，不得标记为通过。
2. 既有Editor编译、Hub persist、WOC协议、npm计数和plugin locked metadata阻断没有因本报告改变。
3. `EngineService`、三个contract helper和`module_context()`的生产零消费者由全仓反向搜索确认；测试、skill和文档引用不计为产品consumer。
4. 生产`EngineModule` owner逐个核对后，名称暂未发现漂移，说明漂移已在`AssetModule`出现；这证明合同允许双真值，而不是证明所有owner当前都错误。
5. factory panic分析基于明确控制流：factory调用位于`resolution.rs`的普通闭包内，调用后才执行error reset；该路径没有局部`catch_unwind`。不需要用动态测试猜测panic会不会越过边界，但并发挂起和恢复细节仍必须用测试限定。

### 2.4 开放failure与旧报告修正

`docs/plans/zircon_runtime/runtime/02/failure-2026-07-17-module-descriptor-regeneration.md`必须保持`open`。它记录的两项源码修复真实存在：`ResolvedPluginGroup`内部会复用已解析descriptor，动态wrapper也不再`Box::leak`名称/说明。但“Builtin selection reports and bootstrap registration read the same frozen group snapshot”只对App最终group内部成立，不覆盖更早的Runtime selection。Runtime cache在返回`RuntimeModuleLoadReport`时被丢弃，App还会重新调用Runtime group resolution，所以完整启动仍没有single-generation证据。

Runtime42第3节所述“descriptor只求值一次并在最终sort复用”也只适用于同一次Runtime profile selection+sort；应保留其局部优化结论，同时用本文纠正端到端外推。Runtime01关于`ServiceFactory`直接接收强`CoreHandle`的旧表述已发生source drift：当前签名是`&CoreWeak`，但factory仍可upgrade并把强handle放进返回的`ServiceObject`，所以“合同允许强环”仍需验证，不能再把参数类型本身写成强handle。

## 3. 必须保留的工程基础

1. 保留`EngineModule`作为`IModule`的Rust owner门面，但收敛到一个权威proposal/snapshot入口，名称和说明从该产物派生。
2. 保留`EngineService`、`EngineDriver`、`EngineManager`、`EnginePlugin`作为metadata层接口族，不要求每个具体manager/driver实例继承一个巨型trait。
3. 保留Core对module/service owner、kind、duplicate、dependency、init-level与dependency direction的typed校验。
4. 保留`FrozenModuleGraph`、service generation、call guard、in-flight drain和反向service order；本文不把生命周期owner搬回门面。
5. 保留`ModuleLifecycle::build/ready/finish/cleanup`与activation panic边界，补齐factory边界而不是删除已有状态机。
6. 保留`RegistryName`缓存解析结果和serde校验，删除会对外部字符串panic的便利入口。
7. 保留Runtime profile从同一target candidate registry补依赖闭包的算法，但让cache成为最终compiled plan的一部分。
8. 保留`ResolvedPluginGroup`对nested group已解析descriptor的传递，升级为验证过的module/descriptor identity pair。
9. 保留动态`DescriptorBackedEngineModule`对owned String的借用，不恢复`Box::leak`或process-global interner。
10. 保留Core devtools对真实registry state的投影，增加compile generation、binding和source provenance，不建立第二状态表。
11. 保留VM package manager从validated package source派生roots的能力，但使用专用、不可伪造的VM context，不复用Core service factory context。
12. 保留App在激活前提供host资源的顺序，把它改为显式HostBindingSet输入，不再按字符串改写descriptor field。

## 4. 当前实现链与断路

```text
Runtime candidate Arc<dyn EngineModule>
  -> module_name() decides BuiltinRuntimeModuleId / HashMap key
  -> descriptor() generation A decides dependency closure and sort
  -> RuntimeModuleLoadReport { modules, availability, diagnostics }
       descriptor cache discarded
  -> App Default/Dev/Headless PluginGroup runs Runtime assembly again
       descriptor generation B for group resolution
  -> Runtime report modules replace group entries and clear cached descriptor
  -> PluginGroupBuilder::try_finish()
       descriptor generation C + module-only sort
  -> ResolvedPluginGroup { modules, descriptors }
  -> module_descriptors() reports generation C
  -> bootstrap clones generation C
       patches PlatformDriver.factory by string
       registers generation C'
  -> Core validates descriptors, freezes topology and activates services
```

| 边界 | 当前事实 | 工程断路 |
|---|---|---|
| Module identity | trait name和descriptor name分别返回 | key、graph与report可分裂；没有pair invariant |
| Description | trait与descriptor各存一次 | `AssetModule`已真实漂移 |
| Materialization | `descriptor()`是任意`&self -> owned value`方法 | 可依赖计数器、锁、环境或可变registry；没有纯度或generation |
| Runtime selection | profile阶段缓存descriptor | cache只服务本地sort，report不持有 |
| App group | 再次运行builtin resolution并replace module | 相同输入被重新解释，descriptor再次生成 |
| Frozen group | 保存module与descriptor平行Vec | 只注释索引 invariant，未验证每对identity；交换name可静默错配 |
| Bootstrap | clone descriptor并改Platform factory | query/report snapshot与Core真实binding不同 |
| Service contract | clone descriptor元数据的wrapper | 无生产consumer，且可构造owner/kind矛盾 |
| Context | public String/CoreWeak/PathBuf字段 | 调用方可伪造；无generation、capability、deadline或binding provenance |
| Factory | opaque Arc closure -> Arc<dyn Any> | 无binding ID、expected type、async/cancel、typed panic或teardown receipt |
| Lazy resolve | factory直接调用 | panic跳过slot reset/notify并越过engine boundary |
| Diagnostics | Core只投影运行态name/count/state | 无composition generation、factory binding、source/build/package provenance |

## 5. P0：正确性与宿主存活阻断

| ID | 阻断 | 直接证据 | 必须修复 / 验收 |
|---|---|---|---|
| MOD-P0-001 | Lazy service/plugin factory panic会越过Core边界并跳过slot复位，service可永久停在`Initializing`，并发waiter可能无终态等待 | `resolve_existing_service_inner()`先把entry设为Initializing并记录owner，随后直接执行`factory(...)`；只有正常返回到函数尾部才在`result.is_err()`时`reset_initializing_service()`；仅module activation外层有`catch_unwind` | 为每次factory invocation建立统一no-unwind trampoline和RAII initialization claim；panic转typed `ServiceFactoryPanicked`，清除owner、复位generation/state、通知所有waiter；Immediate/Lazy/Plugin三路、单线程/多线程、随后重试/卸载/进程退出全部有超时受控测试 |

该P0只登记factory调用边界。Runtime01仍拥有process shutdown、module deactivate和service cleanup父问题；Plugins01/Runtime07仍拥有不可信native/VM插件隔离。修复不得只在App caller外包一层`catch_unwind`，因为Core slot已在panic前变更状态。

## 6. P1：必须完成的工程重构

### 6.1 Module authority、snapshot与composition连续性

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| MOD-P1-01 | `EngineModule::module_name()`与`ModuleDescriptor::name`是两份身份 | `EngineModule`只暴露一个authoritative proposal/snapshot；所有key从validated `ModuleId`派生 | Runtime46；任意pair不可能表达不同名称 |
| MOD-P1-02 | `module_description()`与descriptor description已在`AssetModule`真实漂移 | 删除独立说明authority；diagnostic/display metadata只由proposal拥有 | Runtime46；逐owner parity test不再需要比较两份字段 |
| MOD-P1-03 | `descriptor()`允许任意重复副作用或环境读取 | owner在构造或composition collect阶段只产生一次immutable proposal；编译后禁止再次调用author code | Runtime46；CountingModule端到端call count严格为1 |
| MOD-P1-04 | Runtime profile cache用于闭包/sort后被丢弃，load report只返回modules | `RuntimeModuleLoadReport`返回`Arc<CompiledModuleGraph>`或等价冻结产物，连同descriptor/proposal和诊断 | Runtime42 + Runtime46；Runtime→App零再物化 |
| MOD-P1-05 | App builtin group resolution再次调用Runtime assembly | Profile/target/plugin selection只由Runtime composition compiler执行一次；App只提供product/host inputs | Runtime42；instrumentation证明一次plan build |
| MOD-P1-06 | replace/add清空entry descriptor，`try_finish()`产生第三代descriptor | replacement传递已经验证的proposal/compiled row；不得退回module trait重算 | Runtime46；nested/replaced/disabled组合call count |
| MOD-P1-07 | generic `EngineEntry::module_descriptors()`每次查询重算，bootstrap又调用一次 | `EngineEntry`必须返回共享compiled receipt，不提供可漂移默认实现 | App01 + Runtime46；query/report/bootstrap同Arc generation |
| MOD-P1-08 | `ResolvedPluginGroup`仅用平行Vec注释索引一致，未验证module name等于配对descriptor name | 用单个`ResolvedModuleEntry`持有owner+compiled descriptor；构造时校验identity | Runtime46；交换A/B descriptor的property test必须失败而非静默重排 |
| MOD-P1-09 | `ModuleDescriptor`及service descriptor字段公开可变，冻结后仍可patch | author proposal可构造，compiled descriptor字段private/Arc slice；绑定只能在compile transaction完成 | Runtime46；freeze后编译期无法mutate |
| MOD-P1-10 | `module_descriptors()`看到默认Platform factory，bootstrap注册的是patched factory | report与Core注册消费同一`CompiledBindingSet`；所有host binding进入hash/receipt | Runtime45 + Runtime46；report/registered binding identity完全相同 |
| MOD-P1-11 | stable metadata与`Arc<dyn Fn>`/lifecycle对象混在同一descriptor，无法稳定hash、serde或比较 | 分离`ModuleProposal`、`FactoryBindingKey`、`HostBindingSet`和`CompiledModule` | Runtime46；metadata可确定性编码/hash，binding有独立identity |
| MOD-P1-12 | 没有generation、source/build/package provenance或commit receipt | compiled graph携带composition generation、BuildSet、source fingerprint、provider/package和diagnostic digest | Runtime42/24 + Runtime46；Core、App、Editor、DLL查询同一receipt |

### 6.2 `EngineService`元数据接口收敛

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| MOD-P1-13 | `EngineService`与三个marker没有生产消费者 | 让Core graph、registration diagnostics和devtools消费统一compiled service contract | Runtime46；至少三个真实生产consumer，不以测试引用冒充 |
| MOD-P1-14 | `DriverContract/ManagerContract/PluginContract`复制descriptor字段 | contract应是compiled declaration的borrowed/Arc view，不再clone平行Vec | Runtime46；descriptor与contract无法独立漂移 |
| MOD-P1-15 | `owner_module`由调用方任意传入，可与`registry_name.module_name()`不一致 | owner从validated `ModuleId`和`ServiceId`组合派生，constructor不可伪造 | Runtime46；mismatch在collect gate被typed拒绝 |
| MOD-P1-16 | `service_kind`由wrapper硬编码，可与RegistryName缓存kind不同 | kind由typed declaration决定且只编码一次；字符串只是显示/serde projection | Runtime46；三类kind mismatch构造不可能或返回Result |
| MOD-P1-17 | public marker trait未sealed，自定义`EngineDriver`可报告Manager kind | marker使用sealed typed contract或associated const/type，不能靠实现者诚实 | Runtime46；compile-fail contract test |
| MOD-P1-18 | contract没有expected service type identity | compiled service声明稳定`ServiceTypeKey`和本进程`TypeId`校验策略 | Runtime46 + Runtime24；错误类型在publish前失败 |
| MOD-P1-19 | contract没有factory binding、thread、panic和startup preparation policy | 加`FactoryBindingKey`与execution policy，不暴露closure本体 | Runtime46；binding缺失/重复/错误线程typed reject |
| MOD-P1-20 | contract没有runtime generation、admission或lifecycle projection | runtime service view组合immutable declaration与generation-bound state，不复制第二registry | Runtime01/24 + Runtime46；devtools和resolver同identity |
| MOD-P1-21 | Core注册/图冻结直接读取原始descriptor，绕过`EngineService` | compile阶段输出Core唯一接受的validated contract；Core不再接受uncompiled public field bag | Runtime46；绕过compile的API物理删除 |
| MOD-P1-22 | 未生效contract已进入runtime prelude，形成过早稳定承诺 | prelude只保留完成语义的接口；实验constructor先降为crate-private后硬切 | Runtime46；public API inventory有owner、consumer和version policy |

### 6.3 Identity、声明完整性与诊断

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| MOD-P1-23 | module name/dependency仍是裸`String`，canonical规则分散 | 建立`ModuleId`与fallible/const constructor，descriptor、dependency、registry共用规范 | Runtime24 + Runtime46；serde/property/fuzz corpus |
| MOD-P1-24 | `RegistryName::from_parts()`和prelude `qualified_name()`对运行时字符串`assert!` | 提供`try_from_parts`与const静态路径；公开数据驱动API只返回typed error | Runtime46；空白、dot、Unicode边界和恶意manifest不panic |
| MOD-P1-25 | App group只验证module拓扑，service owner/kind/dependency要到Core registration才失败 | composition compiler预检完整module+service graph，commit前汇总所有diagnostic | Runtime46；Resolved状态保证Core registration不会再发现声明错误 |
| MOD-P1-26 | 五个`InitLevel`不能表达host/target/explicit-load/restart/reload policy | 在proposal加入稳定host/load policy，并映射到Core phase；不把所有策略塞进enum字符串 | Runtime42 + Plugins01；Client/Server/Editor/Program矩阵 |
| MOD-P1-27 | module/service声明缺version、required/optional、conflict、capability和source provider | 使用typed requirement/provision/conflict与provider provenance，composition统一求解 | Runtime42/Plugins06 + Runtime46；冲突和缺失fail closed |
| MOD-P1-28 | Core devtools snapshot只有name/description/count/state，无法回溯编译代或binding | 投影compiled graph id、source/build/package、factory binding key、runtime generation和last transition | Runtime03 + Runtime46；snapshot与receipt可join且有cardinality预算 |

### 6.4 Context authenticity、factory binding与类型边界

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| MOD-P1-29 | `module_context()`无生产caller，却公开制造任意ModuleContext | context constructor只由Core lifecycle transaction拥有，字段private并提供只读accessor | Runtime46；外部代码无法伪造active context |
| MOD-P1-30 | ModuleContext只有String与CoreWeak | 增加module id、composition/runtime generation、phase、transaction id、deadline/cancel和grants | Runtime01/24 + Runtime46；过期context拒绝调用 |
| MOD-P1-31 | 同一PluginContext同时服务Core PluginDescriptor factory与VM package host | 硬切为`CorePluginFactoryContext`和`VmPluginHostContext`，共享最小identity value而非整struct | Runtime07 + Runtime46；两条生命周期不能互相构造 |
| MOD-P1-32 | Core plugin context把canonical service name写进`plugin_name` | 明确区分ServiceId、PackageId、PluginSlotId和display name | Runtime07/24 + Runtime46；diagnostic不混淆四类identity |
| MOD-P1-33 | Core plugin factory的package/source/data roots固定为None | roots由verified package plan/binding显式提供；无package的内建service不伪装package plugin | Plugins01 + Runtime46；real package factory拿到同代verified roots |
| MOD-P1-34 | VM manager clone基础context后直接改三个public root字段 | 用validated constructor从`VmPluginPackageSource`一次生成immutable package context | Runtime07 + Runtime46；不能半更新或跨generation复用roots |
| MOD-P1-35 | context暴露raw可克隆PathBuf，等于ambient filesystem authority | 提供canonical scoped root/capability handle与只读/读写权限，不暴露宿主任意路径 | Runtime25/Plugins01 + Runtime46；escape/symlink/reload权限矩阵 |
| MOD-P1-36 | ServiceFactory只收到CoreWeak，没有当前service/owner/generation/binding信息 | 使用sealed `ServiceFactoryContext`，包含qualified identity、generation、role和typed resolver | Runtime46；factory日志与错误自动带correlation |
| MOD-P1-37 | factory可从整个CoreWeak解析未声明服务，dependency graph不是capability边界 | resolver只授予compiled dependencies与显式capability；debug可检测越权请求 | Runtime01 + Runtime46；undeclared resolve稳定Denied |
| MOD-P1-38 | PluginFactory同样获得全Core和raw roots，没有package grants | package context包含协商后的host capability table和denial reason | Plugins01 + Runtime46；未授权host API不可调用 |
| MOD-P1-39 | ServiceObject只是`Arc<dyn Any + Send + Sync>`，类型错误到resolve/downcast才暴露 | binding声明expected type并在factory commit前验证；Any可保留为内部快速容器 | Runtime24 + Runtime46；错误类型不进入Running |
| MOD-P1-40 | factory同步返回，缺pending/readiness/cancel/deadline；lazy service也无ready阶段 | 定义bounded prepare/ready协议或明确sync budget，复用module lifecycle而不忙等 | Runtime01 + Runtime46；hung init可取消/超时并复位 |
| MOD-P1-41 | factory错误被压为`Initialization(name, error.to_string())` | 保留typed cause、stage、service/binding/generation、retryability和correlation id | Runtime03 + Runtime46；错误roundtrip不丢cause chain |
| MOD-P1-42 | opaque Arc closure没有binding id、source hash、generation、thread affinity或panic policy | `FactoryBindingRecord`登记这些属性，closure只在受控execution cell内 | Runtime46；report、Core和crash evidence能定位同一binding |

### 6.5 Host binding、reload与资格缺口

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| MOD-P1-43 | App按module/service字符串找到PlatformDriver并改写factory | composition前提交typed `PlatformPreferenceBackendBinding`；compiler验证唯一consumer并冻结 | Runtime45 + Runtime46；删除`descriptor_with_preference_storage_backend` |
| MOD-P1-44 | module declaration没有dynamic reload、automatic shutdown、restart-required或state migration policy | declaration显式声明reload tier；Core/Plugin层执行prepare/quiesce/save/restore/commit/rollback | Runtime07/Plugins01 + Runtime46；不支持时返回RestartRequired |
| MOD-P1-45 | 测试只验证一个happy-path module的name/description相等，没有全owner invariant | 对所有生产owner运行identity/description/pairing property suite | Runtime46；Asset drift先RED后由单一authority消失 |
| MOD-P1-46 | `EngineService`测试仅检查driver wrapper复制字段，没有真实Core consumer或矛盾输入 | contract conformance suite覆盖driver/manager/plugin、owner/kind/type/binding和Core/devtools消费 | Runtime46；零平行metadata表 |
| MOD-P1-47 | 开放failure缺Runtime→App→Core总调用数、report/registration parity和动态文本/closure回收 | 加端到端instrumented entry，覆盖1/100/1,000次创建、query、bootstrap、drop、reload | Runtime46；failure满足后才能改名fixed |
| MOD-P1-48 | 无1/100/1k/10k module/service/dependency的物化、排序、clone、RSS与并发failure预算 | 建立source-bound benchmark和fault matrix，记录calls/allocs/bytes/p50/p95/p99 | Runtime46；只在correctness gate通过后比较Unreal/Bevy基线 |

## 7. P2：维护性、可观察性与性能债务

| ID | 差距 | 后续处理 |
|---|---|---|
| MOD-P2-01 | `engine_module`重新导出CoreRuntime/CoreHandle/CoreWeak及大量Core类型，与`core`/prelude重叠 | 收敛到模块作者真正需要的curated facade，生命周期运行态API从Core owner导入 |
| MOD-P2-02 | `EngineModule: Debug`只保证格式化能力，无法提供结构化身份或状态 | Debug保留开发用途；正式诊断只读compiled receipt与runtime snapshot |
| MOD-P2-03 | description使用owned String并在descriptor/query/devtools多次clone | 冻结为`Arc<str>`或artifact string table，并测量，不提前微优化 |
| MOD-P2-04 | descriptor Debug刻意隐藏factory/lifecycle，却没有可替代binding identity | 输出安全的binding key/policy摘要，不打印closure capture或秘密路径 |
| MOD-P2-05 | 三个contract的EngineService impl机械重复 | compiled typed declaration用共享sealed实现，避免宏或继承层膨胀 |
| MOD-P2-06 | contract constructor clone完整dependency Vec | immutable compiled slice共享；统计真实clone bytes |
| MOD-P2-07 | `ResolvedPluginGroup::module_keys()`每次分配Vec | compiled graph提供borrowed iterator/Arc slice；先以profile证明热点 |
| MOD-P2-08 | Runtime/App sorter重复建立String HashMap并移动module | compiler一次建立stable index；排序结果直接是entry index/Arc slice |
| MOD-P2-09 | `RuntimeModuleLoadReport: Clone`复制module bag并鼓励再次装配 | terminal receipt只clone共享Arc，不暴露可重组的public module Vec |
| MOD-P2-10 | context clone会复制String和多个PathBuf | 使用qualified id与scoped root handle；generation retire时统一回收 |
| MOD-P2-11 | `engine_module_declared_layer_does_not_own_runtime_lifecycle`依赖`include_str!`文本包含/排除 | 改为API/compile-fail/behavior测试；结构测试只守明确layer规则 |
| MOD-P2-12 | 模块命名混用`AssetModule`、`DiagnosticsCoreModule`与`animation.runtime`等形态 | 冻结canonical ModuleId与独立display name，提供一次hard-cut migration表 |

## 8. 参考引擎对照与适用边界

| 参考 | 源码事实 | 对Zircon的约束 | 不照搬的内容 |
|---|---|---|---|
| Unreal `IModuleInterface` / `FModuleManager` | implementation interface只有startup/pre-unload/post-load/shutdown与reload政策，不再返回另一份name/description；manager用唯一name持有ready、load order、filename、handle和status，load完成后才publish ready/event，unload先关闭ready再shutdown；可query、abandon和bootstrap state | Module identity/metadata由manager/descriptor单一拥有；publish、query、reload/unload和artifact provenance必须显式 | 不把C++全局singleton、裸module pointer或其已注释的off-thread风险照搬进Rust |
| Unreal `FModuleDescriptor` | host type、loading phase、platform/arch/target/config allow/deny和additional dependencies进入结构化descriptor；load/unload按phase返回typed result | Zircon的phase/host/build policy必须进入proposal和receipt，不能散落`#[cfg]`/App分支 | 不复制所有枚举；按Zircon产品角色和BuildSet建稳定最小集合 |
| Bevy `Plugin` / `App` | `build -> ready -> finish -> cleanup`有显式`PluginsState`，plugin uniqueness由name和registry检查，build panic时保留插入顺序并恢复unwind | Zircon应保留分阶段readiness并让状态由统一owner推进；factory panic必须先恢复内部状态再返回typed failure | Bevy插件主要是同进程Rust App配置，不等价于native ABI或可卸载DLL |
| Godot module/GDExtension | Core/Servers/Scene/Editor按严格层级初始化与逆向反初始化；reload有prepare、instance binding清理、close/open、finish，不能安全reload时返回NeedsRestart | phase顺序、reload eligibility、对象绑定清理和RestartRequired必须是协议，不是布尔宣传 | 不采用Godot Object/ClassDB模型替代Zircon ECS/service generation |
| Fyrox Plugin/DynamicPlugin | registration context与运行context分开；动态plugin在reload前prepare，engine detach/保存状态后load并重新register；context明确提供scene/resource/UI/graphics/time/task/input等能力 | Core service factory context、VM package context和registration context必须分域；reload需要状态与owner交接 | Fyrox context暴露大量可变engine对象，Zircon应改成capability-limited handle以保持并行和卸载安全 |

共同规律不是“trait方法越多越工程化”，而是identity只有一个authority，声明和runtime binding分层，状态由owner推进，publish前验证，失败有typed terminal，reload有prepare/commit/rollback或诚实RestartRequired。Zircon可以比参考实现更少clone、更强generation和更细capability，但不能靠删除这些语义声称更快。

## 9. 目标架构

### 9.1 单一声明与编译产物

```text
EngineModule owner
  -> &ModuleProposal
       ModuleId / metadata / host policy / requirements
       typed ServiceProposal[] / LifecycleBindingKey
  -> RuntimeCompositionCompiler
       project/profile/target/BuildSet/catalog generation
       full module+service validation and deterministic sort
  +  HostBindingSet
       FactoryBindingKey -> execution cell / capabilities / provenance
  -> Arc<CompiledModuleGraph>
       generation + hash + ordered CompiledModuleEntry[]
       diagnostics + capability decisions + binding receipt
  -> CoreRuntime::register_compiled_graph(Arc<...>)
  -> generation-bound runtime state and devtools projection
```

`EngineModule`仍是作者owner，但不再同时提供name、description和每次新建descriptor三个authority。最保守的hard cut是`fn proposal(&self) -> &ModuleProposal`；若部分owner必须延迟materialize，则使用一次性的`collect(self/Arc<Self>, &CollectContext) -> Result<ModuleProposal>`并由compiler记录call count。不能保留旧三个方法再靠debug assertion比较，因为release和动态插件仍可漂移。

### 9.2 Service metadata contract

`EngineService`应由`CompiledServiceDeclaration`或其typed view实现，至少稳定提供：

- `ServiceId`、owner `ModuleId`和`ServiceKind`；
- startup/readiness policy与typed dependency/capability edges；
- expected `ServiceTypeKey`、factory binding key与execution policy；
- declaration/source/build/package provenance；
- runtime view中的slot index、generation、lifecycle、admission和in-flight count。

声明与运行态可以是两个struct，通过同一`ServiceId + graph generation`关联。禁止把mutex、instance或Core registry塞入metadata trait，也禁止继续用一个clone wrapper伪装运行态contract。

### 9.3 Context与factory协议

所有context字段private且只能由对应transaction owner构造：

- `ModuleLifecycleContext`：module/graph/runtime generation、phase、transaction、deadline/cancel、diagnostic span、grants；
- `ServiceFactoryContext`：service/owner/binding identity、generation、只含declared dependencies的resolver、role/target、deadline/cancel；
- `CorePluginFactoryContext`：Core plugin service identity与协商grants，不冒充package；
- `VmPluginHostContext`：verified package/slot/generation、scoped roots、host export/capability table和reload state。

factory执行cell必须在调用前取得RAII initialization claim，所有返回、error、panic、cancel和deadline分支都只走一个terminal commit；waiter总会被notify。factory output先验证expected type和generation，再publish到Running。service-specific teardown若加入，由Runtime01的lifecycle coordinator拥有，不在factory helper里临时增加drop closure。

### 9.4 Composition与运行状态机

```text
Collecting -> Validating -> Resolving -> Binding -> Frozen
     |            |            |          |
     +---------- Failed <------+----------+

Frozen -> Registering -> Activating -> Running
               |             |          |
               +--Rollback---+          +-> Quiescing -> Draining -> Retired

Reload: Running -> Prepare -> Quiesce -> Snapshot -> BindNext
          -> Restore -> CommitNext -> RetireOld
          -> RollbackOld | RestartRequired
```

每个状态迁移产生receipt并绑定graph generation。`RuntimeModuleLoadReport`不再是可随意clone/recompose的module bag，而是Frozen前的typed diagnostic result或Frozen后的terminal receipt。App不能在Frozen之后追加Editor module、替换factory或再次排序；这些输入必须在Collecting/Binding阶段提交。

## 10. 硬切范围与禁止方案

1. 删除`EngineModule::module_name/module_description/descriptor`三authority，迁移为单一proposal/snapshot；不保留默认方法shim。
2. 删除公开`module_context()`、`plugin_context()`伪造helper；Core/VM owner使用私有constructor。
3. 删除`DriverContract/ManagerContract/PluginContract`平行clone实现，或让名称指向真实compiled typed view；不保留两套同名contract。
4. `qualified_name()`公开路径改为fallible/const typed constructor；禁止catch panic后继续。
5. 删除App的descriptor字符串patch和重复builtin group resolution；host binding必须进入compiler输入。
6. Core只接受compiled graph/entry，不再公开注册任意可变descriptor的产品路径；测试fixture可用专用builder。
7. 不得用process-global descriptor cache、永久interner、`Box::leak`、once-cell按name缓存或限制reload次数掩盖所有权问题。
8. 不得只给`descriptor()`加“应纯函数”注释或debug assert；单次求值必须由API所有权保证。
9. 不得在App最外层捕获factory panic后继续；Core必须先复位slot、generation、wait graph和notification。
10. 不得把所有具体manager/driver对象强制实现`EngineService`，metadata和行为接口继续分层。
11. 不得把VM package path、Core plugin service和native ABI plugin压成一个巨型PluginContext。
12. 不得在没有source-bound correctness/performance证据时宣称模块系统达到或超过Unreal。

## 11. 测试先行的重构里程碑

### M0 · Factory panic containment

先写MOD-P0-001 RED测试：Running module中的lazy service factory panic、lazy plugin factory panic、两个并发waiter、panic后重试、deactivate和runtime drop。实现RAII claim与typed no-unwind trampoline，确认所有terminal都会reset/notify。该切片不得等待大规模descriptor重构。

### M1 · Identity与单一proposal

引入`ModuleId/ServiceId`及fallible/const constructor；为所有26个生产owner建立parity表。将`EngineModule`硬切到单一proposal入口，先修复Asset说明漂移。删除public panic name helper和独立name/description读取。

### M2 · EngineService compiled contract

把module/service builder输出编译为typed declaration，完成owner/kind/type/dependency/capability全图预检。Core registration、devtools和diagnostics首先成为真实consumer，再删除clone contract wrapper和prelude过早表面。

### M3 · Context与binding分域

建立sealed lifecycle/factory/plugin contexts和`FactoryBindingRecord`。迁移Platform preference backend到HostBindingSet；拆开Core plugin与VM package context；roots使用scoped capability。保持Runtime01/07/45各自行为owner。

### M4 · Runtime→App→Core single snapshot

让Runtime composition返回`Arc<CompiledModuleGraph>`，App提交Editor/host输入后只完成一次freeze。删除第二次builtin group resolution、replace后descriptor重算和generic EngineEntry默认重算。打开failure中的端到端call-count/parity/reclamation gate。

### M5 · Reload与运行态投影

补module reload tier、RestartRequired、prepare/quiesce/snapshot/restore/commit/rollback adapter；devtools发布graph/runtime generation、binding和source provenance。具体native/VM迁移继续由Plugins01/Runtime07实现。

### M6 · Scale与竞争性证据

在1/100/1k/10k module、每module 0/1/10/100 service、稀疏/稠密依赖、duplicate/cycle/missing binding、panic/cancel/reload场景测calls、allocations、bytes、compile/activate latency、RSS和retire。与参考引擎比较时固定同等图规模、构建模式、硬件和统计协议；先过correctness/failure gate，再讨论性能领先。

## 12. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | 全部生产EngineModule owner只有一个ModuleId/description authority |
| G02 | Runtime→App→Core每个enabled module proposal/materialization call count严格为1 |
| G03 | disabled/unselected module proposal call count为0，除非显式catalog metadata阶段要求 |
| G04 | report、query、sort、Core registration和devtools引用同一graph generation/hash |
| G05 | A/B descriptor名称交换、duplicate与missing module全部在freeze前typed失败 |
| G06 | AssetModule说明漂移测试先RED，单一authority后不再存在比较点 |
| G07 | module/service ID的serde、Unicode、dot、trim、empty和fuzz corpus不panic |
| G08 | `qualified_name`数据驱动路径对所有非法输入返回typed error |
| G09 | module/service owner、kind和dependency direction在compiled gate一次完整验证 |
| G10 | EngineService有Core registration、devtools、diagnostics三个真实生产consumer |
| G11 | custom EngineDriver不能报告Manager/Plugin kind |
| G12 | wrong ServiceObject type在publish前失败，slot不进入Running |
| G13 | undeclared dependency resolve返回Denied并带service/binding/generation |
| G14 | Immediate、Lazy和Plugin factory error保留typed cause chain |
| G15 | Immediate、Lazy和Plugin factory panic均不越过Core API边界 |
| G16 | panic/error/cancel/deadline后initialization owner清除、waiter被唤醒、状态可重试或卸载 |
| G17 | 2/8/64并发waiter在factory失败下都有bounded terminal，无死锁/忙等 |
| G18 | ModuleLifecycle context不可由外部构造，过期generation拒绝使用 |
| G19 | Core plugin service identity、package identity和VM slot identity不可互换 |
| G20 | verified package roots与factory context同generation，reload后旧root handle失效 |
| G21 | root escape、symlink swap和未授权写全部fail closed |
| G22 | Platform preference backend由typed host binding注入，App无descriptor字段patch |
| G23 | missing/duplicate/wrong-target factory binding在Core注册前失败 |
| G24 | Client2D/Client3D/Editor/Dev/Server host/load policy矩阵确定且无`#[cfg]`暗变schema |
| G25 | hot-unload不支持时返回RestartRequired，不尝试破坏性卸载 |
| G26 | reload任一prepare/save/load/restore/commit失败可回滚旧generation |
| G27 | report和devtools能关联BuildSet、source/package/provider、binding和runtime generation |
| G28 | sensitive root/capability不进入Debug、Display或普通diagnostic文本 |
| G29 | 1/100/1,000次dynamic entry create/drop无Box::leak、文本/closure owner可回收 |
| G30 | 1/100/1k/10k模块图记录proposal calls、allocations、clone bytes、RSS和p50/p95/p99 |
| G31 | sparse/dense dependency排序结果确定，输入顺序和HashMap seed不改变receipt hash |
| G32 | compiled graph冻结后无法通过public API修改name、dependency、factory或lifecycle binding |
| G33 | open descriptor-regeneration failure的call-count、parity、reclamation和managed validation全部通过后才能fixed |
| G34 | Runtime01/03/07/24/42/45、App01与Plugins01的父finding不在本文重复关闭 |
| G35 | source fingerprint变化触发recheck，报告与benchmark都绑定source/build/test inventory |
| G36 | Markdown frontmatter、链接、计数、LF/BOM/trailing-space与`git diff --check`通过 |

## 13. Owner与依赖顺序

| 层 | 本报告owner | 依赖/交接 |
|---|---|---|
| L0 identity | ModuleId、ServiceId、proposal/contract single truth | Runtime24 identity规则 |
| L1 factory safety | no-unwind、RAII claim、typed terminal、context authenticity | Runtime01 lifecycle与Runtime03 diagnostics |
| L2 composition | compiled graph、binding set、generation/hash、pair invariant | Runtime42 selection与Plugins06 provider closure |
| L3 package/VM | Core/VM context分域、verified roots/capability | Runtime07、Plugins01 |
| L4 product | App只消费receipt，typed host binding，Core一次注册 | App01、Runtime45 |
| L5 qualification | end-to-end count、fault、reclaim、scale与competitive evidence | 本报告G01-G36 |

实现顺序必须是MOD-P0-001先独立修复，再做M1 identity、M2 compiled contract、M3 binding/context、M4 single snapshot。不能先在App增加descriptor cache，因为那会固化错误owner并让Runtime cache继续丢失。

## 14. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 93文件EngineModule/Service/Context/Factory纵向审查 | review_complete | 2026-08-19 | 26,955行、1,073,201 bytes；fingerprint `a9f0c987...daac83` |
| 26个生产EngineModule owner与生产consumer反查 | review_complete | 2026-08-19 | 名称当前一致；Asset description真实漂移；EngineService/marker无生产consumer |
| Runtime→App→Core descriptor generation链复核 | review_complete | 2026-08-19 | Runtime cache丢弃、App再次resolve、replace清cache、try_finish再生成、bootstrap再patch |
| Lazy factory panic slot状态审查 | review_complete | 2026-08-19 | 新登记MOD-P0-001；动态并发/恢复测试尚未执行 |
| 生产重构与动态资格 | pending | - | 本篇不修改Rust或测试；G01-G36均未通过 |
