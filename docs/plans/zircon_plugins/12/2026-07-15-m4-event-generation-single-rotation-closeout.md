# Plugins 12 M4 event generation single-rotation closeout

Plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
Milestone: M4
Status: completed
Files: ["docs/plans/zircon_plugins/12/2026-07-15-first-party-editor-catalog-m4-milestone-manifest.md","docs/plans/zircon_plugins/12/2026-07-15-first-party-editor-catalog-m4-milestone-manifest.committed.md","docs/plans/zircon_plugins/12/2026-07-15-m4-event-generation-single-rotation-closeout.md","zircon_runtime/src/scene/ecs/schedule_runner.rs","zircon_runtime/src/scene/world/derived_state.rs","zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs","zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs"]
Date: 2026-07-15

## Scope delivered

- Replaced the schedule stage teardown replay with a dirty-derived-state-only flush.
- Kept `UpdateEvents` and `ApplyDeferred` in their single ordered schedule positions instead of executing them a second time at stage completion.
- Preserved late derived-state mutation handling by running only current-stage internal systems whose dirty predicate is still true.
- Added a behavior regression that proves the first tick reads no event and the second tick reads the first generation exactly once.
- Added a structural guard that rejects restoring `run_internal_scene_systems_for_stage` at stage teardown.
- Archived the already committed first-party catalog manifest marker by renaming it to `.committed.md`, preventing a second live M4 manifest while preserving the earlier catalog evidence verbatim.

## Fresh testing evidence

- Managed Windows job `d5440e6acbbb4eddbb1dd9f3c89cfe01` ran `world_driver_rotates_event_generations_once_per_tick`: 1 passed, 0 failed.
- Managed Windows job `1092e4be5e7444e6afdfe55aa63c5448` ran the linked-plugin group after the fix: 3 passed, 0 failed.
- Managed Windows job `6e1251841e9a43eea13a0b360d65b25e` ran the runtime-event-mirror group: 3 passed, 0 failed.
- Managed Windows job `e8215abaece742c4b738fd707fc32d78` ran the editor consumer-host group: 6 passed, 0 failed.
- `rustfmt --check`, `git diff --check`, and the schedule/frame-loop audit passed; the audit reports `missing_source_files = []`, `missing_guard_files = []`, and `risks = []`.

## Review

- The first independent review found one Important issue: stage teardown replayed `UpdateEvents`, clearing the current generation.
- The post-fix independent review reports 0 Critical and 0 Important findings and confirms that late dirty derived state still flushes in stage order.

## Status and completed items

| Milestone | Item | Status | Evidence |
|---|---|---|---|
| M4 | Single event-generation rotation | completed | Stage teardown no longer replays unconditional internal systems. |
| M4 | Late derived-state flush | completed | Only `DerivedStateDirty::should_run` systems execute at teardown. |
| M4 | Behavior and structure guards | completed | Managed 1/1 behavior test and structural source guard passed. |
| M4 | Upward runtime/editor regression | completed | Linked 3/3, mirror 3/3, consumer-host 6/6 passed. |
| M4 | Independent review | completed | Final result: 0 Critical, 0 Important. |
