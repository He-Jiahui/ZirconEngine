---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-05
summary_slug: import-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/ui/host/editor_asset_manager
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_editor/src/core/logging
tests:
  - import completion failure and warning projection
  - repeated import completion cursor/dedup contract
  - asset jump target and canonical LogSource import channel
---

# Editor09: import diagnostics are not projected into EditorLog

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：import admission, completion identity and asset jump data belong to the Editor09 import/asset manager boundary; Editor17 cannot reconstruct them from worker-pool counters.

## 失败现象与复现证据

The import pipeline and editor asset manager own completion and failure facts, but production code does not publish them to `EditorLogService`. Import outcomes are consequently unavailable to the typed Activity log filter/jump model, despite `LogSource::import()` already existing in the canonical core.

## 最低共享层根因

Editor09 owns import completion identity and asset jump data, but its producer boundary has no cursor-based projection to the editor-owned log service.

## 架构修复验收

- Editor09 projects each new import warning/error/completion through `EditorLogService` with `LogSource::import()`, preserving typed asset URI or document jump data when it exists.
- The producer is bounded and cursor-based over its own completion identity; repeated frame observation and snapshot resync cannot multiply identical records.
- Import job Progress and log records remain separate canonical projections: the producer must not manufacture a private history, toast queue or second completion receiver.
- Tests cover success, warning, terminal failure, duplicate observation, asset jump dispatch and bounded retention under a completion storm.

## 禁止临时方案

- Do not parse worker-pool diagnostic text in retained UI, write logs directly from Runtime04, or add a duplicate import diagnostics store.
- Do not use `LogSource::editor()` for import facts or omit their existing typed jump identity.

## 修复结果与回传

Open state: `source_contract_drift_recorded / no_local_rollback / target_validation_pending`. This assigns the missing Editor09 source producer only; Editor17 did not modify asset/import code.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor09 import diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | `LogSource::import()` exists but current production searches find no import producer calling the canonical log service. The handoff records the exact owner boundary and required bounded projection without changing import sources. |
