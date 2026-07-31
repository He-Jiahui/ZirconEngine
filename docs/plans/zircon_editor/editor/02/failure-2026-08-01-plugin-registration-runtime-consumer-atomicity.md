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

## Failure

`register_editor_plugin_registration` committed the plugin's runtime-event consumers before it
validated and committed the extension batch. A scene-mode factory mismatch or another extension
validation error therefore returned failure while the consumer remained registered. In addition,
`EditorRuntimeEventConsumerRegistry::extend` inserted registrations directly, so a duplicate found
late in the batch left earlier consumers committed.

## Required Repair

- Build and validate a candidate consumer registry without mutating the active registry.
- Serialize standalone consumer registration with complete plugin registration.
- Commit the prepared consumer registry only after the extension batch succeeds; the final
  consumer commit must be infallible.
- Prove a rejected plugin can subsequently register the same consumer id and that duplicate batch
  rejection leaves the original registry unchanged.

## Forbidden Workarounds

No best-effort unregister, partial rollback, duplicate-id alias, silent consumer replacement, or
test-only cleanup.

## Repair Result

Implementation complete, managed validation pending. Consumer `extend` now commits a validated
clone, the host prepares a complete candidate behind a registration gate, and extension success
precedes the infallible consumer-registry replacement. The handoff remains open until the current
source receives independent rereview and managed Rust validation.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | plugin registration consumer atomicity | implementation_complete / managed_validation_pending | 二次审查发现 consumer 先提交与 `extend` 逐项写入；现改为候选 registry + registration gate + extension 成功后无失败替换，并新增 rejected extension 不残留 consumer 的 host 回归。 |
