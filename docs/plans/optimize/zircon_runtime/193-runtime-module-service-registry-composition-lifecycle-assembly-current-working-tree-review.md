---
title: Runtime Module、Service Registry、Dependency Resolution、Composition、Lifecycle 与 App Assembly 当前源码复核
category: zircon_runtime
report_id: Runtime193
review_date: 2026-08-30
baseline_head: 399f2318150ae4fa0df3a2543133b03b80099288
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundation_product_incomplete
source_recheck_required: true
canonical_owner: Runtime01
refreshes:
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
related_code:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/core/runtime/handle/registration
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/engine_module
  - zircon_runtime/src/builtin/runtime_modules/composition
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/product_composition/composition.rs
  - zircon_app/src/entry/product_shutdown
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/main/main.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.Compiler.cs
---

# Runtime193 当前源码审查

## 1. 结论

本轮只审查 Runtime module/service registry、composition、dependency resolution、activation/deactivation、plugin assembly 和 App/dynamic session owner。当前工作树的 Runtime 侧选集为约 348 个 Rust 文件、48,167 行、1,785,502 bytes、471 个 test marker；App entry 选集为约 210 个 Rust 文件、26,801 行、1,058,623 bytes。未修改 production Rust、tests、Cargo、ABI 或 tooling，未运行 Cargo、DLL reload、真实 GPU/OS teardown、loom、sanitizer、scale 或 soak。

当前已有可保留的底座：`FrozenModuleGraph` 在第一次生命周期操作前冻结拓扑；module dependency closure、service kind/cross-module/cycle 校验和稳定 activation/reverse service order 已存在；`ServiceHandle`、generation、call admission、in-flight drain、module transition coordinator、panic-to-typed-error 和 veto-before-commit 也有行为测试。问题不在“完全没有 registry”，而在这些部件没有收敛为一个工程级 owner transaction：module 生命周期是 per-module，service teardown 仍隐含在 module cleanup，App composition 只把 `CoreHandle` 返回给上层，dynamic session 又维护另一套关闭次序。

本轮结论：继承 Runtime157 的唯一 P0 仍为 `0 Open / 2 Partial / 1 Closed`，不新增唯一 P0；新增并细化 **32 项 P1（24 Open / 8 Partial / 0 Closed）**、**12 项 P2（8 Open / 4 Partial / 0 Closed）**，资格门 **30 项（20 Fail / 8 Partial / 2 Pass）**。这不是 code-complete 结论，后续实现必须以 Runtime01 的 owner、transaction、receipt 和验证门为准。

## 2. 当前实现闭环

### 2.1 Registry 与 graph

- `ModuleDescriptor` 由公开 `String`、`Vec`、`Arc<dyn ModuleLifecycle>` 和 Driver/Manager/Plugin 三组字段组成；`ModuleDescriptor::new` 默认 `InitLevel::Post` 与 `NoopModuleLifecycle`。
- `register_module` 在 graph 未冻结时持有 frozen-graph mutex，检查模块名、重复模块和每个 service descriptor，再以 1/2/3/4/5/>5 service 五套路径准备并提交 `ServiceEntry`。pending service 在提交前会检查重复，成功后写入 service/module 两张表。
- `FrozenModuleGraph::freeze` 按模块名排序后建立 activation order、dependency/dependent closure、每模块 startup/shutdown service list，并检查 duplicate/missing/kind/cross-module/cycle。
- service entry 保存 index、generation、startup mode、immutable dependency slice、factory、lifecycle、initialization owner、instance、admission 和 in-flight count。generation 在 unload/reactivation 期间递增，但当前是 `u32::wrapping_add` 后回到 1。

### 2.2 Activation、resolution、deactivation

- `activate_module` 先取得完整 dependency closure，然后逐模块调用 `run_module_lifecycle_transition`；`activate_registered_modules` 预先为每个模块获取 transition token，再在批量路径中 build、resolve immediate services、poll ready、finish、commit Running。
- lazy service resolution 可以触发 owner module activation；service factory 通过 `CoreWeak` 或 `PluginContext` 执行，dependency resolution 使用 ThreadId wait graph 和递归 stack 检查 cycle。
- deactivation 先检查 running dependents、blocked service 和 observer veto，再写入 `Stopping`、关闭 admission、等待 guarded calls drain、调用 module cleanup、invalidate service slots，最后写入 `Unloaded`。
- `RuntimePluginBridgeLifecycleState` 被安装为一个单一 `RuntimeModuleLifecycleObserver`；activated 通知的 bridge report 被丢弃，deactivating 只把 bridge error 转成字符串 blocker。

### 2.3 App 与 dynamic owner

- 普通 `EngineEntry::bootstrap` 执行 `CoreRuntime::new -> register_module -> activate_registered_modules` 后返回 `CoreHandle`，不返回带显式 shutdown receipt 的 runtime owner。
- `BuiltinEngineEntry::bootstrap` 在 activation 前后写 config，复制 descriptor 并可能替换 Platform factory；`RuntimeModuleCompositionPlan` 同时保存 `Arc<dyn EngineModule>` 与独立的 `Vec<ModuleDescriptor>`。
- `ProductComposition` 保存 `CoreHandle`、plugin bridge state、compiled plan 和 native plugin host，但没有显式 `shutdown(deadline)`/`Drop` transaction。
- dynamic session 的 `shutdown_before_library_unload` 按 event mirror、project watcher、task scope、module shutdown、task graph、process log 的局部顺序执行；module drain timeout 固定为 `Duration::ZERO`，`RuntimeDynamicSession::Drop` 又忽略返回的 bool。产品 phase enum 又把 `DestroyingRuntime` 放在 `DeactivatingModules` 之前，和 module-first teardown 语义不一致。

## 3. 差距清单

### P1-01：模块 descriptor 是可变公开结构，缺少 sealed declaration boundary（Open）

证据：`zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs:8-17,19-60` 的字段和 builder 都是公开 `String`/`Vec`，注册前没有不可变 declaration object。调用者可在 composition、registration、graph freeze 之间复制和修改 descriptor，难以证明 module identity、factory provenance 与 graph snapshot 一致。

重构：引入不可变 `ModuleDeclaration`/`ServiceDeclaration`，builder 只在 compose 阶段可写；注册提交后只保留 canonical snapshot 和 declaration digest。禁止 production path 直接修改已编译 descriptor。

### P1-02：模块名验证只做 trim/non-empty，命名空间不是稳定 ID（Open）

证据：`registration/validation.rs` 的 `is_canonical_module_name` 只检查非空和首尾无空格；`ModuleDependencySpec` 仍是任意 `String`。与 `RegistryName` 的 namespace/kind/service 校验不一致，控制字符、Unicode 规范化、点号层级和大小写冲突没有统一策略。

重构：使用带规范化、长度上限、ASCII/Unicode 策略和 stable hash 的 `ModuleId`；module dependency 只接受已解析 ID，诊断保留原始输入和 canonical value。

### P1-03：module dependency 只有名字，没有版本、能力、可选边或 provider policy（Open）

证据：`ModuleDependencySpec { module_name: String }` 与 `DependencySpec { name: RegistryName }` 均没有版本范围、capability、optional/weak、target/profile 或 provider selection。当前 graph 只能回答“名字存在且层级正确”，不能回答 ABI/schema 兼容性。

重构：增加 version range、capability set、target/profile constraint、required/optional/weak edge 和 provider selection receipt；compose 阶段解析，runtime 只消费冻结结果。

### P1-04：`InitLevel` 是封闭的五级全局枚举，不能表达动态模块 phase（Partial）

证据：`lifecycle.rs:14-32` 固定 Kernel/Services/Scene/Editor/Post，`sort_module_activation_order` 直接拒绝依赖更晚 init level 的模块。Godot 能按 initialization level 对 built-in modules 和 GDExtension 分层初始化；当前 Zircon 没有扩展 phase、phase owner 或跨 profile 的显式 phase graph。

重构：保留稳定基础 phase，但以 `PhaseId + order constraints` 编译 profile-specific phase graph，禁止模块自己绕过 graph 写隐式顺序。

### P1-05：`ServiceKind` 三分法阻塞未来扩展和服务策略（Open）

证据：`ServiceKind::{Driver,Manager,Plugin}` 在 registry parser、kind rank、dependency validation、manager accessors 多处硬编码；新增 render queue、world service、bridge provider 或 system service 要修改所有 match/名称路径。

重构：将 kind 变成 extensible registry category，策略（dependency rank、teardown policy、thread affinity、visibility）由 category descriptor 提供，而不是散落在 match 中。

### P1-06：factory 只有闭包，没有代码/ABI/allocator/provenance contract（Open）

证据：`ServiceFactory` 和 `PluginFactory` 是 `Arc<dyn Fn(...) -> Result<ServiceObject, CoreError>>`；descriptor debug 只输出 name/startup/dependencies。无法在 failure receipt 中识别 binary build、allocator、source package、capability 或 unload compatibility。

重构：factory 必须携带 provider manifest、BuildSet/ABI identity、allocation domain、thread/phase policy、reload support 和 teardown owner；dynamic provider 禁止只以 Rust closure 伪装静态生命周期。

### P1-07：1/2/3/4/5 service fast paths 与通用路径存在语义漂移风险（Partial）

证据：registration、startup resolution、dependency resolution、blocked unload、unload mutation、dependency slice 都重复展开 exact 1-5 分支；行为测试也按 exact cardinality 扩展。小对象优化没有配套证明所有路径在错误顺序、diagnostic、allocation 和 cancellation 上等价。

重构：先用统一 iterator/graph kernel 作为 correctness authority，再以经过基准证明的 inline-small representation 优化存储；不得让五套控制流拥有独立语义。

### P1-08：registered service index 是进程全局 AtomicU32，不是 runtime generation identity（Open）

证据：`registration/commit.rs:11-22` 使用 static `NEXT_REGISTERED_SERVICE_INDEX`，跨多个 CoreRuntime 共享递增空间；只在溢出时返回 exhausted。runtime 重建、并行测试、长时间 editor session 没有 per-runtime identity/epoch 语义。

重构：使用 runtime-owned identity allocator（BuildSet/runtime id + local index + generation），显式处理 exhaustion、serialization 和 cross-runtime handle comparison。

### P1-09：generation wrap 会重新使用旧值，理论上可复活极老 handle（Open）

证据：`service_entry.rs:115-121` 使用 `wrapping_add`，溢出回到 generation 1。没有 quarantine、epoch salt 或“generation exhausted 后永久废弃”策略。

重构：generation 使用不回绕的宽整数或 `(runtime_epoch, slot_epoch)`，耗尽时 retire slot 并返回 typed error；加入长时 recreate/serialization test。

### P1-10：module graph freeze 只冻结拓扑，不冻结 composition ownership（Partial）

证据：`RuntimeModuleCompositionPlan` 在 `composition/outcome.rs:17-24` 同时保存 modules 和 descriptors；`finish_runtime_module_composition` 以 `module.descriptor()` 生成 descriptor，再把 module/descriptor 分开存储。没有公共 invariant 证明二者的 lifetime、identity digest、factory set 始终相同。

重构：以一个 `CompiledModuleRecord` 同时持有 module object、descriptor snapshot、provider provenance 和 graph node；所有 consumers 从同一 record 读取。

### P1-11：host module append 与 plugin module append 没有统一 owner/namespace conflict receipt（Open）

证据：`RuntimeModuleCompositionCompiler::compile` 先 assemble plugin plan，再 `report.modules.extend(self.host_modules)`，最后才调用 graph freeze。重复名最终可能得到 generic `DuplicateModule`，但没有说明冲突 owner、source package、target profile 或选择原因。

重构：compose 阶段建立 source-labeled candidate set，先做 namespace/resolution policy，再生成 rejection receipt；host override 必须显式声明 replace/augment/deny。

### P1-12：cross-module service dependency 只允许直接声明边，策略没有被记录（Open）

证据：`module_order.rs` 在 service owner 不同时只检查 `node.owner_module` 的直接 `module_dependencies`。传递依赖、optional provider、profile-specific provider 没有 policy object，导致“显式声明要求”与“transitive closure 可用”混在一起。

重构：将 cross-module edge policy 编译进 graph（direct/through-closure/optional/provider capability），并在 rejection/diagnostic 中输出采用的 policy。

### P1-13：single activation 按模块逐个 commit，closure 不是 atomic transaction（Open）

证据：`activation.rs:31-41` 对 closure 逐模块调用 `run_module_lifecycle_transition`；每个模块在 `finish_module_activation` 后才进入 Running。另一个线程可观察到 dependency 已 Running、target 仍 Initializing/Registered 的中间窗口。

重构：为 single、batch、lazy activation 共用 graph transaction；先取得 closure lease，执行 prepare/build/resolve/ready/finish，再一次性发布 epoch 和 Running snapshot。

### P1-14：batch activation 只批量持有 token，不提供读侧 transaction barrier（Partial）

证据：`activation/batch.rs` 预先获取每个 module token，但 service resolution、observer callback 和外部 state query 仍可在 build/finish 之间运行；`LifecycleTransactionSet` 只负责 token completion，不锁住 graph read epoch。

重构：增加 `LifecycleReadEpoch`/transaction snapshot，读侧只能看到 committed generation；失败时发布一个完整 rollback receipt，而不是让调用者猜测哪些模块已经 callback。

### P1-15：activation failure 可能遗留非 startup dependency service（Open）

证据：single failure 只调用 `reset_started_services(startup_services)`，batch failure 也按每个 pending module 的 startup list reset。lazy dependency 在 factory 中被解析后，如果后续 ready/finish 失败且该 service 不在当前 startup list，当前 rollback 没有 owner list 可回收它。

重构：每个 activation transaction 记录所有 newly initialized service claims，按 dependency reverse order rollback；service claim 必须携带 transaction id 和 owner module。

### P1-16：cleanup failure 进入 Stopping 后没有 poison receipt/retry policy（Open）

证据：`deactivate_module_with_graph` 在 `cleanup_module` 返回错误时直接结束；模块保持 `Stopping`，service slots 尚未 invalidate，admission 已关闭。后续只能得到 invalid transition，无法查询失败 owner、重试条件或是否需要强制 abandon。

重构：引入 `Stopping { transaction, blocker }`/`Poisoned` 状态、failure ledger、retry/force-abandon API 和不可变 teardown receipt；失败必须可诊断、可恢复或明确进程级终止。

### P1-17：service teardown 仍没有独立生命周期 contract（Open）

证据：`ServiceEntry` 只有 instance/admission/in-flight/lifecycle，`unload_services` 直接 `invalidate_for_unload`；实际 worker、callback、GPU/OS handle 依赖 module `cleanup` 的隐含约定。service 没有 prepare/drain/cleanup/error provenance。

重构：增加 registration-owned `ServiceLifecycle`，提供 prepare-stop、drain(deadline)、release、poison，并把 service receipt 作为 module transaction 的子证据。

### P1-18：service call drain 只覆盖 owner module 的 service slots（Partial）

证据：deactivation 使用模块预计算的 `shutdown_service_names` 调用 `wait_for_service_calls_to_drain`。module cleanup 期间若仍调用依赖模块 service，依赖 slot 的 in-flight 不在当前 drain 集合中，安全性依赖 callback 自律和 reverse order。

重构：transaction 根据完整 service dependency closure 建立 drain set，先关闭 producer admission，再按反向 service DAG drain/cleanup；禁止 module cleanup 隐式跨 owner 调用未声明服务。

### P1-19：raw `Arc<T>` resolution 仍绕过 admission（Open）

证据：`resolve_driver/resolve_manager/resolve_plugin` 返回 `Arc<T>`；`ServiceHandle::enter` 才执行 generation/admission/in-flight 检查。manager facade `resolve_manager_service` 也返回 `Arc<T>`，因此调用方拿到对象后可以在 unload 之后直接调用。

重构：guarded handle 作为唯一公共 invocation API；raw Arc 只允许 registry-internal snapshot 或明确 immutable service。迁移所有 manager resolver 和 factory back-reference。

### P1-20：`ManagerServiceHandle` 解析后仍返回无 guard 的 Arc（Open）

证据：`core/manager/service.rs` 的 `ManagerServiceResolver::resolve` 先校验 identity，再返回 `RegisteredManagerService::shared()`；之后的 method call 不再经过 `ServiceCallGuard`。这只解决 stale identity，不解决调用期间停机。

重构：manager trait 访问器返回 `ManagerCallGuard<T>` 或 scoped closure；禁止将 service Arc 暴露给 editor/runtime session 长期持有。

### P1-21：resolution wait graph 没有 deadline、cancel、owner diagnostic（Open）

证据：`resolve_existing_service_inner` 在 service Initializing 时以 ThreadId->ThreadId map 等待 condition variable；没有 deadline 参数、cancel token、wait edge receipt 或 stalled owner stack。cycle 可被检测，但 hung factory 只能永久阻塞。

重构：resolution request 携带 deadline/cancellation/trace id；waiter census 和 factory owner stack进入诊断；超时返回 typed `ServiceResolutionTimeout` 并触发 claim rollback。

### P1-22：resolution recursion stack 与 dependency walk 仍有 exact 1-5 specializations（Partial）

证据：`resolution.rs` 的 `resolution_stack_contains` 和 `resolve_dependency_services` 对 1-5 个依赖展开，再走通用 Vec。结构测试验证这些分支存在，但没有等价性/分配/深度压力证明。

重构：统一迭代器和小数组存储，保留单一 cycle/ordering authority；加入 0/1/5/6/1024 深度与随机图 fault tests。

### P1-23：ready contract 是同步 bool，默认 zero timeout 对异步资源不友好（Open）

证据：`ModuleLifecycle::ready` 返回 `CoreResult<bool>`；`wait_until_module_ready` 在 false 后以 1ms `std::thread::sleep` 轮询，zero timeout 立即报错。没有 readiness future、wake source、progress 或 blocked dependency。

重构：改为 `ReadinessReceipt`/future + wake registration，deadline/cancellation 贯穿 transaction；禁止在 UI 或 runtime worker 上进行 OS sleep polling。

### P1-24：module callback context 没有 transaction/deadline/capability scope（Open）

证据：`ModuleContext` 只携带 module name 和 `CoreWeak`；build/ready/finish/cleanup 无 cancellation、phase token、service claim、resource scope。callback 可重新获取任意 Core API，导致 owner boundary 由约定而非类型保证。

重构：引入 phase-scoped `ModuleLifecycleContext`，仅暴露声明的 capabilities、transaction id、deadline、cancel、diagnostic sink 和 owned service/resource scope。

### P1-25：observer 是可覆盖的单槽，没有订阅 token 和多 owner ordering（Open）

证据：`CoreRuntimeInner` 使用 `Mutex<Option<Arc<dyn RuntimeModuleLifecycleObserver>>>`，`install_runtime_module_lifecycle_observer` 直接覆盖，`clear` 无 token；plugin bridge、diagnostics、editor/hot reload 只能竞争同一槽。

重构：由 lifecycle coordinator 管理有序 observer set，返回 registration token，支持 priority、remove、panic isolation、veto phase 和 callback receipt。

### P1-26：activated bridge callback 丢失结果，不能证明 provider 已切换（Open）

证据：`RuntimePluginBridgeLifecycleState::runtime_module_activated` 调用 `activate_provider_at_frame_boundary`，返回的 `RuntimePluginBridgeLifecycleReport` 被丢弃；只有 deactivating 路径把 block 变成 `CoreError`。module Running 与 native provider Applied 不是同一 commit。

重构：provider transition 作为 graph transaction participant，返回 typed report；activate failure、partial slots、frame boundary 和 rollback 必须进入统一 receipt。

### P1-27：普通 EngineEntry 丢弃 CoreRuntime owner，没有显式 shutdown API（Open）

证据：`engine_entry.rs:229-239` 创建 `CoreRuntime`、激活后只返回 `runtime.handle()`；`CoreRuntime` 没有 Drop 级 shutdown，`CoreHandle` 又可 clone。Arc drop 不能表达 deadline、failure、worker join、callback detach 或 dynamic unload quarantine。

重构：返回 `RuntimeOwner`/`ProductRuntimeLease`，显式 `shutdown(deadline)` 一次性 quiesce producer、停 task graph、反向停 module/service 并提交 receipt；Drop 只作为 fail-closed fallback。

### P1-28：BuiltinEngineEntry 的 descriptor override 与 module object 不一致（Partial）

证据：`BuiltinEngineEntry::bootstrap` 对 `descriptor_with_preference_storage_backend` 的副本替换 Platform factory，再注册副本；但 `modules()` 和 composition plan 仍保存原 `Arc<dyn EngineModule>`。selection receipt、module object descriptor 和 registered descriptor 可能不是同一 identity。

重构：在 composition 阶段生成最终 compiled records，factory override 只发生一次；object、descriptor、receipt 和 owner 都引用同一 record。

### P1-29：ProductComposition 仍是 CoreHandle + side owners 的集合，不是 shutdown authority（Open）

证据：`product_composition/composition.rs` 保存 `core`、plugin bridge、compiled plan、native host，但没有 `shutdown` 或 teardown state；注释只描述 drop order。native host、Core、plugin catalog 的相互依赖没有可查询 terminal receipt。

重构：ProductComposition 持有唯一 `RuntimeOwner`，所有 side owner 以 child lease 注册；shutdown 顺序由 owner 驱动并返回 immutable `ProductTeardownReceipt`。

### P1-30：dynamic session Drop 忽略 shutdown failure（Open）

证据：`dynamic_api/session/state.rs:55` 把 `DYNAMIC_SESSION_DESTROY_DRAIN_TIMEOUT` 固定为 `Duration::ZERO`，`193-198` 用它关闭全部 registered modules；只要仍有 guarded call，关闭就会立即失败。`149-152` 的 `Drop` 又使用 `let _ = self.shutdown_before_library_unload()`；函数在 event mirror、watcher、task scope、module shutdown、task graph 任一失败时返回 false。外层 `RuntimeSession` 虽然对 destroy failure abort，但库内 dynamic session 本身没有相同的 fail-closed guarantee。

重构：dynamic session 必须显式 destroy state machine，并由 host deadline budget 派生 module drain deadline；不能把生产销毁策略硬编码为零等待。Drop 只能记录不可恢复 failure 并阻止 library release，不能静默继续析构。ABI destroy 返回 receipt/status，host 持有 quarantine 直到所有 callback/worker 停止。

### P1-31：Product shutdown phase 与 module-first teardown 顺序不一致（Open）

证据：`product_shutdown/phase.rs:33-41` 排列为 Quiescing -> Draining -> ReleasingPlatform -> DestroyingRuntime -> DeactivatingModules -> FlushingDiagnostics；而 dynamic session 先 shutdown registered modules，再 shutdown task graph。两个状态机表达不同的 owner contract。

重构：选择一个 canonical product teardown DAG：停止 ingress/producer -> drain tasks/services -> deactivate modules/providers -> release platform/GPU -> unload runtime library -> flush diagnostics；phase disposition 和 module receipt必须来自同一 coordinator。

### P1-32：lifecycle coordinator 的 Completed entries 和 waiter arithmetic 没有长期治理（Partial）

证据：`core_runtime_state.rs:49-69,87-124` 将完成结果保留在 `HashMap<String, Transition>`，直到下一次同命令覆盖；waiter 使用 `usize += 1`，无上限；等待没有 deadline/cancel。长时间 editor session 的 module recreate、异常 waiter 或高并发可能积累状态。

重构：使用 bounded operation journal + lease-count overflow check，完成后按 receipt retention policy 清理；所有 waiter 带 deadline/cancel/diagnostic，并在 shutdown 时强制收口。

## 4. P2 工程质量与性能差距

### P2-01：generation/index exhaustion 只做单元级边界，没有 long-run quarantine（Open）

加入 2^32 不能实际跑满不是理由；需要模拟 allocator state、序列化/反序列化和数百万次 reactivation，证明 stale handle 永不重新有效。

### P2-02：descriptor/debug 输出缺少 provider provenance（Open）

当前 debug 只打印名称、依赖和 startup mode，缺少 manifest、BuildSet、target、feature、factory source、allocator、thread affinity，故难以从日志重建 composition。

### P2-03：Noop lifecycle 让空模块“看起来已实现”（Open）

`ModuleDescriptor::new` 默认 `NoopModuleLifecycle` 且 ready=true；应该区分 marker-only module、service-only module 和需要显式 lifecycle contract 的 module，避免缺 hook 被误认为可运行。

### P2-04：registry names 是 Arc<str>，但 module identity 仍重复分配 String（Partial）

`RegistryName` 已缓存 offsets/kind，module graph 和 contexts 仍频繁 clone String。应在 canonical snapshot 中使用 interned IDs，诊断再映射字符串。

### P2-05：HashMap 读路径和排序路径没有规模预算（Open）

freeze、blocked unload、service resolution 在大型 graph 上反复构造 HashMap/Vec；缺 1K/10K module、100K service、随机依赖图的内存/p50/p95/峰值测量。

### P2-06：source-shape tests 证明了实现存在，不证明行为等价（Partial）

registration/activation/resolution 结构测试大量 `include_str!`/`.contains`，应保留少量 architectural guard，但把预算转给 controlled interleaving、fault injection、panic/timeout、resource leak 和 model-based state machine。

### P2-07：manager name constants 是跨模块字符串 API（Open）

`core/manager/service_names.rs` 以字符串常量代表 service identity；没有 generated ID/schema version，也没有 compile-time owner validation。高频 manager lookup 应使用 typed key，跨 ABI 才转 canonical string。

### P2-08：activation/deactivation 的 callback panic 诊断缺少 source location/stack/transaction id（Open）

当前只生成 `ModuleLifecycleCallbackPanicked { module, command }`；没有 callback phase、provider、operation id、thread、source package 或 prior state，难以定位生产 failure。

### P2-09：small-cardinality fast path 没有真实 allocation/p95 evidence（Partial）

已有 exact-count tests 和少量 contention benchmark，但没有把 registration、resolution、unload、shutdown 四条路径在 0/1/5/6/32/1K 规模统一测量，也没有证明 code-size/i-cache trade-off。

### P2-10：observer callback 是同步调用，缺少回调预算和诊断隔离（Open）

observer 在 lifecycle owner 线程同步执行，无法声明最大耗时、异步 completion 或 per-observer failure policy。应记录 callback duration，超预算进入 diagnostic ledger，并规定是否阻断 transition。

### P2-11：module status query 不足以支持 Unreal/Godot 级运维诊断（Open）

当前 public API 主要是 mutate/resolve，没有稳定的 module/service status snapshot、load reason、owner/provider、dependency closure、last transition receipt。编辑器和 crash report 不能只依赖内部 locks。

### P2-12：Unity RenderGraph 式 compile/execute/cleanup state guard 尚未映射到 module graph（Partial）

Unity `RenderGraph` 明确区分 BeginRecording、compile、execute、clear/cleanup，并在 active state 禁止非法 API。Zircon module graph 虽有 frozen snapshot，但 registration、activation、service resolution、teardown 的 active-state guard 仍分散，缺统一 illegal-transition diagnostics。

## 5. 参考引擎对照

- Bevy `Plugin` 明确 `build -> ready -> finish -> cleanup`，`App::finish`/`cleanup` 由 App owner 驱动，完成后禁止继续 add plugin。Zircon 已借鉴 hook 名称和 ready 概念，但没有 App-owned runtime transaction，也没有把 service resource、task scope 和 cleanup receipt纳入同一 owner。
- Godot `Main` 按 CORE/SERVERS/SCENE/EDITOR level 初始化并逆序 deinitialize；`GDExtensionManager` 在 unload 前发 unloading、调用 shutdown、按当前 level 逆序 deinitialize。Zircon 的 `InitLevel` 和 reverse module order 是局部相似物，但缺少 extension-level state、unload reason、callback detach 和统一 product phase。
- Fyrox 用 `PluginContainer::Static/Dynamic` 明确静态性能路径与动态库开发路径，Plugin 有 `register`、`on_deinit`、fixed update 和 post-update。Zircon 的 PluginFactory 只表现为闭包，未把 static/dynamic ownership、source copy、watcher 和 plugin object teardown 类型化。
- Unreal `FModuleManager` 提供 `LoadModuleWithFailureReason`、`QueryModule/FModuleStatus`、`UnloadModule`、`UnloadModuleAtShutdown`、Abandon 和 globally unique module name，说明 module status、失败原因、shutdown policy 和 query surface 本身就是工程合同。Zircon 目前只有 typed mutation errors，没有同等运维 status/owner/provenance surface。
- Unity Graphics `RenderGraph` 把 recording/compile/execute/cleanup 分开，维护 resource registry、compiled graph、validation layer 和 cache cleanup，并在 executing/recording 状态阻止非法 API。该模式适合迁移到 Zircon：module/service graph 应拥有 explicit phase state、compiled snapshot、participant receipts 和 cleanup barrier，而不是只靠几个 mutex 与回调约定。

## 6. 重构里程碑与验收门

| Milestone | 交付 | 退出条件 |
|---|---|---|
| M0 canonical declaration | `ModuleId`/`ServiceId`、immutable declaration、provider manifest、composition digest、typed status snapshot。 | composition、registry、App selection 和 dynamic session 使用同一 compiled record。 |
| M1 graph transaction | single/batch/lazy/shutdown 共用 graph transaction、read epoch、claim journal、cancel/deadline。 | 任意并发交错只产生一个确定的 commit/rollback receipt；无 partial Running 可见。 |
| M2 service lifecycle | service prepare/drain/release/poison、dependency-closure drain、guarded manager API。 | service 调用只能通过 generation/admission guard；每个 service 有独立 teardown provenance。 |
| M3 readiness and observers | event/future readiness、observer token set、callback budget、typed bridge participant。 | 无 1ms sleep polling；observer/bridge failure 可回滚或进入明确 poison state。 |
| M4 owner and dynamic unload | `RuntimeOwner`/`ProductRuntimeLease`、module-first product phase、dynamic unload quarantine、worker/callback receipt。 | 普通 App、Editor Play、dynamic ABI 都由同一个 owner contract 关闭，Drop 不静默吞错。 |
| M5 scale and hard cut | 移除 raw Arc public path、统一 cardinality kernel、loom/fault/long-run/1K-100K benchmarks。 | 通过 stale-handle、panic/timeout、recreate、multi-runtime、memory/p95 与 DLL reload gates。 |

| Gate | 当前状态 | 证据 |
|---|---|---|
| G01 immutable declaration and composition identity | Fail | public mutable descriptor；modules/descriptors 双存储。 |
| G02 module dependency policy/version/capability | Fail | dependency 只有 String/name。 |
| G03 service graph duplicate/missing/kind/cycle | Partial | `FrozenModuleGraph` 和 registration tests 已覆盖基础拓扑。 |
| G04 graph-wide activation atomicity | Fail | single closure 逐模块 commit，batch 无 read barrier。 |
| G05 activation claim rollback completeness | Fail | rollback 只覆盖 startup list，未记录所有 lazy dependency claims。 |
| G06 service guarded invocation default | Fail | raw `Arc<T>` 和 manager Arc 仍公开。 |
| G07 service teardown contract | Fail | `ServiceEntry` 无 service lifecycle hooks。 |
| G08 dependency-closure drain | Partial | owner service drain 存在，跨 owner call set 未闭合。 |
| G09 event-driven readiness | Fail | `std::thread::sleep(1ms)` polling。 |
| G10 observer registration ownership | Fail | replaceable single Option observer。 |
| G11 bridge lifecycle receipt | Fail | activate report 被丢弃，非同一 transaction。 |
| G12 status/query/provenance | Fail | 无 Unreal/FModuleStatus 等稳定 snapshot。 |
| G13 explicit ordinary owner shutdown | Fail | EngineEntry 只返回 CoreHandle。 |
| G14 dynamic session drop fail-closed | Fail | Drop 忽略 shutdown bool。 |
| G15 product/module phase consistency | Fail | Product phase enum 与 dynamic module-first顺序冲突。 |
| G16 cleanup failure poison/retry | Fail | Stopping 无 receipt/retry/abandon policy。 |
| G17 lifecycle coordinator bounded journal | Partial | epoch/reentrant/waiters 有基础，retention/deadline/overflow 缺失。 |
| G18 static/dynamic plugin ownership | Fail | PluginFactory closure 无 dynamic/static owner contract。 |
| G19 module context capability scope | Fail | context 只提供 CoreWeak。 |
| G20 registry ID exhaustion/wrap policy | Fail | global AtomicU32 + wrapping generation。 |
| G21 exact cardinality equivalence | Partial | behavior/source tests 有，统一 model/benchmark 无。 |
| G22 controlled interleaving/fault/scale | Fail | 未完成 loom、fault、1K-100K、soak。 |
| G23 reverse module/service order | Pass (local) | frozen graph reverse lists 与 deactivation tests。 |
| G24 veto-before-cleanup | Pass (local) | veto atomicity tests。 |
| G25 panic-to-typed lifecycle error | Partial | typed error 有，source/transaction provenance 不足。 |
| G26 plugin/module composition rejection | Partial | duplicate/fatal diagnostics 有，owner/source policy 不足。 |
| G27 manager typed identity | Partial | generation identity 有，resolved Arc 无 call guard。 |
| G28 runtime task graph adoption | Partial | dynamic session 调用 task graph，但 process/private producers仍外置。 |
| G29 diagnostics flush ordering | Fail | product phase、dynamic session、process log 三套局部顺序。 |
| G30 implementation readiness | Fail | 仅 review 完成，未进入 code change/verification。 |

## 7. 评审边界

本轮未查询、轮询、等待或实时跟踪协调器，遵循用户要求。读取了 Runtime module/service descriptor、registry state、graph freeze、registration、activation/deactivation、resolution、manager facade、engine module/composition、plugin bridge、App EngineEntry/ProductComposition、product shutdown、dynamic session 和对应行为/结构测试；参考了 Unreal、Bevy、Fyrox、Godot 和 Unity Graphics 的 module/plugin/render-graph lifecycle contracts。报告只记录当前工作树差异和后续重构方向，不把局部 graph validation、guarded handle 或 tests 误报为完整引擎生命周期。实现前必须重新读取 HEAD、working-tree diff、owner inventory、dynamic ABI 和所有 producer/worker/callback 证据。
