---
title: Runtime99J Single-Pass Animation Pose Extract
category: zircon_runtime
report_id: Runtime99J-single-pass-animation-pose-extract-2026-08-27
date: 2026-08-27
session_id: root-runtime99j-single-pass-animation-pose-extract-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99J Single-Pass Animation Pose Extract

## Scope

`LevelSystem::build_render_frame_extract` consumes an immutable, frame-sealed
`Arc<BTreeMap<EntityId, Arc<AnimationPoseOutput>>>` while it holds mutable access to the World.
The previous path copied every pose entity into a candidate vector, filtered those candidates into
an `(entity, skeleton)` vector, and then looked every retained entity up in the pose map again to
construct the final `RenderSkeletalPoseExtract` vector.

The extract now iterates the sealed pose map once inside the World access closure and constructs
only the final vector. It retains the world-generation rejection and empty-snapshot fast paths,
the deterministic BTreeMap order, mesh/skeleton eligibility checks, and shared `Arc` pose handles.
No pose payload is deep-cloned.

## Performance Evidence

The isolated Rust model mirrors a 65,536-pose sealed BTreeMap, representative missing-node,
meshless, and missing-skeleton rows, World hash lookups, shared pose handles, and the final extract
layout. It compares the old three-vector/two-map-lookup path with the single-pass final projection.
It runs 31 alternating sample pairs with two repetitions and was compiled with
`rustc +1.94.1 -O -C target-cpu=native` on Windows.

| Metric | Staged projection | Single-pass projection | Change |
|---|---:|---:|---:|
| Allocator calls | 62 | 30 | -51.613% |
| Cumulative requested bytes | 11,534,016 | 6,291,264 | -45.455% |
| P50 | 64,688,300 ns | 45,180,000 ns | -30.157% |
| P95 | 102,241,000 ns | 65,108,000 ns | -36.319% |

Model sources:

- `.codex/state/session-coordinator/runtime99j-single-pass-animation-pose-extract-model.rs`
- `.codex/state/session-coordinator/runtime99j-single-pass-animation-pose-extract-model-result.md`

The model measures the eliminated staging allocations and duplicate pose lookup. It is not a
replacement for managed Cargo behavior tests or whole-frame CPU/GPU profiling.

## Contracts And Validation

- `tools/tests/test_runtime99j_single_pass_animation_pose_extract_performance_contract.py` locks
  direct sealed-map iteration, final DTO construction, and the absence of candidate/skeleton
  staging vectors and pose relookup.
- TDD RED failed all three source-contract checks against the staged path; the implemented contract
  passes 3/3.
- Python bytecode compilation, scoped `rustfmt +1.94.1 --edition 2021 --check`, and scoped
  `git diff --check` pass.
- The post-implementation release model passes its allocation, byte, P50, and P95 gates.
- Cargo type checking and focused animation render-extract behavior/source-guard tests remain
  pending in a managed asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime99J still owns World/Level lifecycle, project I/O, snapshot and clone conservation,
serialization schema, transactionality, generation exhaustion, multi-world scaling, and full
product qualification. This slice only removes redundant per-frame animation pose staging from
the LevelSystem render extract.
