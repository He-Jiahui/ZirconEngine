---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-05
summary_slug: task-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_editor/src/core/logging
tests:
  - bounded task diagnostic observation and no editor-crate dependency
  - runtime diagnostic severity/message projection through the editor host bridge
  - repeated diagnostic snapshot does not create a second log authority
---

# Runtime11: task diagnostics have no EditorLog source bridge

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 six-source logging aggregation.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：runtime task diagnostics own their metrics, lifetime and snapshot accuracy; Editor17 owns `EditorLogService` and must not make `zircon_runtime` depend on `zircon_editor`.

## 失败现象与复现证据

`core/runtime/tasks/diagnostics.rs` exposes scheduler counters and reports, but it has no bounded typed observation contract that a host can convert into editor records. The editor log core therefore receives Play process output only; task warnings, cancellation and panic diagnostics remain isolated from the Activity/log sink.

## 最低共享层根因

Runtime11 owns only internal task diagnostics, while the runtime/editor host boundary has no typed bridge from those facts to the editor-owned canonical log sink.

This is not permission for Runtime11 to import `zircon_editor` or construct `LogEntry`. That would invert the runtime-to-editor dependency and create a second logging authority.

## 架构修复验收

- Runtime11 publishes a bounded, monotonic runtime-neutral task diagnostic observation at its existing scheduler/report boundary; it preserves task identity, severity and message facts without retaining unbounded history.
- The editor host consumes that observation at the existing runtime/editor boundary and emits it through the sole `EditorLogService` as `LogSource::runtime()`.
- Replayed or unchanged observations are deduplicated by their typed identity/cursor; a full snapshot resync may rebuild the consumer view but must not create another runtime log store.
- Runtime11 tests prove no `zircon_editor` dependency, bounded retention and terminal/panic/cancel severity mapping. The cross-plan host test proves a single resulting editor log record per new observation.

## 禁止临时方案

- Do not add an editor dependency, callback into UI code, raw global logger or an unbounded diagnostic queue in `zircon_runtime`.
- Do not add a seventh log channel or stringify task diagnostics into a retained-host-only panel.

## 修复结果与回传

Open state: `source_contract_drift_recorded / no_local_rollback / target_validation_pending`. Editor17 keeps the integrated logging core and Play source forward; this record assigns the missing Runtime11 producer bridge without claiming a runtime test or managed validation result.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Runtime11 task diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | Current-source inspection confirms task diagnostics expose internal scheduler/report facts but no bounded host observation consumed by `EditorLogService`. The target is Runtime11 plus the existing host boundary; no source code, Cargo validation or compatibility logger was added. |
