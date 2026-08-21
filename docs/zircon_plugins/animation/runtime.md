---
related_code:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/mod.rs
  - zircon_plugins/animation/runtime/src/state_machine/mod.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/mod.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/mod.rs
  - zircon_plugins/animation/runtime/src/state_machine/condition_expression/mod.rs
  - zircon_plugins/animation/runtime/src/state_machine/transition/mod.rs
  - zircon_plugins/animation/runtime/src/gpu_skinning/mod.rs
  - zircon_plugins/animation/runtime/src/ik/mod.rs
  - zircon_plugins/animation/runtime/src/ik/postprocess.rs
  - zircon_runtime/src/core/framework/animation/ik_command.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_cache.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_cache.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/simulated_pose_blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/state_machine_layer_diagnostic.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/manager/graph.rs
  - zircon_plugins/animation/runtime/src/manager/parameters.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_plugins/animation/runtime/src/manager/sampling.rs
  - zircon_plugins/animation/runtime/src/manager/state_machine.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/mod.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/channel_sample.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/interpolation.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_target_binding.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/simulated_pose_blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/state_machine_layer_diagnostic.rs
  - zircon_runtime/src/core/framework/physics/skeletal_pose.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/state_machine_interruption.rs
  - zircon_runtime/src/core/framework/physics/skeletal_pose.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_plugins/animation_graph/editor/src/plugin.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/core/framework/animation/asset/state_kind.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets/target_path.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/labeled_subassets.rs
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
  - zircon_runtime/src/scene/runtime_hook/mod.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/world/compiled_binding/mod.rs
implementation_files:
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/evaluate.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/compile.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_1d.rs
  - zircon_plugins/animation/runtime/src/state_machine/blend_space/blend_space_2d.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/evaluate.rs
  - zircon_plugins/animation/runtime/src/state_machine/condition_expression/compile.rs
  - zircon_plugins/animation/runtime/src/state_machine/condition_expression/evaluate.rs
  - zircon_plugins/animation/runtime/src/state_machine/transition/transition_runtime.rs
  - zircon_plugins/animation/runtime/src/gpu_skinning/palette.rs
  - zircon_plugins/animation/runtime/src/gpu_skinning/double_buffer.rs
  - zircon_plugins/animation/runtime/src/gpu_skinning/decision.rs
  - zircon_plugins/animation/runtime/src/ik/two_bone.rs
  - zircon_plugins/animation/runtime/src/ik/look_at.rs
  - zircon_plugins/animation/runtime/src/ik/postprocess.rs
  - zircon_plugins/animation/runtime/src/ik/diagnostic.rs
  - zircon_plugins/animation/runtime/src/ik/execution_error.rs
  - zircon_runtime/src/core/framework/animation/ik_command.rs
  - zircon_runtime/src/core/framework/animation/ik_command_error.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/mod.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/manager/graph.rs
  - zircon_plugins/animation/runtime/src/manager/parameters.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_plugins/animation/runtime/src/manager/sampling.rs
  - zircon_plugins/animation/runtime/src/manager/state_machine.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/mod.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/channel_sample.rs
  - zircon_plugins/animation/runtime/src/channel_sampling/interpolation.rs
  - zircon_plugins/animation/runtime/src/runtime_system.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_target_binding.rs
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src/plugin.rs
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_runtime/src/core/framework/animation/asset/mod.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets/target_path.rs
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
  - user: 2026-07-13 书面设计通过，批准 Runtime02 注册服务 CoreWeak 拆分设计并开始实施
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md
  - docs/plans/zircon_plugins/04-animation.md
  - user: 2026-05-03 继续补独立插件缺口
  - user: 2026-05-08 继续周边设施与插件能力完善计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - docs/superpowers/plans/2026-05-03-physics-animation-aggressive-plugin-migration.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs::registry_owned_services_store_only_weak_runtime_back_references
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/tests/animation_avatar_mask_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_blend_space_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_compiled_graph_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_compiled_state_machine_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_state_kind_asset_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_gpu_skinning_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/ik_postprocess.rs
  - zircon_plugins/animation/runtime/tests/animation_pipeline_structure_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_state_transition_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/animation_assets.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/state_machine_interruption.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/blend_space_state.rs
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
  - cargo +nightly check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked --offline --jobs 1 (2026-07-11 Windows: passed)
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked --offline --jobs 1 (2026-07-11 Windows: 75/75 passed)
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --tests --locked --offline --jobs 1 (2026-07-11 WSL: 75/75 passed)
  - CARGO_INCREMENTAL=0; RUSTFLAGS=-C debuginfo=0; cargo +nightly test --locked --offline -p zircon_plugin_animation_runtime --test animation_ik_contract --jobs 1 --target-dir F:\cargo-targets\zircon-animation-m3-lowmem -- --nocapture (2026-07-11 Windows: 4/4 passed in 457.8s)
  - cargo +nightly test --manifest-path zircon_plugins/Cargo.toml --locked --offline -p zircon_plugin_animation_runtime --test runtime_physics_animation_tick_contract --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-plugin-checks -- --nocapture (2026-07-11 Windows: 34/34 passed in 9m22s build + 0.33s execution)
  - cargo check --manifest-path zircon_plugins/Cargo.toml --locked --target-dir target\codex-shared-a
  - cargo test -p zircon_runtime --locked --lib --target-dir target\codex-shared-a
doc_type: module-detail
---

# Animation Runtime Plugin

`zircon_plugins/animation/runtime` owns the production animation evaluation pipeline. The crate provides the canonical `animation.runtime` descriptor, a persistent `AnimationEvaluationPipeline`, compiled clip/graph/state-machine owners, sequence property writeback, and the runtime scene system that runs animation at `SystemStage::PostUpdate`.

The final concrete-manager hard cut from `zircon_runtime::animation` is still owned by the active Runtime architecture session. The plugin therefore enables that feature only for the remaining neutral manager/sequence seam; production clip/graph/state-machine sampling itself no longer routes through Runtime's string-scanning graph/state evaluators.

## Service Registry Ownership

The plugin `DefaultAnimationManager` may be installed as a Runtime registry service. Its constructor borrows `&CoreHandle` and stores only `CoreWeak`; settings persistence upgrades that weak reference at the operation boundary and becomes a no-op after the Runtime root is gone. The plugin does not retain a parallel strong handle or compatibility owner path.

## 2026-07-11 M1 Evaluation Architecture

- `runtime_system.rs` declares `AnimationEvaluationPipeline` as the scene resource and registers `animation.evaluate` at `PostUpdate`. New worlds receive that resource through the scene-owned `WorldRuntimeExtensionPlan`; the tick keeps an idempotent fallback for an already-live world created before a later plan installation.
- The runtime crate enables both Runtime `animation` and neutral `physics-contracts` features because its production pose-target handoff imports the framework physics DTO directly; this dependency is not left to dev-dependency feature unification.
- The old `src/scene_hook/**` tree is physically deleted. `evaluation/pipeline/{parameter_apply,state_machine_step,graph_evaluate,pose_blend,pose_apply}.rs` are the named phase owners; `tick.rs` only orchestrates their data flow.
- Final poses remain owned framework snapshots, but the `LevelSystem` handoff now retains stable entity entries and copies through reusable bone/name storage. Graph base blending also consumes the first weighted pose by ownership instead of cloning it. Together with `PosePool`, this removes the known stable-rig final-handoff allocation boundary without introducing a borrowed plugin object across the Runtime contract.
- `AnimationClipEvaluator` compiles skeleton targets and clip channels once per `{resource, revision}`, uses skeleton/clip LRU limits of 64/256, bounds diagnostic deduplication at 1,024 entries, recovers through typed invariant errors, and invalidates caches on resource events.
- `evaluation/compiled_graph/**` compiles node edges and parameters to dense slots, validates missing edges/duplicates/cycles, compiles skeleton masks to dense bone rows, and is cached by `{graph_id, skeleton_id, graph_revision, skeleton_revision}`.
- `CompiledAnimationStateMachine` compiles entry/from/to states and transition condition parameters to dense slots. Trigger condition names are compiled beside their selected transition and are consumed only when ordinary or interruption arbitration actually begins that transition. The scene pipeline commits active state and Trigger removal together after clip-event admission; deferred entities retain the complete update, and a same-name value that is no longer `Trigger` is preserved. `pipeline/state_machine_cache.rs` keeps a bounded revision-aware cache, so the production state-machine phase no longer scans state names or condition parameter names each frame.
- `PoseBlend` consumes compiled mask rows by bone index. The legacy string-mask path was deleted from production graph/state evaluation.
- The remaining M1-T3 allocation boundary is `LevelSystem::record_animation_poses(BTreeMap<EntityId, AnimationPoseOutput>)`: ownership moves the final vectors into Runtime and exposes only cloning reads. A reusable final pose owner requires a coordinated Runtime API handback; no plugin-local compatibility wrapper is introduced.
- PoseApply consumes Runtime08's `World::compile_descendant_name_index(...)` projection. Runtime owns each root's hierarchy/name generation; structured name and reparent writes invalidate only affected ancestor roots, while raw hierarchy-component mutation conservatively invalidates cached roots. `pose_target_binding.rs` derives plugin-local exact/short bone-name maps to dense `EntityId` values only when that Runtime binding changes. Transform writes and unrelated subtree edits retain the existing binding, while stable frames do not scan scene node records or reconstruct alias collections.
- M2-T1 adds `mask/{asset,compile,error}.rs`: `.avatar_mask.toml` rules compile subtree inheritance, ordered overrides, and boundary gradients into dense `MaskWeights` aligned with `SkeletonTargetTable`. `AnimationAvatarMask` remains the neutral editor/diagnostic view.
- M2-T2 adds `PoseLayer` and `PoseLayerBlendMode`; `PoseBuffer::blend_layers` applies ordered override/additive layers and multiplies each dense source row by the aligned `MaskWeights` row. Shape and layer weights are validated before mutation.
- M2-T3's Animation half reads Physics-owned `SimulatedPoseFeed` after the final graph/state-machine/layer blend and before IK. Physics supplies a per-bone `normalized_weight` that already combines ragdoll mode, avatar-mask, and interpolation-alpha weights. The plugin resolves exact unique bone names through the compiled skeleton target table and blends local TRS without allocating a per-frame name map; invalid rows do not mutate the pose. The resulting pose is then published as the next `SkeletalPoseTargets` snapshot.

## 2026-07-11 M3-M5 Runtime Extensions

- `state_machine/condition_expression/**` compiles Condition/All/Any/Not trees to postfix instructions and dense parameter slots. Legacy flat asset conditions enter the same evaluator as an implicit All expression.
- `state_machine/transition/**` owns bounded duration/elapsed time, normalized exit-time gating, the four interruption policies, and continuous crossfade weights. The neutral transition asset serializes optional exit time and interruption policy with serde defaults; compiled transitions retain the full descriptor, and production evaluation gates requests using graph-duration-normalized state time. `evaluation/pipeline/state_machine_transition.rs` applies the active-transition policy to current/next candidate states. When A→B is interrupted by B→C, `InterruptedTransitionSource` retains the already blended A/B pose and uses it as the new crossfade source, so the first interrupted frame is continuous instead of snapping to B.
- `evaluation/pipeline/graph_timing_cache.rs` owns a bounded 128-entry graph-duration cache. Its dependency signature includes clip ids, resource revisions, and playback speeds; exit-time normalization therefore reuses ready snapshots and invalidates when a dependent clip changes.
- `state_machine/blend_space/**` sorts 1D samples and compiles 2D samples to deterministic non-overlapping Delaunay triangles. Sampling uses line interpolation, barycentric weights, and nearest convex-hull projection outside the triangulation.
- `AnimationStateAsset` now owns an explicit `AnimationStateKindAsset` instead of an implicit graph field. `GraphRef` preserves the existing graph state, while BlendSpace1D/2D compile parameter names to dense slots and retain a fixed three-entry graph/weight sample array. `evaluation/pipeline/state_graph_sample.rs` owns direct-clip and multi-graph duration, clip-event, and pose sampling. Direct Clip states use the existing revision-aware clip evaluator without a temporary graph. SubMachine states resolve nested compiled machines, isolate their runtime state by entity plus machine lineage, reject cycles, and cap nesting at eight machines.
- `AnimationStateMachineAsset` is the single layered-machine representation. Its `layers` carry machine references, finite normalized weights, Override/Additive modes, and optional dense mask rows; current/v3/v2/v1 payload migration gives historical machines an empty layer list. `state_machine/layer/**` compiles this representation, while `evaluation/pipeline/state_machine_layers.rs` evaluates independent per-layer machine state and folds poses through `PoseLayer`/`PoseBuffer::blend_layers`. Layer transitions use the same interruption-policy selector as the base machine, retain the already blended source pose across A→B→C interruption, advance old/new event windows independently, and clear the retained source on completion. Bone-count/name, pose-row, mask, and blend-shape failures emit `AnimationStateMachineLayerDiagnostic` instead of silently dropping a layer.
- SubMachine runtime keys include the owning parent state in addition to entity and machine lineage, so sibling parent states that reference one child asset do not alias state. `nested_machine_resolve.rs` stops descent when a parent transition is active/requested, and `nested_machine_sample.rs` recursively samples SubMachine pose/event/time on transition endpoints.
- `gpu_skinning/**` builds a maximum-256-joint `SkinningPalette` as posed-world × inverse-bind, preserves current/previous palettes through `SkinningPaletteDoubleBuffer`, and returns typed GPU-versus-CPU decisions from the neutral readiness contract. The Render owner consumes the neutral `RenderSkeletalPoseExtract`, packs the same posed-world × inverse-bind matrices into `SkinnedMeshJointPaletteStorage`, and owns two persistent `STORAGE | COPY_DST` buffers per stable GPUScene instance. Draw build writes the alternate slot; successful submit alone commits it as previous, and a third frame reuses the first slot. Group3 bindings 3/4 expose current/previous palettes as vertex-visible read-only storage, while over-256 or unavailable payloads retain the CPU-skinned draw. No Render dependency on the concrete Animation plugin type is introduced.
- `ik/**` owns the complete M5-T1 postprocess. Components/scripts submit stable-ID `AnimationIkCommand` values through the neutral Manager queue; `ik/postprocess.rs` resolves those IDs through the evaluator's revision-aware `SkeletonTargetTable`, compiles private TwoBone/LookAt slot jobs, evaluates model-space targets after graph/state-machine/layer blending, and writes solved local rotations before Physics pose-target publication and PoseApply. TwoBone chains must be direct root→mid→tip; target/pole values are skeleton model-space points. Invalid bindings, target lookup, pose shape, hierarchy, chain, and solver inputs become typed `AnimationIkDiagnostic` events.
- The focused low-memory Windows executable run passes all four `animation_ik_contract` cases (three solver boundaries plus Manager validation/world isolation/drain semantics). The subsequent Windows nightly locked/offline production Tick executable passes both queued TwoBone and LookAt paths inside the complete 34/34 suite, proving dense target resolution, model-space solve, final local-pose mutation, one-shot command drain, and the required `PoseBlend/SimulatedPoseFeed → IK → SkeletalPoseTargets/PoseApply` ordering.
- `runtime_system.rs` registers `AnimationClipEvent`, `AnimationEvaluationDiagnostic`, `AnimationIkDiagnostic`, and `AnimationStateMachineLayerDiagnostic` through the SDK event builder. The derived `animation.events` catalog identifies clip, IK, and layer diagnostic payloads with explicit v1 schemas; publication remains dormant until a consumer subscription is connected.

## M6 Editor Ownership

- `animation_graph/editor` owns graph/state-machine asset editors, compile/validate/open operations, and graph palettes. BlendSpace1D/2D appear as graph nodes because they participate in pose topology; the editor package does not own generic Animation asset drawers.
- `animation/editor` owns the authoring surface shared by runtime animation assets. It registers dedicated BlendSpace1D, BlendSpace2D, and Avatar Mask drawers under `plugins://animation/editor/**`; the Avatar Mask drawer is therefore not duplicated behind the Animation Graph plugin.
- `timeline_sequence/editor` remains the single sequencer owner and registers the `animation.sequence` timeline editor plus transform, component-property, and event-marker track descriptors. Runtime sequence DTOs stay neutral and no editor-only type crosses into the Animation runtime crate.
- This split follows the fixed Editor/Runtime boundary: authoring packages consume neutral Runtime asset contracts, while evaluation, target compilation, IK, and pose production stay in `animation/runtime`.

## Runtime Boundary

- `AnimationRuntimePlugin` embeds the lifecycle descriptor; `RuntimePluginRegistrationReport::from_plugin(...)` installs it exactly once before provider-specific extension registration.
- The plugin contributes tick behavior through `RuntimePluginModuleRegistration::runtime_scene_system(...)` as `animation.evaluate` in `SystemStage::PostUpdate`, in set `animation.evaluation`, after `zircon.scene.world_transform`.
- `runtime_plugin_descriptor()` is the linked package-manifest source for the Animation runtime crate. It mirrors the static `zircon_plugins/animation/plugin.toml` and built-in catalog metadata: category `runtime`, maturity `beta`, `runtime.plugin.animation` status `partial` with Bevy `bevy_animation` source traceability, and `runtime.feature.animation.timeline_event_track` status `partial`.
- D5 editor authoring macro consumer guard keeps the editor package on the SDK macro path: `zircon_plugins/animation/editor/src/plugin.rs` uses `zircon_plugin_sdk::authoring_plugin!` with `mirrors_runtime_manifest: zircon_plugin_animation_runtime::package_manifest()` and only keeps the Animation-specific extension registration body outside the macro. Status `d5_editor_authoring_macro_consumers_static_passed_cargo_deferred` is locked by `review_d5_editor_authoring_plugins_use_sdk_macro`.
- D9 editor/runtime mirror consumer guard keeps the editor package tied to this runtime package manifest through the SDK declaration projection: editor tests assert `mirrored_runtime_package_id()`, and the package manifest carries both `zircon_plugin_animation_runtime::ANIMATION_RUNTIME_CAPABILITY` and the Animation authoring capability. `tools/audit_plugin_structure.py --json` reports `editor_runtime_mirror_violations = 0` and `d9_editor_runtime_mirror_gate_status = editor-runtime-mirror-clean`; status `d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred` is locked by `review_d9_editor_runtime_mirror_consumers_use_sdk_declaration`.
- D10 animation/physics bridge call migration keeps cross-plugin physics queries on the Plugins 11 bridge path. The runtime contract test resolves `physics.query.v1` from `runtime.extension_report().registry.frozen_bridge_table()` as `WeakBridge<dyn PhysicsQueryInterface>` and calls ray, overlap, and shape-cast through that weak bridge instead of concrete physics manager lookup. Guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`.
- `AnimationRuntimeSystem` resolves `AnimationManagerHandle`, advances scene player clocks, loads animation assets through `ProjectAssetManager`, blends graph/state-machine pose output, and records pose/playback runtime state on `LevelSystem`.
- `AnimationRuntimeSystem` publishes `AnimationClipEvent` values when direct clip players, graph players, state-machine active graphs, or state-machine transition graphs advance across `AnimationClipAsset.event_tracks`, matching Bevy's clip-event precedent for timeline-authored gameplay hooks.
- `runtime_system.rs` is the scheduling entry. `evaluation/pipeline/` is the folder-backed tick implementation; no path attributes or `scene_hook` compatibility modules remain.
- The linked plugin's `DefaultAnimationManager` and `animation.runtime` descriptor own playback settings persistence, graph evaluation, state-machine evaluation, clip pose sampling, and the bounded per-World neutral IK command queue. Concrete target compilation and solving remain plugin-owned; Runtime fallback manager/module types are not re-exported as plugin production types.
- `manager.rs` is the plugin-owned structural `DefaultAnimationManager` facade. `manager/parameters.rs` owns parameter default/value mutation and numeric scalar helpers, `graph.rs` owns graph clip collection plus additive/masked graph evaluation, `state_machine.rs` owns transition condition evaluation and active-state resolution, `pose.rs` owns skeleton bind validation plus clip bone-track sampling, and `sampling.rs` owns finite-value, sample-time, and channel-sample conversion helpers.
- The private `channel_sampling/` module currently supplies channel sampling and interpolation to the manager facade. `apply_sequence_to_world(...)` remains a Runtime interop re-export until Runtime08 publishes the generation-validated generic compiled property accessor; it must not become a second plugin production evaluator.
- `DefaultAnimationManager::evaluate_graph(...)` remains a neutral compatibility-facing contract, while the production pipeline consumes `CompiledAnimationGraph` and dense target masks directly.
- Base graph clips retain their authored positive finite weights until pose composition. The evaluator normalizes only the contributors that target each bone, so a masked clip cannot attenuate an unrelated bone; invalid weights do not participate and every quaternion is canonicalized to an input-order-independent hemisphere before accumulation.
- `DefaultAnimationManager::sample_clip_pose(...)` resolves `AnimationClipBoneTrackAsset.target_id` before the legacy `bone_name` fallback. An explicit target id is the complete canonical slash-joined skeleton path, for example `Root/Hand`; a unique leaf `bone_name` is considered only when `target_id` is `None`. The builtin glTF importer derives explicit paths from the selected skeleton's bone/parent table and rejects channels outside that skeleton during import.
- `apply_sequence_to_world(...)` resolves `AnimationSequenceBindingAsset.target_id` before the legacy `entity_path` fallback. Current runtime target ids accept a stable numeric `EntityId` string or the same canonical `EntityPath` text used by old bindings.
- `zircon_runtime::scene::WorldDriver` dispatches installed runtime scene systems by schedule stage and contains no animation-specific logic.

## Framework Contract

Runtime framework contracts are intentionally concrete-free:

- `apply_sequence_to_world(...)` remains the current Runtime-owned scene-writeback boundary and is reached through the plugin root only while Runtime08 completes compiled property access for every track kind; the neutral `AnimationManager` does not accept `scene::World`.
- `AnimationClipEvent` is the plugin-owned typed scene event for clip event tracks. It records the source entity, optional target id, event name, payload, clip time, and absolute playback time so looping clips can report boundary occurrences deterministically.
- `AnimationGraphBlendMode`, `AnimationGraphClipInstance::target_ids`, and `AnimationGraphEvaluation::mask_target_ids` describe additive/masked graph output without moving concrete graph runtime back into `zircon_runtime`.
- `AnimationClipBoneTrackAsset.target_id`, `AnimationSequenceBindingAsset.target_id`, and `AnimationClipAsset.event_tracks` add stable target/event metadata to the asset contract. Old `bone_name` and `entity_path` fallbacks remain available only when their corresponding explicit target id is absent; an invalid explicit clip path does not fall back to a leaf name.
- `AnimationSequenceApplyReport` reports applied and missing tracks without exposing plugin-owned sequence implementation details.
- `AnimationTimelineDescriptor`, `AnimationTimelineTrackDescriptor`, and `AnimationTimelineClipDescriptor` summarize sequence property tracks, clip bone tracks, event tracks, mask filtering, and clip spans for editor/runtime/VM callers without exposing plugin-owned sampler state.
- `AnimationPlayerRuntimeStatus`, `AnimationRigRuntimeStatus`, and `AnimationRuntimeStatus` expose player state, rig pose coverage, missing targets, GPU-skinning readiness, last tick work, and diagnostics as copied data.
- `ANIMATION_MANAGER_NAME` remains the stable service name consumed by runtime/editor callers.

The linked plugin owns the canonical manager/module identity. Remaining Runtime sequence interop is explicit and transitional; it must converge on Runtime08 compiled property access without reintroducing a Runtime production evaluator.

## Graph Pose Semantics

- Base graph clips are normalized against the total positive base weight before pose blending.
- Additive graph clips are applied after the base pose. Translation is added directly, scale is applied as a delta from `Vec3::ONE`, and rotation is applied as a weighted identity-to-additive rotation delta.
- Mask target ids limit base or additive writes to matching pose bones. Empty target ids mean the clip affects the whole pose; non-empty ids currently match either the bone name or the leaf of a slash path such as `Root/Hand`.
- State-machine transition blending continues to use the same weighted base-pose helper, so state transitions keep their existing cross-fade semantics while graph evaluation can add masked additive layers inside each sampled state graph.

## Binary Compatibility

- New `.zranim` bytes still write the wrapped `AnimationBinaryDocument` shape.
- Decode now also accepts the older stream shape that serialized `AnimationBinaryHeader` followed by the payload. This keeps already-authored version-1 `.zranim` clip, sequence, and graph assets readable without bumping `ANIMATION_BINARY_VERSION`.
- Legacy clip payloads decode with `target_id = None` and empty `event_tracks`; legacy sequence bindings decode with `target_id = None`; legacy graph nodes decode only the original clip/blend/output tags.
- State-machine decoding tries the current StateKind payload, then the former graph-state payload with current transition fields, then the original graph-state/transition payload. Both historical shapes migrate directly to `GraphRef`; no runtime compatibility facade or duplicate state field survives the load boundary.

## Clip Event Semantics

- Direct clip players sample event tracks over the half-open playback range `(previous_time_seconds, current_time_seconds]`.
- Graph players sample the same range from the graph playback clock, then convert each graph clip instance into clip-local time through its playback speed before event sampling.
- State-machine players sample the active state's graph over the state-machine playback clock when no transition is currently blending.
- State-machine transitions sample both the source state's graph and the target state's graph over their own transition-local time ranges. When the transition completes, the runtime stops saving transition state, so the following frame samples only the target active graph and does not repeat the completed transition range.
- Non-looping clips clamp event sampling to the clip duration and never repeat events after the end.
- Looping clips emit each crossed occurrence in playback order, including an event at clip time `0.0` when playback crosses a loop boundary.
- Events are sent to the world's typed event store and become readable after the normal `World::update_events::<AnimationClipEvent>()` step.

## Validation Evidence

- 2026-07-11 M1-T3 current evidence: WSL locked/offline `cargo +nightly check -p zircon_plugin_animation_runtime --tests` exits 0; `animation_compiled_graph_contract` passes 2/2; `animation_compiled_state_machine_contract` passes 2/2; the complete `runtime_physics_animation_tick_contract` passes 20/20 after the declarative-resource fixture boundary was corrected. Scoped rustfmt/diff checks pass and the production evaluation panic/allow scan is clean.
- 2026-07-11 M2-T1 evidence: `animation_avatar_mask_contract` passes 2/2 for subtree inheritance/override and boundary-gradient values after a RED fixture exposed canonical-path validation.
- 2026-07-11 M2-T2 evidence: `animation_pose_buffer_contract` passes 4/4 including `upper_lower_body_split_blend_scenario`; fresh locked/offline Animation `cargo check --tests` exits 0.
- 2026-07-11 M3-T2 hard-cut evidence: Windows nightly/offline `cargo check --lib` exits 0; `animation_compiled_state_machine_contract` passes 3/3, `animation_state_transition_contract` passes 3/3, and the production `runtime_physics_animation_tick_contract` passes 22/22. The production cases include delayed normalized exit-time gating, non-skipped crossfade duration, active A→B interruption by B→C with pose continuity, and source/target clip-event windows. The first aggregate run exposed an event-window regression (0 events instead of 2); after correcting new-transition start times, the same 28-test set passed.
- 2026-07-11 M3-T3 StateKind/BlendSpace evidence: Windows nightly/offline compiled-state tests pass 4/4, StateKind binary/direct-reference tests pass 1/1, and production 1D/2D pose tests each pass 1/1. The 2D case drives three real graph/clip resources through the Delaunay weights and records the expected 7.5 hand translation. Scoped formatting passes; the production sampling split reduces `state_machine_step.rs` from 438 to 325 lines and gives state sampling a dedicated owner. After adding the missing target type to the concurrently introduced project-manifest deserializer, the Windows nightly/offline all-tests check also exits 0; a newer full-suite test count is not yet claimed.
- 2026-07-11 direct Clip state evidence: the production RED first failed with no state pose; after compiling the clip reference directly and routing transition pose/events/exit-time through StateKind dispatch, compiled state-machine tests pass 5/5 and the concentrated direct-Clip/1D/2D production state suite passes 3/3 with expected 10.0/2.5/7.5 hand translations.
- 2026-07-11 SubMachine evidence: compiled state tests pass 6/6; production nested delegation passes 1/1; a nested child transition persists over two ticks and produces the expected 5.0→10.0 crossfade output, 1/1. Runtime keys include entity and machine lineage so sibling/nested state and interruption sources cannot alias.
- 2026-07-11 layer compiler evidence: `animation_state_machine_layer_contract` passes 3/3 for Override/Additive mapping, dense mask compilation, invalid input rejection, and stable direct-reference ordering.
- The complete pre-latest-slice Windows nightly/offline Animation runtime suite passes 78/78 after adding the Physics-owned skeletal target bridge; Physics default tests pass 43/43. The later production Tick executable passes 34/34 and now includes the executable `simulated_pose_blends_under_ragdoll_mask`, masked layer, SubMachine parent-transition, layer-interruption continuity, and TwoBone/LookAt paths. Physics ragdoll focused behavior independently passes 5/5, including the drop golden snapshot and no-pop velocity inheritance. A new all-`--tests` total is not inferred from these focused runs; post-hard-cut full Windows/WSL suites remain the final cross-platform gate, and the earlier WSL 75/75 baseline predates these additions.

- Current D10 animation/physics bridge call migration: the contract test now uses `WeakBridge<dyn PhysicsQueryInterface>` / `physics.query.v1` for physics ray, shape-overlap, and shape-cast calls after ticking the level. Static guard `review_d10_animation_physics_tests_use_sdk_bridge_call` records status `d10_animation_physics_bridge_call_static_passed_cargo_deferred`; Cargo remains deferred for this implementation slice.
- The 2026-06-04 scene hook boundary split reduced `zircon_runtime/src/animation/scene_hook.rs` from a mixed 867-line file to a structural 32-line entry plus `scene_hook/{tick,scan,pending,events,sequences,pose,graph,state_machine}.rs`. `rustfmt --edition 2021 --check --config skip_children=true` passed over the split files. A focused `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-animation-scene-hook-split-0604 --message-format short --color never` attempt timed out after two minutes while other workspace/Hub/editor Cargo lanes were active and did not return Rust diagnostics; compile acceptance for this structural split remains pending.
- The runtime plugin registers its world contributions in `RuntimeExtensionRegistry`; the composition/test host projects them with `world_runtime_extension_plan()` and installs the result through `scene::install_world_runtime_extension_plan(...)`. The obsolete CoreRuntime installation API and Core-owned extension set are absent.
- The 2026-06-04 manager boundary split reduced `zircon_plugins/animation/runtime/src/manager.rs` from a 599-line mixed evaluator/sampler into a 128-line facade plus `manager/{parameters,graph,state_machine,pose,sampling}.rs`. The split preserves `DefaultAnimationManager` and `AnimationManager` behavior while aligning graph, state-machine, clip-pose, parameter, and finite-sampling responsibilities with engine-scale animation runtime boundaries.
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
