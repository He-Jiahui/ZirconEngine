---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: message-inbox-backpressure-and-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/message/envelope.rs
tests:
  - paused subscriber bounded-memory pressure test
  - 1/5/100 subscriber fanout allocation benchmark
  - delivery ordering and dirty-set coalescing regression
---

# Editor02：message inbox 无界且 fanout 深拷贝

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor message 25 个生产 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：subscriber inbox、delivery ownership、dirty coalescing 和 backpressure 是 Editor02 消息协议责任。

## 失败现象与复现证据

每个 subscriber inbox 是无容量限制的 `Vec<EditorMessageDelivery>`；没有 queue depth、age、drop 或 stale-subscriber 诊断。publish/broadcast 先分配 delivered-id `Vec`，随后为每个 subscriber clone topic/message；custom JSON、job label/progress/error 字符串会随 fanout 深拷贝。

`SharedEditorMessageBus::deliveries_for` 还会 clone 整个 inbox，虽当前生产使用很少，但若用于每帧面板轮询会重复放大成本。

## 最低共享层根因

消息协议没有区分 lossless transaction/document edge、latest/coalesced progress/focus/dirty state 与可丢 telemetry，也没有 shared immutable payload 或 subscriber 消费预算。

## 架构修复验收

- paused subscriber 与 1/5/100 fanout 压测报告 RSS、allocations、queue age 和 publish p95。
- 按 topic/protocol 定义 lossless、bounded、latest/coalesced 语义；dirty mask 继续 union。
- 评估共享不可变 message/delivery payload，避免大 JSON/string 为每 subscriber 深 clone。
- unregister、request、broadcast、ordering 和 retained UI 刷新合同全部回归。

## 禁止临时方案

- 不得统一清空或静默丢弃所有 inbox；transaction/document terminal 语义必须保留。
- 不得让各 pane 自建私有去重规则，delivery policy 必须在 Editor02 owner。

## 修复结果与回传

Open state: `待 Editor02 建立 bounded/coalesced delivery 与 fanout ownership`。

