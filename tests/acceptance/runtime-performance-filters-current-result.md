# Runtime 07 performance filters current result

Date: 2026-07-10

Status: in progress

## `ecs_query`

The available default-feature `zircon_runtime` lib-test binary selected 58 tests: 56 behavior tests passed and two stale Runtime 15 naming guards failed. The two current-source guards pass 2/2.

Status: `runtime_07_ecs_query_56_behavior_passed_2_current_guards_passed_fresh_filter_pending`.

## `extract`

The same binary selected 311 tests: 281 passed, 30 failed, 7127 were filtered out, and execution finished in 125.52 seconds.

Seven Runtime 07/15-owned failures were stale evidence routing or source-scan scope. Current-source verification after repair:

- ECS/extract counter and submit-context performance guards: 5/5;
- frame-extract geometry and provider F13 structure guards: 2/2;
- frame-extract snapshot naming guard: 1/1;
- production submit-tree snapshot scan: 50 files, zero forbidden adapters;
- scoped rustfmt and diff check: passed.

The other 23 failures are in active render/HGI/UI/Text owners. They remain visible and are not counted as this slice's success.

Status: `runtime_07_extract_281_passed_30_failed_7_owned_current_source_repaired_23_external_pending`.

## Remaining gate

Both full filters require a newly compiled default-feature lib-test binary. The Runtime 07 FPS/profiling command and complete lib regression also remain pending.
