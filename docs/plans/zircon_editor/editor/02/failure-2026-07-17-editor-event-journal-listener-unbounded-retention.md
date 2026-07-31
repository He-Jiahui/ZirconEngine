---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-event-journal-listener-unbounded-retention
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_event/journal.rs
  - zircon_editor/src/core/editor_event/retention.rs
  - zircon_editor/src/core/editor_event/listener/mod.rs
  - zircon_editor/src/core/editor_event/listener/registry.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
tests:
  - 1000/10000 event paused-listener retention and drop-policy stress
  - journal replay-window and listener ordering parity
  - editor interaction pump p95 and allocation trace
---

# Editor02：editor event journal/listener 无界保留与锁内 fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/editor_event` 27/27 Rust 文件逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：retention、ack/cursor、payload ownership 与 dispatch lock scope 是 Editor02 消息契约的统一责任。

## 失败现象与复现证据

`EditorEventJournal.records` 与 `EditorEventListenerRegistry.deliveries` 都是无上限 `Vec`。每条事件先 clone 到 journal，再为每个匹配 listener 深 clone source、operation arguments/result 等 payload；status/query/ack/sync 又扫描共享 deliveries。`EditorEventService::record` 在持有 service mutex 时完成 journal clone 与全部 listener fanout，因此监听者数量、长会话历史和大 JSON payload 同时放大锁持有时间与内存。

Bevy `Messages` 明确用双缓冲只保留最近两次 update 的消息，并由 cursor 记录每 consumer 进度；其文档也把未调用 update 导致无限增长视为错误使用。Zircon 的 journal 需要 replay/undo 审计，不能照搬两帧窗口，但必须同样明确 retention class、cursor/ack 与清理时机。

## 最低共享层根因

journal、listener inbox 与 service fanout 没有共同的 retention/ownership contract：所有类别都退化为 owned clone + 永久 `Vec`，分发也没有与 service state lock 分离。

## 架构修复验收

- 将 durable audit/replay、frame-local notification、latest-state/coalescible delivery 分成不同 retention class；每类有条数/字节/年龄预算和 dropped/coalesced/lag 指标。
- immutable event payload 共享所有 listener；fanout 不再按 listener 深 clone JSON/字符串。
- service 锁内只分配 sequence 并发布稳定 snapshot，任何 listener delivery、过滤与慢 consumer 工作在锁外执行。
- 1k/10k paused-listener 压测证明内存有界，ack/sync/order/replay/undo 语义和诊断顺序不变。

## 禁止临时方案

- 不得用一个任意 `Vec` 最大长度静默丢弃所有事件类别。
- 不得在 service mutex 内调用用户/插件逻辑或执行全 listener 扫描。
- 不得把 journal、listener inbox 和 operation history 继续作为三个独立无界 authority。

## 修复结果与回传

Open state: `2026-07-18 源码已落地 retention classes、共享 Arc payload、per-listener inbox 与 sequence/journal/listener 分锁；静态契约和 diff gate 已通过。仍待 Coordinator01 immutable full-input barrier fixed return 后执行 source-bound 1k/10k stress、完整 editor_event 回归、独立 review、failure -> fixed return 与 managed commit；在这些证据完成前不得标记 fixed。`

实现与当前阻塞证据：[`2026-07-18-editor-event-retention-and-lock-split.md`](2026-07-18-editor-event-retention-and-lock-split.md)。

2026-07-22 current-source补充：本轮让ack单遍累计removed bytes、diagnostics首末sequence直接读单调队列两端，listener status不再clone+merge+sort全部records。PERF-MVP-067仍open：每事件exact byte accounting继续完整serde traversal、LatestState线性coalesce，listener registry全局锁仍跨逐listener filter/enqueue；Cargo/storm与per-owner锁拆分完成前不得fixed。

2026-07-30 Performance01 current-source校正：32/32生产文件与22/22外部test文件已复读。三类entry/byte/age硬预算、共享Arc、filter预规范化与ack/status止损继续成立，故标题保留为历史failure slug但不得继续描述为当前“无界”。剩余P0根因扩展为：成功dispatch深clone完整record；每event为byte accounting完整serde counting traversal；LatestState在每inbox线性扫描并中段remove；全局listener锁跨filter/prune/coalesce/enqueue；journal/listener polling先全量merge/sort再cursor过滤，并深cloneowned delivery后再次JSON物化。

Editor02后续必须以shared encoded owner/一次accounting、immutable route/filter generation、锁外per-owner enqueue、latest key index和cursor-first bounded k-way page修复，不得退回无界channel或私有线程池。验收增加0/1/1k/10k listeners/events、64B/2MiB/64MiB payload、0/50/100% filter、0/1/99% cursor与1/16 threads，记录serde traversal、record/delivery/JSON clone bytes、coalesce visits/shifts、merge/sort、lock wait/hold、queue bytes/age、p95/RSS，并复跑当前117 tests和F4 retained-host WPR。

当前源码静态边界：`listener/registry.rs` inline test与`src/tests/editor_event/retention.rs` helper的`EditorEventRecord` literal缺新增`binding_path`、`transaction_id`、`save_generation`字段。Performance01未运行Cargo、未编辑这些foreign dirty文件，因此这不是动态RED；current-source gate与独立review完成前failure继续open。
