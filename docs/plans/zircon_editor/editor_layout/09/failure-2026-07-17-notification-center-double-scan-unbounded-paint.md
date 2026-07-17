---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: notification-center-double-scan-unbounded-paint
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/panel.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center/row.rs
---

# Notification center double scan and unbounded paint

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_notification_center*` 11/11 个 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`
- 交接原因：notification generation、unread metadata、retention/backpressure 与 overflow semantics 属于 EditorLayout09 的消息刷新协议责任；EditorUI08 只负责可见行消费与 compiled paint。

## 失败现象与复现证据

Notification header 每次 paint 都 clone 全部 option 来重新计算 unread count。Row loop 随后再次 clone 全部 option，包括最终被 clip 拒绝的行；可见行还会再次复制 title 和 description。消息风暴下 CPU 与保留内存会随完整 notification history 增长。

## 最低共享层根因

消息/通知 owner 没有发布包含 unread count、retention/overflow 状态的 immutable generation。Painter 因而从完整 options 历史反推 header，并且没有可消费的 bounded visible-row authority。

## 架构修复验收

- 1,000-message burst 有 bounded retention 和显式 overflow semantics；每 message 更新为 amortized O(1) 或 bounded batch。
- Same-generation unread scans/rebuilds 为零；closed notification center paint work 为零。
- Open center 只 visit/clone visible+overscan rows；offscreen row clone/text copy 为零。
- Header/unread、severity/state、ordering、scroll/focus、clip 和 pixel behavior 等价。

## 禁止临时方案

- 不得在 painter 再建立一个不受消息 generation 失效约束的 notification cache。
- 不得仅增加 clip leaf early-return，却继续对全部历史执行 `row_data` 或 unread 全量扫描。
- 不得通过静默丢弃通知规避 backpressure；overflow 必须显式、可计数且保序规则清楚。

## 修复结果与回传

Open state: `待 EditorLayout09 回传 generation/retention/unread/overflow counters，并由 EditorUI08 回传 visible-row/clone/command evidence 到 performance plan`。
