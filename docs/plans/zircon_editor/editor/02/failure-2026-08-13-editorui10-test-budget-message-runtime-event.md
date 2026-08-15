---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-message-runtime-event
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/02
related_code:
  - zircon_editor/src/tests/editor_event/runtime/integration.rs
  - zircon_editor/src/tests/editor_message/bus/backpressure.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editor_event --locked
  - cargo test -p zircon_editor --lib editor_message --locked
---

# Editor02: messaging and runtime-event test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：runtime event、message backpressure 和 bounded pump 是 Editor02 data-sync/messaging 行为，必须在该计划内保持测试边界。

## 失败现象与复现证据

结构审计报告 0 个豁免下的 3 个 Editor02 owner：
`tests/editor_event/runtime/integration.rs`（961 行）、`tests/editor_message/bus/backpressure.rs`（964 行）和
`tests/runtime_event_consumer_bounded_pump.rs`（983 行）。它们使 zero-tolerance structure gate 保持 RED。

## 最低共享层根因

runtime event integration、bus backpressure 与 bounded-consumer pump 的独立时序/容量行为被持续追加到 flat
测试 owner，缺少按协议、capacity 和 event-loop 行为划分的 folder-backed 测试边界。

## 架构修复验收

- 将三项按单一 messaging/runtime-event 行为拆分为 folder-backed tests，薄 `mod.rs` 挂载，所有文件不超过 800 行。
- 保留 backpressure、bounded pump、runtime event 顺序与容量断言语义；共享 fixture 必须唯一归属。
- 不得留下 `#[path]` mount、旧 flat 文件、duplicate test tree 或 budget exemption。
- 重审计不再报告这三项；全部 owner 清零后受管 structure gate 才能 GREEN。

## 禁止临时方案

- 不得提高预算、删除时序/容量覆盖或将消息测试移入无关 host/UI 计划。

## 修复结果与回传

Open state: `待修复`。仅完成 Editor02 责任交接，未修改业务测试。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 messaging/runtime-event test-budget handoff | `open` | 从准确 48/0 审计隔离 3 个 Editor02 owner，均超过 960 行。 | 取得源码 lease 后 folder-backed 拆分，受管 messaging/runtime-event 回归和结构审计复验。 |
