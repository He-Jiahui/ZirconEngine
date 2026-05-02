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
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/interop/animation.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_plugins/particles/editor/src/lib.rs
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
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
plan_sources:
  - user: 2026-05-02 ZirconEngine Particles 插件完善计划
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
doc_type: module-detail
---

# Particles Runtime Plugin

## Purpose

The particles plugin owns particle asset descriptions, scene-facing component descriptors, runtime simulation state, editor authoring descriptors, and the plugin render feature descriptor for sprite particles. `zircon_runtime` remains the neutral host: it owns render DTOs, RHI, render graph execution, and the built-in particle renderer, but it does not own concrete particle simulation.

The first implemented backend is CPU sprite simulation. GPU simulation now has the same asset model compiled into a concrete storage layout, WGSL compute program, frame spawn schedule, render graph pass order, and a renderer-owned wgpu executor. `ParticlesManager` still falls back to CPU when it is used without a renderer-attached GPU executor.

## Behavior Model

- `ParticleSystemAsset` contains one or more `ParticleEmitterAsset` records. Each emitter defines capacity, spawn rate, bursts, lifetime, spawn shape, initial velocity, gravity, drag, color over lifetime, size over lifetime, and local/world coordinate space.
- `ParticleSystemComponent` binds an asset to an entity, transform, play state, and time scale. The plugin registers `particles.Component.ParticleSystem` as a dynamic component type.
- `ParticlesManager` instantiates components into stable `ParticleEmitterHandle` values, then controls play, pause, stop, explicit tick, preview rewind, state snapshots, and neutral `ParticleExtract` generation.
- CPU simulation stores particle channels in a structure-of-arrays pool with an explicit free list. The GPU layout uses the same channel names as SoA buffer sections: alive, age, lifetime, position, previous position, velocity, size, initial size, color, start color, rotation, angular velocity, seed, and emitter index.
- `build_particle_extract` turns runtime snapshots into neutral render DTOs and can sort sprites back-to-front when the caller provides a camera position.
- `ParticleGpuFramePlanner` accumulates burst and spawn-rate requests for GPU assets. It produces one emitter parameter block per frame, including capacity ranges, module constants, shape parameters, transform rows, color and size endpoints, and the per-emitter spawn count.
- `ParticleGpuBackend` owns double-buffered particle storage, emitter params, atomic counters, alive index compaction, indirect draw args, and a debug/readback buffer. Its frame order is spawn/update compute, compact-alive compute, build-indirect-args compute.

## Data Flow

1. Runtime plugin registration contributes the `ParticlesModule`, particle component descriptor, particle options, dynamic event catalog, optional physics/animation/GPU feature manifests, and particle render feature.
2. A host or editor tool creates a `ParticleSystemComponent` and calls `ParticlesManager::instantiate`.
3. `ParticlesManager::tick` advances all playing instances. Spawn rate and burst emission are accumulated per emitter; random sampling uses a deterministic local RNG seeded from the asset seed and handle.
4. `ParticlesManager::snapshot` reports emitter state, live sprites, and diagnostics. `ParticlesManager::build_extract` converts that snapshot into `ParticleExtract` for the render framework.
5. The runtime particle renderer builds billboards from `ParticleExtract.sprites`; the shader now preserves vertex alpha and the pipeline uses standard transparent blending.
6. GPU-capable render hosts can compile `ParticleGpuProgram`, use `ParticleGpuFramePlanner` for per-frame spawn requests, and call `ParticleGpuBackend::execute_frame` with their `wgpu::Queue` and command encoder. The generated indirect args buffer uses non-indexed draw layout: vertex count 6, instance count equal to compacted alive particles, first vertex 0, first instance 0.

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
- Non-finite scalar emitter settings are rejected at instantiate time.
- `Gpu` backend requests through `ParticlesManager` run through CPU fallback unless a renderer host separately owns and drives `ParticleGpuBackend`. This keeps the manager usable in headless/editor tests without a `wgpu::Device`.
- GPU capacity is clamped to `PARTICLE_GPU_MAX_PARTICLES`. Per-emitter capacities are assigned in emitter order, so overflow emitters receive zero capacity and a compile diagnostic records the clamp.
- GPU v1 evaluates color and size curves as first-to-last linear endpoints. Assets with more than two keys compile with a warning diagnostic instead of silently pretending to have full curve parity.
- Physics and animation support are capability-gated helper surfaces only in this milestone; they do not create hard dependencies on the physics or animation plugins.

## Test Coverage

`zircon_plugins/particles/runtime/src/tests.rs` covers plugin registration, component/options/event contributions, CPU spawn rate, deterministic seed behavior, lifetime death, free-list reuse, pause/stop/preview rewind, extract sorting, GPU layout/fallback, GPU pass order, WGSL parse coverage, GPU frame spawn planning, capacity clamp diagnostics, and optional physics/animation helper behavior.

`zircon_plugins/particles/editor/src/lib.rs` covers editor views, templates, menu operations, and the particle system component drawer.

`zircon_runtime/src/graphics/tests/m4_behavior_layers.rs` keeps a render-side guard that the particle shader preserves alpha instead of forcing opaque output.

## Open Issues

- GPU transparent rendering from `alive_indices` and `indirect_draw_args` is not yet connected to the built-in particle renderer. The compute backend produces the buffers; the renderer still draws CPU-extracted billboards until a renderer executor consumes those buffers.
- Full CPU/GPU parity for multi-key curves, rotation, angular velocity, and readback comparison is not complete. GPU v1 covers spawn/update, gravity, drag, first-to-last color/size interpolation, alive compaction, and indirect args.
- Runtime scene auto-collection is not wired in this milestone. Hosts can instantiate `ParticleSystemComponent` values through the manager, and future scene integration should collect dynamic components into the manager without moving simulation into `zircon_runtime`.
