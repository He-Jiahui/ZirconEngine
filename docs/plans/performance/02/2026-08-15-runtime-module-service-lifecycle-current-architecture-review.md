---
related_code:
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/handle
  - zircon_runtime/src/core/runtime/state
  - zircon_runtime/src/core/runtime/modules
  - zircon_runtime/src/core/manager
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - module and service production implementation 67 of 67 Rust files reviewed
  - 7348 production lines and 11 inline tests inventoried
  - deterministic 67-file production manifest SHA256 04f3c3d29429c5a0a1e059403b344ad7b0243286ada691fa45a04071c3df81b3
  - direct activation registration resolution plugin and registry tests 44 files 7727 lines 107 tests inventoried
  - deterministic 44-file direct-test manifest SHA256 686f2cc025511ac1b6d42ea2cf7bd61c0fa18188f13097b0faa191ca38235bc9
  - rustfmt edition 2021 check passed for 67 of 67 production files and 44 of 44 direct-test files
  - current-source Cargo product trace allocator and power acceptance blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime 模块与服务生命周期当前架构审查（2026-08-15）

## 范围与结论

已逐文件复读模块/服务生产范围 67/67 个 Rust 文件，共 7,348 行、11 条内联测试；范围为 `core/runtime/{descriptors,handle,state,modules}`、`core/manager/**` 以及 `module_context.rs`、`lifecycle.rs`、`module_lifecycle_observer.rs`、`runtime.rs`。规范化相对路径排序后对每个文件取 SHA256，再拼接清单取总 SHA256，得到 `04f3c3d29429c5a0a1e059403b344ad7b0243286ada691fa45a04071c3df81b3`。67/67 通过 `rustfmt --edition 2021 --check`。

直接 activation/registration/resolution/plugin/registry 测试另清点 44 个文件、7,727 行、107 条测试，清单 SHA256 为 `686f2cc025511ac1b6d42ea2cf7bd61c0fa18188f13097b0faa191ca38235bc9`，44/44 通过 rustfmt。结构测试子集有 18 个文件、2,179 行、20 条测试、779 个源码形状断言和 94 个 `include_str!`；它们大量保护当前 1..5 项手工展开，而不是保护产品复杂度。

结论是：当前实现不是可继续原地微调的 module manager。它把 catalog、拓扑编译、状态转换、同步 ready 轮询、Rust object factory、服务定位、卸载与插件 observer 混在 `CoreHandle` 上；缺少单一 catalog generation、严格 phase owner、TaskGraph readiness、slot-index fast path、可回滚 quiesce 和生命周期诊断。M1 应以一个 Unreal-aligned `EngineRuntime` module/service catalog 硬切替换，不应继续增加计数特化、缓存层或兼容 adapter。

当前源码仍没有可运行产品 binary；managed `zircon_app` build 在 324.2 s 后因 6 个 foreign runtime 编译错误失败，focused runtime lib-test 在 843.4 s 后编译失败，均执行 0 tests。因此本文的次数、行数和复杂度均为 current-source 静态证据，不是 WPR、wall-clock、功耗或 allocator 实测；本模块继续留在 `pending.md`。

## 应保留的行为合同

- module lifecycle、service factory 与 runtime observer 回调通常在 modules/services 锁外调用；新 owner 必须保留 callback-lock separation。
- lazy service factory 在执行依赖和 factory 前释放 services 锁；并发同服务 resolve 已有“单 owner 执行一次 factory”的行为测试。
- registration 会校验 module owner、service kind 和 driver dependency kind，并在提交前构造 startup/shutdown 名单。
- service handle 有 index/generation 和 stale-handle error；卸载期间完成的 lazy factory 不会重新提交旧实例。
- `RegistryName` 缓存解析 offset/kind；state transition dispatch 在 state registry 锁外执行。

这些是迁移后的行为门，不是保留当前 HashMap、String handle、全局 condvar 或 cardinality specialization 的理由。

## P0：计数特化占实现近半，并被源码形状测试反向锁死

registration、service-list、unload 与 dependency matcher 为 1..5 个 service/dependency 分别展开。十个主要特化文件占 2,680 行，`blocked_dependencies/**` 再占 671 行，合计 3,351 行，即本范围 7,348 行的 45.6%。`service_lists/specialized.rs` 单文件包含 77 次固定 `Arc<[RegistryName]>` 数组构造和 224 次 `.clone()` 调用；blocked dependency 的 single/two/three/four/five 文件分别有 16/32/48/64/80 次显式 dependency equality 比较。

这种展开只改变极小 N 的常数，不改变全局 service scan、重复 clone、全局锁和重建 topology 的规模。更严重的是，`service_count_paths.rs`、`service_list_caches.rs`、activation blocked/unload tests 与 `resolution/structure.rs` 明确断言 exact-one 到 exact-five helper、顺序、源码片段和 fallback loop 必须存在。测试把临时实现形状当成产品合同，使删除 3,351 行重复结构本身会 RED。

硬切要求建立一次编译的 module/service dependency graph 和 dense slot tables；遍历复杂度以 `O(M+S+E)` 注册代变化、steady generation `O(1)` 查询为门。迁移同里程碑删除 exact-count helper 及其源码形状测试，改为 0/1/5/100/1000 规模的行为、allocator、visit 与 lock-hold 门。

## P0：稳定批激活仍每次重编 topology

`activation/batch.rs:23-31` 在判断没有 pending module 之前先调用 `sorted_registered_module_order()`；`83-95` 每次锁 modules、clone 全部 `ModuleDescriptor`、按 name 排序，再让 `sort_module_activation_order` 重建 name map、state、DFS stack 和 order。只有 `begin_batch_module_activation` 的 `107-117` 才过滤已经 Running 的 module，并再次 clone name/service lists。

所以稳定 generation 重复调用 `activate_registered_modules()` 仍为 `O(M log M + M + E)`，并产生 descriptor/factory Arc、String、Vec/Box clone；“全部已 Running”不是零工作快路。目标是 catalog mutation 只使 generation 失效一次，生成 immutable compiled activation plan；同 generation 重复 phase 查询和已 active module load 为 `O(1)`，topology build/descriptor clone 为 0。

Unreal `ModuleManager.cpp:992-1023` 先返回已加载 module，这是 common case；`1968-1985` 只处理非空 pending static initializer，并在提交后 `Empty()`。`PluginManager.cpp:2034-2080` 同样只在 pending plugins 非空时配置，完成后清空待处理集合。Zircon 应继承这种 stable fast path 行为，不复制 C++ 容器。

## P0：module 状态机可重入、可倒退且依赖合同可旁路

`activation.rs:28-48` 除 Running 外接受 Registered、Initializing、Stopping、Unloaded 任意状态，并直接写为 Initializing；两个 caller 可同时执行同一 module 的 build/ready/finish/notify。`92-107` 的 deactivate 同样把任何状态写为 Stopping，因此 Registered/Unloaded 可重复 cleanup，Initializing 可被并发 teardown。rollback 只在当前值仍为 Initializing/Stopping 时写回旧枚举，没有 transition owner、CAS、generation 或 wait handle。

batch path 使用 module dependency sort，但单 module `activate_module` 不建立 dependency closure。lazy service resolution 在 `resolution.rs:277-288` 发现 owner module 为 Registered 时直接调用单 module activation，因此可绕过 descriptor 的 module dependencies。当前直接测试覆盖 activation order、rollback 和 service generation，却没有并发 module activation/deactivation、activate-during-stopping、重复 deactivate、非法 phase 或单 module dependency closure 行为门。

目标状态必须由唯一 catalog slot owner 以 monotonic phase 和 transition ticket 发布：`Discovered -> Validated -> Configured -> Loading -> Active -> Quiescing -> Unloaded/Failed`。每条 activation path 都消费同一 compiled dependency closure；同 slot 的并发 caller 共享一个 completion handle，不能重复执行 lifecycle。非法倒退和 unload-in-flight 返回 typed failure。

## P0：ready 在调用线程 1 ms 轮询，批启动最坏串行放大

`module_lifecycle.rs:17-43` 在调用线程同步调用 `ready()`，未就绪时每 1 ms `std::thread::sleep` 后重试。默认 `activate_module()` 与 `activate_registered_modules()` 传 `Duration::default()`，第一次 false 立即超时；显式 timeout 又会阻塞 caller。batch 在 `batch.rs:50-52` 为每个 module 依次给予完整 timeout，因此主线程最坏阻塞可达 `M * timeout`，且 ready callback 次数随 `M * timeout / 1 ms` 增长。

产品启动在 `zircon_app/src/entry/engine_entry.rs:159-168,366-389` 同步 register/activate 后立即 resolve editor manager；dynamic session 在 `construction.rs:123-165` 重复同一模式。M1 必须让 build/readiness/factory 返回 TaskGraph completion，phase owner 使用一个绝对 deadline 聚合等待，并只在 main lane pump 显式 main-affinity work；禁止 sleep-poll foreign callback。

## P0：deactivate 的 rollback 已经发生不可逆 cleanup

`activation.rs:109-130` 先检查 service dependents，随后调用 module `cleanup()`，然后才调用 `notify_runtime_module_deactivating()`。如果 observer 拒绝，`132-134` 把 module enum 写回原状态，但 cleanup side effect 已经发生，service 仍显示 Running。该 rollback 在语义上是假的。

卸载必须分成 `prepare -> quiesce -> observer/capability approval -> cleanup -> publish unloaded -> retire`。所有可失败检查发生在不可逆 cleanup 前；进入 cleanup 后失败进入 typed Failed/Abandoned，不伪装回 Active。Unreal `ModuleManager.cpp:1316-1405` 只对有效且已加载 module 执行卸载，先使其 not-ready，再 shutdown/reset；进程 shutdown 可 abandon code，并避免触发会重新做工作的变更 callback。

## P0：foreign destructor 在全局 services mutex 内执行

`activation.rs:122-126` 持有全局 services mutex 调用 `unload_services`；`service_entry.rs:45-49` 在锁内把 `instance = None`。若这是最后一个 Arc，任意 service/plugin destructor 会在全局 registry 锁内执行，可能阻塞或重入 resolve/deactivate 并死锁。failed activation/reactivation 的 `take()` 也有同类风险。

目标 owner 在短锁内只推进 slot generation/state 并把 instance 移入 retire list；destructor、cleanup、observer 和 plugin callback 在锁外、声明 affinity 上运行。验收必须包含 destructor 重入 resolve、慢 destructor、panic 和 unload deadline；callback-in-registry-lock 恒为 0。

## P0：service fast path 仍是 String/HashMap/全局锁，等待为全局惊群

`registered_manager_identity` 和 `resolve_registered_service_inner` 都用 `RegistryName` 在全局 services HashMap 查找。handle 虽携带 index，却只用于 identity validation，没有直接索引 arena；`ManagerServiceHandle` 仍保存完整 name，resolve 后再 clone 内层 Arc。因此所谓 registered fast path 仍支付 String/hash、全局 mutex 和 Arc clone。

lazy resolve 在 `resolution.rs:229-310` 锁内 clone dependencies/factory，锁外递归 resolve 后在 caller 线程同步执行 Rust factory。factory 没有 affinity、budget、deadline 或 cancel。所有 service 共用 `core_handle.rs:53-64` 的一个 condvar，任一状态变化 `notify_all()`，形成 unrelated waiters 惊群；ThreadId wait graph 没有 timeout/cancel，hung factory 可无限阻塞另一个 resolver。dependency stack 用 Vec 线性检测 cycle，深链可达 `O(D^2)`。

M1 目标是 runtime-owned dense generational slot arena。name map 只服务 catalog/discovery；稳定 handle 以 `{runtime/catalog generation, slot, service generation, capability}` 直接命中 slot，per-slot completion 只唤醒相关 waiter。factory 提交到声明 affinity 的 TaskGraph 并受绝对 deadline/cancel/domain quota 约束。M5 的 VM/dynamic plugin 只跨 ABI 传 stable handle、capability 和序列化状态，不传 Rust `Any` object。

## P1：service index 所有权和诊断都不成立

`register_module.rs:22-24,515-531` 用 process-global `NEXT_REGISTERED_SERVICE_INDEX` 为所有 `CoreRuntime` 分配永不复用的 index；但 services HashMap 是每 runtime 私有。测试甚至要求两个 runtime 的同名 service index 不同。这既扩大进程级共享状态，又使动态 session/test 持续消耗 u32，最终可能在单 runtime 远未耗尽时报告 index exhausted。

index/slot 必须归单一 `EngineRuntime` catalog，slot 复用时推进 generation。当前 generic diagnostic store 和粗粒度 `profile_scope` 存在，但 lifecycle 本身没有 module/service build、ready、finish、cleanup、factory duration，catalog generation/cache hit/topology rebuild，registry lock wait/hold，waiter/wakeup，transition/rollback/retire 或 phase deadline counter。没有这些数据就无法证明结构瓶颈消失。

## Unreal 对照后的目标 owner

唯一 owner 固定为 `zircon_runtime::core::runtime::EngineRuntime`，内部至少分为以下数据面，而不是增加第四个 crate 或兼容 facade：

1. `ModuleCatalog`: descriptor、capability、loading phase、module/service dependency graph 和 catalog generation；mutation 时一次编译并原子发布 immutable plan。
2. `ModuleSlot`/`ServiceSlot`: dense index、generation、monotonic lifecycle、transition ticket、affinity、budget 和 completion；steady handle 直接索引。
3. `LifecycleCoordinator`: 在 TaskGraph 上执行 dependency closure、phase deadline、quiesce/cancel/drain、observer approval 和 retire；main thread 不 sleep-poll。
4. `ServiceResolver`: name 只用于 catalog lookup，stable handle 使用 per-slot completion；factory/destructor/callback 全部锁外执行。
5. `LifecycleDiagnostics`: always-on 低成本 generation/transition/cache/queue/wait/lock/duration counters，高成本 sample 显式开启并与 run/frame/thread 关联。

Unreal `ModuleManager.cpp:1049-1061` 在初始化前先发布可发现但 not-ready 的 module info，以支持受控重入；Zircon 对应行为应由 slot+transition ticket 表达。`PluginManager.cpp:2884-2978` 要求 loading phase 不倒退、不跳阶段，并允许同 phase 重复调用；`3351-3360` 只按需 mount explicitly-loaded plugin。目标继承这些 phase/fast-path 行为，不继承全局 C++ singleton 或动态库细节。

## 实施前 RED 门与动态验收

- 0/1/5/100/1000 modules、services 与 dependency edges：记录 plan builds、descriptor/String/Arc clones、hash lookups、slot lookups、graph visits、alloc bytes、registry lock wait/hold。stable generation 重复 activate/resolve 1,000 次时 topology rebuild/descriptor clone/name hash 为 0，handle resolve 为 `O(1)` slot lookup。
- 两线程同时 activate 同 module 只执行一次 build/ready/finish；activate/deactivate overlap、phase 倒退、dependency missing/cycle、unload-in-flight、重复 unload 均有 typed terminal，回调顺序确定。
- 单 module、batch、lazy service 触发的 activation 使用同一 dependency closure；1/100/1000 module chain 的工作为 `O(M+E)`，不按每 module 重复全图编译。
- ready false/slow/hung/panic、factory slow/hung/panic、cancel 和 deadline 矩阵中 main thread sleep=0；一个 phase 共享绝对 deadline，不出现 `M * timeout`；相关 waiter 定向唤醒，无 unrelated notify-all storm。
- observer reject、cleanup error、destructor reentry/slow/panic 与 process shutdown：不可逆 cleanup 前完成 approval；foreign destructor/callback-in-registry-lock=0；超时返回 typed retire/abandon report。
- `git grep`/结构门证明 exact-one..five registration/service-list/dependency/unload helper、process-global service index、global resolution condvar、old lifecycle adapters 和旧源码形状测试为 0；不保留 alias/re-export/forwarder。
- M0 current-source 产品恢复后，对 F0 runtime/editor 启动与退出各至少 3 次采集 WPR/xperf CPU sampling、context switch/wait、线程峰值、idle wakeups、RSS、I/O、wall time 和功耗，并关联 lifecycle counters。没有同机 current-source 数据前不声明接近 Unreal。

本轮不修改 Rust 实现：上述 P0 同时跨 module catalog、TaskGraph、service ABI、插件 observer、app bootstrap 和测试合同；任何“修一个 clone/锁/计数 helper”的局部补丁都会延长错误 owner，并与 Plan02 M1 hard cut 冲突。
