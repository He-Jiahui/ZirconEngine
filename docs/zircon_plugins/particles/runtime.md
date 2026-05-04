---
related_code:
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/module.rs
  - zircon_plugins/particles/runtime/src/package.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/simulation/pool.rs
  - zircon_plugins/particles/runtime/src/simulation/rng.rs
  - zircon_plugins/particles/runtime/src/render/extract.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/particles/runtime/src/render/gpu/mod.rs
  - zircon_plugins/particles/runtime/src/render/gpu/layout.rs
  - zircon_plugins/particles/runtime/src/render/gpu/program.rs
  - zircon_plugins/particles/runtime/src/render/gpu/planner.rs
  - zircon_plugins/particles/runtime/src/render/gpu/readback.rs
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/interop/animation.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/particles/editor/authoring.ui.toml
  - zircon_plugins/particles/editor/preview.ui.toml
  - zircon_plugins/particles/editor/particle_system.drawer.ui.toml
  - zircon_plugins/particles/templates/cpu_sprite_system.toml
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/gpu_feedback.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/runtime_feedback_batch.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/particle_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
implementation_files:
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/simulation/pool.rs
  - zircon_plugins/particles/runtime/src/render/extract.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/gpu/layout.rs
  - zircon_plugins/particles/runtime/src/render/gpu/program.rs
  - zircon_plugins/particles/runtime/src/render/gpu/planner.rs
  - zircon_plugins/particles/runtime/src/render/gpu/readback.rs
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/authoring.ui.toml
  - zircon_plugins/particles/editor/preview.ui.toml
  - zircon_plugins/particles/editor/particle_system.drawer.ui.toml
  - zircon_plugins/particles/templates/cpu_sprite_system.toml
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/gpu_feedback.rs
  - zircon_runtime/src/graphics/particle_runtime_provider/runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/runtime_feedback_batch.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/particle_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
plan_sources:
  - user: 2026-05-02 ZirconEngine Particles 插件完善计划
  - .codex/plans/ZirconEngine Particles 插件完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - docs/superpowers/specs/2026-05-03-particles-full-render-graph-refactor-design.md
  - docs/superpowers/plans/2026-05-03-particles-full-render-graph-refactor.md
tests:
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - tests/acceptance/particles-gpu-readback-mailbox.md
validation:
  - 2026-05-04: cargo check -p zircon_runtime --tests --locked --offline --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed with existing warnings)
  - 2026-05-04: cargo test -p zircon_runtime --lib graph_execution --locked --offline --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed 16/16 filtered tests with existing warnings)
  - 2026-05-04: cargo test -p zircon_runtime --lib compile_preserves_renderer_stage_for_each_graph_pass --locked --offline --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed 1/1 filtered test with existing warnings)
  - 2026-05-04: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_runtime --locked --offline --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed 20/20 tests with existing runtime warnings)
  - 2026-05-04: cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_particles_editor --locked --offline --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed 1/1 test with existing runtime/editor warnings)
  - 2026-05-04: rustfmt --edition 2021 --check <scoped render graph/runtime prepare/particles files> (passed)
  - 2026-05-04: git diff --check -- <scoped render graph/runtime prepare/particles/docs/session/plan files> (no whitespace errors; LF-to-CRLF warnings only)
doc_type: module-detail
---

# Particles Runtime Plugin

## Purpose

The particles plugin owns particle asset descriptions, scene-facing component descriptors, runtime simulation state, editor authoring descriptors, and the plugin render feature descriptor for sprite particles. `zircon_runtime` remains the neutral host: it owns render DTOs, RHI, render graph execution, and the built-in particle renderer, but it does not own concrete particle simulation.

The first implemented backend is CPU sprite simulation. GPU simulation now has the same asset model compiled into a concrete storage layout, WGSL compute program, frame spawn schedule, render graph pass order, renderer-owned staged graph executor surface, and neutral counter/indirect readback DTO. `ParticlesManager` still falls back to CPU when it is used without a renderer-attached GPU executor.

## Behavior Model

- `ParticleSystemAsset` contains one or more `ParticleEmitterAsset` records. Each emitter defines capacity, spawn rate, bursts, lifetime, spawn shape, initial velocity, gravity, drag, material and texture handles, optional physics options, optional animation bindings, color over lifetime, size over lifetime, and local/world coordinate space.
- `ParticleSystemComponent` binds an asset to an entity, transform, play state, and time scale. The plugin registers `particles.Component.ParticleSystem` as a dynamic component type.
- `ParticlesManager` instantiates components into stable `ParticleEmitterHandle` values, then controls play, pause, stop, explicit tick, preview rewind, state snapshots, and neutral `ParticleExtract` generation.
- CPU simulation stores particle channels in a structure-of-arrays pool with an explicit free list. The GPU layout uses the same channel names as SoA buffer sections: alive, age, lifetime, position, previous position, velocity, size, initial size, color, start color, rotation, angular velocity, seed, and emitter index.
- `build_particle_extract` turns runtime snapshots into neutral render DTOs and can sort sprites back-to-front when the caller provides a camera position. The extract also carries per-entity particle bounds and the camera position used for sorting, so renderer and editor consumers can reason about culling/debug metadata without depending on the particles plugin.
- CPU sprite snapshots preserve per-emitter material handles, texture handles, and particle rotation. The built-in runtime particle billboard builder applies rotation around the camera-facing right/up basis while retaining the existing alpha-preserving transparent path.
- Optional physics is capability-gated by `runtime.feature.particles.physics`. Without that capability, physics modules produce diagnostics and run as no-op; with it, CPU simulation applies external force and collision damping settings from the emitter options. Enabling the capability after instantiation propagates to existing CPU particle instances.
- Optional animation control is capability-gated by `runtime.feature.particles.animation_control`. Without that capability, events and bindings produce diagnostics and do not mutate particles; with it, spawn-once and timed emission begin/end events can target an emitter handle or resolve by entity.
- `ParticleGpuFramePlanner` accumulates burst and spawn-rate requests for GPU assets. It produces one emitter parameter block per frame, including capacity ranges, module constants, shape parameters, transform rows, color and size endpoints, and the per-emitter spawn count.
- `ParticleExtract.gpu_frame` carries the neutral renderer-facing GPU frame summary for GPU-backed emitters: alive count, spawned total, per-emitter spawned counts, and non-indexed indirect draw args. This is runtime-framework DTO data, not a particles concrete type.
- `ParticleGpuBackend` owns double-buffered particle storage, emitter params, atomic counters, alive index compaction, indirect draw args, and a debug/readback buffer. Its frame order is spawn/update compute, compact-alive compute, build-indirect-args compute.
- `particle_render_pass_executor_registrations` exposes four normal graph executors: `particle.gpu.spawn-update`, `particle.gpu.compact-alive`, `particle.gpu.indirect-args`, and `particle.transparent`. The executors validate their pass metadata/resource contract in metadata-only tests and, when the renderer attaches a GPU context, can consume named graph resources or emit neutral particle readback outputs through `RenderPluginRendererOutputs`.
- `ParticleGpuCounterReadback` decodes the debug/readback counter words and projects them into neutral `RenderParticleGpuReadbackOutputs`, including alive count, spawned total, debug flags, per-emitter spawned counts, and indirect draw args. `SceneRendererAdvancedPluginOutputs` stores this payload in the shared plugin renderer output mailbox and can take the particle slot without clearing VG/HGI slots.
- `ParticleRuntimeFeedback` and `ParticleGpuFeedback` are neutral runtime feedback carriers in `zircon_runtime`. Runtime submission now drains the renderer particle mailbox and merges prepared sideband particle outputs into this feedback packet, updates particle GPU feedback stats, and leaves concrete state application to the particles plugin manager or host. `ParticlesManager::apply_gpu_feedback` stores the last non-empty neutral particle readback packet for diagnostics/parity without mutating the CPU simulation snapshot; empty feedback means no new GPU packet and does not erase the prior diagnostic packet.
- The editor authoring surface is registered from `zircon_plugins/particles/editor/src/authoring.rs`. It contributes the `particles.authoring` and `particles.preview` views, concrete `.ui.toml` templates for authoring/preview/component drawer surfaces, a `ParticleSystemComponent` drawer, a `particles.system` asset editor, a CPU sprite particle-system creation template, and descriptor-level operations for create, add component, open, add emitter, add module, edit curve, validate, preview play, pause, stop, rewind, and warmup.
- The CPU sprite creation template points at `zircon_plugins/particles/templates/cpu_sprite_system.toml`, a starter TOML document for a local-space CPU sprite emitter. Until concrete editor operation handlers are added, non-view particles authoring menu rows are registered disabled and the corresponding operations are not callable from remote/CLI. This keeps schema/template discovery visible without presenting an enabled click path that would fail with an unhandled operation.

## Data Flow

1. Runtime plugin registration contributes the `ParticlesModule`, particle component descriptor, particle options, dynamic event catalog, optional physics/animation/GPU feature manifests, and particle render feature.
2. Editor plugin registration contributes capability-gated authoring descriptors. The root `lib.rs` delegates to the `authoring` module so the crate entry remains structural while the descriptor batch owns authoring operations and asset-template wiring.
3. A host or editor tool creates a `ParticleSystemComponent` and calls `ParticlesManager::instantiate`.
4. `ParticlesManager::tick` advances all playing instances. Spawn rate and burst emission are accumulated per emitter; random sampling uses a deterministic local RNG seeded from the asset seed and handle.
5. Optional animation events enter through `ParticlesManager::apply_animation_event`. The manager verifies the animation-control capability before applying spawn-once or timed emission state changes.
6. `ParticlesManager::snapshot` reports emitter state, live sprites, and diagnostics. `ParticlesManager::build_extract` converts that snapshot into `ParticleExtract` for the render framework, including sorted sprites, bounds, material/texture handles, and the sort camera metadata.
7. The runtime particle renderer builds rotated billboards from `ParticleExtract.sprites`; the shader now preserves vertex alpha and the pipeline uses standard transparent blending.
8. GPU-capable render hosts can compile `ParticleGpuProgram`, use `ParticleGpuFramePlanner` for per-frame spawn requests, and call `ParticleGpuBackend::execute_frame` with their `wgpu::Queue` and command encoder. The generated indirect args buffer uses non-indexed draw layout: vertex count 6, instance count equal to compacted alive particles, first vertex 0, first instance 0.
9. The renderer-owned compiled graph path executes particle pass descriptors at the `Transparent` stage while the scene command encoder is live. Shared runtime imports `scene-color` and `scene-depth` into `RenderGraphExecutionResources`; particle executors refer to those names and particle buffer names through graph resource contracts rather than direct runtime/plugin coupling.
10. When counter readback is requested, `ParticleGpuBackend::read_render_outputs_readback` decodes counter words plus indirect draw args into `RenderParticleGpuReadbackOutputs`. During staged graph execution, `particle.gpu.indirect-args` can also project `ParticleExtract.gpu_frame` into the neutral `RenderPluginRendererOutputs.particles` sink. The scene-renderer advanced plugin output mailbox can store that neutral payload and expose it through `take_last_particle_gpu_readback_outputs` while leaving VG/HGI readbacks intact.
11. Runtime submission collects particle GPU readback from both renderer mailbox output and prepared runtime sideband output. Renderer-produced readback is treated as authority when both exist; otherwise the sideband packet is used. Non-empty readback becomes `ParticleRuntimeFeedback`, contributes `RenderStats` particle GPU counters, and can be handed to `ParticlesManager::apply_gpu_feedback` by a host/plugin runtime owner. The manager records only non-empty packets, so frames without particle readback do not clear the last diagnostic/parity packet.

## Design and Rationale

The module follows the current plugin architecture rather than adding particle-specific runtime ownership. The asset and CPU simulation live in `zircon_plugins/particles`, while render DTOs remain neutral. This matches the independent-plugin plan and leaves room for GPU simulation without forcing `zircon_runtime` to depend on plugin concrete types.

The CPU pool stores `alive`, `age`, `lifetime`, `position`, `previous_position`, `velocity`, `size`, `color`, `rotation`, `angular_velocity`, `seed`, and `emitter_index` as separate arrays. The GPU path extends that channel list with `initial_size` and `start_color` because color/size-over-life evaluation needs stable spawn-time values.

Reference evidence used for the GPU direction:

- Unreal Niagara: `NiagaraComputeExecutionContext.h`, `NiagaraDataSet.h`, `NiagaraDataInterface.h`, and `NiagaraDataInterfaceRW.h` show GPU compute dispatch, double-buffered data buffers, simulation stage hooks, and indirect dispatch argument generation.
- Unity VFX Graph: `VFXDataParticle.cs`, `VFXGraphCompiledData.cs`, and `VFXCodeGenerator.cs` show attribute layout compilation, indirect buffer allocation, generated compute shader code, active indirection, and compute bounds handling.
- Bevy render: `bevy_render/src/batching/gpu_preprocessing.rs` shows Rust/wgpu-oriented indirect parameter buffers, compute preprocessing, CPU metadata, and debug-copy settings.

The intentional Zircon divergence is that particle assets stay in `zircon_plugins/particles` and the GPU executor is an explicit renderer-owned object. The shared runtime graph sees only external resource names and pass descriptors; it does not gain a plugin-specific dependency.

Particles M6 uses the same neutral renderer graph surface as other advanced plugins. `zircon_runtime` owns stage metadata, `RenderPassExecutionContext`, `RenderGraphExecutionResources`, `RenderGraphExecutionRecord`, and the renderer command-encoder lifetime. The particles plugin owns only descriptor contracts, executor functions, GPU layout/program/readback logic, and particle DTO projection. The transparent graph slice runs before CPU billboard fallback, so the CPU path remains available until named particle buffers are provided by a renderer-attached GPU backend.

The feedback continuation keeps the same ownership split. `zircon_runtime` does not own particle simulation and does not create a particle provider registry in this slice; it only exposes neutral `ParticleGpuFeedback` / `ParticleRuntimeFeedback` packets and records render-framework stats from them. `zircon_plugins/particles` consumes that neutral feedback through `ParticlesManager::apply_gpu_feedback`, which records the last non-empty GPU readback for diagnostics and parity tooling without changing CPU fallback particle state.

## Edge Cases and Constraints

- Delta time must be finite and non-negative.
- Assets must contain at least one emitter.
- Non-finite scalar, vector, shape, physics, color, curve, burst-time, and animation-binding settings are rejected at instantiate time.
- Particle bounds are generated from sprite size and position after CPU extraction. Empty extracts carry no bounds.
- `Gpu` backend requests through `ParticlesManager` run through CPU fallback unless a renderer host separately owns and drives `ParticleGpuBackend`. This keeps the manager usable in headless/editor tests without a `wgpu::Device`.
- GPU capacity is clamped to `PARTICLE_GPU_MAX_PARTICLES`. Per-emitter capacities are assigned in emitter order, so overflow emitters receive zero capacity and a compile diagnostic records the clamp.
- GPU v1 evaluates color and size curves as first-to-last linear endpoints. Assets with more than two keys compile with a warning diagnostic instead of silently pretending to have full curve parity.
- Physics and animation support are capability-gated helper surfaces only in this milestone; they do not create hard dependencies on the physics or animation plugins. Their unavailable paths are explicit diagnostics rather than silent behavior changes.
- A neutral particle GPU readback payload is considered present only when count/debug/per-emitter/indirect fields are non-default. Taking the particle payload drains only the particles slot and keeps other plugin renderer outputs available to their own consumers.
- Empty/default particle runtime feedback is treated as no new packet instead of a reset signal. Hosts that need explicit diagnostic clearing must add a separate lifecycle decision rather than relying on absent GPU readback for clearing.
- Staged graph execution currently validates named particle resources and emits neutral readback DTOs; it does not yet issue the full concrete particle compute/draw dispatch from `RenderGraphExecutionResources` alone. Missing `scene-color` or `scene-depth` in the renderer resource registry is reported as a graph resource binding error.

## Test Coverage

`zircon_plugins/particles/runtime/src/tests.rs` covers plugin registration, component/options/event contributions, CPU spawn rate, deterministic seed behavior, lifetime death, free-list reuse, pause/stop/preview rewind, extract sorting, material/texture/rotation extraction, bounds and sort metadata, non-finite asset rejection including burst times and animation binding progress, capability-gated physics diagnostics, late physics capability propagation, external force application, capability-gated animation diagnostics and event control, GPU layout/fallback, GPU pass order, WGSL parse coverage, GPU frame spawn planning, neutral GPU frame extract projection, particle graph executor resource-contract validation, capacity clamp diagnostics, and optional physics/animation helper behavior.

`zircon_plugins/particles/editor/src/tests.rs` covers editor views, templates, asset editor registration, CPU sprite asset creation template registration, disabled descriptor-level authoring operations, preview operations, capability gating, the particle system component drawer, compile-time include guards for the starter particle asset template, and compile-time include guards for the concrete authoring, preview, and component drawer `.ui.toml` documents.

`zircon_runtime/src/graphics/tests/m4_behavior_layers.rs` keeps a render-side guard that the particle shader preserves alpha instead of forcing opaque output.

Inline runtime tests in `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs`, `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs`, and `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs` cover neutral particle readback empty detection, mailbox storage, collection, and particle-only take semantics. Runtime graph tests in `compiled_render_pipeline.rs`, `render_pipeline_asset/compile.rs`, `render_graph_execution_record.rs`, `render_graph_execution_resources.rs`, `render_pass_execution_context.rs`, and `render_pass_executor_registry.rs` cover stage preservation, named resource errors, GPU-context requirement errors, staged execution record counts, and object-backed executor invocation.

Runtime submission feedback tests in `prepared_runtime_submission.rs`, `submit/collect_runtime_feedback.rs`, and `submit/submit_runtime_frame.rs` cover particle sideband preservation, particle readback merge authority, renderer-vs-sideband selection, and prepared sideband projection on the direct runtime-frame submit path. Particles runtime tests cover `ParticlesManager::apply_gpu_feedback` storing neutral readback while preserving CPU snapshot state and preserving the prior non-empty packet across empty feedback.

2026-05-04 scoped validation for the M6 graph refactor used `target\codex-shared-a` with `--locked --offline`. Runtime test targets compiled, runtime graph execution tests passed 16/16 filtered tests, the pipeline stage-preservation regression passed, particles runtime passed 20/20 tests, and particles editor passed 1/1 test. These are scoped gates for the particles/render-graph lane; full workspace validation was not run from this dirty checkout.

2026-05-04 scoped validation for the particle feedback continuation used `target\codex-shared-a` with `--locked --offline` after an offline `zircon_plugins/Cargo.lock` refresh added the dependency edges Cargo required. Runtime and particle test targets compiled, targeted particle feedback merge/sideband/direct-submit tests passed, the manager feedback ingest regression passed, scoped `rustfmt --check` passed, and scoped `git diff --check` found no whitespace errors beyond LF-to-CRLF warnings. Full workspace validation was still not run from this dirty checkout.

## Open Issues

- GPU transparent rendering from `alive_indices` and `indirect_draw_args` is not yet fully connected to the built-in particle renderer draw path. The renderer now provides a GPU-capable `RenderPassExecutionContext` and staged graph execution; the current particle executors validate named resources and emit neutral readbacks, while the standard renderer still draws CPU-extracted billboards until concrete particle GPU buffers are submitted through the renderer resource registry.
- Full CPU/GPU parity for multi-key curves, material/texture metadata, bounds, rotation, angular velocity, and readback comparison is not complete. CPU extraction now carries those fields; GPU v1 covers spawn/update, gravity, drag, first-to-last color/size interpolation, alive compaction, indirect args, and neutral counter/indirect readback projection.
- A neutral particles runtime feedback packet now exists and runtime submission routes particle GPU readbacks into stats/feedback. The remaining feedback gap is automatic provider/manager lifecycle ownership: there is still no registered particles runtime provider equivalent to VG/HGI that the render framework can instantiate and update without host/plugin handoff.
- Runtime scene auto-collection is not wired in this milestone. Hosts can instantiate `ParticleSystemComponent` values through the manager, and future scene integration should collect dynamic components into the manager without moving simulation into `zircon_runtime`.
