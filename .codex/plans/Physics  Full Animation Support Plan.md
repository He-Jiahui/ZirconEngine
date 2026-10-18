# Physics + Full Animation Support Plan

## Summary
- Build physics as an optional runtime extension owned by `zircon_runtime`, with backend-neutral core logic in `zircon_physics` and a new `zircon_physics_jolt` adapter crate as the first real backend.
- Build animation as a full stack in `zircon_animation`: sequence/property animation, skeletal clips, animation graphs, and state machines. Shared property-path foundations are declared up front, then implemented first for scene ECS plus the first-wave physics/animation components.
- Keep repository layering consistent: `zircon_framework` holds stable manager contracts and DTOs, `zircon_manager` exposes resolvers/handles, `zircon_runtime` owns module registration and runtime-facing interfaces, and `zircon_scene` remains the ECS/world authority.
- Reference baseline: Fyrox leads physics/component data and property-path animation, Bevy informs animatable-property targeting, Theatre leads sequence UI hierarchy, and Unreal MovieScene/Sequence leads binding-track-channel ownership while v1 stays simpler.

## Key Changes
- Shared Foundations: add `zircon_framework::physics` and `zircon_framework::animation` modules, plus matching `zircon_manager` service names/resolvers. `zircon_runtime::extensions::{physics, animation}` become folder-backed owners and expose runtime-facing `physics_interface` / `animation_interface` facades.
- Shared Foundations: introduce stable `EntityPath` and `ComponentPropertyPath` addressing. Canonical animation/inspector binding form is `entity_path:component.property[...]`; assets store the structured path and runtime compiles it to cached lookup IDs. First implementation coverage is all current scene ECS components plus new physics/animation components.
- Shared Foundations: replace the current fixed inspector surface with generic property entries keyed by subject path + property path, reusing the existing editor binding/reflection plumbing instead of inventing a separate animation-only inspector path.
- Shared Foundations: extend `zircon_resource` / `zircon_asset` with `PhysicsMaterial`, `AnimationSkeleton`, `AnimationClip`, `AnimationSequence`, `AnimationGraph`, and `AnimationStateMachine` asset kinds. Animation assets are authoritative versioned binary files; physics materials stay in the normal typed asset flow.
- Physics: convert `zircon_runtime::extensions::physics` into a real module with project settings, manager wiring, backend selection, and schedule hooks. Backend switching is project-settings driven and takes effect on module/world restart, not mid-simulation hot swap.
- Physics: move backend-agnostic runtime integration into `zircon_physics`: backend trait, world lifecycle, transform sync, query/event DTOs, rigid body/collider/joint schemas, layer/group types, and physics settings. Add new workspace crate `zircon_physics_jolt` to initialize and drive Jolt through that trait surface.
- Physics: add scene ECS components for rigid bodies, colliders, joints/constraints, and physics material references. Colliders carry shape, sensor flag, layer/group data, and optional local overrides; material assets carry friction, restitution, and combine rules.
- Physics: add project settings for backend choice, layer names, collision/filter matrix, solver groups, fixed tick rate, max substeps, and simulation mode. Runtime stepping is explicit: pre-physics sync, fixed-step simulate, contact/query collection, post-physics writeback.
- Animation: split `zircon_animation` into `sequence`, `property`, `skeleton`, `graph`, `state_machine`, and `runtime` subdomains. The sequence model is a simplified Unreal-style `Sequence -> Binding -> Track -> Channel -> Keys`; v1 excludes nested subsequences/directors/movie-render features.
- Animation: use the shared property-path registry for all property animation. Sequence tracks are grouped by entity/component/property path so the editor tree, runtime binding, and serialization all use the same hierarchy.
- Animation: use cubic Hermite as the authored/evaluated curve model. Auto tangents, fitting, and key reduction use a Householder-based solver so editor tooling, import, and runtime sampling all share one math path.
- Animation: full stack scope includes skeletal animation. Extend import/runtime data so models can produce `AnimationSkeleton` and `AnimationClip` assets, add skeleton/player components, solve poses before `RenderExtract`, and allow sequence/property tracks plus skeletal graphs to target the same entity tree.
- Animation: add `AnimationGraph` and `AnimationStateMachine` assets with clip, blend, transition, and parameter nodes. Runtime order is fixed: parameter updates and sequence/property overrides in `Update`, graph/state solve in animation stage, pose/skinning output before `RenderExtract`.
- Editor And Tooling: add `AnimationSequenceEditorView` and `AnimationGraphEditorView` to the existing workbench/activity system. UI shape follows Theatre: outline/track tree on the left, dope sheet + curve graph in the center, details/parameters on the right.
- Editor And Tooling: expand the current animation binding payloads beyond `PositionOfTrackAndFrame` to cover track creation/removal, target rebinding, key editing, scrubbing, playback, graph editing, and state transition editing. All editor interaction stays inside the existing native-binding/reflection architecture.
- Editor And Tooling: make inspector and sequence editor share one canonical property model. Picking a field in inspector yields the exact path used for track creation and runtime evaluation.

## Public Interfaces
- `zircon_framework::physics`: stable `PhysicsManager` trait plus DTOs/handles for simulation settings, world/query operations, layer masks, collision groups, and physics material metadata.
- `zircon_framework::animation`: stable traits/handles for sequence playback, skeletal clip playback, graph instances, state machines, and parameter control.
- `zircon_manager`: new handles, service names, and resolver helpers for physics and animation managers.
- `zircon_scene`: new ECS component types for physics and animation plus stable `EntityPath` / `ComponentPropertyPath` support and generic property serialization helpers.
- `zircon_asset` / `zircon_resource`: new resource kinds and imported asset variants for `PhysicsMaterial`, `AnimationSkeleton`, `AnimationClip`, `AnimationSequence`, `AnimationGraph`, and `AnimationStateMachine`.
- `zircon_runtime`: `extensions::physics` and `extensions::animation` become folder-backed module owners with runtime-facing facades, settings registration, and schedule wiring.
- `zircon_physics_jolt`: new workspace member implementing the backend-neutral physics trait surface on top of Jolt.

## Test Plan
- Contract And Wiring: module descriptors, manager resolvers, and builtin runtime registration resolve physics/animation through the same framework/manager path as existing subsystems.
- Shared Property Paths: parse, serialize, resolve, and cache entity/component/property paths across scene save/load; inspector edits and sequence edits must hit the same canonical subject/property pair.
- Physics Behavior: rigid body/collider/joint/material serialization roundtrip, layer/filter matrix behavior, sensor behavior, fixed-step tick/update correctness, transform sync, and Jolt smoke simulation.
- Physics Queries And Events: ray cast, overlap/contact notifications, and backend-unavailable failure paths respect masks/groups and restart semantics.
- Animation Assets And Math: binary roundtrip for skeleton/clip/sequence/graph/state-machine assets; Hermite sampling and Householder-based tangent/key-reduction math remain deterministic and numerically stable.
- Animation Runtime: property-sequence playback, skeletal clip playback, graph blending, state transitions, and pose/skinning extraction all produce deterministic results.
- Editor Tooling: sequence editor tree/dopesheet/graph interactions, inspector-to-track creation flow, graph/state editing, binding payload roundtrip, and workbench activity integration through the Slint host.

## Assumptions And Defaults
- `physics_interface` remains runtime-owned as requested, but stable cross-crate contracts still live in `zircon_framework` so the subsystem matches existing resolver patterns.
- Property-path foundations are declared generically now, but first implementation coverage is scene ECS plus the new physics/animation components; later runtime modules adopt the same registry interfaces instead of adding ad hoc path parsing.
- Physics backend switching is supported at project/module restart boundaries, not as live hot swap inside an active step.
- Sequence v1 is intentionally simpler than full Unreal MovieScene: no subsequences, no director/event tracks, and no render-capture pipeline in the first implementation.
- “Full animation stack” means sequence/property animation, skeletal clips, graphs, and state machines are all in scope for the program, but implementation order stays bottom-up: shared path/asset foundations first, then sequence/property + physics, then skeletal runtime, then graph/state-machine authoring.
