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
  - zircon_runtime/src/core/framework/animation/clip_event_sampling.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/animation/clip_event.rs
implementation_files:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/clip_event_sampling.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/clip_event.rs
plan_sources:
  - user: 2026-07-13 书面设计通过，批准 Runtime02 注册服务 CoreWeak 拆分设计并开始实施
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md
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
  - tools/tests/test_frameworks_01_scene_animation_boundary.py::Frameworks01SceneAnimationBoundaryTests::test_scene_does_not_depend_on_optional_animation_implementation
  - zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs::registry_owned_services_store_only_weak_runtime_back_references
  - zircon_runtime/src/animation/manager/mod.rs::animation_manager_playback_settings_recover_poisoned_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/sequence/channel_sample.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs::review_f5_animation_manager_uses_animation_error
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs::runtime_animation_backlog_boundary_requires_doc_update
  - "pending: cargo test -p zircon_runtime --lib animation --locked"
doc_type: module-detail
---

# Runtime Animation Module

`core::framework::animation` is the neutral contract owner. `zircon_runtime::animation` owns the manager and concrete sampling implementation, registers the canonical `animation.runtime` module identity, and installs `DefaultAnimationManager`; the plugin-owned `animation.evaluate` system is the sole production `SystemStage::PostUpdate` evaluator. The evaluator publishes immutable Level pose snapshots through `LevelSystem::record_animation_pose_snapshot`; unchanged paused instances retain their last snapshot and skeletal targets without a new sample or transform apply.

The current implementation includes clip event sampling, sequence application, skeleton clip pose sampling, graph evaluation, state machine evaluation, weighted base/additive graph pose blending, and scene-world pose application for named descendants.

State-machine `Trigger` parameters are one-shot only in the production plugin
transaction. The compiled selected transition supplies shared Trigger-name
metadata, but the player mutation removes current Trigger values only after
pose sampling succeeds and clip-event admission accepts the entity. A failed
pose sample or deferred event batch retains active state, transition state, and
Trigger values for retry. The neutral manager evaluator remains pure and never
consumes caller parameters.

## Runtime Shape

The runtime is split into four local responsibilities:

| Area | Runtime owner | Current status | Boundary judgement |
|---|---|---|---|
| Module registration | `animation/module.rs` | Implemented | Keep at crate-root module family because it registers the manager service. |
| Manager API implementation | `animation/manager/mod.rs` and submodules | Implemented | Keep as the runtime implementation of `core::framework::animation::AnimationManager`; the root is folder-backed and the old flat `animation/manager.rs` path is retired. |
| Clip-event sampling boundary | `core/framework/animation/clip_event_sampling.rs` and `animation/clip_event.rs` | Implemented, validation pending | Neutral request/result/sampler contracts point from scene to the optional implementation; project-asset loading and sampling remain in animation. |
| Plugin evaluation | `zircon_plugins/animation/runtime/src/runtime_system.rs` and `evaluation/pipeline/` | Implemented | `animation.evaluate` consumes the runtime contracts and is the only production pose/event evaluator. |
| Sequence helpers | `animation/sequence.rs` and submodules | Implemented | Keep local because sequence sampling is animation-domain behavior, not generic scene behavior. |

## Reference Comparison

Runtime 14 uses Bevy, Fyrox, and Godot only as boundary references, not as feature-complete parity requirements.

| Capability | Reference anchor | Zircon status | Judgement |
|---|---|---|---|
| Clip asset sampling | Bevy `AnimationClip`, Fyrox animation tracks | Present via `sample_clip_pose` and sequence channel sampling | Implemented for current runtime needs. |
| Clip events | Bevy `animation_event.rs` | Present via budgeted `sample_clip_events_budgeted` and plugin event publishing | Resumable count/time/byte bounded drain. |
| Animation graph | Bevy `AnimationGraph`, Godot `AnimationTree` | Present as asset graph evaluation plus runtime base/additive pose blending | Implemented baseline, but not a full editor-authored blend tree. |
| State machine | Fyrox machine, Godot `animation_node_state_machine` | Present via manager contracts and plugin state-machine evaluation | Implemented baseline. |
| Transitions | Bevy `AnimationTransitions`, state machine references | Present for state machine transition sampling and pose blending | Implemented baseline. |
| Root motion | Godot `root_motion_view`, engine animation controllers | Not extracted as a runtime motion output | Backlog debt. Needs explicit component/output contract before implementation. |
| Morph targets | Bevy `morph.rs` | Present through asset/scene property/sequence tracks and graphics CPU extraction paths, but not as a dedicated animation-system morph solver | Implemented baseline outside the animation manager; future expansion must coordinate asset, render, and graphics owners. |
| GPU skinning upload | Render pipelines in mature engines | Not owned here | Deliberate non-goal. Rendering plans own GPU upload and skinning buffers. |
| Editor authoring tools | Fyrox editor animation plugin, Godot animation editors | Not owned here | Deliberate non-goal for `zircon_runtime::animation`; editor tooling must live outside this module. |

## Ownership

`zircon_runtime::animation` should keep its crate-root seat for manager and playback implementation. Neutral DTOs and inversion interfaces belong in `core::framework::animation`; moving project-asset loading or playback algorithms there would mix contracts with implementation, while moving them into `scene` would make scene own asset-specific animation behavior.

`DefaultAnimationManager` is installed in the Runtime service registry, so its runtime back-reference is `CoreWeak`. Construction may borrow `&CoreHandle`, but the manager upgrades only at the playback-settings persistence boundary; a dead Runtime root skips persistence while the manager's already-owned local playback settings remain readable. This prevents the registry entry from retaining the root through `ServiceEntry.instance`.

The stable boundary is:

- `core::framework::animation` defines contracts and DTOs.
- `asset` owns serialized animation, graph, skeleton, and state machine asset shapes.
- `animation` owns manager access, sequence helpers, project-asset clip-event sampling, and the concrete sampler.
- `scene` owns the bounded Level event queue, age, retry, and overflow policy through the neutral sampler contract; it does not import the optional animation implementation.
- `zircon_plugins/animation/runtime` owns plugin package metadata and the sole `animation.evaluate` runtime-system registration and evaluation pipeline.
- `render` and `graphics` own GPU skinning and draw submission.

## Evaluation Diagnostics

The prior `AnimationSceneFrameDiagnostics` and `animation.scene.scanned_entities` rows are Runtime 07 historical evidence, not a current production owner. The plugin evaluator now reports through its evaluation diagnostics while `LevelSystem` owns the bounded event backlog, age, overflow, unavailable-asset, and oversized-event state.

This status is `animation_scene_frame_diagnostics_static_passed_cargo_deferred`. It gives Runtime 07 an evidence path for animation scene-hook frame cost before any M2 optimization is proposed; the neutral clip-event sampling contract does not move playback implementation into core and does not claim GPU skinning or draw submission ownership.

Runtime 14 M0.1 is therefore complete as an architecture judgement. No code migration is required for this slice.

Runtime 14 M1 adds `runtime_animation_backlog_boundary_requires_doc_update` as a backlog/non-goal guard. The guard locks this document to the current code facts: `compile_sequence_for_world` owns the import/edit boundary and `apply_compiled_sequence_to_world` applies its retained typed writers, `compiled_sequence_applies_mesh_renderer_morph_weight_track` proves the existing morph-weight property track baseline, root motion remains backlog debt, `render` and `graphics` own GPU skinning and draw submission, and editor authoring tools stay outside `zircon_runtime::animation`.

Runtime 15 M1 animation manager folder-backed cutover is recorded as `runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred`. Runtime 15 M1 adds `runtime_15_animation_manager_is_folder_backed` as the structure guard for the manager entry cutover. The guard locks the old flat `animation/manager.rs` path as retired, requires `animation/manager/mod.rs` to own `DefaultAnimationManager` and the child module mounts, and keeps graph, parameters, pose, sampling, and state-machine behavior in `animation/manager/{graph,parameters,pose,sampling,state_machine}.rs`. This closes the `manager.rs` plus `manager/` coexistence debt for animation without changing the service-registration behavior or public `DefaultAnimationManager` facade; the current canonical module identity is `animation.runtime`.

Runtime 15 F5 animation typed errors is recorded as `runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred`. `core::framework::animation` owns `AnimationError` and `AnimationResult`; `AnimationManager::sample_clip_pose`, concrete clip sampling, channel helpers, and `animation::sequence::{compile_sequence_for_world, apply_compiled_sequence_to_world(...)}` return `AnimationResult` instead of public `Result<_, String>`. The Frameworks05 cut removed scene writeback from the neutral manager trait, so `animation.evaluate` retains compiled sequence writers and invokes the compiled upper sequence function directly. `review_f5_animation_manager_uses_animation_error` keeps typed errors synchronized without reintroducing `scene::World` into framework.

## IK Ownership

The former global IK command queue had no production producer. It duplicated a
`Mutex<HashMap<WorldHandle, ...>>` in both fallback and plugin managers, then
forced the animation tick to drain and partition that inbox on every frame.
The 2026-08-24 hard cut removes the queue, its replacement-epoch state, its
postprocess adapter, and the benchmark that measured only the retired adapter.

TwoBone and LookAt mathematical solvers remain in the Animation plugin. Product
IK is not accepted until a compiled animation graph/evaluation node owns its
parameters, target slots, scratch storage, diagnostics, and per-instance state.
This follows Unreal's skeletal-control node model: pose input and node-local
state are evaluated together, rather than submitted through a process-wide
manager inbox. No performance or power claim is made for the hard cut; future
graph-local integration requires product traces and focused measurements.

## Playback Settings Lock Recovery

Runtime 15 M3 animation manager lock poison recovery is recorded as `runtime_15_animation_manager_lock_poison_recovery_static_passed_cargo_deferred`. `DefaultAnimationManager` now uses the private `lock_playback_settings()` helper for both `store_playback_settings` and `AnimationManager::playback_settings()`, so a poisoned playback settings mutex is recovered instead of panicking with `animation playback mutex poisoned`.

The module-local `animation_manager_playback_settings_recover_poisoned_lock` test poisons the playback settings lock and verifies that store/read still works afterward. `structure_convention/lock_poison_policy.rs::runtime_15_animation_manager_lock_poison_recovery_guard_covers_playback_settings` keeps this behavior tied to Runtime 15 status output, `docs/plans/engine-code-structure-convention.md`, `docs/plans/engine-code-review-findings-2026-06.md`, and this animation owner document. Full `module_convention_gate` and full animation Cargo sweep remain pending because the implementation slice used scoped rustfmt/static validation while external Cargo/Rust lanes were active.
