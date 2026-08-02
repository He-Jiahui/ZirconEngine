---
related_code:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/ik_command.rs
  - zircon_runtime/src/core/framework/animation/ik_command_error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/animation/playback_settings.rs
  - zircon_runtime/src/core/framework/animation/pose_bone.rs
  - zircon_runtime/src/core/framework/animation/pose_output.rs
  - zircon_runtime/src/core/framework/animation/pose_source.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/core/framework/animation/state_machine_evaluation.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/animation/track_path_error.rs
  - zircon_runtime/src/core/framework/animation/tests.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_runtime/src/animation/sequence.rs
implementation_files:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/ik_command.rs
  - zircon_runtime/src/core/framework/animation/ik_command_error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/state_machine_evaluation.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/animation/tests.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_runtime/src/core/framework/animation/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs::review_f5_animation_manager_uses_animation_error
  - avatar_mask_filters_exact_leaf_and_excluded_targets
  - animation_tick_contract_records_work_events_and_sanitized_delta
  - gpu_skinning_readiness_requires_enabled_gpu_resources
  - timeline_track_masks_and_clip_status_sanitize_contract_values
  - runtime_status_reports_player_rig_and_gpu_readiness
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs::manager_ik_commands_are_validated_and_drained_per_world
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/ik_postprocess.rs
  - cargo +nightly check --locked --offline -p zircon_plugin_animation_runtime --test animation_ik_contract --jobs 1 (2026-07-11: passed)
  - cargo +nightly check --locked --offline -p zircon_plugin_animation_runtime --test runtime_physics_animation_tick_contract --jobs 1 (2026-07-11: passed)
  - CARGO_INCREMENTAL=0; RUSTFLAGS=-C debuginfo=0; cargo +nightly test --locked --offline -p zircon_plugin_animation_runtime --test animation_ik_contract --jobs 1 --target-dir F:\cargo-targets\zircon-animation-m3-lowmem -- --nocapture (2026-07-11: 4/4 passed in 457.8s)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/animation/*.rs (planned for current slice)
  - git diff --check -- zircon_runtime/src/core/framework/animation docs/zircon_runtime/core/framework/animation.md docs/zircon_plugins/animation/runtime.md (planned for current slice)
  - cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never (pending while active Cargo lanes are busy)
doc_type: module-detail
---

# Animation Framework Contracts

## Purpose

`zircon_runtime::core::framework::animation` is the neutral animation contract layer. It defines the versioned skeleton/clip/sequence/graph/state-machine resource schemas, timeline descriptors, playback settings, evaluation DTOs, pose output records, avatar masks, GPU-skinning readiness, stable-ID IK commands, tick requests/reports, runtime status snapshots, typed animation errors, and the `AnimationManager` trait. It does not own project loading, concrete clip sampling, graph blending, IK target-slot compilation/solving, scene mutation, authored timeline UI, GPU buffer allocation, or event dispatch.

Concrete runtime behavior remains in `zircon_plugins/animation/runtime`. The framework gives runtime, editor, scripting, and future VM plugin callers a shared vocabulary for animation service access without importing the plugin crate or sharing plugin-owned objects.

## Related Files

The framework is folder-backed and `mod.rs` is only the public re-export surface.

- `error.rs` defines `AnimationError` and `AnimationResult` for framework-facing manager, pose sampling, and sequence writeback failures.
- `asset/` is the unique versioned animation resource-schema owner, including `ZRANIM01` binary conversion and typed `AnimationAssetError`; asset import/cache code consumes these records without re-exporting them from the asset facade.
- `manager.rs` defines `AnimationManager` and its neutral status, control, evaluation, and command surface. It does not project concrete asset models into timeline descriptors or accept `scene::World` for mutation.
- `ik_command.rs` defines script/component-facing TwoBone and LookAt commands in skeleton model space; `ik_command_error.rs` classifies unsupported managers, non-finite input, invalid weight, degenerate axis, and bounded-queue overflow.
- `tick.rs`, `event.rs`, and `runtime_status.rs` define world tick inputs, emitted clip-event records, per-player status, per-rig status, and aggregate runtime status.
- `timeline.rs`, `track_path.rs`, and `sequence_apply_report.rs` describe property tracks, bone tracks, event tracks, timeline clip spans, sequence writeback paths, and missing-track reporting.
- `avatar_mask.rs`, `graph_clip_instance.rs`, `graph_evaluation.rs`, and `state_machine_evaluation.rs` describe masked/additive graph output and state-machine transition reports.
- `pose_bone.rs`, `pose_output.rs`, `pose_source.rs`, and `gpu_skinning.rs` describe sampled pose output and GPU skinning readiness without owning renderer resources.

`AnimationPoseOutput::clone_from_reusing_storage` is the neutral handoff primitive for a stable rig. It updates source/state data in place, reuses the existing bone vector, and reuses each bone-name string allocation when topology and names remain stable. The contract remains an owned snapshot; callers do not receive plugin-owned pool buffers or renderer resources.

## Behavior Model

Animation is contract-first. Runtime scene components and animation assets may refer to clips, sequences, graphs, state machines, skeletons, and stable target ids, but the framework only describes what a manager can report or request. The plugin decides how to sample clips, blend graph layers, apply sequence property tracks, dispatch events, and record poses on `LevelSystem`.

Timeline descriptors summarize authored or imported animation data:

- sequence descriptors expose property tracks as `AnimationTrackPath` records plus optional stable target ids;
- clip descriptors expose bone-transform tracks and event tracks;
- track descriptors can carry an `AnimationAvatarMask`, allowing editor/runtime tools to reason about masked targets before a concrete graph evaluator runs;
- clip spans sanitize start, duration, playback speed, and weight so callers can display or validate bad data without panicking.

The framework owns only these descriptor DTOs and their validation helpers. The retired `from_sequence`, `from_clip`, `sequence_timeline_descriptor`, `clip_timeline_descriptor`, and `sequence_track_paths` convenience APIs directly imported concrete asset models and had no production callers; the hard cut deletes them instead of preserving asset projections or compatibility wrappers. A concrete animation/editor adapter that actually needs a descriptor must assemble it at the implementation boundary.

Runtime status snapshots summarize live state:

- player status records identify clip, sequence, graph, or state-machine players and whether they are stopped, playing, paused, waiting for assets, or invalid;
- rig status records identify skeleton readiness, pose coverage, missing targets, avatar mask state, and GPU-skinning readiness;
- aggregate status records combine players, rigs, last tick work, and diagnostics for editor panels, VM calls, diagnostics, and headless tooling.

## Reference Alignment

The contract split follows local reference-engine evidence:

- Bevy keeps animation graphs, graph masks, animation targets, players, transitions, and events in separate Rust modules under `dev/bevy/crates/bevy_animation/src`. Zircon mirrors that split with framework DTOs for graph clips, masks, events, timeline descriptors, and runtime player status.
- Godot separates `AnimationPlayer`, `AnimationMixer`, `AnimationTree`, blend spaces, state machines, track caches, audio tracks, and root-motion concerns under `dev/godot/scene/animation`. Zircon keeps mixer/player/tree concepts visible as neutral timeline and runtime-status data, but leaves concrete cache and mutation behavior in the plugin.
- Unreal's runtime animation source tree separates animation assets, asset player nodes, state-machine nodes, pose data, montage/composite concepts, and GPU-skinning-adjacent runtime types under `dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation`. Zircon adopts the long-lived contract shape while landing it as Rust-friendly framework DTOs.

The deliberate divergence is that Zircon does not add a non-network `server` layer for animation. Consumers resolve `AnimationManager` through `zircon_runtime::core::manager` and exchange framework records.

## Control Flow

The Runtime animation absorption layer registers the default manager and manager handle. `zircon_plugins/animation/runtime` registers `animation.evaluate`, resolves that neutral `AnimationManager`, advances scene player components, loads assets through `ProjectAssetManager`, samples/blends poses, consumes IK commands after the final layer blend, emits events, and records pose/playback state on `LevelSystem`.

Framework callers can request:

1. default playback settings;
2. normalized track paths;
3. graph or state-machine evaluations;
4. pose sampling for a clip;
5. sequence evaluation records; concrete sequence writeback is invoked by the Plugin evaluation pipeline through the Runtime sequence owner, not through the neutral manager;
6. validated, bounded IK work through `queue_ik_command`, drained by the owning world tick;
7. runtime status through the data-only manager methods; timeline descriptors are assembled by concrete implementation adapters rather than by the neutral manager.

The framework defaults return inert status and descriptor records so optional managers can remain small while still satisfying the trait.

Runtime 15 F5 animation manager typed errors records the typed boundary as `runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred`. `AnimationManager::sample_clip_pose` returns `AnimationResult`; the upper animation sequence apply function also returns `AnimationResult` while owning concrete `scene::World` mutation. Concrete animation owners map skeleton bind, clip sample, quaternion normalization, and sequence channel conversion failures into `AnimationError` variants without putting `World` back into the framework trait.

## Edge Cases

Framework DTOs sanitize non-finite and negative times, playback speeds, and weights. Target matching accepts exact ids and slash-path leaf matches, matching current clip target-id and sequence target-id behavior. Empty or muted target descriptors do not match runtime targets. Missing skeletons, missing targets, GPU resource gaps, invalid players, and waiting-for-asset states are represented as status data rather than panics.

IK commands are one-shot and world-scoped. The neutral boundary validates finite target/pole/axis/clamp values, normalized weights, and non-degenerate LookAt axes before enqueue. Stable `AnimationTargetId` values deliberately cross the framework boundary; skeleton-scoped dense slots never do. Optional custom managers return `Unsupported` by default instead of pretending to accept work.

`AnimationPlayerRuntimeStatus` also sanitizes its JSON boundary. `time_seconds` and `playback_speed` serialize and deserialize as finite non-negative values, and `weight` serializes and deserializes as a finite `0.0..=1.0` value. `AnimationRuntimeStatus::sanitized_snapshot()` exposes the same comparison shape used by the serde round-trip guard, so diagnostics and editor panels do not receive JSON `null` values from `NaN` or infinite runtime floats.

Concrete managers must still validate asset availability, clip duration, graph cycles, state-machine transition validity, malformed quaternion channels, skeleton/track mismatch, GPU resource allocation, and event dispatch ordering. The framework-level error surface intentionally covers manager/apply failures that callers can act on without parsing strings: non-finite skeleton bind fields, zero-length bind rotations, sample type mismatches, non-finite samples, zero-length quaternion samples, non-finite sequence channel samples, and zero-length sequence channel quaternions.

The 2026-06-04 Plugin runtime follow-up temporarily split sequence writeback under `zircon_plugins/animation/runtime/src/sequence/`. The 2026-08-01 hard cut deleted that duplicate facade and its children: `zircon_runtime/src/animation/sequence.rs` and `sequence/{apply,channel_sample,compiled,conversion,interpolation,target,tests,time}.rs` are now the sole sequence implementation owners, re-exported by the Plugin crate root and called from `evaluation/pipeline/sequences.rs`. The earlier Frameworks05 cut remains intact: `AnimationManager` does not accept `scene::World`, and no retired scene-hook or Plugin-local sequence compatibility path survives.

## Test Coverage

Framework tests lock avatar mask target filtering, tick/event report behavior, GPU-skinning readiness, track mask matching, clip status sanitization, runtime player/rig aggregation, and serde round-trips for runtime status records. `test_animation_timeline_contract_does_not_project_asset_models` keeps asset-to-timeline conversion out of framework, while `test_animation_manager_contract_does_not_mutate_scene_world` keeps sequence writeback in the concrete runtime/scene owner. Animation plugin contracts additionally cover Manager IK validation/world isolation. `review_f5_animation_manager_uses_animation_error` still protects `AnimationError`/`AnimationResult` across manager sampling and upper sequence conversion owners.

Focused Cargo validation for the current framework-contract update is pending while active Cargo lanes are running. The intended focused check is:

```powershell
cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never -- --nocapture --test-threads=1
```
