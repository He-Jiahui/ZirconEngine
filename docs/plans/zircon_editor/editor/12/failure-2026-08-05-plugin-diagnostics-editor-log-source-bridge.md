---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-05
summary_slug: plugin-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/plugin/catalog.rs
  - zircon_editor/src/core/plugin/extension_materialization.rs
  - zircon_editor/src/core/plugin/lifecycle_message_bridge.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_editor/src/core/logging
tests:
  - plugin-id-preserving diagnostic projection
  - plugin lifecycle and materialization error severity mapping
  - duplicate/replay diagnostics do not create a second log authority
---

# Editor12: plugin diagnostics do not reach the canonical EditorLog source

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：plugin catalog, materialization and lifecycle diagnostics have package identity only under Editor12; Editor17 owns the canonical log service and must not duplicate plugin state.

## 失败现象与复现证据

Current plugin catalog/materialization paths accumulate diagnostic strings and the plugin ABI exposes diagnostics emission, but none project those facts into `EditorLogService`. A user can therefore see plugin registration failure state without a filterable Activity/log record whose source is the responsible plugin.

## 最低共享层根因

Plugin-owned diagnostics carry package identity in Editor12, but the catalog/materialization/lifecycle boundary has no bounded canonical-log projection or replay identity.

## 架构修复验收

- At the Editor12 host boundary, catalog, materialization, lifecycle and ABI diagnostic facts emit through the existing `EditorLogService` with `LogSource::plugin(plugin_id)`; plugin identity is validated and never inferred from display text.
- Severity mapping is explicit and preserves diagnostics after fault isolation/revoke without running a callback twice or mutating plugin state because log delivery is retried.
- The projection uses a bounded cursor/dedup identity over existing plugin diagnostics; it does not keep a second retained history or reintroduce raw `Vec<String>` as UI authority.
- Tests cover a valid plugin id, empty-id rejection, callback panic/materialization failure, ABI diagnostic delivery and duplicate replay. Editor17 consumes the resulting canonical records unchanged.

## 禁止临时方案

- Do not build a plugin-only toast/history or write directly to retained host controls.
- Do not introduce a fallback `LogSource::editor()` for a known plugin, a raw global logger, or a second plugin diagnostic queue.

## 修复结果与回传

Open state: `source_contract_drift_recorded / no_local_rollback / target_validation_pending`. No Plugin/runtime-interface source was changed by Editor17; the existing M3.1 core remains the sole sink.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor12 plugin diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | Current-source inspection found plugin diagnostic collection and ABI emission surfaces but no production `LogSource::plugin` projection outside tests. The handoff fixes ownership and acceptance without adding source code or a compatibility channel. |
