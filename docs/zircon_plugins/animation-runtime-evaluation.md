---
related_code:
  - zircon_runtime/src/core/framework/animation/target_id.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/animation_clip_compile_error.rs
  - zircon_plugins/animation/runtime/src/evaluation/skeleton_target_table.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compiled_animation_clip.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_clip_track.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_slot.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_table.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_table_error.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/pose_buffer.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/storage.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/animation_clip_evaluator.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/channel_validation.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/clip_sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs
  - zircon_runtime/src/core/resource/snapshot.rs
implementation_files:
  - zircon_runtime/src/core/framework/animation/target_id.rs
  - zircon_plugins/animation/runtime/src/evaluation/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/animation_clip_compile_error.rs
  - zircon_plugins/animation/runtime/src/evaluation/skeleton_target_table.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compiled_animation_clip.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_clip_track.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_slot.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_table.rs
  - zircon_plugins/animation/runtime/src/evaluation/target_table_error.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/pose_buffer.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/storage.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/animation_clip_evaluator.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/channel_validation.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/clip_sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs
  - zircon_runtime/src/core/resource/snapshot.rs
plan_sources:
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_plugins/animation/runtime/tests/animation_target_table_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_compiled_evaluator_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_runtime/tests/resource_snapshot_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_allocation_contract.rs
validation:
  - 2026-07-10: historical focused M1-T1 DTO suite passed 4/4 after a focused Windows type check passed.
  - 2026-07-10: review-corrected target ownership and weighted-pose implementation passed WSL cargo check for all animation runtime test targets in 13m08s.
  - 2026-07-10: standalone direct-owner harnesses compiled the current production owners with the original integration tests and passed target-table 10/10 plus weighted-pose 4/4; temporary harness files were deleted.
  - 2026-07-10: formal WSL Cargo passed target-table 10/10, weighted-pose 4/4, compiled evaluator 10/10, and production scene tick baseline 16/16.
  - 2026-07-10: latest production tick binary passed 19/19, covering the original 16 contracts plus remove/re-add cache invalidation, invalid transition duration, and bind-reference additive boundaries.
doc_type: module-detail
---

# Animation Runtime Evaluation

## Purpose and current boundary

Plugins 04 M1-T1 introduces stable animation target identity and loading-time dense resolution. M1-T2 introduces weighted structure-of-arrays pose storage, a reusable buffer pool, and override/additive blend operators. These are implementation foundations for the planned four-stage evaluator.

The production clip sampler now consumes `CompiledAnimationClip` through `AnimationClipEvaluator`, and graph/state-machine clip sampling routes through the same evaluator. Graph mask matching and state/pose merge still contain string-oriented lookups, and final `AnimationPoseOutput` construction still allocates/clones names. Therefore this document does not claim that the whole production animation tick is runtime-string-free or allocation-free. Those remaining gates belong to M1-T3.

## Ownership and module shape

- `zircon_runtime::core::framework::animation::AnimationTargetId` is the shared 128-bit DTO in the plan-named `target_id.rs` owner.
- `SkeletonTargetTable` is compiled once from one skeleton and then shared with compiled clips through `Arc`. Its public API exposes stable IDs and resolved bone indices, not raw dense slots.
- `TargetTable<T>` and `TargetSlot` are crate-private implementation details. A slot from one skeleton cannot be passed through public API to another skeleton table.
- `CompiledAnimationClip` owns cloned channel payloads and an `Arc<SkeletonTargetTable>`. `CompiledClipTrack` does not expose its raw slot or retain target strings.
- `AnimationClipEvaluator` is a plugin-declared ECS resource registered through `RuntimePluginModuleRegistration::resource(...)`; the tick path only binds the current `ResourceManager` event stream and never lazily inserts the resource.
- `PoseBuffer` owns aligned translation, rotation, scale, and effective-weight vectors. `PosePool` reuses all four vectors across evaluations.
- Multi-responsibility owners use folder-backed façades: `evaluation/mod.rs`, `compiled_animation_clip/mod.rs`, and `pose_buffer/mod.rs`; declaration, compilation, storage, and blend behavior remain in narrow children.

## Stable identity and canonical paths

`AnimationTargetId` hashes the namespace `zircon.animation.target.v1`, followed by every ordered UTF-8 segment encoded as `u64 little-endian byte length + bytes`; the first 128 BLAKE3 bits are stored. The golden path `Armature/Hips/Spine/Chest` is locked to `[135, 4, 92, 99, 71, 46, 103, 133, 47, 42, 67, 184, 28, 111, 114, 206]`. A segment matrix covers concatenation, slash, empty-segment, and non-ASCII boundaries.

Skeleton bone names must be non-empty, already trimmed, and contain no `/`. Explicit clip target paths must round-trip through `EntityPath` unchanged; normalized aliases such as leading/trailing whitespace, repeated separators, or padded segments are rejected instead of hashing to a different skeleton identity.

## Loading-time compilation

1. `SkeletonTargetTable::compile` walks every parent chain, validates the immediate child/parent relation, rejects cycles, and derives each full-path ID once for the skeleton.
2. The internal table binds every ID to a dense row and preserves the stable ID for diagnostics.
3. `CompiledAnimationClip::compile` receives `Arc<SkeletonTargetTable>` and resolves each explicit full path or unique legacy leaf name without rebuilding the skeleton table.
4. Two source tracks resolving to the same row return `DuplicateTrackTarget` with both track indices and the stable ID; source order never silently selects the winner.
5. Public consumers can query the resolved bone index for a track but cannot extract or mix skeleton-scoped slots.

This follows Bevy's full-path stable identity and Fyrox's pre-bound track-handle model. Zircon intentionally uses a dense, skeleton-owned table because later pose/mask/skinning arrays share its row order.

## Weighted pose storage and blending

`PoseBuffer` uses four aligned vectors: translations, rotations, scales, and weights. Reset produces identity TRS with zero effective weight. `set_transform` validates finite TRS, normalizes rotation, and marks the row fully valid; `set_weight` accepts only finite values in `[0, 1]`.

Override blending multiplies the operator weight by the source row weight, uses vector lerp and shortest-path quaternion slerp, and accumulates destination validity. Additive blending applies weighted translation delta, identity-to-source rotation delta, and scale delta from one; output validity is the maximum existing/effective additive weight. Shape mismatches and invalid operator weights are structured errors.

`PosePool::with_buffers` preallocates all four channel capacities. Steady-state acquire/reset/blend/release must not allocate; capacity misses are counted explicitly rather than hidden.

## Failure and boundary behavior

- Empty, padded, or slash-containing bone names fail skeleton compilation.
- Non-canonical explicit paths and fallback leaf names fail clip compilation.
- Missing parents report the immediate bad child; parent cycles are typed errors.
- Duplicate skeleton identities and duplicate clip tracks are separate typed errors.
- `u32` slot exhaustion maps to `TargetCapacityExceeded` instead of being misreported as a duplicate.
- Source strings can change after clip compilation without changing the resolved row, but this proves only the compiled asset contract, not yet the production sampler path.

## Reference evidence

- Bevy `dev/bevy/crates/bevy_animation/src/lib.rs`: stable full-path animation target identity, evaluator caches, and blend accumulation.
- Fyrox `dev/Fyrox/fyrox-animation/src/track.rs`, `pose.rs`, and `value.rs`: pre-bound targets, pose storage, and weighted blending.
- Unreal `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimationRuntime.cpp`: additive translation/rotation/scale accumulation semantics.

## Validation state

Historical implementation anchor: `plugins_04_m1_t1_target_identity_and_dense_compile_path_implemented`.

Historical focused-test anchor: `plugins_04_m1_t1_target_identity_focused_tests_passed` (4/4 for the earlier DTO slice).

Current review-correction anchor: `plugins_04_m1_t3_production_compiled_evaluator_review_corrections_in_progress`.

Current weighted-pose anchor: `plugins_04_m1_t2_weighted_pose_formal_cargo_4_of_4_passed`.

Formal WSL Cargo now passes the target suite 10/10, weighted pose/pool 4/4, compiled evaluator 10/10, atomic resource snapshot 1/1, and the latest production tick binary 19/19. Earlier external compile-drift attempts remain historical only and are not used as acceptance evidence. M1 remains in progress because graph/mask/state dense compilation, reusable production output, diagnostics, bounded eviction, and full four-stage ownership are still pending.
