---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: plugin-registration-runtime-consumer-atomicity
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/02
related_code:
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
tests:
  - rejected plugin extension does not publish runtime event consumers
  - runtime consumer registry extend is atomic on duplicate rejection
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

Open state: `待修复`; implementation complete, managed validation pending. Consumer `extend` now commits a validated
clone, the host prepares a complete candidate behind a registration gate, and extension success
precedes the infallible consumer-registry replacement. The handoff remains open until the current
source receives independent rereview and managed Rust validation.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | plugin registration consumer atomicity | implementation_complete / managed_validation_pending | 二次审查发现 consumer 先提交与 `extend` 逐项写入；现改为候选 registry + registration gate + extension 成功后无失败替换，并新增 rejected extension 不残留 consumer 的 host 回归。 |
