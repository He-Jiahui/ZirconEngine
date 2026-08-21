# Runtime08C Reusable IK Model-Pose Scratch Record

- Date: 2026-08-20
- Owner: `optimize-runtime08c-ik-model-scratch-r1-01a00797-20260820`
- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-17
- Execution plan: `docs/plans/optimize/zircon_runtime/08c-p1-17-ik-model-pose-scratch.md`
- Status: implementation and regression definition complete; managed validation pending

## Problem

Every IK command allocated a fresh optional model-bone vector and traversal
vector. A two-bone command did that twice because the mid solve correctly
rebuilds model transforms after changing the root rotation. The bounded queue
therefore paid repeated allocator cost even when consecutive commands used the
same or smaller skeleton topology.

## Change

- `apply_ik_commands` now owns one batch-local `ModelPoseScratch` and lends it
  to each ordered command.
- The scratch retains a dense `ModelBone` buffer and explicit unresolved,
  visiting, and resolved traversal states. Length follows the current
  topology, while retained heap capacity grows only when a larger topology
  exceeds the prior capacity.
- Each model-pose rebuild resets traversal states and recomputes every model
  transform from the latest local pose. The optimization does not reuse stale
  matrices across the root mutation in a two-bone solve.
- The old allocating algorithm remains only as a test oracle and release
  benchmark baseline.

## Performance Contract

The ignored release gate evaluates a fixed three-bone hierarchy 16,384 times
per sample for 21 alternating-order sample pairs. It emits raw sample arrays,
nearest-rank P50/P95, and `p95_ratio` under marker
`IK_MODEL_POSE_SCRATCH_BENCH_V1`. Acceptance requires scratch P95 to be at most
75% of allocating P95. Absolute latency and the measured ratio remain pending
until the serialized coordinator batch completes.

## Acceptance

- `reusable_model_pose_scratch_matches_allocating_hierarchy_evaluation` locks
  matrices, positions, and rotations against the allocating oracle.
- `reusable_model_pose_scratch_retains_buffers_for_stable_topology` locks model
  and state buffer addresses and capacities across repeated samples.
- `reusable_model_pose_scratch_resets_traversal_state_after_error` locks cycle
  rejection and subsequent valid reuse.
- `ik_model_pose_scratch_two_bone_path_rebuilds_after_root_write_and_recovers_after_failed_command`
  runs the production two-bone helper with a failed cyclic hierarchy followed
  by a valid command on the same scratch and checks the solved tip against its
  model-space target, which requires the post-root-write rebuild.
- `ik_model_pose_scratch_release_benchmark_evidence` defines the 21-pair
  alternating nearest-rank release gate and 75% P95 ceiling.
- Exact-file Rust 1.94.1 rustfmt and scoped `git diff --check`: passed.
- Cargo regressions and release P50/P95: pending the next managed multi-task
  Runtime08C batch; no direct or competing Cargo process was started.

## Remaining Plan Work

This slice removes repeated model-pose workspace allocations only. Prepared
skeleton residency, dense rig programs, incremental subtree updates,
one-model-pose-per-rig execution, command priority/conflict semantics, joint
limits, and orientation/contact behavior remain under Runtime08C P1-17.
