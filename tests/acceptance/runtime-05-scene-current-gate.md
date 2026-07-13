---
related_code:
  - zircon_runtime/src/scene
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene
plan_sources:
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
output_records:
  - docs/plans/zircon_runtime/runtime/05/2026-07-09-scene-editor-boundary-closeout-output-records.md
status: focused_runtime05_gates_accepted_broad_render_scene_regression_blocked
---

# Runtime 05 Scene Current Gate Acceptance

Date: 2026-07-11

## Fresh managed-lane result

The default-feature current-source `scene::` filter completed with 1518
passed, 19 failed, 2 ignored, and 5953 filtered tests in 238.35s.

A subsequent newly compiled package rerun exited with the Windows access-
violation status `-1073741819` after 105.04s, so it produced no trustworthy
aggregate count. Before termination it exposed the already classified mesh-
renderer geometry-order failure and entered the long-running active Render
post-process family; this run is retained as failure evidence, not presented as
a green or complete scene gate.

## Failure ownership

- Render/resource-streamer/post-process/sprite/SDF: 10 failures in active
  Render and text owners.
- Scene behavior and current structure: 6 failures covering dynamic-scene
  owner shape, mobility preflight/freshness, scene-patch typed error/preview,
  and mesh-renderer geometry ordering.
- Runtime 05/Frameworks status routing: the 3 stale guards were routed to the
  numbered Runtime 05, Runtime 15, and Frameworks 02 output records; their
  current-source standalone run is 3/3 passed.
- The dynamic-scene root owner guard now follows the completed Runtime 15 hard
  cutover from retired `document/legacy.rs` to
  `document/v1_project_document.rs`; the component-structure standalone run is
  2/2 passed.
- Mobility source guards now match the typed `SceneError` variants, and patch
  preview workload coverage derives its total from all serialized scene
  components while separately checking the single plugin-owned component.
- Fresh package evidence passes `scene_patch_document` 5/5, dynamic-scene
  absorption guards 8/8, dynamic-scene owner-tree 2/2, mobility preflight 1/1,
  and no-op derived-state behavior 1/1.

## Decision

Not accepted as a complete Runtime 05 milestone. The focused rebuild and all
repaired Runtime 05 subsets are now green. The current broad rerun terminated
without a result, so the earlier 1518/19/2 snapshot remains the last complete
aggregate. The mesh geometry-order failure and active Render/text failures
remain the known broad-gate blockers. No render-side or compatibility path was
added.
