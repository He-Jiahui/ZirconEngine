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
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/tests/events/benchmark_evidence.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib event_bus_runtime07_ --locked --jobs 1 -- --ignored --nocapture --test-threads=1 (run twice; pending)
  - cargo +1.94.1 test -p zircon_runtime --lib core::runtime::tests::events:: --locked --jobs 1 -- --nocapture --test-threads=1 (pending)
  - cargo +1.94.1 test -p zircon_runtime --lib foundation::tests:: --locked --jobs 1 -- --nocapture --test-threads=1 (pending)
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

- 完成 1/2/5/100 subscriber × payload size 与 paused consumer 压测，报告 shared-allocation identity、bounded retained payload bytes、p95、RSS、queue age 和 lock wait。分配验收针对本失败的逐订阅者深拷贝根因，不冒充 allocator 内部字节统计。
- Runtime02/07 定义 lossless、bounded/drop-oldest、latest/coalesced 类别和顺序语义。
- 按证据采用 shared immutable event payload、bounded/coalesced queue、per-topic ordering 或组合；断连 prune 和 shutdown 保持正确。
- 增加低开销 queue depth/drop/age/publish duration 指标，诊断关闭不能成为新热点。

## 禁止临时方案

- 不得统一静默丢弃事件，或删除全局锁而不定义并验证顺序。
- 不得只在 editor/plugin caller 定期 drain 来掩盖共享 EventBus 无界契约。

## 修复结果与回传

Runtime07/02 已在当前源码中完成显式 `Lossless`、`BoundedDropOldest`、`Latest` delivery policy，共享不可变 `Arc<EngineEvent>` fanout、per-topic 顺序锁、bounded queue、断连 prune/shutdown，以及可关闭的 queue/drop/age/publish/delivery-lock-wait 诊断。并发行为与源码结构 guard 已覆盖同 topic 顺序、不同 topic 并行进展、capacity-one 峰值、blocked receiver shutdown、订阅 reservation 与 lock-wait 计数。Publish 先走 poison-safe `try_lock` 快路径，只有观察到 `WouldBlock` 才计 waiting publisher 和 lock-wait sample；单线程无竞争 benchmark 反向断言这些字段保持零，避免把锁获取次数冒充等待次数。

Foundation 在 Core 已销毁时返回的断连订阅也已 hard cut 为零状态 typed `Disconnected` 实现；`recv`、`try_recv` 与 `recv_timeout` 均立即返回，不再为回退路径创建 `crossbeam_channel::unbounded()` 或 heap-backed receiver。

独立静态审查在补齐 disabled snapshot 全字段零值断言和 payload 字节数断言后为 Critical 0 / Important 0；生产路径不再采用旧 unbounded crossbeam queue、逐订阅者 payload 深拷贝或全局 delivery lock。

Open state: `实现完成，等待同一 source manifest 的 behavior/structure managed Cargo 门，以及两次 managed Runtime07 benchmark 终态与原始 EVENTBUS_BENCH_V1 shared-Arc/retained-bytes/p95/RSS/queue-age/lock-wait 证据后转 fixed`。三项受管验证全部完成前不得把本记录标记为 fixed，也不得用机器相关阈值弱化硬行为断言。

## 2026-07-27 F2 poison-recovery increment

- 在 `events/topic.rs` 的模块私有单元测试中，分别 poison topic map、per-topic delivery、subscriber snapshot 与 subscriber queue mutex；随后经公共 `publish`、`recv` 与 `diagnostic_report` 验证同一 EventBus 继续交付事件。
- 生产 EventBus 模块静态扫描没有 `.lock().unwrap()` 或 `Condvar::wait(...).unwrap()`；四处裸 `unwrap` 只用于上述受控 poison 测试的持锁 panic。
- Rust 1.94.1 `rustfmt --check` 和 scoped `git diff --check` 已通过。当前 source snapshot 为 `1127`，包含 17 个 EventBus DTO、生产模块和行为/结构/基准测试路径。
- Text01 受管 test lane 正在运行时，本切片没有创建抢占 FIFO 的通用 Cargo reservation；本记录仍为 `open`，待 snapshot `1127` 的 behavior/structure gate 和两条受管 benchmark 终态后才可转为 fixed。
