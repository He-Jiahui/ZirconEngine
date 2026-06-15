---
related_code:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
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
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
implementation_files:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/state_machine_evaluation.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/animation/tests.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_runtime/src/core/framework/animation/tests.rs
  - avatar_mask_filters_exact_leaf_and_excluded_targets
  - animation_tick_contract_records_work_events_and_sanitized_delta
  - gpu_skinning_readiness_requires_enabled_gpu_resources
  - timeline_descriptor_summarizes_sequence_property_tracks
  - timeline_descriptor_summarizes_clip_bone_and_event_tracks
  - timeline_track_masks_and_clip_status_sanitize_contract_values
  - runtime_status_reports_player_rig_and_gpu_readiness
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/animation/*.rs (planned for current slice)
  - git diff --check -- zircon_runtime/src/core/framework/animation docs/zircon_runtime/core/framework/animation.md docs/zircon_plugins/animation/runtime.md (planned for current slice)
  - cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never (pending while active Cargo lanes are busy)
doc_type: module-detail
---

# Animation Framework Contracts

## Purpose

`zircon_runtime::core::framework::animation` is the neutral animation contract layer. It defines timeline descriptors, playback settings, graph evaluation DTOs, state-machine evaluation DTOs, pose output records, avatar masks, GPU-skinning readiness, tick requests/reports, runtime status snapshots, and the `AnimationManager` trait. It does not own concrete clip sampling, graph blending, scene mutation, authored timeline UI, GPU buffer allocation, or event dispatch.

Concrete runtime behavior remains in `zircon_plugins/animation/runtime`. The framework gives runtime, editor, scripting, and future VM plugin callers a shared vocabulary for animation service access without importing the plugin crate or sharing plugin-owned objects.

## Related Files

The framework is folder-backed and `mod.rs` is only the public re-export surface.

- `manager.rs` defines `AnimationManager`, including default status and timeline descriptor accessors.
- `tick.rs`, `event.rs`, and `runtime_status.rs` define world tick inputs, emitted clip-event records, per-player status, per-rig status, and aggregate runtime status.
- `timeline.rs`, `track_path.rs`, and `sequence_apply_report.rs` describe property tracks, bone tracks, event tracks, timeline clip spans, sequence writeback paths, and missing-track reporting.
- `avatar_mask.rs`, `graph_clip_instance.rs`, `graph_evaluation.rs`, and `state_machine_evaluation.rs` describe masked/additive graph output and state-machine transition reports.
- `pose_bone.rs`, `pose_output.rs`, `pose_source.rs`, and `gpu_skinning.rs` describe sampled pose output and GPU skinning readiness without owning renderer resources.

## Behavior Model

Animation is contract-first. Runtime scene components and animation assets may refer to clips, sequences, graphs, state machines, skeletons, and stable target ids, but the framework only describes what a manager can report or request. The plugin decides how to sample clips, blend graph layers, apply sequence property tracks, dispatch events, and record poses on `LevelSystem`.

Timeline descriptors summarize authored or imported animation data:

- sequence descriptors expose property tracks as `AnimationTrackPath` records plus optional stable target ids;
- clip descriptors expose bone-transform tracks and event tracks;
- track descriptors can carry an `AnimationAvatarMask`, allowing editor/runtime tools to reason about masked targets before a concrete graph evaluator runs;
- clip spans sanitize start, duration, playback speed, and weight so callers can display or validate bad data without panicking.

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

`zircon_plugins/animation/runtime` registers the concrete module, default manager, manager handle, and scene runtime hook. The scene hook resolves `AnimationManager`, advances scene player components, loads animation assets through `ProjectAssetManager`, samples/blends poses, emits clip events, and records pose/playback runtime state on `LevelSystem`.

Framework callers can request:

1. default playback settings;
2. normalized track paths;
3. graph or state-machine evaluations;
4. pose sampling for a clip;
5. sequence writeback through `apply_sequence_to_world`;
6. status or timeline descriptors through the data-only methods.

The framework defaults return inert status and descriptor records so optional managers can remain small while still satisfying the trait.

## Edge Cases

Framework DTOs sanitize non-finite and negative times, playback speeds, and weights. Target matching accepts exact ids and slash-path leaf matches, matching current clip target-id and sequence target-id behavior. Empty or muted target descriptors do not match runtime targets. Missing skeletons, missing targets, GPU resource gaps, invalid players, and waiting-for-asset states are represented as status data rather than panics.

`AnimationPlayerRuntimeStatus` also sanitizes its JSON boundary. `time_seconds` and `playback_speed` serialize and deserialize as finite non-negative values, and `weight` serializes and deserializes as a finite `0.0..=1.0` value. `AnimationRuntimeStatus::sanitized_snapshot()` exposes the same comparison shape used by the serde round-trip guard, so diagnostics and editor panels do not receive JSON `null` values from `NaN` or infinite runtime floats.

Concrete managers must still validate asset availability, clip duration, graph cycles, state-machine transition validity, malformed quaternion channels, skeleton/track mismatch, GPU resource allocation, and event dispatch ordering.

2026-06-04 plugin runtime follow-up split `zircon_plugins/animation/runtime/src/sequence.rs` into a structural facade plus `sequence/{apply,channel_sample,conversion,interpolation,target,tests,time}.rs`. This did not change the neutral framework contracts; sequence binding iteration, channel sampling, interpolation, target-id fallback, and scene property writeback remain plugin-owned runtime behavior behind the same `AnimationManager::apply_sequence_to_world(...)` capability.

## Test Coverage

Framework tests lock avatar mask target filtering, tick/event report behavior, GPU-skinning readiness, sequence timeline descriptor generation, clip bone/event descriptor generation, track mask matching, clip status sanitization, runtime player/rig aggregation, and serde round-trips for runtime status records. `runtime_animation_status_json_boundary_sanitizes_non_finite_values` keeps the Runtime 14 plan and module-family audit tied to the same JSON boundary guard.

Focused Cargo validation for the current framework-contract update is pending while active Cargo lanes are running. The intended focused check is:

```powershell
cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never -- --nocapture --test-threads=1
```
