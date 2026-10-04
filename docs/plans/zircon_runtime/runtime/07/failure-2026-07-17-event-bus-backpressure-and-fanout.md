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
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/events/publish/single_disconnect_inline_tests.rs
  - zircon_runtime/src/core/runtime/tests/events/structure/event_bus/publish.rs
  - zircon_runtime/src/core/runtime/tests/events/structure/event_bus/subscribe.rs
  - zircon_runtime/src/foundation/tests.rs
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

## 2026-09-09 current-source regression repair

Session `failure-roll-01a07160-runtime07` retains this lifecycle and acquired
the archived-owner record through fingerprint
`28d6a788f22f055b386d83db4b251ee95a86fb148327998bac4324c716ef6da4`.
Preimage snapshot 3274 preserves all historical evidence at
`20dfffbb3657dc6e455bcfb27e7f2b09de911645b0e6592aaa7386330ce8dd2a`;
the current record was Git-clean before this continuation.

All 22 current EventBus contract, owner, handle and test files match immutable
input `runtime15-production-view-3271-20260909`, manifest
`532caeb7e8cda84b9b5e5a05ad099875a958a8f9a87803aa55ff683a595712b7`.
The producer already uses a read/write topic registry, per-topic delivery,
shared immutable payloads, batch drain accounting and sampled diagnostics.
The default timing interval is 64; explicit `Lossless` remains unbounded while
`BoundedDropOldest` and `Latest` have their declared capacities. No complete
lossless memory-budget solution is claimed by this historical policy repair.
Foundation now publishes only its behavioral ConfigManager and no EventManager;
its 16 current source/test files also match the input. The historical disconnected
Foundation event fallback is retired, and must not be restored for this gate.

Two Windows static/no-default/locked RED jobs executed actual tests:

- `5942fe1eb71a4151a29230f2f9cd09d8`, filter `core::runtime::events::`:
  14 passed, 1 failed, 8 ignored, 6827 filtered out. The single-disconnect
  guard cuts `publish.rs` at its first `#[cfg(test)]` helper, before the
  production `publish` implementation. Artifact:
  `results/runtime07-eventbus-owner-support-3271.{json,log}`.
- `31a47db2427c41f5bdec8a85e228ba7a`, original filter
  `core::runtime::tests::events::`: 21 passed, 4 failed, 5 ignored,
  6820 filtered out. All 19 behavior tests passed; four structure tests still
  require the old allocation order, scalar drain body, inline modulo sampling
  or exclusive `lock_topics` helper. Artifact:
  `results/runtime07-eventbus-original-red-3271.{json,log}`.

The two guard files and single-disconnect test were acquired from terminal
owners through fingerprints
`9257793bbe463a876847c58d079cad1f0338bb809f36bddde35eb7987e69064e`
and `8d30c4fb7e53cb5d637f7fc3c7e0010fb2676025288134392cdab3d6ff66ad38`.
Preimage snapshots 3275/3276 preserve their exact bytes. The single-disconnect
preimage already contained an uncommitted trait import and a capacity-constructor
guard update; those meanings are preserved. Both other guards were Git-clean.

Source snapshot 3277 changes only these three test files:

- `single_disconnect_inline_tests.rs` now executes the actual one-subscriber
  ID collector, proving the first ID remains inline and additional storage
  has zero length and zero capacity before and after insertion. The public
  disconnect-cleanup behavior regression remains in the same batch.
- `structure/event_bus/publish.rs` retains per-topic locking and shared-payload
  assertions, including empty-subscriber checking before allocation. It follows
  the current ID collector's removal call, verifies enqueue/dequeue/drain order
  within each responsibility, batch drain accounting and both power-of-two and
  arbitrary-interval sampling routes.
- `structure/event_bus/subscribe.rs` follows the current `write_topics` admission
  owner while retaining reservation, drop, policy and lifecycle assertions.

Rustfmt and scoped whitespace checks pass. Production EventBus code, policies,
metrics, performance thresholds and ignored benchmark definitions are unchanged.
Fresh managed owner/original/Foundation gates and both required Runtime07
benchmark runs, independent review, formal binding and canonical closeout
remain pending. External zr_vm is skipped.

## 2026-09-09 lower GREEN and execution-configuration handoff

Input `runtime07-eventbus-guards-3277-20260909` freezes the three test changes
over the unchanged producer, manifest
`ef35355b6952453dac4d0f899a2d62843c17e63e394e0f6bf4748028ead278db`.
Managed job `0c1cf55c6dde4b39b0dab91ba311b8b5` actually ran
`core::runtime::events::`: 15 passed, 0 failed, 8 ignored, 6827 filtered out.
This includes the repaired inline-ID regression and real disconnect cleanup.
Artifacts: `results/runtime07-eventbus-owner-support-3277.{json,log}`.

The caller's JSON recorded requested `RUST_TEST_THREADS=1`, but investigation
proved that managed compiler environment cleanup removes all `rust`-prefixed
ambient variables, including this setting and `RUST_TEST_NOCAPTURE`. The current
validator has no equivalent explicit harness parameters. This result is valid
functional evidence only; it does not establish serial execution. The original
artifact is preserved. The source hashes, reproduction and required explicit
argument repair are in the [Tooling01 handoff](../../../optimize/zircon_tooling/01/failure-2026-09-09-managed-test-harness-arguments-missing.md).

Both Runtime07 performance runs remain unsubmitted until their required serial
execution and visible raw output can be bound to the actual managed command.
No ineffective configuration is retried, and no benchmark or full-lifecycle
acceptance is inferred from the passing lower functional batch.

Two further Windows static/no-default/locked functional jobs completed on
input `runtime15-ui-upload-3282-20260909`, manifest
`bd0a27003ded9b3acb85a59cf3ac28f7fd4251a710770c8b34ca8d2c34d05d77`.
This input retains all three EventBus snapshot-3277 hashes and changes only the
unrelated Runtime15 UI-upload guard relative to the earlier EventBus input.

- `551501e1752d4719aa7aaabbc3006a0e`, `core::runtime::tests::events::`:
  25 passed, 0 failed, 5 ignored, 6820 filtered out. All 19 behavior and six
  current structure tests ran, including all four formerly failing guards.
  Artifact: `results/runtime07-eventbus-original-functional-3282.{json,log}`.
- `6c625797f2944b40bbad7eaa99ae3f71`, `foundation::tests::`:
  7 passed, 0 failed, 0 ignored, 6843 filtered out. This verifies the current
  Foundation configuration owner; it does not reinstate or accept a retired
  Foundation EventManager. Artifact:
  `results/runtime07-foundation-functional-3282.{json,log}`.

Both receipts bind the input and real Cargo commands, without claiming a serial
test harness. The remaining obligations are explicit-configuration reruns,
two actual performance batches, independent review and formal lifecycle
acceptance. The 15/25/7 passing functional counts are not performance evidence.

The prepared bounded review was submitted once to the designated existing
task `01a07063-6f03-7803-a12d-13ea015ca645` on 2026-09-09. Native resume
failed before the review started with `thread-store conflict: thread ... already
has an active writer` (JSON-RPC -32600, process exit 1). No new C0 report exists
for the EventBus increment. The prompt remains at
`.codex/tmp/runtime07-eventbus-3277-review-20260909.txt`; it includes all source
hashes, 15/25/7 functional results and the explicit Tooling01 handoff. This item
is suspended pending that task's availability, with no repeated resume, new
review task, closeout attempt or WeCom send.
