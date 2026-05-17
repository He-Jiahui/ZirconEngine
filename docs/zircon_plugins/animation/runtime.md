---
related_code:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/clip_event.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
implementation_files:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/clip_event.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/scene/level_system.rs
plan_sources:
  - user: 2026-05-03 继续补独立插件缺口
  - user: 2026-05-08 继续周边设施与插件能力完善计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - docs/superpowers/plans/2026-05-03-physics-animation-aggressive-plugin-migration.md
tests:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - runtime_physics_animation_tick_contract::level_tick_emits_animation_clip_event_tracks_crossed_by_player_time
  - runtime_physics_animation_tick_contract::clip_event_sampling_reports_loop_boundary_occurrences_in_playback_order
  - runtime_physics_animation_tick_contract::graph_player_emits_clip_events_using_graph_clip_playback_speed
  - runtime_physics_animation_tick_contract::state_machine_player_emits_active_graph_clip_events
  - runtime_physics_animation_tick_contract::state_machine_transition_emits_from_and_to_graph_clip_events
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked --quiet (blocked: unrelated active scene world/ECS compile errors)
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
doc_type: module-detail
---

# Animation Runtime Plugin

`zircon_plugins/animation/runtime` owns the concrete animation runtime after the hard cutover. The crate provides the `AnimationModule` descriptor, the plugin-local `AnimationDriver`, the `DefaultAnimationManager` evaluator/sampler, sequence property writeback, and the scene hook that runs animation at `SystemStage::PostUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::animation` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::animation`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and generic scene hook scheduling.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The plugin contributes tick behavior through `RuntimeExtensionRegistry::register_scene_hook(scene_hook_registration())`.
- `AnimationSceneRuntimeHook` resolves `AnimationManagerHandle`, advances scene player clocks, loads animation assets through `ProjectAssetManager`, blends graph/state-machine pose output, and records pose/playback runtime state on `LevelSystem`.
- `AnimationSceneRuntimeHook` publishes `AnimationClipEvent` values when direct clip players, graph players, state-machine active graphs, or state-machine transition graphs advance across `AnimationClipAsset.event_tracks`, matching Bevy's clip-event precedent for timeline-authored gameplay hooks.
- `DefaultAnimationManager` owns playback settings persistence, graph evaluation, state-machine evaluation, clip pose sampling, and sequence-to-world application.
- `DefaultAnimationManager::evaluate_graph(...)` preserves additive clip roles and mask target ids in neutral framework DTOs, while `AnimationSceneRuntimeHook` consumes those roles during graph pose blending.
- `DefaultAnimationManager::sample_clip_pose(...)` resolves `AnimationClipBoneTrackAsset.target_id` before the legacy `bone_name` fallback. Target ids can match a bone name or the slash-joined skeleton path, for example `Root/Hand`.
- `apply_sequence_to_world(...)` resolves `AnimationSequenceBindingAsset.target_id` before the legacy `entity_path` fallback. Current runtime target ids accept a stable numeric `EntityId` string or the same canonical `EntityPath` text used by old bindings.
- `zircon_runtime::scene::WorldDriver` dispatches installed hooks by schedule stage and contains no animation-specific logic.

## Framework Contract

Runtime framework contracts are intentionally concrete-free:

- `AnimationManager::apply_sequence_to_world(...)` defines the manager-side sequence writeback capability.
- `AnimationClipEvent` is the plugin-owned typed scene event for clip event tracks. It records the source entity, optional target id, event name, payload, clip time, and absolute playback time so looping clips can report boundary occurrences deterministically.
- `AnimationGraphBlendMode`, `AnimationGraphClipInstance::target_ids`, and `AnimationGraphEvaluation::mask_target_ids` describe additive/masked graph output without moving concrete graph runtime back into `zircon_runtime`.
- `AnimationClipBoneTrackAsset.target_id`, `AnimationSequenceBindingAsset.target_id`, and `AnimationClipAsset.event_tracks` add stable target/event metadata to the asset contract while keeping old `bone_name` and `entity_path` fallbacks available.
- `AnimationSequenceApplyReport` reports applied and missing tracks without exposing plugin-owned sequence implementation details.
- `ANIMATION_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The plugin can evolve graph blending, state-machine semantics, and importer-driven animation assets without reintroducing `zircon_runtime::animation`.

## Graph Pose Semantics

- Base graph clips are normalized against the total positive base weight before pose blending.
- Additive graph clips are applied after the base pose. Translation is added directly, scale is applied as a delta from `Vec3::ONE`, and rotation is applied as a weighted identity-to-additive rotation delta.
- Mask target ids limit base or additive writes to matching pose bones. Empty target ids mean the clip affects the whole pose; non-empty ids currently match either the bone name or the leaf of a slash path such as `Root/Hand`.
- State-machine transition blending continues to use the same weighted base-pose helper, so state transitions keep their existing cross-fade semantics while graph evaluation can add masked additive layers inside each sampled state graph.

## Binary Compatibility

- New `.zranim` bytes still write the wrapped `AnimationBinaryDocument` shape.
- Decode now also accepts the older stream shape that serialized `AnimationBinaryHeader` followed by the payload. This keeps already-authored version-1 `.zranim` clip, sequence, and graph assets readable without bumping `ANIMATION_BINARY_VERSION`.
- Legacy clip payloads decode with `target_id = None` and empty `event_tracks`; legacy sequence bindings decode with `target_id = None`; legacy graph nodes decode only the original clip/blend/output tags.

## Clip Event Semantics

- Direct clip players sample event tracks over the half-open playback range `(previous_time_seconds, current_time_seconds]`.
- Graph players sample the same range from the graph playback clock, then convert each graph clip instance into clip-local time through its playback speed before event sampling.
- State-machine players sample the active state's graph over the state-machine playback clock when no transition is currently blending.
- State-machine transitions sample both the source state's graph and the target state's graph over their own transition-local time ranges. When the transition completes, the runtime stops saving transition state, so the following frame samples only the target active graph and does not repeat the completed transition range.
- Non-looping clips clamp event sampling to the clip duration and never repeat events after the end.
- Looping clips emit each crossed occurrence in playback order, including an event at clip time `0.0` when playback crosses a loop boundary.
- Events are sent to the world's typed event store and become readable after the normal `World::update_events::<AnimationClipEvent>()` step.

## Validation Evidence

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a` passed with 7 plugin contract tests.
- `cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a` passed for the independent plugin workspace with animation included but still outside the root workspace.
- `cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a` passed with 767 runtime lib tests, validating scene hook dispatch, manager contracts, and hard-cutover structural assertions without depending on the plugin crate.
- Current additive/mask/event metadata seam: `cargo check --manifest-path "zircon_plugins/Cargo.toml" -p zircon_plugin_animation_runtime --tests --locked --quiet` is blocked before animation test execution by unrelated active scene world/ECS errors: `rebuild_fixed_component_presence_for_entity` visibility and missing `flush_pending_scene_systems_if_ready` call sites. The written contract tests cover additive mask pose application, clip target-id resolution, sequence target-id resolution, and legacy stream `.zranim` decode.
- The 2026-05-16 direct clip-event slice passed `rustfmt --edition 2021 --check` and `git diff --check` for the touched animation runtime files.
- `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir target\codex-animation-event --message-format short --color never` passed with `zircon_plugins/Cargo.lock` protected and restored after the run.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir target\codex-animation-event --test runtime_physics_animation_tick_contract clip_event -- --nocapture` passed the two new clip-event tests with `zircon_plugins/Cargo.lock` protected and restored after the run.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir target\codex-animation-event --test runtime_physics_animation_tick_contract graph_player_emits_clip_events -- --nocapture` passed the graph player clip-event test with `zircon_plugins/Cargo.lock` protected and restored after the run.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir target\codex-animation-event --test runtime_physics_animation_tick_contract state_machine_player_emits_active_graph_clip_events -- --nocapture` passed the state-machine active graph clip-event test with `zircon_plugins/Cargo.lock` protected and restored after the run.
- `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir target\codex-animation-event --test runtime_physics_animation_tick_contract state_machine_transition_emits_from_and_to_graph_clip_events -- --nocapture` passed the state-machine transition clip-event test with `zircon_plugins/Cargo.lock` protected and restored after the run.
- A repeat aggregate `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_animation_runtime --offline --jobs 1 --target-dir target\codex-animation-event --test runtime_physics_animation_tick_contract event -- --nocapture` attempt was stopped after it blocked on the shared Cargo package cache and left orphaned local cargo/rustc processes; the earlier protected targeted tests for direct, graph, and state-machine event paths had already passed.
- Direct `--locked` validation remains blocked by the pre-existing `zircon_plugins/Cargo.lock` delta; the current lockfile diff is still limited to the existing `zircon_runtime_reflection_macros` entry.
