---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: bridge-import-stable-call-double-mutex
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
tests:
  - single and 16-thread stable bridge-call benchmark
  - concurrent reload/disable generation race test
  - provider lifetime and poison-recovery regression
---

# Plugins01：稳态 BridgeImport call 每次双 mutex

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：runtime plugin bridge 稳态调用静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：Runtime06 插件生命周期与并发 bridge 基准
- 交接原因：binding publication、provider generation、reload/unbind 与调用期 provider lifetime 必须统一设计，不能删除单把锁制造悬垂 provider。

## 失败现象与复现证据

`BridgeImport::call` 先通过 `lock_binding()` 获取 `Mutex<Option<WeakBridge<T>>>`，clone `WeakBridge` 后释放；
`WeakBridge::provider_with_slot` 再获取 `cached: Arc<Mutex<Option<(generation, Arc<T>)>>>`。即使 generation
长期不变，调用仍需两次 mutex acquire，并 clone cached provider Arc。多 worker 高频调用同一 import 时，cache mutex
成为共享串行点；`is_enabled`/`pin` 也进入相同 provider cache 路径。

Provider reload/disable 本身已有 atomic generation，但 stable call 没有直接消费一个可无锁读取的 generation/provider
snapshot。当前锁保证了 provider lifetime 与 poison recovery，不能简单删除。

## 最低共享层根因

Bridge import binding 与 weak provider cache 都以“低频可变容器”建模，没有把 catalog freeze/reload 转换成一次发布、
多次读取的 immutable generation snapshot；读写比例极端偏读却仍使用独占 mutex。

## 架构修复验收

- catalog freeze/reload/unbind 发布带 generation/status/provider Arc 的不可变 snapshot；stable call 快路不获取独占 mutex。
- generation miss 进入有界慢路刷新 snapshot；并发调用持有的旧 provider Arc 在完成前保持有效。
- disabled/not-enabled diagnostics 与 generation 语义不变；reload 不得把旧 callback/provider 提前释放。
- 1 与 16 calling threads 各 1M stable calls，记录 mutex acquisition/wait、throughput、p95/p99；快路无全局独占串行。
- 并发 reload/disable/unbind、poison recovery、`pin` guard lifetime 和 absent/disabled 错误测试通过。

## 禁止临时方案

- 不得缓存裸指针或绕过 Arc lifetime。
- 不得只把 `Mutex` 改成 `RwLock` 就宣称完成；必须用指标证明 stable 多读不再被单写锁路径串行。
- 不得把 generation 检查移除或让 disabled provider 继续可调用。

## 修复结果与回传

Open state: `待 Plugins01 建立按 generation 发布的 bridge import/provider snapshot 并完成多线程基准`。
