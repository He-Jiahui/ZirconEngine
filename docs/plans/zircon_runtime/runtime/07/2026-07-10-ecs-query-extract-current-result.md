# Runtime 07 ecs-query/extract current result

Date: 2026-07-10

Status: in progress

## `ecs_query`

The available default-feature lib-test binary selected 58 tests. All 56 behavior tests passed; two stale Runtime 15 naming guards failed in that old binary. Both current-source naming guards pass 2/2 after numbered-output routing.

Status: `runtime_07_ecs_query_56_behavior_passed_2_current_guards_passed_fresh_filter_pending`.

## `extract`

The same binary ran 311 tests as 281 passed / 30 failed / 7127 filtered in 125.52 seconds.

Seven Runtime 07/15-owned failures were stale evidence routing or an over-broad source scan. Current-source verification passes:

- performance-hotspots ECS/extract and submit-context guards: 5/5;
- Plan 09 frame-extract geometry and provider F13 structure guards: 2/2;
- frame-extract naming guard: 1/1;
- production-only snapshot-adapter scan: 50 files / 0 offenders;
- scoped rustfmt and diff check.

The remaining 23 failures belong to actively changing render/HGI/UI/Text owners and are not promoted by this record.

Status: `runtime_07_extract_281_passed_30_failed_7_owned_current_source_repaired_23_external_pending`.

Both full filters, FPS/profiling, and the complete lib regression require fresh executable evidence.

Acceptance mirror: `tests/acceptance/runtime-performance-filters-current-result.md`.
