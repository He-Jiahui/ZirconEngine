---
related_code:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/graph.rs
  - zircon_runtime/src/animation/manager/state_machine.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/animation/scene_hook/graph.rs
  - zircon_runtime/src/animation/scene_hook/state_machine.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/animation/scene_hook/events.rs
  - zircon_runtime/src/animation/scene_hook/node_pose.rs
  - zircon_runtime/src/animation/scene_hook/pending.rs
  - zircon_runtime/src/animation/scene_hook/scan.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/clip_event.rs
implementation_files:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/animation/scene_hook/events.rs
  - zircon_runtime/src/animation/scene_hook/node_pose.rs
  - zircon_runtime/src/animation/scene_hook/pending.rs
  - zircon_runtime/src/animation/scene_hook/scan.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/clip_event.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Fyrox/fyrox-animation/src
  - dev/Fyrox/fyrox-impl/src/scene/animation
  - dev/godot/scene/animation
tests:
  - zircon_runtime/src/animation/manager/mod.rs::animation_manager_playback_settings_recover_poisoned_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/sequence/channel_sample.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs::review_f5_animation_manager_uses_animation_error
  - zircon_runtime/src/animation/scene_hook/node_pose.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_animation_backlog_boundary_requires_doc_update
  - "pending: cargo test -p zircon_runtime --lib animation --locked"
doc_type: module-detail
---

# Runtime Animation Module

`zircon_runtime::animation` is a runtime-owned playback module, not a thin registration shell. It registers `AnimationModule`, installs `DefaultAnimationManager`, and exposes a scene runtime hook that samples animation requests during `SystemStage::PostUpdate`.

The current implementation includes clip event sampling, sequence application, skeleton clip pose sampling, graph evaluation, state machine evaluation, weighted base/additive graph pose blending, and scene-world pose application for named descendants.

## Runtime Shape

The runtime is split into four local responsibilities:

| Area | Runtime owner | Current status | Boundary judgement |
|---|---|---|---|
| Module registration | `animation/module.rs` | Implemented | Keep at crate-root module family because it registers manager and scene hook services. |
| Manager API implementation | `animation/manager/mod.rs` and submodules | Implemented | Keep as the runtime implementation of `core::framework::animation::AnimationManager`; the root is folder-backed and the old flat `animation/manager.rs` path is retired. |
| Scene hook playback | `animation/scene_hook.rs` and submodules | Implemented | Keep behavior local to animation; it consumes asset/scene/framework contracts and writes scene pose/event results. |
| Sequence helpers | `animation/sequence.rs` and submodules | Implemented | Keep local because sequence sampling is animation-domain behavior, not generic scene behavior. |

## Reference Comparison

Runtime 14 uses Bevy, Fyrox, and Godot only as boundary references, not as feature-complete parity requirements.

| Capability | Reference anchor | Zircon status | Judgement |
|---|---|---|---|
| Clip asset sampling | Bevy `AnimationClip`, Fyrox animation tracks | Present via `sample_clip_pose` and sequence channel sampling | Implemented for current runtime needs. |
| Clip events | Bevy `animation_event.rs` | Present via `sample_clip_events` and scene hook event publishing | Implemented. |
| Animation graph | Bevy `AnimationGraph`, Godot `AnimationTree` | Present as asset graph evaluation plus runtime base/additive pose blending | Implemented baseline, but not a full editor-authored blend tree. |
| State machine | Fyrox machine, Godot `animation_node_state_machine` | Present via manager and scene hook state machine evaluation | Implemented baseline. |
| Transitions | Bevy `AnimationTransitions`, state machine references | Present for state machine transition sampling and pose blending | Implemented baseline. |
| Root motion | Godot `root_motion_view`, engine animation controllers | Not extracted as a runtime motion output | Backlog debt. Needs explicit component/output contract before implementation. |
| Morph targets | Bevy `morph.rs` | Present through asset/scene property/sequence tracks and graphics CPU extraction paths, but not as a dedicated animation-system morph solver | Implemented baseline outside the animation manager; future expansion must coordinate asset, render, and graphics owners. |
| GPU skinning upload | Render pipelines in mature engines | Not owned here | Deliberate non-goal. Rendering plans own GPU upload and skinning buffers. |
| Editor authoring tools | Fyrox editor animation plugin, Godot animation editors | Not owned here | Deliberate non-goal for `zircon_runtime::animation`; editor tooling must live outside this module. |

## Ownership

`zircon_runtime::animation` should keep its crate-root seat. Moving it into `core::framework::animation` would mix framework contracts with playback implementation, while moving playback into `scene` would make scene own asset-specific animation behavior.

The stable boundary is:

- `core::framework::animation` defines contracts and DTOs.
- `asset` owns serialized animation, graph, skeleton, and state machine asset shapes.
- `animation` owns runtime evaluation and scene-hook application.
- `zircon_plugins/animation/runtime` owns plugin package metadata and runtime-system registration, but currently wraps and re-exports the runtime-owned animation family instead of deleting `zircon_runtime::animation`.
- `render` and `graphics` own GPU skinning and draw submission.

## Scene Hook Diagnostics

Runtime 07 M1.1 adds `AnimationSceneFrameDiagnostics` inside the scene hook owner. The hook records count-only `DiagnosticStore` rows for `animation.scene.scanned_entities`, sequence/clip/graph/state-machine samples, `animation.scene.output_poses`, `animation.scene.applied_transforms`, `animation.scene.published_events`, and `animation.scene.state_transitions`. Empty manager and disabled playback frames write zeroes through the same paths.

This status is `animation_scene_frame_diagnostics_static_passed_cargo_deferred`. It gives Runtime 07 an evidence path for animation scene-hook frame cost before any M2 optimization is proposed; it does not move animation contracts into `core::framework::animation` and does not claim GPU skinning or draw submission ownership.

Runtime 14 M0.1 is therefore complete as an architecture judgement. No code migration is required for this slice.

Runtime 14 M1 adds `runtime_animation_backlog_boundary_requires_doc_update` as a backlog/non-goal guard. The guard locks this document to the current code facts: `apply_sequence_to_world` remains the public sequence application hook, `sequence_applies_mesh_renderer_morph_weight_track` proves the existing morph-weight property track baseline, root motion remains backlog debt, `render` and `graphics` own GPU skinning and draw submission, and editor authoring tools stay outside `zircon_runtime::animation`.

Runtime 15 M1 animation manager folder-backed cutover is recorded as `runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred`. Runtime 15 M1 adds `runtime_15_animation_manager_is_folder_backed` as the structure guard for the manager entry cutover. The guard locks the old flat `animation/manager.rs` path as retired, requires `animation/manager/mod.rs` to own `DefaultAnimationManager` and the child module mounts, and keeps graph, parameters, pose, sampling, and state-machine behavior in `animation/manager/{graph,parameters,pose,sampling,state_machine}.rs`. This closes the `manager.rs` plus `manager/` coexistence debt for animation without changing the `AnimationModule` service registration or public `DefaultAnimationManager` facade.

Runtime 15 F5 animation manager typed errors is recorded as `runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred`. `core::framework::animation` now owns `AnimationError` and `AnimationResult`, while `AnimationManager::sample_clip_pose`, `AnimationManager::apply_sequence_to_world`, `DefaultAnimationManager`, clip pose sampling, channel sample helpers, and sequence channel conversion return `AnimationResult` instead of public `Result<_, String>`. The typed variants distinguish non-finite skeleton bind fields, zero-length bind rotations, sample type mismatches, non-finite samples, zero-length quaternion samples, non-finite sequence channel samples, and zero-length sequence channel quaternions. `review_f5_animation_manager_uses_animation_error` keeps this document, Runtime 15 status, review findings, and the framework animation contract document synchronized with the code owner.

## Playback Settings Lock Recovery

Runtime 15 M3 animation manager lock poison recovery is recorded as `runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred`. `DefaultAnimationManager` now uses the private `lock_playback_settings()` helper for both `store_playback_settings` and `AnimationManager::playback_settings()`, so a poisoned playback settings mutex is recovered instead of panicking with `animation playback mutex poisoned`.

The module-local `animation_manager_playback_settings_recover_poisoned_lock` test poisons the playback settings lock and verifies that store/read still works afterward. `structure_convention/lock_poison_policy.rs::runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings` keeps this behavior tied to Runtime 15 status output, `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, and this animation owner document. Full `module_convention_gate` and full animation Cargo sweep remain pending because the implementation slice used scoped rustfmt/static validation while external Cargo/Rust lanes were active.
