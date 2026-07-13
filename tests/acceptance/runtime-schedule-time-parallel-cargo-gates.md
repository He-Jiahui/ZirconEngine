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
status: accepted_current_source
---

# Runtime Schedule, Time, and Parallel Cargo Gate Acceptance

Date: 2026-07-11

A freshly compiled default-feature, locked `zircon_runtime` lib-test binary in
a coordinator-managed lane produced these focused results:

- `ecs_schedule`: 77 passed, 0 failed, 7415 filtered out, 0.52 seconds.
- `tests::time::`: 4 passed, 0 failed, 7488 filtered out, 0.02 seconds.
- `session`: 162 passed, 0 failed, 10 ignored, 7320 filtered out, 44.83 seconds.
- `schedule_parallel`: 15 passed, 0 failed, 7477 filtered out, 0.04 seconds.

The direct Runtime 03 audit is also green: source owners 19/19, guard/test
owners 11/11, stage variants 9/9, fixed-loop stages 3/3, behavior anchors
13/13, missing Cargo anchors empty, and `risks = []`.

These results accept Runtime 03's schedule graph, runtime-time/fixed-step,
dynamic-session, and parallel-executor filters. The ten ignored session cases
require the documented real ZR VM/runtime-library environment. The downstream
`zircon_app` package gate is separately green at 135 passed, 0 failed, and one
documented ignored capture. This acceptance does not claim the full Runtime
package or the complete runtime architecture program green.
