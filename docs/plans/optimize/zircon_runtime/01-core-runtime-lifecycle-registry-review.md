---
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
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
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

# 01 · Core Runtime 生命周期与 Registry 工程化差距

## 1. 结论

当前 runtime kernel 已具有 descriptor、module dependency sort、四阶段 `ModuleLifecycle`、lazy service resolution、service generation、跨线程 resolution cycle 检测和局部 rollback，不能归类为“纯临时 demo”。但它尚未形成工程级生命周期闭环，`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md` 中对 lifecycle code-complete 的结论必须按 current source 重开。

本轮确认 3 项 P0、5 项 P1 和 4 项 P2。最高风险集中在：生产停机不进入 module cleanup、deactivation 失败后状态与副作用不一致、模块状态转换可并发重入。它们是 plugin hot reload、动态库安全、长期运行和可靠测试的下层阻塞项，应先于更高级 plugin SDK 或 editor extension 功能修复。

本篇只审查 module/service lifecycle 与 registry。runtime tasks、events、diagnostics 仍未完成深审；报告中的 shutdown coordinator 会规定它们的接入点，但不替代后续专篇。

## 2. 当前实现闭环

### 2.1 已建立的基础

- `LifecycleState` 有 `Registered/Initializing/Running/Stopping/Unloaded` 五态，`ModuleLifecycle` 有 `build/ready/finish/cleanup` 四阶段（`lifecycle.rs:37-90`）。
- `ModuleDescriptor` 声明 init level、module dependencies、lifecycle 和 driver/manager/plugin descriptors。
- 批量 activation 调用 `sort_module_activation_order`，先 build 全部模块，再 startup service、ready、finish 和 commit；失败时逆序 cleanup 已 build 模块（`activation/batch.rs:19-80,183-205`）。
- service resolution 有 per-thread initialization owner、wait graph、跨线程 dependency-cycle 检测和 commit 时 generation 校验（`handle/resolution.rs:221-330`）。
- blocked unload 能拒绝仍被已实例化外部 service 依赖的 owner service（`activation/blocked_unload.rs:14-84`）。

这些机制值得保留为迁移输入，但状态数量或测试数量本身不能证明并发和卸载语义完整。

### 2.2 生产调用链

普通 host 和 builtin host 的 `EngineEntry::bootstrap` 都执行 `CoreRuntime::new → register_module* → activate_registered_modules → handle`，随后只返回 cloneable `CoreHandle`（`zircon_app/src/entry/engine_entry.rs:159-168,366-389`）。全仓 production 调用点未发现 `deactivate_module` 或“按依赖反向 shutdown all”；该 API 的调用主体是 runtime tests。

动态 session 在 construction 中同样调用 `activate_registered_modules`。其 `Drop`/`shutdown_before_library_unload` 只关闭 plugin event mirror、project watchers 和 process log（`zircon_runtime/src/dynamic_api/session/state.rs:119-162`），没有驱动 module `cleanup` 或 registry unload。Host-side `RuntimeSession::drop` 会调用 FFI `destroy_session`，失败时为避免 DLL worker 残留而 abort（`zircon_app/src/entry/runtime_library/runtime_session.rs:491-522`），但 dynamic-side destroy 成功并不能证明所有 module/service 都进入了反向生命周期。

## 3. 差距清单

### P0-1：产品停机没有进入统一的反向模块生命周期

**证据**

- 两个 `EngineEntry::bootstrap` 仅 activation 后返回 handle；production 搜索没有 `deactivate_module` 调用。
- `RuntimeDynamicSession::shutdown_before_library_unload` 的成功条件只由 event mirror 与 process log 组成，project watcher 是手工特例；其余模块的 `ModuleLifecycle::cleanup` 没有被调用。
- `CoreRuntime/CoreHandle` 没有拥有“最后一个产品 owner 关闭全部模块”的协议；普通 `Arc` drop 只能释放对象，无法保证线程、callback、GPU/OS 资源先停止。

**后果**

模块作者在测试中实现的 cleanup 在真实退出路径可能永远不执行。对进程退出这会产生日志/持久化/flush 丢失；对同进程 session 重建和动态库卸载，则可能留下仍执行旧代码的 callback/worker，破坏 host 侧“destroy 成功即可安全卸载 DLL”的前提。

**目标契约**

由 `zircon_app` 持有唯一 `RuntimeOwner`，`CoreHandle` 只是借用能力。`RuntimeOwner::shutdown(deadline)` 必须执行：停止 admission → quiesce 外部 producer → 逆依赖 pre-shutdown → drain in-flight calls/tasks → service cleanup → module cleanup → invalidate handles → release library/resources。Drop 只能作为 fail-closed 后备，并记录/终止无法证明安全的动态库卸载，不能静默替代显式 shutdown。

### P0-2：deactivation veto 发生在 cleanup 之后，rollback 恢复了错误状态

**证据**

`deactivate_module` 先把 module 写为 `Stopping`，随后执行 `cleanup_module`，再调用可返回错误的 `notify_runtime_module_deactivating`（`handle/activation.rs:92-133`）。observer 拒绝时，代码只把 lifecycle 从 `Stopping` 恢复为先前状态；已发生的 cleanup 副作用不可逆。

现有 `core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` 测试注册的是无 lifecycle 的空模块（`tests/plugin.rs:155-200`），只验证 state 和 bridge 仍在，无法发现真实 cleanup 已执行，因此形成 false green。

**后果**

模块会被标记回 `Running`，但内部线程、订阅或资源可能已停止/释放；后续调用表现为 use-after-cleanup、双 cleanup 或部分功能失效。这不是普通错误处理不足，而是状态机原子性被破坏。

**目标契约**

deactivation 必须分成无副作用的 `prepare/veto` 与不可回退的 `commit`。所有 dependency/live-object/bridge 检查先完成；进入 commit 后不再恢复 `Running`。cleanup 失败要进入显式 `StopFailed/Poisoned` 终态并阻止重新解析，而不是伪装成成功运行。

### P0-3：同一模块的生命周期转换没有串行化或合法迁移校验

**证据**

`activate_module` 只对 `Running` 返回；任何其他状态都直接改为 `Initializing`，释放 modules mutex 后调用 lifecycle（`handle/activation.rs:28-68`）。`deactivate_module` 同样把任意状态直接改为 `Stopping`（`:94-107`）。批量 activation 也会把所有非 Running 项改为 Initializing（`activation/batch.rs:97-119`）。

因此两个线程可以先后取得旧状态并都执行 build；activate 与 deactivate 也可能分别在锁外执行 build/cleanup。service resolution 对 service 有 initialization owner/wait graph，但 module transition 没有对应 owner、condition variable、epoch 或 cancellation token。`CoreError` 也没有 invalid transition/concurrent transition 类别。

**后果**

重复注册 callback、重复启动线程、build 与 cleanup 并发、commit 覆盖较新状态。错误依赖时序，难以复现，尤其会在 lazy resolution、hot reload 与并行 editor/runtime 工作流中放大。

**目标契约**

每个 module slot 持有 transition epoch、owner、phase、result 和 waiters。所有 public lifecycle command 进入单一 `LifecycleCoordinator`；重复同向请求共享结果，冲突请求等待或取消，非法迁移返回 typed error。lifecycle callback 在锁外运行，但 commit 必须以 epoch compare-and-commit 完成。

### P1-1：公开强 `Arc<T>` 使卸载不可撤销，factory 还可形成 runtime 强环

`resolve_driver/manager/plugin` 直接返回 `Arc<T>`（`handle/resolution.rs:27-35,80-83`），registry unload 只对 slot 调用 `invalidate_for_unload`（`activation/unload_mutation.rs:50-53`）。外部已持有的 Arc 继续可调用旧对象；generation 只保护再次通过 registry identity 解析的内部路径。

普通 `ServiceFactory` 接收 `&CoreHandle`，服务可 clone 成强 handle（`descriptors/service_factory.rs:7-8`），而 registry 又用 `Arc<dyn Any>` 持有服务（`service_object.rs:6`），因此契约允许 service → core → registry → service 环。Plugin factory 使用 weak context 是更合理的方向，但三类 service 并不一致。

目标是 generation-aware `ServiceHandle/CallGuard`：停止 admission 后新调用失败，in-flight call 可计数 drain，旧 handle 在 generation 变化后得到 `Unloaded/Stale`，服务只能持 `CoreWeak` 或受审计的 scoped capability。确实需要裸 Arc 的内部零开销热路径必须由生命周期静态约束或 benchmark 证明，不得作为默认公共模型。

### P1-2：module dependency 只约束 activate-all，单模块 activation 绕过 dependency closure

`activate_registered_modules` 会排序所有 descriptor（`activation/batch.rs:83-87`），但 `activate_module` 直接 build 目标模块，不读取 `module_dependencies`。lazy service resolution 只在 owner module 为 `Registered` 时激活 owner（`handle/resolution.rs:277-292`），也不先激活 module dependency closure。

这造成同一 descriptor 在批量启动与按需启动时具有不同语义。deactivation 也只检查已实例化 service dependency，不检查 module-level dependent 是否 Running。目标必须用同一个 immutable dependency graph 服务单模块 closure、批量启动、按需解析和 reverse shutdown。

### P1-3：service dependency 校验不完整，shutdown 不是依赖图反序

registration validation 只禁止 Driver 依赖非 Driver（`registration/validation.rs:10-95`）。它没有在完整模块集提交时统一验证：

- Manager 是否错误依赖 Plugin；
- dependency target 是否存在；
- 重复 dependency；
- 跨模块 service dependency 是否由 module dependency 声明；
- service dependency cycle 的全图诊断。

shutdown list 仅按 Plugin → Manager → Driver 分类，分类内部保持 descriptor 顺序；只有单一 kind 时甚至直接复用 owner order（`registration/service_lists/shutdown.rs:6-56`）。`first_blocked_unload` 跳过 unload set 内部 dependent（`blocked_unload.rs:64-79`），所以同 kind A→B 能否按 A 后 B 清理完全依赖声明顺序，而不是图。

目标是在 descriptor freeze 时建立 module/service 两层 DAG，输出确定性的 activation topological order 与 shutdown reverse-topological order。缺失 target、层级违规、cycle 和未声明跨模块边必须在任何 lifecycle callback 前一次性报告。

### P1-4：service 没有受内核约束的 shutdown/cleanup 协议

Service descriptor 只有 factory，实例类型擦除为 `Arc<dyn Any + Send + Sync>`。unload 只失效 registry entry，没有 per-service `prepare_shutdown/drain/cleanup`，内核无法证明 service 的线程、callback、文件/GPU 资源已停止。Module `cleanup` 可以手工做这些工作，但没有 service 清单/状态上下文强制一一覆盖，也无法处理外部 Arc。

目标应为受控的 `ServiceLifecycle` 或 registration-owned destructor record，并明确 module hook 与 service hook 的顺序。热路径服务对象可以保持类型擦除，但 teardown metadata 不能被擦除。

### P1-5：默认产品启动不支持异步 readiness，显式路径用阻塞轮询

默认 `activate_module`/`activate_registered_modules` 传零 timeout；`ready()` 首次返回 false 就立即 `ModuleReadyTimeout`。只有调用带 timeout 的变体才会每 1ms `std::thread::sleep` 轮询（`activation/module_lifecycle.rs:9-43`）。

这使 `ready` 阶段名义上支持异步，默认产品路径实际上只接受同步 ready；显式等待又占用 OS thread，没有 wake、取消、分阶段 deadline 或阻塞原因诊断。目标是事件/condition/future 驱动 readiness，runtime owner 提供整体 deadline 和 cancellation，各 module 提供可观测 blocker；严禁在 task worker 或 UI owner thread 上固定轮询。

### P2-1：1-5 元 cardinality 微特化扩大了关键内核的审计面积

registration 子树约 2,470 行，其中 `descriptor_entries_three/four/five.rs`、`service_lists/specialized.rs` 和 `register_module.rs` 大量展开 1-5 个 service/dependency 组合；blocked unload、unload mutation、validation 也重复相同分支。相关测试使用源文本断言锁住“不得 Vec/HashSet”和特定分支形状。

当前没有与这些分支绑定的端到端 benchmark 证明收益，却显著扩大生命周期内核的正确性矩阵。目标先建立 registry build、resolve hit/miss、并发 lazy init、unload graph 的基准，再硬切为可审计的统一算法（例如 small inline storage + graph index）。只保留 profile 证明有效且不复制状态语义的局部特化。

### P2-2：结构文本测试掩盖了缺失的状态机行为测试

activation/registration/resolution 共 96 个 test，但有 94 次 `include_str!` 和 641 次 `.contains(...)`。这些守卫适合约束禁止路径和 canonical owner，不适合证明并发、异常与资源寿命。

必须删除与具体局部分支数量绑定的守卫，把预算转向 model-based state-machine tests、loom/可控 interleaving（若依赖政策允许）、fault injection 和 product shutdown integration。保留的结构守卫只检查架构边界，不检查实现语句。

### P2-3：module lifecycle observer 是可静默替换的单槽扩展点

`install_runtime_module_lifecycle_observer` 写入 `Mutex<Option<Arc<_>>>`，后安装者覆盖前者。随着 plugin bridge、diagnostics、editor 和 hot reload 都需要生命周期通知，单槽会制造隐藏 owner 冲突。

目标不是无界全局 event bus，而是 coordinator 拥有的 ordered observer set：注册 token、优先级/phase、卸载、panic/error policy 和重入规则明确；veto observer 只允许出现在 prepare 阶段。

### P2-4：既有计划的 code-complete 状态与 current behavior 不一致

Frameworks02 已记录大量结构守卫和局部里程碑，但当前源代码仍存在上述 P0/P1。后续不得继续往该文件追加“测试数量增加”的完成记录；应重开 lifecycle milestone，以本篇行为验收矩阵覆盖原有 source-shape acceptance。Frameworks01 的 `zr_kernel` 迁移也不能先把不稳定契约原样封装进新 crate，再宣称边界完成。

## 4. 参考引擎证据与适用边界

| 参考 | 已核对机制 | Zircon 应吸收 | 不应误读 |
|---|---|---|---|
| Bevy | `Plugin` build/ready/finish/cleanup（`plugin.rs:16-104`）；App 统一聚合 finish/cleanup（`app.rs:232-295`）；placeholder + `catch_unwind` 保护 plugin registry 重入/失败（`:536-567`） | phase ownership、callback 外 registry mutation safety、reentrancy tests | Bevy plugin cleanup 不是动态热卸载协议，不能证明 Zircon Arc 卸载安全 |
| Fyrox | Static/Dynamic `PluginContainer`、`prepare_to_reload/reload`（`plugin/mod.rs:59-98`）；dylib 显式 Unloaded/Loaded 状态（`dylib.rs:217-306`）；`on_deinit` | reload 前准备、状态迁移、plugin state handoff、动态库边界 | 不能直接复制 panic-on-unloaded accessor；Zircon 应返回 typed stale/unloaded error |
| Godot | Core→Servers→Scene→Editor 初始化，退出反向分层清理（`main.cpp:789-845,5245-5327`）；GDExtension reload 先 prepare/unload/clear binding，再 reopen/init/finish（`gdextension_manager.cpp:81-110,158-209`） | 显式层级、反向停机、reload transaction、清理 instance binding 后再换库 | Godot 的全局单例/宏注册不是 Zircon Rust ownership 目标 |
| Unreal | module 启动完成前 `bIsReady=false`，完成后广播（`ModuleManager.cpp:980-1120`）；unload 先 not-ready，再 Shutdown/destroy（`:1316-1405`）；shutdown 按实际 load order 逆序并有 PreUnload（`:1437-1494`）；live-object/unload callbacks（`ModuleManager.h:769-783`） | readiness publication、reverse actual order、pre-unload/live-object gate、广播时点 | Unreal 注释也承认手工 unload 的 dependency 风险；不能声称其自动解决所有依赖或线程安全 |

共同原则不是 API 命名，而是：对象在“可发现”和“可调用”之间有明确发布点；卸载先停止新访问，再排空活对象/调用；清理顺序由实际依赖/装载关系决定；动态库关闭前必须证明旧代码不再执行。

## 5. 目标架构

### 5.1 核心对象

- `RuntimeOwner`：产品级唯一 owner，拥有显式 shutdown；不能从普通 service clone。
- `LifecycleCoordinator`：串行化 module graph transaction，拥有 transition epoch、deadline、cancellation、observer phases 和 journal。
- `FrozenModuleGraph`：所有 descriptor 完成后一次构建，包含 module/service DAG、init level、startup/reverse shutdown order 和诊断 provenance。
- `ModuleSlot`：stable identity + generation + state + transition result/waiters；状态至少区分 `Registered/Starting/Running/Quiescing/Stopping/Unloaded/Failed`。
- `ServiceSlot`：stable identity + generation + admission gate + in-flight counter + erased object + erased teardown record。
- `ServiceHandle<T>`：解析时不泄漏无约束 registry Arc；每次进入 call guard 校验 generation/admission，并在退出时递减 in-flight。

### 5.2 Shutdown transaction

```text
Running
  -> PrepareStop (dependency/live-object/veto checks; no side effects)
  -> Quiescing (close admission; cancel producers)
  -> Draining (wait in-flight/tasks with deadline)
  -> Stopping (reverse-topological service cleanup, then module cleanup)
  -> Unloaded (generation++, publish event)
```

任一步骤失败都必须有明确终态：PrepareStop 失败可回到 Running；Quiescing 之后不得假装回到 Running，必须 retry/force policy 或 `StopFailed`。所有事件必须在状态提交之后、锁外派发，并定义 observer panic/reentrancy 规则。

### 5.3 Hard cut

- 不保留直接 public `Arc<T>` resolve 与 generation-aware handle 两套长期 API；在调用方迁移完成后删除旧 resolve surface。
- 不保留单模块与 batch 两套生命周期语义；两者都调用同一 graph transaction。
- 不保留 1-5 元重复状态算法；benchmark 只允许保留不改变语义的 storage specialization。
- 不以 `Drop` 静默吞错兼容旧 host；所有 product owner 显式 shutdown，动态库路径 fail closed。
- 不新增 `legacy/compat/shim` 模块；跨 milestone 通过同一分支内调用方迁移完成硬切。

## 6. 测试先行重构里程碑

| 里程碑 | 先写的失败证据 | 实现范围 | 晋级条件 |
|---|---|---|---|
| M0 | cleanup-veto、activate/activate、activate/deactivate、产品 exit、held service reference 复现 | 只补行为 harness/fault injection | 每个 P0 有稳定红测，现有绿测不能规避副作用 |
| M1 | missing target、manager→plugin、跨模块未声明边、same-kind shutdown、module closure | `FrozenModuleGraph` + typed graph diagnostics | single/batch/lazy 使用同一 order，cycle provenance 可读 |
| M2 | 可控 interleaving、callback panic、reentrant resolve/activate、timeout/cancel | `LifecycleCoordinator` + transition epoch/journal | 同模块 callback 至多一次；冲突命令结果确定 |
| M3 | stale handle、in-flight drain、service→core cycle、stop deadline | ServiceSlot/Handle/admission/quiescence/teardown record | unload 后旧 handle 不可进入新调用，动态库前活调用为零 |
| M4 | builtin host exit、dynamic session destroy/recreate、partial failure | `RuntimeOwner::shutdown` 接入 app/dynamic API | 所有 production path 反向停机；失败不会报告可安全卸载 |
| M5 | registry/resolve/unload benchmark 与规模测试 | 删除 cardinality 状态特化，按 profile 决定 storage | 正确性矩阵收敛，性能无未解释回退 |
| M6 | workspace consumers、plugin bridge、editor/runtime integration | 移除旧 Arc/single-slot/旧 lifecycle API | 无 shim/双轨；上层回归和 plan guard 全通过 |

M0-M4 是 MVP 基础正确性；M5 的性能重写必须后置于语义收敛，但 benchmark harness 可在 M0 同时建立。

## 7. 验收矩阵

### 7.1 状态与并发

- 两线程同时 activate 同一 module：build/finish/notification 各一次，第二调用共享同一结果。
- activate 与 deactivate、batch 与 lazy resolve、shutdown 与 factory commit 的所有关键 interleaving 有确定结果。
- lifecycle callback 内 resolve/register/activate 的允许与禁止矩阵明确；禁止项返回 typed error，不死锁。
- build/ready/finish/cleanup、observer 和 service teardown 的 error/panic 都进入可诊断终态。

### 7.2 依赖与顺序

- single module activation 自动闭包依赖；缺失 target/level/cycle 在 callback 前失败。
- Driver/Manager/Plugin 层级和跨 module declaration 一致性在 graph freeze 校验。
- 同 kind 和跨 kind service 均按 reverse topology 停机，不依赖 descriptor 顺序。
- module dependent 或 live service handle 存在时，prepare stop 给出完整、稳定排序的 blocker provenance。

### 7.3 寿命与产品闭环

- 外部持有旧 service handle 时，quiesce 后不能开始新调用；已进入调用完成后才能 unload。
- service 尝试保存强 CoreHandle 时 registration/build 失败或类型层面不可表达。
- builtin runtime/editor、dynamic runtime library 和 session recreate 均证明 cleanup 顺序。
- destroy failure、deadline、worker 不退出、callback 残留时不得卸载 DLL；host 得到明确 fatal/retry 结果。

### 7.4 性能与规模

- 1/5/32/1K/10K modules/services 的 graph build、startup、resolve hit/miss、shutdown benchmark。
- 1/8/32 threads 的 concurrent resolve 与 lifecycle contention；记录 p50/p95/p99 和 allocation。
- 对比统一算法与现有 1-5 特化；只有测得的真实回退才允许局部优化。
- 长时 create/activate/shutdown/recreate 循环检测线程、handle、allocation 和 callback 泄漏。

## 8. 既有计划纠正

1. `frameworks/02-module-kernel-and-lifecycle-unification.md`：重开 lifecycle completion。原有四阶段 API/结构守卫可作为历史输入，但不得继续表示产品级 lifecycle 完成。
2. `frameworks/01-runtime-crate-decomposition.md`：`zr_kernel` 物理迁移应依赖 M1/M2 的稳定 graph/transaction contract，避免把错误语义固化为跨 crate API。
3. `runtime/06-plugin-surface-and-lifecycle.md`：plugin hot reload 必须依赖 M3/M4 的 quiescence、live handle 和 product shutdown；plugin 层不得另建平行卸载机制。
4. 后续 Runtime event/task/diagnostics 审查必须验证它们如何参加 admission、cancellation、drain 与 shutdown deadline。

## 9. 工作区复核标记

本轮读取期间 `handle/core_handle.rs`、`runtime.rs`、`state/core_runtime_state.rs`、tasks 和 diagnostics 存在其他会话修改；核心 finding 主要落在未修改的 activation/registration/resolution 文件及产品调用点。开始 M0 前仍必须重新读取这些重叠文件，确认没有新加入的 shutdown coordinator、observer set 或 state ownership 改变结论。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
