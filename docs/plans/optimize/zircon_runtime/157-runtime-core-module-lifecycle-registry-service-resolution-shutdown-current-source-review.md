---
title: Runtime Core Module Lifecycle、Registry、Service Resolution、Activation、Shutdown 与 Dynamic Session 当前源码复核
category: zircon_runtime
report_id: Runtime157
review_date: 2026-08-29
baseline_head: b2e76ff33cc298ad76f7b801a1d06d1e2faa046d
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
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation
  - zircon_runtime/src/core/runtime/handle/registration
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
reference_engines:
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/godot/main/main.cpp
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
---

# Runtime157 当前源码审查

## 1. 结论

本轮刷新 Runtime01 的 CoreRuntime/module registry/service resolution 生命周期，不把已有问题重复纳入总账。物理选择集为 99 个文件、18,253 行、716,286 bytes、143 个测试属性、2 个 ignored；当前工作树中该选择集有 28 个 tracked 修改、8 个 untracked。选择集指纹为 `a4e58145352ddae23e871d17ff2d303797417db5260646e6f4cfa6993bddf713`。参考树为 8 个文件、12,658 行、469,610 bytes，指纹 `fdb09b2eedf7b6c82e66daf294a847545403d0f95c3748df3a37e3f84febd9ab`。

相较 Runtime01，源码已加入 immutable `FrozenModuleGraph`、module dependency closure、service dependency validation/topological order、generation-bound `ServiceHandle`/`ServiceCallGuard`、in-flight drain、prepare-before-veto、module transition coordinator、reentrant/invalid-transition errors，以及 `shutdown_registered_modules_with_drain_timeout`。这些是可保留的工程底座，但不等于完整 runtime owner 生命周期：普通 `EngineEntry` 仍只返回可 clone 的 `CoreHandle`，没有自动拥有/提交最终 shutdown；裸 `Arc<T>` resolve API 仍公开；ready 仍在显式等待路径中以 1ms OS sleep 轮询；service cleanup 仍由 module hook 间接承担；lifecycle observer 仍是可覆盖的单槽。

本轮重判：P0 **0 Open / 2 Partial / 1 Closed**；P1 **2 Open / 1 Partial / 2 Closed**；P2 **2 Open / 2 Partial**。Runtime01 的 3/5/4 项数量不变，仅状态随当前证据更新，不新增唯一 finding。

## 2. 当前实现闭环

### 2.1 已形成的基础

- `LifecycleState` 具备 Registered、Initializing、Running、Stopping、Unloaded；`ModuleLifecycle` 具备 build/ready/finish/cleanup。
- `FrozenModuleGraph::freeze` 在第一次生命周期操作前冻结 module graph，并同时校验缺失/重复/层级错误、module cycle、service kind、缺失 service、跨 module 未声明边、service cycle，生成 activation 与 reverse service order。
- `activate_module` 使用同一 graph 的 `module_activation_closure`；批量 activation 用 `LifecycleTransactionSet` 获取每个 module transition token，并在失败时逆序 cleanup/reset。
- `LifecycleCoordinator` 以 epoch、command、owner、waiters 串行化同一 module；同线程重入返回 typed error，其他线程等待并共享完成结果。测试覆盖 concurrent activation、activation/deactivation contention、reentrant callback 和 batch token completion。
- deactivation 先检查 running dependents 与 blocked service，再调用 observer veto；只有 veto 成功后才写入 Stopping、关闭 service admission、等待 call drain、cleanup、invalidate service slot、进入 Unloaded。当前 `veto_atomicity` 测试验证 cleanup 不发生在 veto 之前。
- `ServiceHandle::enter` 通过 `CoreWeak`、registered index/generation 和 admission guard 约束调用；Drop 释放 in-flight count，服务槽在 drain 后才允许 invalidate。旧的 `resolve_* -> Arc<T>` 仍保留为公共旁路。

### 2.2 生产调用路径

`EngineEntry::bootstrap` 的两个生产路径仍是 `CoreRuntime::new -> register_module* -> activate_registered_modules -> CoreHandle`。`CoreRuntime::shutdown_registered_modules_with_drain_timeout` 已存在，并且 dynamic session destroy 路径在 `zircon_runtime/src/dynamic_api/session/state.rs` 调用它；但普通 App/host owner 没有把该 shutdown 绑定到唯一 owner、process close、task graph、event producer、plugin bridge、GPU/OS surface 与 persistence flush 的共同 receipt。

dynamic session 的 teardown 已能报告 module shutdown 错误并阻止后续库卸载，这是重要进展；它仍把 event mirror、process log、project watcher、module shutdown 分散在多个 owner/阶段，缺少一个可证明“所有 producer quiesced、所有 service calls drained、所有 worker stopped、所有 callbacks detached”的统一 transaction。

## 3. 差距清单

### P0-1：普通产品 owner 仍不能证明统一反向停机（Partial）

dynamic session 已调用 `shutdown_registered_modules_with_drain_timeout`，并按 active order 逆序 deactivate；但普通 `EngineEntry` 没有等价的 owner-managed shutdown。`CoreHandle` 可被 clone，Arc drop 不能表达最后 owner、shutdown deadline、failed cleanup、worker/callback residue 或动态库 unload quarantine。结论从 Open 降为 Partial，不是 Closed。

必须建立 `RuntimeOwner::shutdown(deadline)`：停止新 admission，quiesce producer，停止 task/event/plugin bridge，按 module/service reverse DAG drain，执行 cleanup，invalidate handles，提交不可变 shutdown receipt；Drop 只能 fail-closed 后备，不能静默代替显式 shutdown。

### P0-2：deactivation veto 的 cleanup 之后 rollback（Closed）

当前 `deactivate_module_with_graph` 先执行 running-dependent/service blocker 检查，再调用 `notify_runtime_module_deactivating`，成功后才写 Stopping 和 cleanup。`veto_atomicity.rs` 覆盖 observer veto、registered module、running dependents 和 shutdown order，证明拒绝时不会产生 cleanup 副作用。旧问题已闭合；后续不得把 observer 重新放回 commit/cleanup 之后。

### P0-3：生命周期转换缺乏串行化/合法迁移（Partial）

同 module 的 transition 已由 `LifecycleCoordinator` 的 epoch/owner/waiter/reentrant 检查保护，非法状态有 typed `InvalidModuleLifecycleTransition`；batch activation 也持有 transaction set。剩余风险是 `activate_module` 对 dependency closure 逐 module 完成 transition，而不是将整条 closure 作为一个跨 module atomic commit：另一个 operation 可在依赖已 Running、目标尚未 commit 的窗口介入。shutdown 也以连续单 module operation 提交，缺少全局 lifecycle epoch/operation receipt。应把 closure/batch/shutdown 统一成可取消的 graph transaction，或显式公开中间状态。

### P1-1：裸 `Arc<T>` 仍绕过 generation-aware call admission（Partial）

`resolve_driver/manager/plugin` 仍返回 `Arc<T>`，虽然新增 `resolve_*_handle` 和 `ServiceCallGuard`。外部持有裸 Arc 后，registry generation、admission close 和 unload 不能阻止直接调用；ServiceFactory 仍收到 `&CoreHandle`，服务还可形成 service -> core -> registry -> service 强环。应将 guarded handle 变为默认 public API，裸 Arc 限定为 registry-internal/静态生命周期对象，并审计所有 factory back-reference。

### P1-2：single-module activation 绕过 dependency closure（Closed）

当前 `activate_module` 先取得 `FrozenModuleGraph::module_activation_closure`，再按同一全局 order 激活闭包；lazy service resolution 通过 owner module activation 进入这条路径。`module_order_tests` 覆盖 closure filtering。旧的 batch/single semantic split 已闭合，但仍需把 closure 作为 P0-3 的一个 transaction unit。

### P1-3：service dependency validation 与 shutdown reverse graph 不完整（Closed）

`module_order.rs` 已验证 duplicate/missing/kind/cross-module declaration/service-cycle，并生成按 service DAG 的稳定 topological order 和 reverse `shutdown_service_names`；测试覆盖 same-kind reverse order、missing dependency、manager-to-plugin、undeclared cross-module 和 cycle diagnostic。旧 finding 已闭合。新增 service kind 或 descriptor 时必须继续通过 graph freeze，禁止恢复仅按 Driver/Manager/Plugin 分类的旁路。

### P1-4：service 没有独立的内核 shutdown/cleanup contract（Open）

`ServiceEntry` 只有 admission/in-flight/instance/lifecycle 状态；`unload_services` 使 slot 失效，实际线程、callback、GPU/OS resource 仍依赖 module `cleanup` 的约定。内核无法检查每个 service 是否完成 prepare/drain/cleanup，也无法为一个 service 输出独立 failure provenance。应增加 registration-owned `ServiceLifecycle`/destructor record，并明确 service hook 与 module hook 顺序和错误终态。

### P1-5：ready readiness 仍是 sleep polling（Open）

`wait_until_module_ready` 在 `ready` 返回 false 后使用 `MODULE_READY_POLL_INTERVAL = 1ms` 的 `std::thread::sleep`，没有 wake source、phase deadline、cancellation、blocked dependency diagnostics；zero timeout 的默认 product activation 仍把异步 ready 视为立即失败。应采用 condition/future/event-driven readiness，按 runtime owner deadline 传递取消和 blocker receipt，禁止占用 UI/task worker 轮询。

### P2-1：1-5 cardinality specialization（Partial）

registration、blocked unload、service list 和 unload mutation 仍为 1/2/3/4/5 service 写出大量 `unreachable!` 分支；6 项以上才走统一扫描。保留 small-inline storage 可能有价值，但当前没有完成 1/2/3/4/5/6/32/1K 的真实 allocation/p50/p95/shutdown-order benchmark。先以统一 graph correctness 为基线，再用 profile 证明局部 specialization 的收益。

### P2-2：结构文本测试与行为测试并存（Partial）

当前已有 concurrent/reentrant/veto/drain/reverse-order 行为测试，明显改善了 Runtime01 的 false-green 风险；同时 activation/registration/resolution 仍大量使用 `include_str!`、`.contains` 和 source-shape guard，无法替代 loom/controlled interleaving、fault injection、worker/resource leak 与 dynamic session integration。应把结构守卫降为边界检查，把预算转向可重复状态机与资源寿命测试。

### P2-3：lifecycle observer 是可静默覆盖的单槽（Open）

`install_runtime_module_lifecycle_observer` 直接写入 `Mutex<Option<Arc<dyn RuntimeModuleLifecycleObserver>>>`，后安装者覆盖前者，clear 也没有 owner token/generation。plugin bridge、diagnostics、editor 和 hot reload 若同时需要通知，会发生隐藏 owner 冲突。应由 coordinator 持有有序 observer set，具备 token、phase/priority、remove、panic/error policy 与 reentrancy 规则；veto 仅允许 prepare phase。

### P2-4：既有计划仍可能把局部底座写成 code-complete（Open）

旧 framework/runtime 计划不能只用 descriptor、source-shape tests 或新增 API 数量标记 lifecycle 完成。当前仍缺普通 owner shutdown、service teardown contract、async readiness 和 cross-module transaction。`zr_kernel` crate migration、plugin hot reload、editor/runtime session 必须依赖本报告的行为和 receipt gates。

## 4. 参考引擎对照

Bevy `Plugin` 文档把 build/ready/finish/cleanup 作为 App-owned lifecycle，并禁止在 plugin build 期间重入 App update；Zircon 应吸收阶段顺序和 owner 约束，但保留 Rust 的 explicit ownership。Godot `Main` 对 servers 按层初始化并在 cleanup 中反向 deinitialize，GDExtension unload 先发 unloading、逐 level deinitialize，再清 callbacks；这对应 Zircon 的 graph reverse order、quiesce 和 callback detach。Unreal `FModuleManager` 区分 load/unload result、compatibility、module status 与 change reason，说明 lifecycle result 不能只用 bool/Arc existence。Fyrox dynamic plugin 明确由库加载对象持有 library、source copy 和 watcher，且 reloading 是 unsafe boundary；Zircon 不能把 dynamic destroy 成功等同于所有 service 已安全卸载。

## 5. 重构里程碑与验收门

| Milestone | 交付 | 退出条件 |
|---|---|---|
| M0 owner/shutdown inventory | 找出每个 CoreRuntime/CoreHandle owner、producer、worker、callback、GPU/OS resource 与 dynamic unload path。 | 普通与 dynamic session 都有唯一 shutdown owner；无 silent drop。 |
| M1 graph transaction | closure/batch/single/shutdown 共用 graph transaction、epoch、cancel、receipt。 | 并发、重入、跨 module interleaving 具备确定结果。 |
| M2 service teardown | registration-owned service prepare/drain/cleanup、failure/poison state、guarded handle default。 | 旧 generation 不能新调用；所有 service 有 teardown provenance。 |
| M3 async readiness | wake/future readiness、phase deadline、cancellation、blocked diagnostics。 | 无 `thread::sleep` readiness polling；UI/task worker 不被占用。 |
| M4 dynamic qualification | real DLL/session create-destroy/reload、fault/worker residue、callback detach、unload quarantine。 | destroy failure 不卸载库；receipt 可审计。 |
| M5 performance and hard cut | 统一 graph + 可证明 inline-small fast path、loom/fault/soak、旧 API 迁移。 | 1K/10K module/service 与长时 recreate 通过预算。 |

| Gate | 当前状态 | 证据 |
|---|---|---|
| G01 普通 owner 显式 shutdown | Partial | dynamic session 有 shutdown，普通 EngineEntry 无统一 owner。 |
| G02 reverse module/service order | Pass | frozen graph、reverse service order 与 shutdown tests。 |
| G03 veto 无 cleanup 副作用 | Pass | `veto_atomicity` 行为测试。 |
| G04 transition serialization/reentrancy | Partial | coordinator/epoch/reentrant tests；closure 非 atomic。 |
| G05 dependency closure single/batch 一致 | Pass | `module_activation_closure` 与 batch transaction。 |
| G06 service dependency graph validation | Pass | missing/kind/cross-module/cycle/reverse-order tests。 |
| G07 guarded service call/drain | Partial | handle/guard/in-flight 已有，裸 Arc 仍公开。 |
| G08 service teardown provenance | Fail | 无独立 service lifecycle contract。 |
| G09 event-driven readiness | Fail | 仍 1ms `thread::sleep`。 |
| G10 observer owner/token set | Fail | 单槽 Option observer 可覆盖。 |
| G11 dynamic DLL unload qualification | Fail | 未完成真实 DLL/fault/worker/callback receipt。 |
| G12 scale/soak/controlled interleaving | Fail | 未完成 1K/10K 与长时资源寿命证据。 |

## 6. 评审边界

本轮读取 CoreRuntime lifecycle、descriptor/frozen graph、activation/registration/resolution、module/service state、activation behavior tests、App EngineEntry、runtime-library session 与 dynamic session teardown，并对照 8 个 Bevy/Fyrox/Godot/Unreal 文件。没有修改 production Rust/test/Cargo/ABI，没有运行 Cargo、真实 DLL、跨进程 fault、reload、sanitizer、loom、scale、soak 或动态 benchmark。实现前必须重新获取 HEAD、working-tree diff、selection fingerprint 和 owner inventory；本报告只更新 Runtime01 状态，不声明 runtime lifecycle 已经工程化完成。
