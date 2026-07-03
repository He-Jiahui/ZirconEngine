---
related_code:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/manager/graph.rs
  - zircon_plugins/animation/runtime/src/manager/parameters.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_plugins/animation/runtime/src/manager/sampling.rs
  - zircon_plugins/animation/runtime/src/manager/state_machine.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/sequence/apply.rs
  - zircon_plugins/animation/runtime/src/sequence/channel_sample.rs
  - zircon_plugins/animation/runtime/src/sequence/conversion.rs
  - zircon_plugins/animation/runtime/src/sequence/interpolation.rs
  - zircon_plugins/animation/runtime/src/sequence/target.rs
  - zircon_plugins/animation/runtime/src/sequence/tests.rs
  - zircon_plugins/animation/runtime/src/sequence/time.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/scene_hook/events.rs
  - zircon_plugins/animation/runtime/src/scene_hook/graph.rs
  - zircon_plugins/animation/runtime/src/scene_hook/pending.rs
  - zircon_plugins/animation/runtime/src/scene_hook/pose.rs
  - zircon_plugins/animation/runtime/src/scene_hook/scan.rs
  - zircon_plugins/animation/runtime/src/scene_hook/sequences.rs
  - zircon_plugins/animation/runtime/src/scene_hook/state_machine.rs
  - zircon_plugins/animation/runtime/src/scene_hook/tick.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/plugin/scene_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
implementation_files:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/manager/graph.rs
  - zircon_plugins/animation/runtime/src/manager/parameters.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_plugins/animation/runtime/src/manager/sampling.rs
  - zircon_plugins/animation/runtime/src/manager/state_machine.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/sequence/apply.rs
  - zircon_plugins/animation/runtime/src/sequence/channel_sample.rs
  - zircon_plugins/animation/runtime/src/sequence/conversion.rs
  - zircon_plugins/animation/runtime/src/sequence/interpolation.rs
  - zircon_plugins/animation/runtime/src/sequence/target.rs
  - zircon_plugins/animation/runtime/src/sequence/tests.rs
  - zircon_plugins/animation/runtime/src/sequence/time.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/scene_hook/events.rs
  - zircon_plugins/animation/runtime/src/scene_hook/graph.rs
  - zircon_plugins/animation/runtime/src/scene_hook/pending.rs
  - zircon_plugins/animation/runtime/src/scene_hook/pose.rs
  - zircon_plugins/animation/runtime/src/scene_hook/scan.rs
  - zircon_plugins/animation/runtime/src/scene_hook/sequences.rs
  - zircon_plugins/animation/runtime/src/scene_hook/state_machine.rs
  - zircon_plugins/animation/runtime/src/scene_hook/tick.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/core/framework/animation/graph_blend_mode.rs
  - zircon_runtime/src/core/framework/animation/graph_clip_instance.rs
  - zircon_runtime/src/core/framework/animation/graph_evaluation.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_runtime/src/core/framework/animation/sequence_apply_report.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
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
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/animation_assets.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - runtime_physics_animation_tick_contract::level_tick_emits_animation_clip_event_tracks_crossed_by_player_time
  - runtime_physics_animation_tick_contract::clip_event_sampling_reports_loop_boundary_occurrences_in_playback_order
  - runtime_physics_animation_tick_contract::graph_player_emits_clip_events_using_graph_clip_playback_speed
  - runtime_physics_animation_tick_contract::state_machine_player_emits_active_graph_clip_events
  - runtime_physics_animation_tick_contract::state_machine_transition_emits_from_and_to_graph_clip_events
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - animation_registration_contributes_runtime_module
  - animation_plugin_toml_matches_catalog_beta_partial_metadata
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --test runtime_physics_animation_tick_contract --target-dir target\codex-shared-a
  - cargo test --manifest-path zircon_plugins\animation\runtime\Cargo.toml animation_registration_contributes_runtime_module --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-animation-runtime-metadata --color never --quiet
  - cargo test --manifest-path Cargo.toml -p zircon_runtime --lib animation_plugin_toml_matches_catalog_beta_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-animation-runtime-metadata --color never --quiet
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked --quiet (blocked: unrelated active scene world/ECS compile errors)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never (2026-06-12 plugin-architecture runtime-system migration: passed with existing warnings; zircon_plugins/Cargo.lock protected/restored)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --lib --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never -- --nocapture (2026-06-12 attempted; timed out after 10 minutes during compile/link; no plugin test pass claimed)
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
doc_type: module-detail
---

# Animation Runtime Plugin

`zircon_plugins/animation/runtime` owns the concrete animation runtime after the hard cutover. The crate provides the `AnimationModule` descriptor, the plugin-local `AnimationDriver`, the `DefaultAnimationManager` evaluator/sampler, sequence property writeback, and the runtime scene system that runs animation at `SystemStage::PostUpdate`.

`zircon_runtime` no longer exports `zircon_runtime::animation` and does not depend on the plugin crate. Runtime keeps only neutral contracts under `zircon_runtime::core::framework::animation`, manager service names/resolvers under `zircon_runtime::core::manager`, scene ECS state, and generic runtime scene-system scheduling.

## Runtime Boundary

- The plugin contributes the lifecycle module through `RuntimeExtensionRegistry::register_module(module_descriptor())`.
- The plugin contributes tick behavior through `RuntimePluginModuleRegistration::runtime_scene_system(...)` as `animation.evaluate` in `SystemStage::PostUpdate`, in set `animation.evaluation`, after `zircon.scene.world_transform`.
- `runtime_plugin_descriptor()` is the linked package-manifest source for the Animation runtime crate. It mirrors the static `zircon_plugins/animation/plugin.toml` and built-in catalog metadata: category `runtime`, maturity `beta`, `runtime.plugin.animation` status `partial` with Bevy `bevy_animation` source traceability, and `runtime.feature.animation.timeline_event_track` status `partial`.
- D5 editor authoring macro consumer guard keeps the editor package on the SDK macro path: `zircon_plugins/animation/editor/src/plugin.rs` uses `zircon_plugin_sdk::authoring_plugin!` with `mirrors_runtime_manifest: zircon_plugin_animation_runtime::package_manifest()` and only keeps the Animation-specific extension registration body outside the macro. Status `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred` is locked by `review_d5_editor_authoring_plugins_use_sdk_macro`.
- D9 editor/runtime mirror consumer guard keeps the editor package tied to this runtime package manifest through the SDK declaration projection: editor tests assert `mirrored_runtime_package_id()`, and the package manifest carries both `zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY` and the Animation authoring capability. `tools/audit_plugin_structure.py --json` reports `editor_runtime_mirror_violations = 0` and `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`; status `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred` is locked by `review_d9_editor_runtime_mirror_consumers_use_sdk_declaration`.
- D10 animation/physics bridge call migration keeps cross-plugin physics queries on the Plugins 11 bridge path. The runtime contract test resolves `physics.query.v1` from `runtime.extension_report().registry.frozen_bridge_table()` as `WeakBridge<dyn PhysicsQueryInterface>` and calls ray, overlap, and shape-cast through that weak bridge instead of concrete physics manager lookup. Guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`.
- `AnimationRuntimeSystem` resolves `AnimationManagerHandle`, advances scene player clocks, loads animation assets through `ProjectAssetManager`, blends graph/state-machine pose output, and records pose/playback runtime state on `LevelSystem`.
- `AnimationRuntimeSystem` publishes `AnimationClipEvent` values when direct clip players, graph players, state-machine active graphs, or state-machine transition graphs advance across `AnimationClipAsset.event_tracks`, matching Bevy's clip-event precedent for timeline-authored gameplay hooks.
- `runtime_system.rs` is the scheduling entry. The existing `scene_hook/` child directory is now an internal tick implementation subtree loaded by path attributes: `tick.rs` owns tick orchestration, `scan.rs` walks scene animation players into pending sample requests, `pending.rs` carries those request DTOs, `sequences.rs` applies property-track sequences, `pose.rs` samples direct clip poses, `graph.rs` owns graph clip-event sampling and additive/masked graph blending, `state_machine.rs` owns state-machine transition pose/event resolution, and `events.rs` publishes typed clip events into the scene world.
- `DefaultAnimationManager` owns playback settings persistence, graph evaluation, state-machine evaluation, clip pose sampling, and sequence-to-world application.
- `manager.rs` is now the structural `DefaultAnimationManager` facade. `manager/parameters.rs` owns parameter default/value mutation and numeric scalar helpers, `graph.rs` owns graph clip collection plus additive/masked graph evaluation, `state_machine.rs` owns transition condition evaluation and active-state resolution, `pose.rs` owns skeleton bind validation plus clip bone-track sampling, and `sampling.rs` owns finite-value, sample-time, and channel-sample conversion helpers.
- `sequence.rs` is now a structural sequence facade. `sequence/apply.rs` owns sequence binding iteration and scene property writeback, `target.rs` resolves stable target ids and legacy entity paths, `time.rs` owns loop/clamp sample-time normalization, `channel_sample.rs` owns channel key selection, `interpolation.rs` owns Hermite and quaternion interpolation, `conversion.rs` owns channel-to-scene-property validation/conversion, and `tests.rs` keeps private sequence coverage out of the facade.
- `DefaultAnimationManager::evaluate_graph(...)` preserves additive clip roles and mask target ids in neutral framework DTOs, while `AnimationRuntimeSystem` consumes those roles during graph pose blending.
- `DefaultAnimationManager::sample_clip_pose(...)` resolves `AnimationClipBoneTrackAsset.target_id` before the legacy `bone_name` fallback. Target ids can match a bone name or the slash-joined skeleton path, for example `Root/Hand`.
- `apply_sequence_to_world(...)` resolves `AnimationSequenceBindingAsset.target_id` before the legacy `entity_path` fallback. Current runtime target ids accept a stable numeric `EntityId` string or the same canonical `EntityPath` text used by old bindings.
- `zircon_runtime::scene::WorldDriver` dispatches installed runtime scene systems by schedule stage and contains no animation-specific logic.

## Framework Contract

Runtime framework contracts are intentionally concrete-free:

- `AnimationManager::apply_sequence_to_world(...)` defines the manager-side sequence writeback capability.
- `AnimationClipEvent` is the plugin-owned typed scene event for clip event tracks. It records the source entity, optional target id, event name, payload, clip time, and absolute playback time so looping clips can report boundary occurrences deterministically.
- `AnimationGraphBlendMode`, `AnimationGraphClipInstance::target_ids`, and `AnimationGraphEvaluation::mask_target_ids` describe additive/masked graph output without moving concrete graph runtime back into `zircon_runtime`.
- `AnimationClipBoneTrackAsset.target_id`, `AnimationSequenceBindingAsset.target_id`, and `AnimationClipAsset.event_tracks` add stable target/event metadata to the asset contract while keeping old `bone_name` and `entity_path` fallbacks available.
- `AnimationSequenceApplyReport` reports applied and missing tracks without exposing plugin-owned sequence implementation details.
- `AnimationTimelineDescriptor`, `AnimationTimelineTrackDescriptor`, and `AnimationTimelineClipDescriptor` summarize sequence property tracks, clip bone tracks, event tracks, mask filtering, and clip spans for editor/runtime/VM callers without exposing plugin-owned sampler state.
- `AnimationPlayerRuntimeStatus`, `AnimationRigRuntimeStatus`, and `AnimationRuntimeStatus` expose player state, rig pose coverage, missing targets, GPU-skinning readiness, last tick work, and diagnostics as copied data.
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

- Current D10 animation/physics bridge call migration: the contract test now uses `WeakBridge<dyn PhysicsQueryInterface>` / `physics.query.v1` for physics ray, shape-overlap, and shape-cast calls after ticking the level. Static guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`; Cargo remains deferred for this implementation slice.
- The 2026-06-04 scene hook boundary split reduced `zircon_plugins/animation/runtime/src/scene_hook.rs` from a mixed 867-line file to a structural 32-line entry plus `scene_hook/{tick,scan,pending,events,sequences,pose,graph,state_machine}.rs`. `rustfmt --edition 2021 --check --config skip_children=true` passed over the split files. A focused `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-scene-hook-split-0604 --message-format short --color never` attempt timed out after two minutes while other workspace/Hub/editor Cargo lanes were active and did not return Rust diagnostics; compile acceptance for this structural split remains pending.
- The 2026-06-12 plugin-architecture slice replaces the old root `scene_hook.rs` scheduling entry with `runtime_system.rs`, keeps `scene_hook/` as internal tick child modules, declares `system_sets = ["animation.evaluation"]` and `system_anchors = ["animation.evaluate"]` in `plugin.toml`, and installs physics/animation world runtime extensions through `CoreRuntime::install_world_runtime_extensions(...)`. The obsolete plugin-local `clip_event.rs` duplicate is removed; `scene_hook/events.rs` and `graph.rs` now sample and publish the shared `zircon_runtime::animation::AnimationClipEvent` type. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never` passes with existing warnings. `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks --message-format short --color never` also passes with existing warnings; a follow-up `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --lib --offline ...` timed out after 10 minutes during compile/link, so no fresh animation plugin test pass is claimed for this slice yet.
- The 2026-06-04 manager boundary split reduced `zircon_plugins/animation/runtime/src/manager.rs` from a 599-line mixed evaluator/sampler into a 128-line facade plus `manager/{parameters,graph,state_machine,pose,sampling}.rs`. The split preserves `DefaultAnimationManager` and `AnimationManager` behavior while aligning graph, state-machine, clip-pose, parameter, and finite-sampling responsibilities with engine-scale animation runtime boundaries.
- The 2026-06-04 sequence boundary split reduced `zircon_plugins/animation/runtime/src/sequence.rs` from a 379-line mixed sequence implementation into an 11-line facade plus `sequence/{apply,channel_sample,conversion,interpolation,target,tests,time}.rs`. The split follows Theatre's sequence/keyframe separation and Unreal's sequence/track/section runtime separation while preserving current property-track writeback, target-id fallback, sample-time handling, channel sampling, Hermite/quaternion interpolation, and private coverage. `rustfmt --edition 2021 --check` passed over the sequence facade and all child files. `git diff --check -- zircon_plugins/animation/runtime/src/sequence.rs zircon_plugins/animation/runtime/src/sequence docs/zircon_plugins/animation/runtime.md docs/zircon_runtime/core/framework/animation.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with only expected LF-to-CRLF warnings on tracked files; trailing-whitespace and conflict-marker scans over the same files returned empty. Focused Cargo validation remains pending while active workspace Cargo/rustc lanes are running.
- The 2026-05-31 linked metadata parity slice first proved the gap with `cargo test --manifest-path zircon_plugins\animation\runtime\Cargo.toml animation_registration_contributes_runtime_module --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-animation-runtime-metadata --color never --quiet`: the linked package manifest still reported `Experimental` while the static TOML and built-in catalog reported `Beta`.
- After updating `runtime_plugin_descriptor()`, the same focused command passed with 1 Animation runtime test and 0 failures, validating category, maturity, and both partial capability-status rows for the linked runtime package manifest. Existing output was limited to unrelated `zircon_runtime` warnings.
- `cargo test --manifest-path Cargo.toml -p zircon_runtime --lib animation_plugin_toml_matches_catalog_beta_partial_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-animation-runtime-metadata --color never --quiet` passed with 1 runtime static-manifest/catalog test and 0 failures, validating `zircon_plugins/animation/plugin.toml` and the built-in catalog still agree on beta/partial Animation metadata.
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

## D11 animation/physics TestRuntime fixture migration

状态：`d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred`。

The animation/physics contract test now uses `zircon_plugin_sdk::TestRuntime` instead of rebuilding CoreRuntime, foundation/asset/scene modules, fixed-step clocks, scene hooks, and world runtime extensions inside the test. `runtime_physics_animation_tick_contract/runtime_helpers.rs` owns `TestRuntime::builder()` setup for the physics and animation plugins plus manager lookup helpers; `runtime_physics_animation_tick_contract/animation_assets.rs` owns the large animation asset fixtures; `runtime_physics_animation_tick_contract/target_resolution.rs` owns the clip/sequence target-id fallback contracts. The main test file stays focused on behavior and calls `runtime.create_default_level()` / `runtime.tick_level_seconds(...)`. Guard `review_d11_animation_physics_tests_use_sdk_test_runtime_fixture` locks this migration; Cargo is deferred for this status slice.
