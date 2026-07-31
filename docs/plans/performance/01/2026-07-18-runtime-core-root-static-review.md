---
related_code:
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/weak.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - ten production Rust files reviewed
  - two source-level RED to GREEN performance guards added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, allocation counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core root逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime`根目录除聚合`tests.rs`外10/10个生产Rust文件，当前946行/3个inline测试。范围覆盖runtime facade、config store、clock/time、lifecycle/error、event facade与weak handle；子目录另行逐批登记。

## PERF-MVP-318：typed config load深复制JSON

原`ConfigStore::load<T>`先在Mutex内通过`load_value`深clone完整`serde_json::Value`，再在锁外反序列化；大数组/对象且被多个manager重复读取时，读取成本和allocation bytes随配置大小重复。已以RED→GREEN守卫把私有map值改为`Arc<Value>`：写入拥有单份JSON，typed load锁内仅clone Arc、锁外从`&Value`反序列化；`load_value`与`snapshot_values`仍返回owned Value，公开语义不变。

该止损为每次store增加一个Arc owner allocation，需用读多写少的产品配置负载验证净收益；最终Runtime02应提供generation snapshot/typed config cache，而不是每个manager重复反序列化。

## PERF-MVP-319：CoreRuntime facade每调用一次Arc增减

原`CoreRuntime`持`Arc<CoreRuntimeInner>`，绝大多数公开方法先调用`handle()`构造临时`CoreHandle`，因此time/state/event/config/diagnostics等每帧入口每次都执行一次共享Arc增减。已让`CoreRuntime`直接持有`CoreHandle`，facade内部借用已有handle；只有调用者显式索取owned `handle()`时才clone。weak、scheduler、task pool及所有委托语义不变。

## 其余风险

`ConfigStore`仍是单全局Mutex，snapshot与owned load会深clone全部/单项JSON；`record_diagnostic`等facade虽删除临时Arc，底层store/queue的锁与分配仍由各子模块预算。`EventBus::default`默认启用诊断，必须在event子目录的高频publish基准中计入诊断本身的开销。

## 验收要求

对1 KiB/1/100 MiB config、1/100/10k keys、read:write 1:1/100:1及1M facade calls记录JSON clone bytes、Arc RMW、Mutex hold/wait、deserialize calls与CPU p95：typed load JSON clone bytes=0，内部facade Arc clone=0；并验证poison recovery、missing/parse errors、CoreRuntime clone/weak/lifecycle/event/time/state parity。当前源码Cargo、规模counter和F2/F4产品trace完成前，这10文件留在`pending.md`。
