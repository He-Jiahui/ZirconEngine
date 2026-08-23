---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: plugin-registration-runtime-consumer-atomicity
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
tests:
  - rejected plugin extension does not publish runtime event consumers
  - runtime consumer registry extend is atomic on duplicate rejection
  - plugin registration atomicity structure contract
---

# Editor02: Plugin registration can leave runtime consumers after extension rejection

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：M1.3 executable extension registry 独立复审整改批次 3
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：最低共享根因位于 Editor02 拥有的 runtime-event consumer registry transaction boundary，而不是 Editor05 的 scene-mode factory 调用点。

## 失败现象与复现证据

`register_editor_plugin_registration` committed the plugin's runtime-event consumers before it
validated and committed the extension batch. A scene-mode factory mismatch or another extension
validation error therefore returned failure while the consumer remained registered. In addition,
`EditorRuntimeEventConsumerRegistry::extend` inserted registrations directly, so a duplicate found
late in the batch left earlier consumers committed.

## 最低共享层根因

Plugin registration lacked one atomic preparation/commit boundary spanning the candidate runtime-event consumer registry and the extension batch. Both the host sequence and registry `extend` could mutate active state before all fallible validation completed.

## 架构修复验收

- Build and validate a candidate consumer registry without mutating the active registry.
- Serialize standalone consumer registration with complete plugin registration.
- Commit the prepared consumer registry only after the extension batch succeeds; the final
  consumer commit must be infallible.
- Prove a rejected plugin can subsequently register the same consumer id and that duplicate batch
  rejection leaves the original registry unchanged.

## 禁止临时方案

No best-effort unregister, partial rollback, duplicate-id alias, silent consumer replacement, or
test-only cleanup.

## 修复结果与回传

Open state: `候选实现、duplicate-batch atomic regression 与二次审查已完成，方可进入受管验证`; Consumer `extend` now commits a validated
clone, the host prepares a complete candidate behind a registration gate, and extension success
precedes the infallible consumer-registry replacement. The handoff remains open until the current
source receives independent rereview and managed Rust validation.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-05 | plugin registration consumer atomicity | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | 候选 registry + registration gate + extension 成功后无失败替换仍成立；补齐按 `BTreeMap` key 排序先处理 `a-new`、后处理 active `z-duplicate` 的重复 batch 回归，断言失败后 active 不含先处理的 `a-new` 且原 consumer 保持。最终独立二审 `0/0/0`；尚未运行 Cargo，failure 保持 open。 |
| 2026-08-05 | plugin registration atomicity receipt | `source_forward_repair_static_green / independent_second_review_green / managed_validation_queued` | current source manifest 封存 candidate registry、host registration gate、extension registration/controller 与 rejected-plugin regression。 | Ticket `73b0e8bd4ee04408b062eb6bb34115a8` 已收到 queued receipt；未轮询、未写 fixed，failure 保持 open。 |
| 2026-08-05 | registration regression module boundary | `source_forward_repair_static_green / independent_second_review_pending / managed_validation_queued` | 将 1,746 行 `extensions_registration.rs` 拆为只负责 wiring 的根模块，以及 overlay lifecycle（582 行）、operation/view registration（393 行）和 plugin contribution/inspector（774 行）三个语义子模块；20 个 `#[test]` 函数与 rejected-plugin 原子性回归均守恒。 | 新增 plugin atomicity 静态合同 `3/3 GREEN`，锁定 candidate registry、plugin extension 成功后安装 consumer、独立 consumer 注册与 plugin 注册共享 `plugin_registration_gate`，以及模块预算/测试守恒；连同其余 Editor02 合同为 `29/29 GREEN`。`rustfmt --check`、Python compile 与 scoped diff 通过；本次结构增量须完成独立二审，已存在 queued ticket 不轮询且不作为通过证据。 |
