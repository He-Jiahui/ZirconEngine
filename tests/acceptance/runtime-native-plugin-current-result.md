# Runtime 06 native-plugin current result

Date: 2026-07-10

Status: in progress

## Executable baseline

The latest available default-feature `zircon_runtime` lib-test binary predating the current repairs ran the `native_plugin` filter as:

- 138 selected;
- 124 passed;
- 14 failed;
- 7300 filtered out;
- 395.79 seconds.

One failure was behavioral: `hot_reload_state_restore_failure_rolls_back_and_reports`. The previous native package was restored and reinserted, but the lifecycle diagnostic still reported that rollback was unavailable. The current source replaces that ambiguous boolean state with an explicit previous-package disposition and adds `native_live_host_rollback_plan_reports_when_previous_plugin_was_restored`.

The other 13 failures were evidence-owner drift:

- nine typed-error review guards read route-only priority plan parents;
- four structure/status guards referenced retired status subtree names.

## Current-source verification

- Native-plugin typed-error review harness: 18 passed / 0 failed.
- Typed-error review `include_str!` resolution: 212 paths / 0 missing.
- Runtime 15 review-guard status source inventory: 84 paths / 0 missing.
- Scoped rustfmt: passed.
- Scoped `git diff --check`: passed, with line-ending notices only.

## Remaining gate

A freshly compiled default-feature lib-test binary must rerun `native_plugin` before Runtime 06 can promote this filter. Current status anchors:

- `runtime_06_native_hot_reload_restore_rollback_state_machine_repaired_fresh_cargo_pending`;
- `runtime_06_native_plugin_typed_error_review_guards_18_passed_fresh_filter_pending`;
- `runtime_15_native_plugin_typed_error_numbered_output_and_current_child_routing_static_passed`.
