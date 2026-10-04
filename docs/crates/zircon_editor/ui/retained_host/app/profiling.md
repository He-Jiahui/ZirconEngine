---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/commands.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/status.rs
  - zircon_editor/src/ui/retained_host/app/profiling/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/commands.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/profiling/actions/status.rs
  - zircon_editor/src/ui/retained_host/app/profiling/diagnostics.rs
  - zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app profiling action/diagnostics ownership scan
  - app profiling action subowner ownership scan
  - git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Profiling App Boundary

## Purpose

The retained-host profiling app boundary owns performance timeline controls and runtime diagnostics enrichment for the Workbench performance pane. It keeps the app-facing `RetainedEditorHost` methods stable while separating action dispatch, diagnostics snapshot access, and profile snapshot merge policy.

This split supports the 08 M3.S2 retained-host cleanup by making `app/profiling.rs` a structural owner instead of a mixed action/diagnostics/merge implementation file.

## Related Files

- `zircon_editor/src/ui/retained_host/app/profiling.rs` declares profiling child modules and exposes `PERFORMANCE_TIMELINE_ACTION_CONTROL_ID`.
- `zircon_editor/src/ui/retained_host/app/profiling/actions.rs` is the structural performance timeline action entry.
- `zircon_editor/src/ui/retained_host/app/profiling/actions/commands.rs` owns performance timeline action ids, action-to-command mapping, and action mapping tests.
- `zircon_editor/src/ui/retained_host/app/profiling/actions/dispatch.rs` owns profiling-enabled command dispatch, non-profiling status fallback, and presentation invalidation.
- `zircon_editor/src/ui/retained_host/app/profiling/actions/status.rs` owns combined editor/runtime profiling status-line text.
- `zircon_editor/src/ui/retained_host/app/profiling/diagnostics.rs` owns `runtime_diagnostics_with_profile(...)` and the dynamic runtime profile snapshot request.
- `zircon_editor/src/ui/retained_host/app/profiling/snapshot_merge.rs` owns editor/runtime `ProfileSnapshot` merge rules when the `profiling` feature is enabled.

## Behavior Model

Performance timeline button clicks enter through `dispatch_performance_timeline_action(...)`. Non-profiling builds return a status-line message that the controls require a profiling build. Profiling builds map stable Workbench action ids to `ProfileControlCommand`, send the command to the editor diagnostics profiler and the dynamic runtime client, combine both responses into one status line, and invalidate presentation data so the performance pane can refresh.

Runtime diagnostics enter through `runtime_diagnostics_with_profile(...)`. Non-profiling builds return the editor manager diagnostics snapshot directly. Profiling builds request a dynamic runtime profile snapshot and merge it into the editor snapshot before returning diagnostics to pane projection.

Snapshot merging preserves existing editor samples when present, offsets runtime span ids after the editor's current maximum span id, ORs active/feature flags, combines session ids when the editor and runtime sessions differ, and appends runtime frames/spans/counters.

## Design and Rationale

Performance action dispatch and profile snapshot merging are separate behavior families. Action dispatch is user-command handling with status-line side effects, now split again into command grammar, dispatch, and status text children. Diagnostics enrichment is pane data preparation. Snapshot merge is pure profile data policy gated behind the `profiling` feature. Splitting these files lets future performance controls or merge rules change without reopening an unrelated owner.

The `snapshot_merge.rs` module is compiled only with the `profiling` feature because the underlying profile DTOs are not needed in normal editor builds.

## Edge Cases and Constraints

- Unknown performance timeline actions write a status-line diagnostic instead of mutating profiler state.
- Runtime profiler unavailability is represented in the combined status message and does not block editor profiler control.
- Empty editor snapshots can be fully replaced by the runtime snapshot when the editor profiler is inactive and has no samples.
- Span id remapping uses saturating addition to avoid panics on unusually large ids.

## Test Coverage

Implementation-slice validation covers formatting, ownership scans, scoped diff checks, and the current practical Cargo check status. `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` is currently blocked before editor code by unrelated active-worktree `zircon_runtime` post-process render errors. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 profiling action subowner split reduced `profiling/actions.rs` from 101 lines to a 3-line structural entry. `actions/commands.rs` is 50 lines and owns action ids plus command mapping tests, `actions/dispatch.rs` is 42 lines and owns editor/runtime profiler command dispatch plus non-profiling fallback, and `actions/status.rs` is 13 lines and owns combined status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app profiling action subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Keep future performance timeline command ids in `actions/commands.rs`, dispatch side effects in `actions/dispatch.rs`, status-line formatting in `actions/status.rs`, diagnostics access in `diagnostics.rs`, and profile merge policy in `snapshot_merge.rs`.
