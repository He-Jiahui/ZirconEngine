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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
plan_sources:
  - user: 2026-05-02 ZirconEngine Particles 插件完善计划
  - .codex/plans/ZirconEngine Particles 插件完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - tests/acceptance/particles-gpu-readback-mailbox.md
doc_type: module-detail
---

# Particles Runtime Plugin

## Purpose

The particles plugin owns particle asset descriptions, scene-facing component descriptors, runtime simulation state, editor authoring descriptors, and the plugin render feature descriptor for sprite particles. `zircon_runtime` remains the neutral host: it owns render DTOs, RHI, render graph execution, and the built-in particle renderer, but it does not own concrete particle simulation.

The first implemented backend is CPU sprite simulation. GPU simulation now has the same asset model compiled into a concrete storage layout, WGSL compute program, frame spawn schedule, render graph pass order, renderer-owned wgpu executor, and neutral counter/indirect readback DTO. `ParticlesManager` still falls back to CPU when it is used without a renderer-attached GPU executor.

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
- `ParticleGpuBackend` owns double-buffered particle storage, emitter params, atomic counters, alive index compaction, indirect draw args, and a debug/readback buffer. Its frame order is spawn/update compute, compact-alive compute, build-indirect-args compute.
- `ParticleGpuCounterReadback` decodes the debug/readback counter words and projects them into neutral `RenderParticleGpuReadbackOutputs`, including alive count, spawned total, debug flags, per-emitter spawned counts, and indirect draw args. `SceneRendererAdvancedPluginOutputs` stores this payload in the shared plugin renderer output mailbox and can take the particle slot without clearing VG/HGI slots.
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
9. When counter readback is requested, `ParticleGpuBackend::read_render_outputs_readback` decodes counter words plus indirect draw args into `RenderParticleGpuReadbackOutputs`. The scene-renderer advanced plugin output mailbox can store that neutral payload and expose it through `take_last_particle_gpu_readback_outputs` while leaving VG/HGI readbacks intact.

## Design and Rationale

The module follows the current plugin architecture rather than adding particle-specific runtime ownership. The asset and CPU simulation live in `zircon_plugins/particles`, while render DTOs remain neutral. This matches the independent-plugin plan and leaves room for GPU simulation without forcing `zircon_runtime` to depend on plugin concrete types.

The CPU pool stores `alive`, `age`, `lifetime`, `position`, `previous_position`, `velocity`, `size`, `color`, `rotation`, `angular_velocity`, `seed`, and `emitter_index` as separate arrays. The GPU path extends that channel list with `initial_size` and `start_color` because color/size-over-life evaluation needs stable spawn-time values.

Reference evidence used for the GPU direction:

- Unreal Niagara: `NiagaraComputeExecutionContext.h`, `NiagaraDataSet.h`, `NiagaraDataInterface.h`, and `NiagaraDataInterfaceRW.h` show GPU compute dispatch, double-buffered data buffers, simulation stage hooks, and indirect dispatch argument generation.
- Unity VFX Graph: `VFXDataParticle.cs`, `VFXGraphCompiledData.cs`, and `VFXCodeGenerator.cs` show attribute layout compilation, indirect buffer allocation, generated compute shader code, active indirection, and compute bounds handling.
- Bevy render: `bevy_render/src/batching/gpu_preprocessing.rs` shows Rust/wgpu-oriented indirect parameter buffers, compute preprocessing, CPU metadata, and debug-copy settings.

The intentional Zircon divergence is that particle assets stay in `zircon_plugins/particles` and the GPU executor is an explicit renderer-owned object. The shared runtime graph sees only external resource names and pass descriptors; it does not gain a plugin-specific dependency.

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

## Test Coverage

`zircon_plugins/particles/runtime/src/tests.rs` covers plugin registration, component/options/event contributions, CPU spawn rate, deterministic seed behavior, lifetime death, free-list reuse, pause/stop/preview rewind, extract sorting, material/texture/rotation extraction, bounds and sort metadata, non-finite asset rejection including burst times and animation binding progress, capability-gated physics diagnostics, late physics capability propagation, external force application, capability-gated animation diagnostics and event control, GPU layout/fallback, GPU pass order, WGSL parse coverage, GPU frame spawn planning, capacity clamp diagnostics, and optional physics/animation helper behavior.

`zircon_plugins/particles/editor/src/tests.rs` covers editor views, templates, asset editor registration, CPU sprite asset creation template registration, disabled descriptor-level authoring operations, preview operations, capability gating, the particle system component drawer, compile-time include guards for the starter particle asset template, and compile-time include guards for the concrete authoring, preview, and component drawer `.ui.toml` documents.

`zircon_runtime/src/graphics/tests/m4_behavior_layers.rs` keeps a render-side guard that the particle shader preserves alpha instead of forcing opaque output.

Inline runtime tests in `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs`, `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs`, and `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs` cover neutral particle readback empty detection, mailbox storage, collection, and particle-only take semantics.

## Open Issues

- GPU transparent rendering from `alive_indices` and `indirect_draw_args` is not yet automatically connected to the built-in particle renderer or neutral render-framework executor path. The compute backend and transparent render hook can produce/consume the buffers, and the neutral mailbox can carry counter/indirect readbacks, but `RenderPassExecutionContext` remains metadata-only and the standard renderer still draws CPU-extracted billboards until a renderer-hosted GPU executor consumes those buffers.
- Full CPU/GPU parity for multi-key curves, material/texture metadata, bounds, rotation, angular velocity, and readback comparison is not complete. CPU extraction now carries those fields; GPU v1 covers spawn/update, gravity, drag, first-to-last color/size interpolation, alive compaction, indirect args, and neutral counter/indirect readback projection.
- No particles runtime feedback provider equivalent to the current VG/HGI feedback path exists yet. Host code can take `RenderParticleGpuReadbackOutputs` from the scene-renderer output mailbox, but the runtime submission feedback batch does not automatically route particle GPU readbacks into a particle manager/provider update loop.
- Runtime scene auto-collection is not wired in this milestone. Hosts can instantiate `ParticleSystemComponent` values through the manager, and future scene integration should collect dynamic components into the manager without moving simulation into `zircon_runtime`.
