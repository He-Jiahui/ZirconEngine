# Runtime 14 module-family filter current result

Date: 2026-07-10

Status: in progress

## Executable results

- `module_family`: 2 passed / 2 stale plan guards failed. Current folder-backed module-family suite passes 6/6.
- `diagnostic_log`: 15/15 passed.
- `engine_module`: the old default-feature binary passed 6 and failed 1, exposing a real declared-layer public-surface omission: `ServiceFactory` was no longer re-exported beside the other canonical `core::runtime` contracts. The direct re-export is restored; a later current-source target-client lib-test binary passes the full `engine_module` filter 7/7.
- `navigation`: 109 passed / 4 failed. The stale Runtime09 navigation guard passes in the current 11/11 legacy-route suite; three UI behavior failures remain external.
- `animation`: 39 passed / 6 failed. All six failures were stale Physics owner, review-output, typed-error, or Runtime14 plan guards and are reconciled in current source; animation behavior tests in the filter passed.

The direct `module_family_boundary` audit reports four root families with counts animation/navigation/diagnostic_log/engine_module = 28/9/7/8, all missing lists empty, and `risks = []`.

Status anchors:

- `runtime_14_module_family_old_2_passed_2_failed_current_root_family_6_passed`;
- `runtime_14_diagnostic_log_15_passed`;
- `runtime_14_engine_module_service_factory_restored_target_client_filter_7_passed`;
- `runtime_14_navigation_109_passed_4_failed_1_guard_current_passed_3_external_ui_pending`;
- `runtime_14_animation_39_passed_6_failed_all_6_current_guards_reconciled_fresh_filter_pending`.
