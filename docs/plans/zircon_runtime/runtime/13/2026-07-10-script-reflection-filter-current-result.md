# Runtime 13 script and reflection filter current result

Date: 2026-07-10

Status: in progress

## Executable results

- `script::`: 60/60 passed, 7378 filtered, in 1.45 seconds.
- `script_binding`: 5 passed / 1 failed, 7432 filtered, in 0.84 seconds. The only failure was the stale route-owner split guard; current standalone script-binding guards pass 3/3 after numbered-output/current-status-child routing.
- `reflection`: 63 passed / 3 failed / 2 ignored, 7370 filtered, in 91.72 seconds. Script reflection and scene reflection paths passed. One stale SSR structure guard now passes 1/1 in current source; two reflection-probe render-product behavior failures remain with the active render owner.

Status anchors:

- `runtime_13_script_filter_60_passed_script_binding_5_passed_1_guard_current_passed`;
- `runtime_13_reflection_filter_63_passed_3_failed_2_ignored_1_current_guard_passed_2_external_render_pending`.

Runtime13 remains in progress until a fresh binary reruns the script-binding guard and the plan's broader package gates are closed.
