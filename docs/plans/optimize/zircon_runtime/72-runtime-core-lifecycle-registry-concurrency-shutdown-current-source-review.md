---
title: Runtime Core Lifecycle、Registry、Concurrency、Service Quiescence 与 Product Shutdown 当前源码工程化差距
category: zircon_runtime
report_id: Runtime72
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
  - zircon_runtime/src/core/runtime/contexts
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_runtime/src/core/runtime/handle/registration
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/weak.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/entry_runner/editor/composition.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
tests:
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/tests/plugin.rs
  - zircon_runtime/src/core/runtime/tests/weak.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/core/extension/gdextension_manager.h
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/main/main.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/ContextContainer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven/GPUResidentDrawer.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 72 · Runtime Core Lifecycle、Registry、Concurrency、Service Quiescence 与 Product Shutdown 工程化差距

## 1. 结论

当前 Core Runtime 已经不是最初报告所描述的空壳。`FrozenModuleGraph` 会校验 module/service 缺失依赖、初始化层级、跨模块声明、service kind 顺序和环，并预计算 module activation closure、dependent closure、service startup/shutdown 拓扑；单模块激活已经补上依赖闭包，批量激活有逆序 cleanup，deactivation veto 已移到副作用之前，`ServiceHandle`/`ServiceCallGuard` 也提供 generation 与 in-flight drain。动态 Runtime Session 现在会先停 event mirror 和 watcher，再逆序关闭 module，host 侧销毁失败会保留 session 供重试或在最终 `Drop` 中 fail-stop。这些是真实底座，应保留并强化，不能退回字符串 service locator、无状态 callback 或只靠 `Arc` 析构的实现。

但这套底座仍未形成工程级生命周期内核。本轮确认6项 Runtime72 独有 P0：同一 lifecycle waiter 会因任意 module 的全局 `notify_all` 被重复计数并留下陈旧结果；单模块依赖闭包逐项取得 transition，依赖完成后可被并发卸载，使 dependent 最终 Running 而 dependency 已 Unloaded；service instance 在持有 registry mutex 时析构，合法的 `Drop` 回调可重入 Core 并自锁；批量激活先取得部分 token 后若后续取得失败，不会释放先前 token；静态 `EngineEntry`/Editor composition 只保留 `CoreHandle` 并直接 drop，完全绕过 module cleanup；全局 shutdown 又按“全部声明模块”而非“实际启动 ledger”遍历，合法部分激活会被一个 `Registered` module 提前中止。

另登记18项 P1、8项 P2 与40项资格门。Runtime46 的 lazy factory panic 永久卡住 `Initializing` 是既有唯一 P0 owner；Runtime50 的裸 `Arc` resolver 绕过 call guard、Runtime24 的 process-global index/generation、Runtime42 的 composition truth、Runtime43 的 dynamic FFI/session 上层边界均只作为依赖引用，不在本篇重复累计。完成并发模型、fault injection、真实静态/动态产品 shutdown、DLL unload、soak 和同硬件性能证据前，不能宣称生命周期可靠性或性能达到、更不能宣称超过当前 Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner 边界

| 领域 | Canonical owner | Runtime72 责任 | 不重复登记 |
|---|---|---|---|
| Module graph、transition、activation、deactivation | Runtime01 / Runtime72 current-source follow-up | transaction identity、闭包原子性、状态机、rollback、shutdown ledger | Runtime42 的 builtin catalog/profile/capability 组合 |
| Service slot、factory、handle、call lease | Runtime46 / Runtime50 | activation/unload 对 service lifecycle 与 quiescence 的集成 | Runtime46 lazy panic P0、Runtime50 resolver facade 的完整合同 |
| Identity、generation、epoch | Runtime24 | lifecycle 采用 runtime-scoped、不回绕的身份与 stale 拒绝 | 全引擎 handle/index/epoch 收敛 |
| Dynamic Session / FFI / DLL unload | Runtime43、Interface01、App06 | Core shutdown receipt 与 quiescence 下限 | ABI handle、allocation、action barrier、host gateway 父问题 |
| Static App / Editor composition | App01、Editor application owner | 提供 owned runtime close，并消费 Core shutdown report | entry profile、window loop、Editor retained-host 产品组合 |
| Concurrency / transaction qualification | Tooling24/35/37 | lifecycle model、fault/soak/linearizability gate | 通用工具、evidence archive 与 CI control plane |

本篇是对 legacy Runtime01 的 current-source 增量复审，不接管其计划族，也不修改其开放 failure。Runtime01 仍是历史 owner；本篇新增的 `RCL-*` finding 只拥有当前实现中新发现、且未被其它报告登记的缺陷。

### 2.2 Zircon 物理冻结

本轮逐文件聚焦109个 Zircon 文件，共19,925行、777,486 bytes；其中63个 production/product 文件，入选测试含131个 Rust test attribute。以排序后的 `path=per-file SHA-256` 逐行 LF 连接、末尾无 LF 再做 SHA-256，working-tree 指纹为 `44ab9cf76588998fb1c2011277facfc3b5c949255f1a857d155d85ab1169a42c`。冻结时18个入选路径 dirty，结论绑定当前共享 working copy，不把 HEAD 单独当成源码事实。

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Lifecycle vocabulary、descriptor、context、state、registration | 36 / 4,367 / 160,961 | graph freeze、注册事务、service identity、module/service state 与 callback surface |
| Activation、resolution、observer、runtime facade | 20 / 3,508 / 129,301 | single/batch transition、rollback、call admission、drain、unload、shutdown |
| Runtime behavior/structure tests | 46 / 8,556 / 357,947 | 115个 test attribute；并发 join、veto、rollback、reactivation 与 resolver 行为 |
| App/Editor/Dynamic product owner | 7 / 3,494 / 129,277 | 6个 test attribute；static bootstrap/drop、dynamic teardown/retry、FFI fail-stop |
| 去重合计 | **109 / 19,925 / 777,486** | manifest fingerprint 如上；18个路径 dirty |

本轮只做 review，不修改 production/tests，不运行 Cargo、动态库、Editor、fault、soak、loom 或 benchmark。两个 Runtime01 open failure 都只有源码形态变化与“受管验证待执行”记录，没有本轮新测试证据，因此保持 open。

### 2.3 参考物理冻结

参考侧12个文件、13,783行、518,624 bytes，按同一 manifest 算法得到指纹 `07c90c6448b988e0504d3807ffc60e1fa5c7c9ba02b2b58db4caff5e5e2e0f91`。

| 参考 | 本轮采用的工程事实 | 对 Zircon 的约束 | 不外推的内容 |
|---|---|---|---|
| Unreal | `IModuleInterface` 区分 startup、pre-unload、shutdown、dynamic reload、automatic shutdown；`FModuleManager` 维护已加载实例、变更事件、unload/abandon 与实际启动/关闭顺序 | 必须有真实 loaded ledger、reload policy、pre-stop/quiesce/cleanup/retire、main-thread/owner 约束与可观测事件 | 不照搬全局 singleton、C++ ABI 或具体热重载实现 |
| Bevy | App owner 驱动 plugin build、ready、finish、cleanup 与 runner 状态，`PluginsState` 明确阶段 | composition owner 必须持有 runtime lifetime，ready/finish/cleanup 是 app progression，不应由偶然最后一个 `Arc` 触发 | Bevy 本身不是 dynamic unload 基线，不据此要求相同 plugin model |
| Fyrox | Plugin 提供 init/deinit，dynamic plugin loader 明确 reload/unload 与 library lifetime | 动态扩展必须先停调用与回调、执行 deinit、退休实例，再释放 library | 不复制其 scene/UI 对象模型 |
| Godot | Extension manager 按初始化层级 initialize/deinitialize，Main 有显式全局 teardown 顺序 | shutdown 必须分层、逆序、可重复且由产品 owner 调用，不能用声明图替代实际 active set | 不复制 Godot Object/RenderingServer singleton 架构 |
| Unity Graphics | `ContextContainer.Dispose` 明确释放所拥有项；GPU resident owner 在 cleanup/code unload 中逐项释放 GPU/CPU 资源 | 即使资源最终支持 RAII，owner 仍须有显式、可测试、顺序化的 dispose/cleanup 边界 | Unity Graphics 只作为 resource-owner 证据，不冒充完整 module manager 参考 |

## 3. 相对 Runtime01 已实质推进的底座

| Runtime01 历史结论 | 当前源码状态 | 本篇处理 |
|---|---|---|
| 产品 shutdown 不进入 module cleanup | Dynamic Runtime Session 已显式 shutdown；静态 App/Editor 仍缺失 | 缩窄为 RCL-P0-005，不再声称所有产品路径都缺失 |
| deactivation veto 在 cleanup 之后 | observer veto 已在 `Stopping`、admission close、cleanup 前执行 | 记为已修正底座，不保留旧 P0 |
| module lifecycle 完全未串行化 | `LifecycleCoordinator` 已按 module/command/epoch 串行和 join | 旧缺口关闭为源码形态；新协调器缺陷由 P0-001/004 接管 |
| 单模块激活不激活依赖闭包 | `module_activation_closure` 已进入 public single activation | 旧缺口关闭；闭包非原子由 P0-002 接管 |
| service dependency 不校验/不排序 | frozen graph 已校验 missing/cycle/kind/cross-module declaration 并提供 topo startup/shutdown | 保留实现，后续只优化执行与 receipt |

开放交接 `failure-2026-08-17-concurrent-module-activation-preflight-rejects-join.md` 的源码路径现在允许 `Initializing` 进入 coordinator，测试源码也有并发 join gate；`failure-2026-08-16-runtime-core-dependent-collection-type.md` 所需的 `Vec<String>` 类型也已出现在当前源码。但两份交接都要求受管 focused/original/upward 验证，本轮未运行 Cargo，所以不得改名、移动或标记 `fixed`。

## 4. 六项新增 P0

### RCL-P0-001：全局 Condvar 唤醒会重复登记同一 waiter，并把旧 transaction 结果留给未来调用

`LifecycleCoordinator::begin` 每次看到同命令 `InFlight` 都执行 `waiters += 1`；`acquire_module_lifecycle_transition` 从 Condvar 醒来后会重新调用 `begin`。所有 module 又共享一个 `lifecycle_transition_changed`，任何 module completion 都 `notify_all`。因此等待 A 的同一个调用会因 B 完成或 spurious wake 被再次计入 A；owner 把膨胀后的计数复制到 `Completed`，真实 waiter 只消费一次，残余旧结果留给未来同命令调用。

成功结果常被状态幂等性掩盖，失败结果却会被重复回放：原始 activation 已 rollback 到 `Registered`/`Unloaded`，外部条件修复后若再次 activate，仍可能在不执行 build 的情况下收到旧错误，直到虚假 waiter 计数耗尽。计数还没有 checked overflow、operation ID、每 waiter ticket 或 per-module predicate。

必须让每个调用只注册一次 waiter，并等待明确 `(module, epoch)` terminal state；推荐使用 operation record + waiter ticket/sequence，或 per-transition Condvar/channel。完成后删除以裸计数模拟 receipt 的状态，加入 unrelated module completion、spurious wake、失败后立即重试和多轮 activate/deactivate 模型测试。

### RCL-P0-002：单模块 activation closure 不是一个原子 transition 集合

`activate_module(B)` 只在循环前验证一次 `[A, B]` 状态，然后分别执行 A transition 和 B transition。A 完成到 B 进入 `Initializing` 之间没有 closure lease。并发 `deactivate_module(A)` 只阻止 lifecycle 为 `Running` 的 dependent；此时 B 仍可为 `Registered`，所以 A 能 cleanup/unload，原调用随后仍可把 B 标记为 `Running`。若 B 的 callback 恰好不解析 A service，最终状态就是 dependent Running / dependency Unloaded。

批量激活已经证明“先取得整个 transition 集合”是可行方向，但 single activation 没有复用同一 transaction。必须以稳定拓扑顺序一次取得 dependency closure token，预检和 commit 都验证 graph generation；deactivation 必须把 `Initializing`、quiescing 和 pending dependent transaction 视为 live dependent。任何失败都逆序释放 token，不能靠第二次状态检查缩小窗口。

### RCL-P0-003：service instance 在 registry mutex 内析构，合法 Drop 重入可自锁

deactivation 持有 `services` mutex 调用 `unload_services`，`invalidate_for_unload` 直接执行 `self.instance = None`。失败 activation/reactivation 的 `instance.take()` 也在同一锁内。service/plugin factory 接收 `CoreWeak`，所以扩展对象的 `Drop` 合法地可以 upgrade 并解析、注销或记录另一个 service；若 registry 中最后一个 `Arc` 在这里释放，析构会重入同一 mutex 并永久自锁。panic 还会留下部分 slot 已失效、module 仍 `Stopping` 的混合状态。

必须在锁内只做 slot CAS、generation bump 和 `Option::take`，把 retired instances 收集到有序 batch，释放 mutex 后才执行 service stop/cleanup 和最终 Drop。析构重入要么由明确的 retire phase 支持，要么以 typed policy 拒绝，不能依赖“Drop 通常不调用 Core”。

### RCL-P0-004：batch 取得部分 transition token 后遇到取得错误会永久泄漏 token

`activate_registered_modules_with_ready_timeout` 在循环中用 `?` 取得每个 module token；只有显式 `Completed(Err)` 分支会完成已取得 token。若当前 lifecycle callback 已拥有后序 module B 的 activation，然后重入 batch activation，batch 可先为前序 Running module A 创建新 owner token，再在 B 收到 `ModuleLifecycleCommandReentrant` 并由 `?` 直接返回。A token 从未 complete，之后 A 的 activate/deactivate 都会永远等待这个已不存在的 owner。

必须用 RAII transaction set 持有所有已取得 token：成功 commit terminal result，任何 return/panic/drop 都以结构化 abort result 完成并唤醒 waiter。加入“single B callback -> batch activate -> typed reentrant error -> A/B 后续命令仍可运行”的确定性测试，不能只测同一 module 立即重入。

### RCL-P0-005：静态 App/Editor bootstrap 丢失 runtime owner，产品关闭绕过 module cleanup

`EngineEntry::bootstrap` 和 `BuiltinEngineEntry::bootstrap` 创建 `CoreRuntime`、激活全部 module 后只返回 `CoreHandle`。`EntryRuntimeBootstrap`、`NativePluginRuntimeBootstrap` 与 `RetainedHostRuntimeLease` 也只拥有 handle，没有 `close`/`Drop` shutdown contract；`EditorApplicationComposition::close` 和 retained-host 返回路径直接 `drop(core)`。`CoreRuntime` 可 clone 且自身没有 Drop shutdown，`CoreHandle` 又不暴露全局 shutdown，所以静态 editor/runtime/native-plugin module 的 cleanup 永远不会由产品 owner 调用。

必须让 composition root 持有不可克隆的 `OwnedCoreRuntime`/`RuntimeLease`，借用方只拿 weak/typed access；owner `close(deadline)` 返回 `RuntimeShutdownReport`，Drop 只能做有记录的 fail-stop fallback。Native plugin host 必须晚于 module/service retire 和 callback quiescence 释放。不得把 shutdown 加到任意 `CoreHandle::drop`，否则多 owner 无法判定最后责任方。

### RCL-P0-006：shutdown 遍历声明图而非实际启动 ledger，部分激活会阻断全部清理

`shutdown_registered_modules_with_drain_timeout` 逆序遍历 frozen graph 的全部 `module_activation_order`，遇到 `Registered` module 时 `deactivate_module_with_graph` 返回 invalid transition，shutdown 立即 `?` 返回。公共 API 允许只激活一个 module dependency closure；若反向顺序先遇到从未启动的独立 module，真正 Running 的 module 将完全不执行 cleanup。

必须维护实际成功达到 Running 的 activation ledger，并在 shutdown 时逆序处理该 ledger；`Registered`/已 `Unloaded` 应作为显式 skipped outcome，而不是阻断 unrelated cleanup。单个 module 失败不能抹掉其余 module 的 cleanup 结果，最终 report 要包含 cleaned/skipped/failed/blocked/remaining 与 deadline consumption。

## 5. 目标架构

| 组件 | 所属 | 责任 |
|---|---|---|
| `RuntimeCompositionBuilder` | App/Core boundary | 注册 module/service/capability，显式 `seal()` 生成 immutable plan 与 diagnostics receipt |
| `FrozenRuntimePlan` | Core descriptors | graph generation、module/service topo、activation/dependent closure、reload policy 与 stable identity |
| `LifecycleTransactionSet` | Core lifecycle | 按稳定顺序取得 closure token，RAII abort/complete，支持同 operation join 与相反 command 排队 |
| `ModuleRuntimeRecord` | Core state | declared、prepared、initializing、running、quiescing、stopping、failed、retired/abandoned 与 terminal cause |
| `ActivationLedger` | Core runtime | 记录实际成功 startup/finish/publish 顺序，shutdown/reload 只消费 active generation |
| `ServiceSlot` + `ServiceCallLease` | Core service | generation、admission、in-flight、retire state；调用必须持 lease，slot detach 与对象 Drop 分离 |
| `ModuleLifecycleExecutor` | Product/Core | callback affinity、safe point、deadline、cancel、panic boundary 与不可重入策略 |
| `RuntimeLifecycleEventStream` | Core diagnostics | operation ID、epoch、phase、latency、result、rollback、remaining work 和 bounded history |
| `RuntimeShutdownReport` | Core/App/FFI | per-module/service outcome、deadline、blocked calls、cleanup failure、DLL-retirement readiness |
| `OwnedCoreRuntime` | App composition | 唯一 close 责任方；向子系统发布 weak/typed access，不把裸 handle 当 owner |

状态和锁的最低规则：registry lock 内不得执行 user callback、factory 或对象析构；同一 transaction 的 preflight/lease/commit 使用同一 graph generation；任何 token、guard、observer subscription 和 native callback 都有 generation 与 bounded terminal path；错误 rollback 不得把未知清理状态伪装成可安全重试的 `Registered`。

## 6. P1 差距与重构完成定义

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RCL-P1-001 | `built` 只在 `build()` 返回 Ok 后置位；callback 做了部分副作用再返回 Err 时不调用 cleanup | build 改为 prepare/commit 或返回 compensation token；任何部分副作用都有确定性 rollback |
| RCL-P1-002 | activation cleanup rollback 失败后仍把 module/service 恢复为 `Registered`/`Unloaded` | 进入 Failed/Poisoned，保留原错误与 compensation 错误；只允许显式 recover/abandon |
| RCL-P1-003 | service 只有 factory 与最终 `Arc` Drop，没有 prepare/stop/cleanup/retire 合同 | service descriptor 提供受 deadline/affinity 管理的生命周期，module cleanup 不再猜 service 内部状态 |
| RCL-P1-004 | 生产 resolver 大量返回裸 `Arc<T>`；close admission 只约束 `ServiceHandle::enter` | Runtime50 完成 canonical call lease；新 resolve 在 Stopping 后拒绝，所有产品调用纳入 drain |
| RCL-P1-005 | drain timeout 只覆盖已登记 guard，observer、module cleanup、service cleanup 可无限阻塞 | 一个 operation deadline 贯穿 veto、drain、cleanup、retire、notify；每阶段报告剩余预算 |
| RCL-P1-006 | shutdown 遇首个错误立即返回，丢失其它独立 module 的清理机会和状态 | best-effort dependency-safe continuation，返回完整 report；hard dependency blocker 与 independent failure 分开 |
| RCL-P1-007 | `ready()` 在调用线程每1ms sleep/poll，默认 timeout 为零 | event/future 驱动 readiness，支持 wake、cancel、deadline、progress 与无忙轮询测试 |
| RCL-P1-008 | batch 对每个 module 重用完整 `ready_timeout`，总耗时可放大为 N 倍 | API 明确 total deadline 与可选 per-module slice，receipt 记录每阶段 budget consumption |
| RCL-P1-009 | build/ready/finish/cleanup/observer 没有 main/render/worker affinity 与 safe-point 合同 | descriptor 声明 executor/phase，错误线程调用被拒绝；product loop 提供确定性 safe point |
| RCL-P1-010 | activation observer 在 module 已 Running 后通知；panic 会 rollback Core，但已通知的外部 side effect 无 inverse | publish 是 transaction phase，observer 返回 receipt/compensation；batch 后序失败逆序撤销已发布项 |
| RCL-P1-011 | runtime lifecycle observer 是可静默替换/clear 的单 slot | generational subscription set，install/remove 有 token、quiescence 和 duplicate/priority policy |
| RCL-P1-012 | `LifecycleState` 只有五态，无法表达 prepared、quiescing、failed、retiring、abandoned | module/service 分离状态机，terminal cause 与 recovery policy 可查询、可序列化到 diagnostics |
| RCL-P1-013 | graph 在首次 lifecycle/resolve 时隐式冻结，没有显式 seal receipt | composition root 显式 seal 并获得 graph generation、diagnostics、capability 与 owner snapshot |
| RCL-P1-014 | 所有 module 默认可 unload/reactivate，没有 permanent、reloadable、abandon、code generation 策略 | descriptor 声明 unload/reload policy；不支持 reload 的 module fail-close，动态 code owner 有 generation |
| RCL-P1-015 | shutdown 不记录实际 startup/finish/publish ledger，也不区分 build 完成但未 publish 的对象 | ledger 覆盖每个 phase，rollback/shutdown 针对真实已提交阶段逆序补偿 |
| RCL-P1-016 | independent module build、startup service resolution 与 ready 全部串行 | correctness 先闭合，再按 DAG level 并行；保持 deterministic publication 与 bounded worker budget |
| RCL-P1-017 | lifecycle 只有即时 `Result`，无 operation ID、phase latency、failure history、retry/remaining state | Runtime03/O11 接入 bounded typed event/receipt，支持 crash/evidence 关联但不泄露敏感 callback 数据 |
| RCL-P1-018 | service index 使用 process-global `AtomicU32`，generation 可 wrapping 回初始值 | Runtime24 收敛为 runtime-scoped non-reusing identity/epoch；耗尽 typed fail，不把 wrap 当新对象 |

## 7. P2 工程债

| ID | 当前差距 | 收敛方向 |
|---|---|---|
| RCL-P2-001 | registration、dependency match、startup、unload 对1至5 cardinality 大量手写特化 | 用 profile 证明收益；保留必要 fast path，其余由生成/共享 iterator 实现并做语义等价测试 |
| RCL-P2-002 | 多个 structure test 只用 `include_str!` 断言源码形状 | 结构 guard 只守边界，正确性改为公开行为、状态模型与 fault tests |
| RCL-P2-003 | 无 loom、状态机/property test 或线性化 history checker | 为 coordinator、closure transaction、service call/unload 建小状态模型并覆盖调度交错 |
| RCL-P2-004 | coordinator 每 module 永久保留 Completed record，缺 retention/metrics | terminal waiter 清空后回收或压缩为 bounded history，暴露 live/stale operation 数 |
| RCL-P2-005 | `ready()` 可能被重复调用，但幂等、线程安全与副作用合同未写明 | readiness 改为 query/subscription；若保留 callback，明确 pure/idempotent 要求并验证 |
| RCL-P2-006 | lifecycle trait 无 schema/API version 或 capability negotiation | descriptor snapshot 带 contract version/capabilities，插件不兼容在加载前拒绝 |
| RCL-P2-007 | 并发 join 只测理想等待集合，未覆盖 unrelated notify、失败重试、mixed command | 建 deterministic scheduler fixture，禁止用 sleep 扩大窗口冒充修复 |
| RCL-P2-008 | 当前 contention gate 只有绝对750ms，没有 workload/hardware/baseline 分布与资源成本 | 记录环境、样本、P50/P95/P99、CPU、allocation、锁等待，并与 correctness gate 绑定 |

## 8. 继承阻塞与非重复计数

| Owner | 已有事实 | Runtime72 依赖 |
|---|---|---|
| Runtime46 | lazy service factory panic 会跳过 reset/notify，使 slot 永久 `Initializing` | G17 必须先通过；本篇不另计 P0 |
| Runtime50 | manager/driver/plugin 产品调用仍大量返回裸 `Arc`，未进入 `ServiceCallGuard` | G18/G19 要求 canonical lease；本篇只登记 shutdown integration P1 |
| Runtime24 | process-global index 与 wrapping generation | identity gate 由 Runtime24 实施，本篇消费其 runtime-scoped identity |
| Runtime42/App01 | 多层 descriptor/composition 与 static bootstrap owner | 本篇定义 seal/owned close 接口，不复制 catalog/profile finding |
| Runtime43/Interface01/App06 | dynamic session registry、FFI action barrier、allocation 与 DLL owner | 本篇只提供 Core shutdown/quiescence receipt，ABI 与 host 重试由上层 owner 验收 |

## 9. 重构里程碑

### M0 · Current-source failures 与模型冻结

- 保持两个 Runtime01 failure open，完成 focused/original/upward 受管验证后才返回 fixed；
- 为6项 P0 写最小 deterministic reproduction 和状态序列；
- 冻结 module/service state machine、lock order、callback affinity 与 shutdown report schema。

### M1 · Coordinator 与 closure transaction

- waiter ticket/operation epoch 替代重复计数；
- single/batch activation 统一 `LifecycleTransactionSet`，所有退出 RAII complete/abort；
- dependency closure activation 与 dependent unload 在同一 graph generation 上线性化。

### M2 · Service quiescence 与 rollback

- canonical `ServiceCallLease` 进入全部生产调用；
- slot detach、service stop/cleanup、Drop 分阶段且锁外执行；
- lazy factory panic、partial build、cleanup failure 进入 Failed/Poisoned 和显式 recovery。

### M3 · Owned runtime 与 shutdown ledger

- `OwnedCoreRuntime` 持有实际 activation ledger；
- static App/Editor/native plugin 与 dynamic session 共用 `close(deadline) -> RuntimeShutdownReport`；
- cleanup/retire 完成前禁止卸载 native library，失败可重试或 fail-stop。

### M4 · Readiness、affinity、event 与 diagnostics

- callback executor/safe point、event-driven readiness、total deadline/cancel；
- generational observer subscription 与可补偿 publish；
- bounded lifecycle event/history、per-phase latency 和 terminal receipt。

### M5 · 性能与产品资格

- correctness/fault/loom/soak 先通过，再做 DAG parallel startup 与 fast-path 优化；
- static Editor、linked dynamic、native DLL、headless 与 export product 都执行真实 open/close/reopen；
- 同硬件、同 workload 对比 startup/shutdown latency、CPU、RSS、allocation、lock wait，证据绑定 source/build。

## 10. 资格门

| Gate | 验收内容 |
|---|---|
| RCL-G01 | composition 显式 seal，返回 frozen plan generation 与完整 graph diagnostics |
| RCL-G02 | missing/cycle/init-level/kind/cross-module edge 在任何 callback 前 fail-close |
| RCL-G03 | 同命令 join 在 unrelated notify 与 spurious wake 下每 caller 只登记/消费一次 |
| RCL-G04 | 失败 operation 的外部条件修复后，下一次调用执行新 transaction 而非回放旧结果 |
| RCL-G05 | activate/deactivate mixed command 对同 module 线性化，无 lost wake 或永久 wait |
| RCL-G06 | single activation 一次取得整个 dependency closure 的 transaction lease |
| RCL-G07 | dependency activation 与 provider unload 竞争不能产生 Running dependent / Unloaded provider |
| RCL-G08 | batch token set 任意 return/panic/drop 都完成或 abort 全部已取得 token |
| RCL-G09 | lifecycle callback 重入 batch 返回 typed error 后，所有 module 后续仍可 activate/deactivate |
| RCL-G10 | 只激活一个闭包后 shutdown 能跳过未启动 module 并清理全部实际 Running module |
| RCL-G11 | activation ledger 精确记录 build/ready/finish/publish 的已提交 generation |
| RCL-G12 | 正常 shutdown 按实际成功 activation 的逆依赖序执行，顺序有行为测试 |
| RCL-G13 | deactivation observer veto 前后零状态、service admission 和 generation 变化 |
| RCL-G14 | activation publish 后序失败会逆序补偿 observer side effect |
| RCL-G15 | registry lock 内不执行 factory、lifecycle callback、observer、service cleanup 或对象 Drop |
| RCL-G16 | 最后一个 service `Arc` 的 Drop 重入 Core 不死锁，结果符合明确 policy |
| RCL-G17 | Runtime46 lazy factory panic focused/original/upward gates 通过且 slot waiter 被唤醒 |
| RCL-G18 | production resolver/caller 不再绕过 canonical ServiceCallLease |
| RCL-G19 | close admission 后无新调用进入，已有调用 drain 后才开始 service retire |
| RCL-G20 | 一个 total deadline 贯穿 wait、observer、cleanup、retire、notification |
| RCL-G21 | hung cleanup 到期返回 typed incomplete report，不无限占用产品关闭线程 |
| RCL-G22 | startup/shutdown 支持 cancel，terminal state 与 compensation 明确 |
| RCL-G23 | 某 module cleanup 失败时继续处理 dependency-safe independent modules并汇总 report |
| RCL-G24 | rollback cleanup 失败进入 Failed/Poisoned，不伪装 Registered/Unloaded |
| RCL-G25 | recover/retry/abandon/quarantine 有 policy、generation 与 audit receipt |
| RCL-G26 | service prepare/start/stop/cleanup/retire 顺序和错误合同有行为测试 |
| RCL-G27 | permanent/reloadable/abandonable module policy 在 code load 前校验 |
| RCL-G28 | callback executor/thread affinity 错误调用 fail-close，safe point 可验证 |
| RCL-G29 | readiness 由 wake/future 驱动，无1ms caller-thread polling |
| RCL-G30 | batch startup 总 deadline 不随 module 数量无界放大 |
| RCL-G31 | DAG parallel startup 保持 deterministic publish/rollback，并受 worker/memory budget 限制 |
| RCL-G32 | observer install/remove 使用 generational token，remove 会等待 in-flight callback quiesce |
| RCL-G33 | static `EngineEntry` 产品 close 真实调用每个已启动 module cleanup |
| RCL-G34 | Editor composition/retained host 正常、错误、panic 路径都消费 shutdown report |
| RCL-G35 | native plugin library 在 service/module/callback retirement 前不可释放 |
| RCL-G36 | dynamic session destroy 失败保留可重试对象，最终 owner fail-stop 有诊断证据 |
| RCL-G37 | build/ready/finish/observer/cleanup/service Drop 每阶段 fault injection 无沉默半状态 |
| RCL-G38 | coordinator/closure/service quiescence 的 loom/model/property tests 覆盖关键交错 |
| RCL-G39 | startup/shutdown benchmark 报告 P50/P95/P99、CPU、allocation、lock wait、RSS 与环境 |
| RCL-G40 | focused Runtime01/46/50、App static/dynamic、Editor product gates、links、fingerprint 和 diff check 全通过 |

## 11. 实施禁止项

- 不得用 sleep、扩大 timeout、固定线程调度或减少 joiner 隐藏竞态。
- 不得在 `CoreHandle::Drop` 猜测最后 owner，也不得因 `Arc::strong_count` 恰为1就执行 shutdown。
- 不得在 registry mutex 内调用扩展代码或析构扩展对象。
- 不得用兼容 facade 保留裸 `Arc` 与 call lease 两套等价生产路径。
- 不得把 cleanup error 打日志后恢复为 Running/Registered/Unloaded。
- 不得把声明 graph reverse 当成实际 startup ledger。
- 不得为了更快而删除 deadline、receipt、rollback、quiescence 或 fault gate。

## 12. 状态

| 工作 | 状态 | 证据 |
|---|---|---|
| 109文件 current-source 与 product owner 复审 | review_complete | 19,925行、777,486 bytes、131个 test attribute、18个 dirty path、fingerprint 如上 |
| 五类参考 source 对照 | review_complete | 12文件、13,783行、518,624 bytes、fingerprint 如上 |
| 新增 finding | review_complete | 6 P0 / 18 P1 / 8 P2，40项资格门 |
| Runtime01 open failure | validation_pending | 源码形态已变化；本轮无 Cargo 证据，不关闭、不移动 |
| Production/tests 重构 | pending | 本轮 review-only，未修改源码或测试 |
