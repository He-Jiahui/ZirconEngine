---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: event-bus-backpressure-and-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
tests:
  - 1/2/5/100 subscriber payload fanout benchmark
  - paused consumer bounded-memory pressure test
  - concurrent publisher ordering and lock-wait test
---

# Runtime07：EventBus 无 backpressure、fanout 深拷贝且全 topic 串行

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：core EventBus 五个生产 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：事件队列预算、fanout payload ownership 与 publish contention 是共享 runtime hotpath；需 Runtime07 联合 Runtime02 冻结契约。

## 失败现象与复现证据

每个 topic subscriber 使用 `crossbeam_channel::unbounded()`，没有 queue depth/drop/age 诊断；暂停或遗忘的 receiver 会无限保留消息。多订阅者发布对 `{String, serde_json::Value}` 做逐订阅者深 clone，成本随 fanout 与 payload 相乘。

所有 topic publish 还共同持有一个 `delivery_lock`，send 和断连 prune 都在锁内；不同 topic、多后台 producer 与主线程 publisher 因而完全串行。当前没有锁等待和 publish duration 数据。

## 最低共享层根因

EventBus 未表达事件类别的 delivery policy，也没有共享 payload ownership 与 per-topic/global ordering 的显式契约。上层消费者无法安全决定丢弃、合并或反压，只会各自积累 unbounded receiver。

## 架构修复验收

- 完成 1/2/5/100 subscriber × payload size 与 paused consumer 压测，报告分配、p95、RSS、queue age 和 lock wait。
- Runtime02/07 定义 lossless、bounded/drop-oldest、latest/coalesced 类别和顺序语义。
- 按证据采用 shared immutable event payload、bounded/coalesced queue、per-topic ordering 或组合；断连 prune 和 shutdown 保持正确。
- 增加低开销 queue depth/drop/age/publish duration 指标，诊断关闭不能成为新热点。

## 禁止临时方案

- 不得统一静默丢弃事件，或删除全局锁而不定义并验证顺序。
- 不得只在 editor/plugin caller 定期 drain 来掩盖共享 EventBus 无界契约。

## 修复结果与回传

Open state: `待 Runtime07/02 建立 delivery policy、压测并修复`。

