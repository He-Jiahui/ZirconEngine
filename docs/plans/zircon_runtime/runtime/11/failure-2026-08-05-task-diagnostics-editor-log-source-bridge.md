---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-26
summary_slug: task-diagnostics-editor-log-source-bridge
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostic_observation
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_editor/src/core/logging/runtime_task_diagnostics
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
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

Implementation state: `bounded_source_implemented / canonical_editor_projection_implemented / focused_static_passed / independent_source_review_accepted / managed_validation_pending`. Runtime11 owns a 256-entry terminal journal, hard 64-entry read batches, typed scheduler/task identity, a monotonic observation cursor, and 4 KiB UTF-8-safe messages. The terminal source has an independent enable flag, so editor log consumption does not activate full lifecycle counter/timing sampling. Task IDs use the existing 64 diagnostics shards rather than a scheduler-global allocator. Panic maps to runtime-neutral Error and cancellation to Warning. The retained editor host advances one cursor and emits through the existing `EditorLogService` with `LogSource::runtime()`; no runtime-to-editor dependency or second log store was added. Dependency panic before launch now increments `panicked` rather than the previous incorrect `cancelled` path, and the first dependency terminal callback exclusively owns handle state, metrics and observation. Folder-backed scheduler tests and diagnostic snapshots leave `job_scheduler.rs = 347` and `diagnostics.rs = 427`. The focused structure/dependency audit passes 2/2; selected JobSystem audit fields report `expected_module_count = 13`, `behavior_test_anchor_count = 46`, `missing_behavior_test_anchors = []`, `missing_api_snippets = {}`, `oversized_modules = []`, and `runtime_editor_dependency_references = []`. Independent source review reports zero Critical/Important/Minor findings and accepts the slice for managed Cargo validation. The aggregate audit remains blocked only by the pre-existing mesh-builder direct-Rayon path outside this slice. Managed Cargo execution and final acceptance are still pending, so this handoff is not closed or accepted.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Runtime11 task diagnostics -> Editor17 M3.1 source bridge | `open_handoff_recorded` | Current-source inspection confirms task diagnostics expose internal scheduler/report facts but no bounded host observation consumed by `EditorLogService`. The target is Runtime11 plus the existing host boundary; no source code, Cargo validation or compatibility logger was added. |
| 2026-08-26 | Runtime11 bounded terminal observation + EditorLog projection | `implementation_pending_validation` | Implemented 256 retained observations, hard 64-entry batches with cursor-progression coverage, 4 KiB messages, shard-local typed identity, observation-only enablement, exact gap count, panic/error and cancel/warning mapping, first-terminal-winner consistency, canonical runtime-source projection, and deduplicated repeat pump. Folder-backed splits restore `job_scheduler.rs = 347` and `diagnostics.rs = 427`. Focused structure/no-editor-dependency unittests pass 2/2; selected static audit fields are 13 modules / 46 behavior anchors with missing/oversized/dependency-reference sets empty; rustfmt check and scoped `git diff --check` pass. Independent source review reports 0 Critical / 0 Important / 0 Minor findings and accepts the slice for managed Cargo validation. The aggregate audit is blocked by the pre-existing `graphics/.../mesh_draw_command_list/builder.rs` direct-Rayon path; managed Cargo and final acceptance receipts remain pending. |
