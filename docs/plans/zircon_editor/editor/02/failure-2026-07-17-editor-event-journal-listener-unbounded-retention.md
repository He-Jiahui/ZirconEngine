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

Open state: `2026-08-05 已前向修复 listener page 的全量三队列 clone/sort、latest-state 线性定位与每事件独立 serde counting：queue 改为 delivery-cursor/age/event-sequence/key 索引，控制面硬切为 bounded page，shared record 只计算一次编码长度而不常驻第二份 JSON。cursor 采用 listener-local arrival ordinal，避免迟到的低 event sequence 被 continuation 跳过。registry 已按配置变更生成 immutable route snapshot，service 与 control 在 global registry guard 外执行 filter/per-owner enqueue/status/page/ack。source-bound 1k/10k stress、完整 editor_event 回归、独立 review、failure -> fixed return 与 managed commit 仍未完成；在这些证据完成前不得标记 fixed。`

实现与当前阻塞证据：[`2026-07-18-editor-event-retention-and-lock-split.md`](2026-07-18-editor-event-retention-and-lock-split.md)。

2026-07-22 current-source补充：本轮让ack单遍累计removed bytes、diagnostics首末sequence直接读单调队列两端，listener status不再clone+merge+sort全部records。PERF-MVP-067仍open：每事件exact byte accounting继续完整serde traversal、LatestState线性coalesce，listener registry全局锁仍跨逐listener filter/enqueue；Cargo/storm与per-owner锁拆分完成前不得fixed。

2026-07-30 Performance01 current-source校正：32/32生产文件与22/22外部test文件已复读。三类entry/byte/age硬预算、共享Arc、filter预规范化与ack/status止损继续成立，故标题保留为历史failure slug但不得继续描述为当前“无界”。剩余P0根因扩展为：成功dispatch深clone完整record；每event为byte accounting完整serde counting traversal；LatestState在每inbox线性扫描并中段remove；全局listener锁跨filter/prune/coalesce/enqueue；journal/listener polling先全量merge/sort再cursor过滤，并深cloneowned delivery后再次JSON物化。

Editor02后续必须以shared encoded owner/一次accounting、immutable route/filter generation、锁外per-owner enqueue、latest key index和cursor-first bounded k-way page修复，不得退回无界channel或私有线程池。验收增加0/1/1k/10k listeners/events、64B/2MiB/64MiB payload、0/50/100% filter、0/1/99% cursor与1/16 threads，记录serde traversal、record/delivery/JSON clone bytes、coalesce visits/shifts、merge/sort、lock wait/hold、queue bytes/age、p95/RSS，并复跑当前117 tests和F4 retained-host WPR。

2026-08-05 forward repair：`listener/registry.rs` inline test与`src/tests/editor_event/retention.rs` helper 已补齐 `binding_path`、`transaction_id`、`save_generation`。同步删除旧无界 delivery query，并把 `AckDeliveriesThrough` 硬切为 delivery cursor；新增混合 retention-class 的 cursor page continuation、先确认后迟到低 sequence 仍可拉取、终页后空页与非法页大小回归；新实现不保留旧 API 或 compatibility shim。immutable route snapshot/per-owner inbox 的旧 snapshot、filter/disable/unregister 线性化和 detached inbox 投递亦已通过最终独立二审 `0/0/0`。详情见 `2026-08-05-retention-cursor-page-hardcut.md`；该 P0 仍须 managed current-source gate，failure 保持 open。

## 产出记录与时间

| 时间 | 切片 | 状态 | 完成项目与后续门禁 |
| --- | --- | --- | --- |
| 2026-08-05 CST | PERF-MVP-067 cursor page / latest-state index | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | 删除 `QueryDeliveries`/`QueryDeliveriesSince`，三类 queue 硬切为 delivery-cursor `BTreeMap`、age/event-sequence `BTreeSet`、latest-state key index；listener polling 变为 cursor-first bounded k-way page，shared record 在构造期一次 JSON length accounting 后释放临时 bytes。registry guard 仅产生 shared page，owned delivery/JSON 在 guard 析构后投影。ACK 同样以 delivery cursor 截断，覆盖 delayed-low-sequence/terminal-empty page 与非法页大小；公开架构文档、acceptance 审计和控制回归名称均已硬切到 `QueryDeliveriesPage` 与 delivery-cursor ACK，避免新调用方或维护者重接旧无界协议。`rustfmt`、scoped diff 与旧路径静态审计已复跑，最终独立二审 `0/0/0`。尚未执行 Cargo；route snapshot/per-owner enqueue 已由下一行继续修复。 |
| 2026-08-05 CST | PERF-MVP-067 immutable route / per-owner inbox | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | registry 在 listener config mutation 时重建 immutable `Arc<[EditorEventListenerRoute]>`；record 仅在 registry lock 内复制 route snapshot，filter 与每个 inbox lock 均在其后，control 的 status/page/ack 同样采用 handle 后锁外 retention。回归覆盖 old in-flight snapshot 与新 filter/disable/unregister 的线性化语义、detached inbox 的旧 route 实际投递；静态锁边界与 legacy-path 审计和最终独立二审 `0/0/0` 通过。remaining gate 是 source-bound Cargo 和 0/1k/10k contention/stress。 |
| 2026-08-05 CST | PERF-MVP-067 retention stress receipt | `source_forward_repair_static_green / independent_second_review_green / managed_validation_queued` | current source manifest 封存 journal、retention、immutable route/registry、service 与 1,000-listener cursor-page regression；focused stress 由 coordinator 执行。 | Ticket `0a59c4cedd8a43da9270ec689e17b22a` 已收到 queued receipt；未轮询，未将排队状态记为 test pass，failure 保持 open。 |
