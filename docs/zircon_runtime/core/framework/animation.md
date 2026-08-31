---
related_code:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/compiler/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/diagnostic.rs
  - zircon_runtime/src/core/framework/animation/compiler/graph.rs
  - zircon_runtime/src/core/framework/animation/compiler/parameter.rs
  - zircon_runtime/src/core/framework/animation/compiler/product.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/model.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_set.rs
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
  - zircon_runtime/src/asset/assets/scene/animation.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_runtime/src/animation/sequence.rs
implementation_files:
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/error.rs
  - zircon_runtime/src/core/framework/animation/compiler/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/diagnostic.rs
  - zircon_runtime/src/core/framework/animation/compiler/graph.rs
  - zircon_runtime/src/core/framework/animation/compiler/parameter.rs
  - zircon_runtime/src/core/framework/animation/compiler/product.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/model.rs
  - zircon_runtime/src/core/framework/animation/avatar_mask.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/core/framework/animation/gpu_skinning.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_set.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/state_machine_evaluation.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/animation/tests.rs
  - zircon_runtime/src/asset/assets/scene/animation.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_runtime/src/core/framework/animation/tests.rs
  - zircon_runtime/src/core/framework/animation/compiler/tests.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/tests.rs
  - zircon_runtime/src/core/framework/animation/compiler/state_machine/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/animation_resource.rs::review_f5_animation_manager_uses_animation_error
  - avatar_mask_filters_exact_leaf_and_excluded_targets
  - animation_tick_contract_records_work_events_and_sanitized_delta
  - gpu_skinning_readiness_requires_enabled_gpu_resources
  - timeline_track_masks_and_clip_status_sanitize_contract_values
  - runtime_status_reports_player_rig_and_gpu_readiness
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/animation/*.rs (planned for current slice)
  - git diff --check -- zircon_runtime/src/core/framework/animation docs/zircon_runtime/core/framework/animation.md docs/zircon_plugins/animation/runtime.md (planned for current slice)
  - cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never (pending while active Cargo lanes are busy)
doc_type: module-detail
---

# Animation Framework Contracts

## Purpose

`zircon_runtime::core::framework::animation` is the neutral animation contract layer. It defines the versioned skeleton/clip/sequence/graph/state-machine resource schemas, timeline descriptors, playback settings, evaluation DTOs, pose output records, avatar masks, GPU-skinning readiness, tick requests/reports, runtime status snapshots, typed animation errors, and the `AnimationManager` trait. It does not own project loading, concrete clip sampling, graph blending, IK node evaluation, scene mutation, authored timeline UI, GPU buffer allocation, or event dispatch.

Concrete runtime behavior remains in `zircon_plugins/animation/runtime`. The framework gives runtime, editor, scripting, and future VM plugin callers a shared vocabulary for animation service access without importing the plugin crate or sharing plugin-owned objects.

## Related Files

The framework is folder-backed and `mod.rs` is only the public re-export surface.

- `error.rs` defines `AnimationError` and `AnimationResult` for framework-facing manager, pose sampling, and sequence writeback failures.
- `asset/` is the unique versioned animation resource-schema owner, including `ZRANIM01` binary conversion and typed `AnimationAssetError`. All binary animation inputs are admitted through one 64 MiB byte budget and matching bounded `bincode` decode before document/stream or version-migration fallback; oversized input returns `AnimationAssetError::InputTooLarge`. Asset import/cache code consumes these records without re-exporting them from the asset facade.
- `compiler/` is the single pure semantic-compilation boundary. Its `compile_animation_source` dispatches Sequence, Graph, and State Machine inputs through one typed product surface, and its builtin schema registry defines the `zircon.runtime.animation` owner/version plus typed Graph pin/cardinality, state-kind, parameter, and asset-kind facts. It currently validates Sequence timing/channel semantics, Graph topology/parameters, State Machine transitions/layers/Blend Space admission, then lowers local string links into immutable index-based IR with stable source diagnostics. It does not load external resources, own editor sessions, or mutate runtime state.
- `manager.rs` defines `AnimationManager` and its neutral status, control, and evaluation surface. It does not project concrete asset models into timeline descriptors, accept `scene::World` for mutation, or expose a process-wide IK inbox.
- `parameter_map.rs` defines the deterministic parameter value-map shape. `parameter_set.rs` is the unique scene-schema, ECS-player, and runtime-request owner: it wraps the map in copy-on-write storage and owns opaque process-local content revision/fingerprint values. Neither runtime identity is serialized or interpreted as an asset generation.
- `tick.rs`, `event.rs`, and `runtime_status.rs` define world tick inputs, emitted clip-event records, per-player status, per-rig status, and aggregate runtime status.
- `timeline.rs`, `track_path.rs`, and `sequence_apply_report.rs` describe property tracks, bone tracks, event tracks, timeline clip spans, sequence writeback paths, and missing-track reporting.
- `avatar_mask.rs`, `graph_clip_instance.rs`, `graph_evaluation.rs`, and `state_machine_evaluation.rs` describe masked/additive graph output and state-machine transition reports.
- `pose_bone.rs`, `pose_output.rs`, `pose_source.rs`, `pose_snapshot.rs`, and `gpu_skinning.rs` describe sampled pose output, sealed publication ownership, and GPU skinning readiness without owning renderer resources.

`AnimationPoseOutput` remains owned and mutable while a plugin samples, blends, and applies pose modifiers. The final publication boundary seals each entity row once as `AnimationPoseHandle = Arc<AnimationPoseOutput>` and publishes an `AnimationPoseSnapshot = Arc<BTreeMap<EntityId, AnimationPoseHandle>>`. Frame, render-extract, physics-publication, and history consumers clone handles rather than bone/name payloads. `AnimationPoseOutput::clone_from_reusing_storage` remains available only for owners that intentionally maintain mutable owned scratch storage; it is not the production frame-publication contract.

## Behavior Model

Animation is contract-first. Runtime scene components and animation assets may refer to clips, sequences, graphs, state machines, skeletons, and stable target ids, but the framework only describes what a manager can report or request. The plugin decides how to sample clips, blend graph layers, apply sequence property tracks, dispatch events, and record poses on `LevelSystem`.

Timeline descriptors summarize authored or imported animation data:

- sequence descriptors expose property tracks as `AnimationTrackPath` records plus optional stable target ids;
- clip descriptors expose bone-transform tracks and event tracks;
- track descriptors can carry an `AnimationAvatarMask`, allowing editor/runtime tools to reason about masked targets before a concrete graph evaluator runs;
- clip spans sanitize start, duration, playback speed, and weight so callers can display or validate bad data without panicking.

The framework owns only these descriptor DTOs and their validation helpers. The retired `from_sequence`, `from_clip`, `sequence_timeline_descriptor`, `clip_timeline_descriptor`, and `sequence_track_paths` convenience APIs directly imported concrete asset models and had no production callers; the hard cut deletes them instead of preserving asset projections or compatibility wrappers. A concrete animation/editor adapter that actually needs a descriptor must assemble it at the implementation boundary.

Scene graph/state-machine assets and their ECS player components store `AnimationParameterSet`
directly. Custom serde exposes only the deterministic `AnimationParameterMap` value shape and
reconstructs a fresh process-local revision during load. Cloning a set shares its immutable map;
insert/remove/clear perform copy-on-write and advance `AnimationParameterRevision` only for actual
content changes. Consumers may compare or cache that revision during the process lifetime, but may
not persist it, derive asset currentness from it, or replace resource/ECS generations with it.
Mutable access to the underlying map is intentionally not exposed because it would bypass revision
advancement. Construction and successful mutation also refresh
`AnimationParameterContentFingerprint`; equal values, including signed-zero variants, have the same
fingerprint. Equality rejects unequal fingerprints early and still compares full values after a
fingerprint match, so a collision cannot change parameter semantics.

Graph and state-machine scan requests clone the component-owned set directly. The runtime does not
retain a second per-entity parameter snapshot or expose the retired map-synchronization API. Stable
request admission is therefore one shared-owner clone, O(1) in parameter count; actual content
mutation remains O(P) for P parameters because COW and fingerprint refresh must visit owned values.
There is no raw-map component/schema field, compatibility field, alias, or synchronization bridge.

Within one evaluation frame, graph results are indexed by graph id, skeleton id, and the parameter
content fingerprint in a bounded `BTreeMap`. A candidate hit is accepted only after complete
parameter equality. The cache stops admitting new distinct keys after 256 entries and is cleared at
the next frame boundary; it does not evict early entries or linearly scan unrelated instance maps.
This is a same-content evaluation deduplication index, not a persisted asset key and not a substitute
for graph/skeleton resource revisions.

Runtime graph lowering consumes the shared compiler artifact directly and retains its
dependency-first evaluation order. Evaluation seeds the output and walks that order in reverse, so
every compiled node is dispatched once after all consumers have contributed. Repeated DAG paths
merge their weights at the child node rather than recursively expanding one invocation per path.
The common single-context path uses an inline accumulator; only nodes reached through distinct
mask/additive contexts allocate an ordered context map. The nearest mask to a clip overrides outer
masks, additive mode is monotonic, and a clip emits once for each distinct mask/additive context.

Compiled graph clip output has a deterministic semantic order: Base contributions precede Additive
contributions, then clip source slot and mask source slot decide order. Authoring edge traversal order
is not a public ordering contract. This is important because event collection and additive rotation
application both consume this sequence. The current evaluator still materializes weighted clip
contributions before pose sampling; it is not yet the final reusable node-local pose program or the
graph-local IK execution model.

## Semantic Compiler

`compile_animation_source`, `compile_animation_sequence`, `compile_animation_graph`, and `compile_animation_state_machine` are deterministic and side-effect free. Sequence compilation validates duration/FPS, optional target identity, duplicate property writers, canonical key time ordering/range, stable value domains, interpolation/tangent compatibility, finite channel values, and normalizable quaternions. Graph compilation validates node and parameter identities, output cardinality, node references, scalar weight-parameter contracts, finite numeric authoring values, and cyclic dependencies. State Machine compilation validates state identities and entry ownership, transition endpoints/times, condition/operator/value type contracts, source-order-preserved multiple transitions, layer uniqueness/weights, and the current 1D/2D Blend Space point admission rules. Valid artifacts store source-order-stable dense slots for internal states, nodes, parameters, and sequence tracks; graph artifacts additionally expose a dependency-first evaluation order for the output-reachable graph. Unreachable but valid graph nodes remain in the artifact and emit warnings, so authoring tools can report them without silently deleting source.

Topology construction uses an iterative Kahn pass plus an explicit work stack for output reachability; compile depth therefore does not consume the process call stack. The compiler is deliberately stricter than the pre-existing evaluator while migration is in progress: it validates every authored node, including unreachable cycles. Callers must not treat a successful compile as proof that an external clip resource has loaded, nor treat a failed compile as permission to replace a last-known-good preview/runtime artifact.

This is the first shared Sequence, Graph, and State Machine compiler contract, not the final integration state. The binary envelope now bounds raw byte admission and every fallback decode attempt, but external dependency resolution, plugin schema registration, generation/currentness ownership, and artifact installation remain to be converged. The runtime graph evaluator consumes the shared Graph artifact; Editor preview and other compiler consumers still require their own audited product integration. No compatibility facade is provided for the old Editor-only compile summary.

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

The Runtime animation absorption layer registers the default manager and manager handle. `zircon_plugins/animation/runtime` registers `animation.evaluate`, resolves that neutral `AnimationManager`, advances scene player components, loads assets through `ProjectAssetManager`, samples/blends poses, emits events, and records pose/playback state on `LevelSystem`.

Framework callers can request:

1. default playback settings;
2. normalized track paths;
3. graph or state-machine evaluations;
4. pose sampling for a clip;
5. sequence evaluation records; concrete sequence writeback is invoked by the Plugin evaluation pipeline through the Runtime sequence owner, not through the neutral manager;
6. runtime status through the data-only manager methods; timeline descriptors are assembled by concrete implementation adapters rather than by the neutral manager.

The framework defaults return inert status and descriptor records so optional managers can remain small while still satisfying the trait.

Runtime 15 F5 animation manager typed errors records the typed boundary as `runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred`. `AnimationManager::sample_clip_pose` returns `AnimationResult`; the upper animation sequence apply function also returns `AnimationResult` while owning concrete `scene::World` mutation. Concrete animation owners map skeleton bind, clip sample, quaternion normalization, and sequence channel conversion failures into `AnimationError` variants without putting `World` back into the framework trait.

## Edge Cases

Framework DTOs sanitize non-finite and negative times, playback speeds, and weights. Target matching accepts exact ids and slash-path leaf matches, matching current clip target-id and sequence target-id behavior. Empty or muted target descriptors do not match runtime targets. Missing skeletons, missing targets, GPU resource gaps, invalid players, and waiting-for-asset states are represented as status data rather than panics.

Product IK belongs to compiled animation graph/evaluation nodes, with graph-instance-local state and skeleton-scoped dense target slots. The framework does not expose the retired one-shot world command queue. `AnimationTargetId` remains the stable asset/authoring identifier used by animation tracks and masks; dense slots never cross the plugin boundary.

`AnimationPlayerRuntimeStatus` also sanitizes its JSON boundary. `time_seconds` and `playback_speed` serialize and deserialize as finite non-negative values, and `weight` serializes and deserializes as a finite `0.0..=1.0` value. `AnimationRuntimeStatus::sanitized_snapshot()` exposes the same comparison shape used by the serde round-trip guard, so diagnostics and editor panels do not receive JSON `null` values from `NaN` or infinite runtime floats.

Concrete managers must still validate asset availability, clip duration, graph cycles, state-machine transition validity, malformed quaternion channels, skeleton/track mismatch, GPU resource allocation, and event dispatch ordering. The framework-level error surface intentionally covers manager/apply failures that callers can act on without parsing strings: non-finite skeleton bind fields, zero-length bind rotations, sample type mismatches, non-finite samples, zero-length quaternion samples, non-finite sequence channel samples, and zero-length sequence channel quaternions.

The 2026-06-04 Plugin runtime follow-up temporarily split sequence writeback under `zircon_plugins/animation/runtime/src/sequence/`. The 2026-08-01 hard cut deleted that duplicate facade and its children: `zircon_runtime/src/animation/sequence.rs` and `sequence/{apply,channel_sample,compiled,conversion,interpolation,target,tests,time}.rs` are now the sole sequence implementation owners, re-exported by the Plugin crate root and called from `evaluation/pipeline/sequences.rs`. The earlier Frameworks05 cut remains intact: `AnimationManager` does not accept `scene::World`, and no retired scene-hook or Plugin-local sequence compatibility path survives.

## Test Coverage

Framework tests lock avatar mask target filtering, tick/event report behavior, GPU-skinning readiness, track mask matching, clip status sanitization, runtime player/rig aggregation, and serde round-trips for runtime status records. Compiler tests lock unified source dispatch, Sequence timing/type/interpolation/quaternion guards, successful dense index lowering, duplicate/missing/cyclic graph rejection, unreachable-node warnings, deep non-recursive topology, state/transition/condition/layer validation, typed parameter-slot resolution, and Blend Space admission parity. `test_animation_timeline_contract_does_not_project_asset_models` keeps asset-to-timeline conversion out of framework, while `test_animation_manager_contract_does_not_mutate_scene_world` keeps sequence writeback in the concrete runtime/scene owner. Animation plugin contracts retain the pure TwoBone and LookAt solver boundaries. `review_f5_animation_manager_uses_animation_error` still protects `AnimationError`/`AnimationResult` across manager sampling and upper sequence conversion owners.

`AnimationParameterSet` unit coverage locks shared clone semantics, copy-on-write mutation,
no-op revision stability, iterator construction, fingerprint collision safety, and serde revision
reconstruction. The Frameworks01 source boundary locks the scene-schema/ECS owner hard cut and
rejects retained projection snapshots or player-map synchronization.

Focused Cargo validation for the current framework-contract update is pending while active Cargo lanes are running. The intended focused check is:

```powershell
cargo test -p zircon_runtime --lib animation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-framework-contract --message-format short --color never -- --nocapture --test-threads=1
```
