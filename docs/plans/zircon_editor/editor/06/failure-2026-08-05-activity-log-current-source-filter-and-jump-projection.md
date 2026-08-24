---
handoff_kind: failure
status: open
created_at: 2026-08-05
updated_at: 2026-08-10
summary_slug: activity-log-current-source-filter-and-jump-projection
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/logging
  - zircon_editor/src/ui/activity/view.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/console_output.rs
  - zircon_editor/src/ui/retained_host/app.rs
tests:
  - filtered current log snapshot replaces stale retained rows
  - source and severity filter projection uses the canonical LogFilter
  - typed LogJump dispatch has one host action path
---

# Editor06: Activity log projection is not consumed by host filter and jump controls

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.1 logging aggregation and Activity read-model projection.
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：Editor17 owns `EditorLogService` and the immutable read-only projection; Editor06 owns Workbench/reflection/retained-host control materialization and user action routing.

## 失败现象与复现证据

`ui/activity/view.rs` now exposes `activity_log_views(&[LogRecord])`, preserving typed source, severity and `LogJump`. Current production references are confined to its module export and unit test; no host reflection or retained-host control invokes it. As a result, the Activity/console surface cannot render the canonical filtered snapshot or dispatch a typed jump.

## 最低共享层根因

The core log service and UI read model are separate by design, but the Editor06 control boundary has no current-snapshot projection from `EditorLogService::snapshot(&LogFilter)` and no single action route for `LogJump`.

## 架构修复验收

- Editor06 materializes the Activity/console log rows from a current filtered `EditorLogService` snapshot and `activity_log_views`; source/severity filtering is expressed with the existing `LogFilter`, not copied into a retained model.
- Each refresh replaces the prior retained row projection. Removed/filtered records disappear immediately; the control must not retain an independent history, queue or clone of `LogRecord` as authority.
- Rendering preserves sequence, typed source, severity, message, frame and optional `LogJump`. Clicking a jump uses one host action path and does not parse display text or mutate the log store.
- Tests cover filter changes, snapshot replacement after eviction, typed asset/document jump dispatch and empty/no-jump rows.

## 禁止临时方案

- Do not rebuild source/severity filtering in retained host, add a console-local history, or expose raw path strings as jump authority.
- Do not make `core/logging` depend on retained host controls, introduce callback state into `ActivityLogView`, or add compatibility DTOs around `LogRecord`.

## 修复结果与回传

Current state: `host_consumer_implemented / second_review_clean / managed_validation_pending`. Editor06 now rebuilds the Console/Activity rows on every `EditorDataSnapshot` and `EditorChromeSnapshot` projection from `EditorLogService::snapshot(&LogFilter)` followed by `activity_log_views`; the retained payload owns only bounded display fields plus an optional record sequence and never retains `LogRecord` history as authority.

Severity and source controls dispatch typed `MenuAction` values and rebuild the canonical `LogFilter`. Dynamic jump rows carry only `workbench.activity_log.jump.<sequence>`; click handling re-queries `EditorLogService::record`, reads the typed `LogJumpTarget`, and sends the existing `EditorAssetEvent::OpenAsset` route. Asset and script locators therefore continue through the existing project-relative virtual locator resolver without a Windows-only prefix scheme or display-text parsing. Evicted rows fail closed, and rows without a jump have no action token. The existing Clear action now clears the canonical in-memory log store while preserving monotonic sequences and event-sink resync ordering.

Focused regressions cover canonical source/severity filtering, replacement after store eviction, asset/script/no-jump action classification, source-control projection, and opaque jump action tokens. Cargo and live product screenshot validation remain outside this static implementation step; the failure is not returned fixed until the second review and managed Windows evidence are accepted.

The second review found and closed two retained-host interaction gaps: resize now rebuilds scroll extent from the canonical Activity Log projection instead of legacy status history, and Console hit-testing applies the same scroll transform and viewport clipping as paint. Dynamic log rows cannot dispatch outside the clipped viewport, while non-log header controls remain interactive. Popup `Hit/Blocked/Miss` routing runs before scrolled rows, so an open source dropdown cannot click through to a covered jump action. The final targeted review reported `Critical=0 / Important=0`.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor17 Activity log read model -> Editor06 host controls | `open_handoff_recorded` | Production reference search finds `activity_log_views` only in its module export and unit test. The handoff fixes the filter/snapshot/jump ownership boundary without adding a second log state or editing retained-host source. |
| 2026-08-10 | Editor06 current Activity Log consumer | `implementation_complete_second_review_running_validation_pending` | Production Chrome projection consumes `EditorLogService::snapshot(&LogFilter)` plus `activity_log_views`; retained rows expose bounded display fields and opaque sequences only; typed asset/script/no-jump tests and source/severity control tests are present. `git diff --check` passed for the scoped files; no Cargo or screenshot acceptance is claimed. |
| 2026-08-10 | Editor06 retained interaction review closure | `second_review_clean_validation_pending` | Canonical resize projection, scrolled-row hit geometry, viewport clipping, hidden-row exclusion, header control routing, and popup-over-jump priority now have focused regressions. Targeted rereview returned `0 Critical / 0 Important`; managed Cargo and live screenshot evidence remain pending. |
