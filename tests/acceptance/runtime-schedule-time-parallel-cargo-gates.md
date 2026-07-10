---
related_code:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/activation/schedule
  - zircon_runtime/src/scene/ecs_schedule
  - zircon_runtime/src/scene/schedule_parallel
plan_sources:
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
output_records:
  - docs/plans/zircon_runtime/runtime/03/2026-07-09-schedule-and-frame-loop-alignment-output-records.md
---

# Runtime Schedule, Time, and Parallel Cargo Gate Acceptance

Date: 2026-07-10

A successfully compiled default-feature, locked `zircon_runtime` lib-test binary produced these focused results:

- `ecs_schedule`: 77 passed, 0 failed, 7361 filtered out, 0.76 seconds.
- `tests::time::`: 4 passed, 0 failed, 7434 filtered out, 0.01 seconds.
- `fixed_update`: 3 passed, 0 failed, 7435 filtered out, 0.02 seconds.
- `schedule_parallel`: 15 passed, 0 failed, 7423 filtered out, 0.04 seconds.

These results accept Runtime 03's schedule graph, runtime-time/fixed-step, and parallel-executor focused filters. They do not accept the separate `session` filter or the full runtime package. The same binary's `session` filter reported 154 passed, 8 failed, and 10 ignored; all eight failures were stale structure guards, and a newly compiled binary is still required after their current-owner/archive-routing repair.
