---
related_code:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/handle/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/handle/service_identity.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - ten root production Rust files reviewed
  - one source-level RED to GREEN state-lock guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, state scale counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core handle root逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/handle`根目录10/10个生产Rust文件，当前1,496行/5个inline测试。范围覆盖module activation/deactivation、service resolution、time/state/event/config/diagnostic facade、runtime observer与registry锁辅助；`activation/**`、`registration/**`另行逐批登记。

## PERF-MVP-320：state重复锁与无界transition history

重复`init_state<T>`原先先锁`StateRegistry`调用`init_state`，已有state时释放锁后又通过`self.state<T>`重新锁一次读取当前值。已用RED→GREEN守卫把检查与existing state读取合并到一次锁，dispatch仍在锁外执行，避免hook重入死锁，返回语义不变。

更大的结构性问题位于`StateMachine<T>`：每次transition都永久push完整event，`state_transition_events<T>`每次又clone完整history；长期运行或高频游戏状态会形成无界内存和O(history)查询。每次transition还分别线性扫描enter/exit/transition三组hooks并分配三组Vec。历史公开语义不明确，本轮不擅自改成ring/drain；Runtime02/07需定义consumer cursor、bounded retention与按state-pair索引。

## 其余热点

time每帧在一个diagnostic store Mutex下写4条series；service activation复制service/startup/shutdown name lists，resolution pending路径复制RegistryName/dependencies/factory并等待全局Condvar。前者回链既有time/diagnostic预算，后者需在activation/registration子目录结合规模和并发证据再决定arena/Arc owner，不能仅凭clone语句改变生命周期所有权。

## 验收要求

对state types 1/100/10k、transitions/hooks/history各1/100/100k与60/120 Hz运行记录registry locks、history bytes/clones、hook probes/Vec bytes和transition p95：重复init lock=1；默认产品history有明确byte/event上限或cursor drain，查询与新增事件而非总历史成正比，hook probe与匹配候选成正比。另记录time diagnostic locks/records及service activation/resolution clone/wait；当前源码Cargo与F2/F4产品trace完成前，这10文件留在`pending.md`。
