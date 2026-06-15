---
related_code:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager.rs
  - zircon_runtime/src/animation/manager/graph.rs
  - zircon_runtime/src/animation/manager/state_machine.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/animation/scene_hook/graph.rs
  - zircon_runtime/src/animation/scene_hook/state_machine.rs
  - zircon_runtime/src/animation/scene_hook/tick.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/clip_event.rs
implementation_files:
  - zircon_runtime/src/animation/mod.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager.rs
  - zircon_runtime/src/animation/scene_hook.rs
  - zircon_runtime/src/animation/sequence.rs
  - zircon_runtime/src/animation/clip_event.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Fyrox/fyrox-animation/src
  - dev/Fyrox/fyrox-impl/src/scene/animation
  - dev/godot/scene/animation
tests:
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/sequence/channel_sample.rs
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
| Manager API implementation | `animation/manager.rs` and submodules | Implemented | Keep as the runtime implementation of `core::framework::animation::AnimationManager`. |
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

Runtime 14 M0.1 is therefore complete as an architecture judgement. No code migration is required for this slice.

Runtime 14 M1 adds `runtime_animation_backlog_boundary_requires_doc_update` as a backlog/non-goal guard. The guard locks this document to the current code facts: `apply_sequence_to_world` remains the public sequence application hook, `sequence_applies_mesh_renderer_morph_weight_track` proves the existing morph-weight property track baseline, root motion remains backlog debt, `render` and `graphics` own GPU skinning and draw submission, and editor authoring tools stay outside `zircon_runtime::animation`.
