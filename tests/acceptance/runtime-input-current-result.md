# Runtime 09 / Runtime 12 input current result

Date: 2026-07-10

Status: in progress

## Results

| Evidence | Result | Acceptance meaning |
|---|---:|---|
| available default-feature `input` filter | 429 passed / 25 failed / 1 ignored | old-binary triage baseline only |
| Runtime12 input-stack current source | 11/11 | current owner, docs, inventory, and split guards pass |
| Runtime15 input naming current source | 3/3 | mouse-wheel, DOM keycode, and winit baseline guards pass |
| Runtime09 route/name current source | 11/11 | numbered evidence and current split owners pass |
| scene-world visibility owner current source | 1/1 | numbered Plan09/priority evidence passes |
| Runtime12 structure audit | risks = [] | counts 12/20/7/6 and all missing lists empty |

All twelve stale guard failures from the old binary are covered by current-source evidence. Thirteen UI behavior failures remain with active UI/Text owners, so the broader `input` gate is not promoted. A newly compiled default-feature runtime binary must rerun the filter before acceptance.

Status anchors:

- `runtime_09_input_filter_old_binary_429_passed_25_failed_1_ignored_12_current_guards_reconciled_13_ui_behavior_pending`;
- `runtime_12_input_stack_audit_risks_empty_standalone_11_passed_fresh_filter_pending`;
- `runtime_15_runtime09_runtime12_input_guard_current_owner_reconciliation_static_passed`.
