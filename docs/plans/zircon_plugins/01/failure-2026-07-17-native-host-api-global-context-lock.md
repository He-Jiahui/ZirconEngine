---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-host-api-global-context-lock
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
tests:
  - native host context registry 1/16-thread benchmark
  - concurrent scope-drop and in-flight call lifetime test
  - stale/wrong-kind handle status regression
---

# Plugins01：native host API context registry 全局锁

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：native host bridge ABI 热路径完整静态复读
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：Runtime06 native lifecycle/ABI 并发与压力测试
- 交接原因：handle allocation、lookup、scope drop、reload generation 与 callback/dynamic-library lifetime 必须共同收敛，不能缓存裸 context 指针。

## 失败现象与复现证据

性能审计已把 `NativeHostApiV3Context::BridgeCall` 改为 `Arc<NativeHostBridgeCallContext>`，使
`bridge_context_for` 不再逐 call 深 clone method `BTreeMap`。但每次 `native_host_bridge_call_v1_inner` 仍调用
`lock_contexts()`：一个进程级 `OnceLock<Mutex<BTreeMap<u64, NativeHostApiV3Context>>>`。

所有 native plugins、registration scopes、bridge scopes 与 calling threads 共享这把 mutex；稳定 handle 的每次调用
都独占锁、做 BTreeMap lookup、clone Arc 后才锁外 dispatch。高频多线程 bridge workload 仍会在 context registry
串行，只是锁持有时间比旧实现短。

## 最低共享层根因

Host handle 被设计为单调 u64 + 全局可变 map，没有 slot generation 与并发读 owner。scope drop 通过 map remove
表达失效，因此每次 call 都必须进全局锁确认 lifetime。

## 架构修复验收

- 使用带 slot generation 的 handle；lookup 能并发读取并取得 Arc context，注册/remove/reuse 走有界写慢路。
- scope drop 原子阻止新的 lookup；已经取得 Arc 的在途 call 完成前，其 method table、bridge table 与动态库 owner 有效。
- stale/reused handle 必须返回 `NotFound`，registration handle 调 bridge 继续返回 `UnsupportedVersion`。
- 1 与 16 threads 各 1M stable calls，记录 context registry exclusive mutex acquire/wait=0、lookup p95/p99 和吞吐。
- 并发 drop/reload/panic、disabled interface、missing method 与 method output buffer 合约全部回归。

## 禁止临时方案

- 不得把 u64 handle reinterpret 为裸指针或让插件持有 Rust Arc 指针。
- 不得永不 remove context 以换取无锁读取，造成 generation 混淆和内存泄漏。
- 不得把 method callback 调用移回任何 registry 写锁内。

## 修复结果与回传

Open state: `待 Plugins01 引入 generational concurrent-read host context registry 并完成多线程/生命周期验收`。
