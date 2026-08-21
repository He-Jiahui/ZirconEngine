# Runtime08C P1-17 Reusable IK Model-Pose Scratch Plan

- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-17
- Owner: `optimize-runtime08c-ik-model-scratch-r1-01a00797-20260820`
- Scope: IK post-process model-pose workspace allocation

## Change

1. Lock model-pose output and invalid-hierarchy behavior against the allocating
   implementation before changing production storage.
2. Reuse model-bone and traversal-state buffers across commands in one IK
   drain while preserving command order and pose recomputation.
3. Prove that repeated same-topology samples retain both buffer allocations.
4. Add a 21-pair alternating-order release gate with nearest-rank P50/P95 and
   raw samples for the allocating and scratch implementations.
5. Run this gate in the next serialized Runtime08C managed batch; do not start
   a standalone Cargo process while another coordinator job is active.

## Acceptance

- Scratch and allocating model-pose evaluation produce identical matrices,
  positions, and rotations for a non-trivial hierarchy.
- A cyclic hierarchy remains a typed `InvalidSkeletonHierarchy` failure after
  a successful prior sample, proving traversal state is reset.
- The production two-bone path recovers from a failed cyclic-hierarchy command,
  reaches the model-space tip target that requires the post-root-write rebuild,
  and reuses the same scratch for the following valid command.
- Repeated same-topology evaluation keeps the model and traversal buffer
  addresses and capacities stable.
- The release gate emits exactly 21 alternating pairs, nearest-rank P50/P95,
  raw sample arrays, and requires scratch P95 to be at most 75% of allocating
  P95 on the fixed three-bone repeated-workload fixture.
- This slice does not claim prepared skeleton residency, one-model-pose-per-rig,
  joint limits, or multi-command arbitration from the remaining P1-17 work.
