# Runtime 08 ECS filter current result

Date: 2026-07-10

Status: in progress

## Executable results

- `entity`: 78 passed / 3 failed / 7357 filtered. Two stale review/structure guards pass 2/2 in current source; one LUT graph behavior failure remains with the active render owner.
- `observer`: 16 behavior tests passed; the one stale naming guard passes 1/1 in current source.
- `command`: 144 passed / 5 failed / 7289 filtered. Four stale Plan 02/08 structure guards pass 4/4 in current source; one real render GPU-context file-budget failure remains (`gpu.rs` has 800 lines and the guard requires `< 800`).
- `change_tick`: 4/4 passed.
- `messages`: 24/24 passed.
- `ecs`: 330 passed / 10 failed / 7098 filtered. Current-source reconciliation covers all ten old failures: owner-tree 3/3, ECS-kernel split 1/1, naming M2 44/44 aggregate evidence including the six selected guards, and Runtime 07 ECS/extract guards 2/2.

## Current-source changes

- F17 review evidence and Plan 02/08 structure evidence read numbered output records.
- Runtime 08 owner-tree inventory recognizes `observer/callback_registry.rs` and `storage/component_storage/component_results.rs` instead of retired `utils.rs` owners.
- ECS-kernel split evidence reads current status-row/status-map children plus numbered Runtime 15 and Frameworks 02 records.

Status anchors:

- `runtime_08_entity_78_passed_3_failed_2_owned_current_guards_passed_1_external_render_pending`;
- `runtime_08_observer_16_behavior_passed_current_guard_1_passed_fresh_filter_pending`;
- `runtime_08_command_144_passed_5_failed_4_owned_current_guards_passed_1_render_budget_pending`;
- `runtime_08_change_tick_4_passed_messages_24_passed`;
- `runtime_08_ecs_330_passed_10_old_failures_current_source_reconciled_fresh_filter_pending`.

Acceptance mirror: `tests/acceptance/runtime-ecs-kernel-filters-current-result.md`.
