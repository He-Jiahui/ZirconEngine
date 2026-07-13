---
related_code:
  - zircon_plugins/particles/plugin.toml
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
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/particles/runtime/src/interop/animation.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_plugins/particles/runtime/src/tests/mod.rs
  - zircon_plugins/particles/runtime/src/tests/cpu_simulation.rs
  - zircon_plugins/particles/runtime/src/tests/extract.rs
  - zircon_plugins/particles/runtime/src/tests/gpu.rs
  - zircon_plugins/particles/runtime/src/tests/graph.rs
  - zircon_plugins/particles/runtime/src/tests/manager_resolution.rs
  - zircon_plugins/particles/runtime/src/tests/optional_features.rs
  - zircon_plugins/particles/runtime/src/tests/registration.rs
  - zircon_plugins/particles/runtime/src/tests/support.rs
  - zircon_plugins/particles/runtime/src/tests/validation.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/particles/editor/authoring.zui
  - zircon_plugins/particles/editor/preview.zui
  - zircon_plugins/particles/editor/particle_system.drawer.zui
  - zircon_plugins/particles/templates/cpu_sprite_system.toml
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
implementation_files:
  - zircon_plugins/particles/plugin.toml
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
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/particles/editor/src/lib.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/authoring.zui
  - zircon_plugins/particles/editor/preview.zui
  - zircon_plugins/particles/editor/particle_system.drawer.zui
  - zircon_plugins/particles/templates/cpu_sprite_system.toml
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle.wgsl
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
plan_sources:
  - user: 2026-05-02 ZirconEngine Particles 插件完善计划
  - .codex/plans/ZirconEngine Particles 插件完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - docs/superpowers/specs/2026-05-03-particles-full-render-graph-refactor-design.md
  - docs/superpowers/plans/2026-05-03-particles-full-render-graph-refactor.md
tests:
  - zircon_plugins/particles/runtime/src/tests/mod.rs
  - zircon_plugins/particles/runtime/src/tests/cpu_simulation.rs
  - zircon_plugins/particles/runtime/src/tests/extract.rs
  - zircon_plugins/particles/runtime/src/tests/gpu.rs
  - zircon_plugins/particles/runtime/src/tests/graph.rs
  - zircon_plugins/particles/runtime/src/tests/manager_resolution.rs
  - zircon_plugins/particles/runtime/src/tests/optional_features.rs
  - zircon_plugins/particles/runtime/src/tests/registration.rs
  - zircon_plugins/particles/runtime/src/tests/support.rs
  - zircon_plugins/particles/runtime/src/tests/validation.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs
  - cargo test -p zircon_runtime --lib runtime_15_particle_gpu_readback_output_accessor_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_advanced_plugin_output_test_accessor_cleanup --no-default-features --features core-min --locked
  - 2026-06-01: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime --locked --jobs 1 --message-format short --color never (passed 21 tests after package option strictness fix)
  - 2026-06-18: cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never (passed with existing zircon_runtime warnings)
  - 2026-06-18: cargo test -p zircon_plugin_particles_runtime particles_runtime_plugin_module_and_runtime_prepare_share_manager --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture (passed 1 test)
  - 2026-06-18: cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture (passed 1 test after shader/control-flow and compact-bind usage fixes)
  - 2026-06-18: cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture (passed 4 focused owner tests, including transparent draw recording and offscreen RGBA8 visual readback)
  - 2026-06-18: cargo test -p zircon_plugin_particles_runtime render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture (passed 1 focused CPU/GPU count parity test after map-aligned readback and spawned-counter fixes)
  - 2026-06-18: cargo test -p zircon_runtime --lib render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618 --message-format short --color never -- --nocapture (passed 1 focused scene neutral GPU-frame auto-collection test)
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
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\particles\runtime\Cargo.toml particles_plugin_registration_contributes_runtime_module_render_feature_and_component --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-particles-runtime-metadata --color never --quiet (red before linked status metadata, then passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\Cargo.toml -p zircon_runtime --lib particles_plugin_toml_matches_catalog_optional_feature_metadata --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-particles-runtime-metadata --color never --quiet (red before static category/catalog feature parity, then passed with existing runtime warnings)
  - 2026-06-18: rustfmt --edition 2021 <scoped particles runtime-prepare files> passed; cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-runtime-prepare-0618 --message-format short --color never first exposed DTO drift in ParticleExtract.previous_sprites and RenderParticleSpriteSnapshot fields, then passed after CPU extract DTO sync with the existing zircon_runtime warning set.
doc_type: module-detail
---

# Particles Runtime Plugin

## Purpose

The particles plugin owns particle asset descriptions, scene-facing component descriptors, runtime simulation state, editor authoring descriptors, and the plugin render feature descriptor for sprite particles. `zircon_runtime` remains the neutral host: it owns render DTOs, RHI, render graph execution, and the built-in particle renderer, but it does not own concrete particle simulation.

The first implemented backend is CPU sprite simulation. GPU simulation now has the same asset model compiled into a concrete storage layout, WGSL compute program, frame spawn schedule, render graph pass order, renderer-owned staged graph executor surface, and neutral counter/indirect readback DTO. `ParticlesManager` still falls back to CPU when it is used without a renderer-attached GPU executor.

## Behavior Model

- `ParticleSystemAsset` contains one or more `ParticleEmitterAsset` records. Each emitter defines capacity, spawn rate, bursts, lifetime, spawn shape, initial velocity, gravity, drag, material and texture handles, optional physics options, optional animation bindings, color over lifetime, size over lifetime, and local/world coordinate space.
- `ParticleSystemComponent` binds an asset to an entity, transform, play state, and time scale. The plugin registers `particles.Component.ParticleSystem` as a dynamic component type.
- `ParticlesManager` instantiates components into stable `ParticleEmitterHandle` values, then controls play, pause, stop, explicit tick, preview rewind, state snapshots, neutral `ParticleExtract` generation, and concrete GPU runtime instance snapshots for the plugin-owned runtime-prepare path.
- Package metadata now classifies Particles consistently as runtime, experimental, and partial across `zircon_plugins/particles/plugin.toml`, the linked runtime descriptor, and `RuntimePluginDescriptor::builtin_catalog()`. The same three owner optional feature rows are exposed everywhere: `particles.physics`, `particles.animation_control`, and `particles.gpu_simulation`; this is manifest/dependency gating only and does not promote advanced VFX parity.
- CPU simulation stores particle channels in a structure-of-arrays pool with an explicit free list. The GPU layout uses the same channel names as SoA buffer sections: alive, age, lifetime, position, previous position, velocity, size, initial size, color, start color, rotation, angular velocity, seed, and emitter index.
- `build_particle_extract` turns runtime snapshots into neutral render DTOs and can sort sprites back-to-front when the caller provides a camera position. The extract also carries per-entity particle bounds and the camera position used for sorting, so renderer and editor consumers can reason about culling/debug metadata without depending on the particles plugin.
- CPU sprite snapshots preserve per-emitter material handles, texture handles, and particle rotation. The built-in runtime particle billboard builder applies rotation around the camera-facing right/up basis while retaining the existing alpha-preserving transparent path.
- Optional physics is capability-gated by `runtime.feature.particles.physics`. Without that capability, physics modules produce diagnostics and run as no-op; with it, CPU simulation applies external force and collision damping settings from the emitter options. Enabling the capability after instantiation propagates to existing CPU particle instances.
- Optional animation control is capability-gated by `runtime.feature.particles.animation_control`. Without that capability, events and bindings produce diagnostics and do not mutate particles; with it, spawn-once and timed emission begin/end events can target an emitter handle or resolve by entity.
- `ParticleGpuFramePlanner` accumulates burst and spawn-rate requests for GPU assets. It produces one emitter parameter block per frame, including capacity ranges, module constants, shape parameters, transform rows, color and size endpoints, per-emitter timing, and the per-emitter spawn count.
- `ParticleExtract.gpu_frame` carries the neutral renderer-facing GPU frame summary for GPU-backed emitters: alive count, spawned total, per-emitter spawned counts, and non-indexed indirect draw args. It can come from the particles plugin's runtime manager path or from scene-authored dynamic particle payloads projected by `World::collect_render_particles(...)`. This is runtime-framework DTO data, not a particles concrete type.
- `ParticleGpuBackend` owns double-buffered particle storage, emitter params, atomic counters, alive index compaction, indirect draw args, and a debug/readback buffer. Its frame order is spawn/update compute, compact-alive compute, build-indirect-args compute. `ParticleGpuBuffers` exposes all of those WGPU buffers as read-only handles, including emitter params and debug readback; its `particles_a` and `particles_b` fields are graph-facing input/output aliases over the backend ping-pong pair for the last executed frame. Compact bind groups keep read-only current storage and writable next storage on distinct WGPU buffers so validation does not see conflicting storage usages inside one dispatch.
- `ParticleGpuRuntimeOwner` is the plugin-side WGPU owner shared by runtime prepare and the transparent graph executor. It keeps planner timing state per `ParticleEmitterHandle`, synthesizes one aggregate GPU asset/backend from all playing GPU instances exposed by the shared `ParticlesManager`, executes `ParticleGpuBackend::execute_frame(...)`, exposes graph-facing backend buffer aliases for materialization, and records the transparent render pass from the last executed backend when the runtime graph reaches `particle.transparent`. The aggregate backend is rebuilt only when the playing GPU emitter set changes; per-instance planners still own their own burst/spawn-rate timing before their emitter params are remapped into the aggregate layout.
- The runtime plugin now owns one shared `ParticlesManager` and registers `particles.runtime-prepare` with that same manager used by the resolved runtime module service. When concrete GPU instances exist, the collector executes `ParticleGpuRuntimeOwner`, registers real backend WGPU buffers for the declared `particles.gpu.*` external set, and returns neutral `RenderPluginRendererOutputs.particles`. When no shared-manager GPU instance exists, or when the stateless collector registration is used directly by tests/tools, the neutral `ParticleExtract.gpu_frame` buffer producer remains the fallback.
- `particle_render_pass_executor_registrations` exposes four normal graph executors: `particle.gpu.spawn-update`, `particle.gpu.compact-alive`, `particle.gpu.indirect-args`, and `particle.transparent`. The descriptor targets the runtime `Transparent3d` graph stage, and the runtime registry no longer supplies a descriptor-created noop for particle executor ids, so linked particles must contribute explicit executor registrations. The transparent executor validates its resource contract, then asks `RenderPassGpuExecutionContext::record_particle_gpu_transparent_to_resources("scene-color", "scene-depth", ...)` for the active color/depth views, scene bind group/layout, target formats, queue, encoder, and camera billboard basis. It records `ParticleGpuRuntimeOwner::record_transparent_render(...)` against the shared runtime-prepare owner when a backend executed this frame, and falls back to `record_particle_billboards_to_resources(...)` for CPU sprites or frames without a concrete backend.
- `ParticleGpuCounterReadback` decodes the debug/readback counter words and projects them into neutral `RenderParticleGpuReadbackOutputs`, including alive count, spawned total, debug flags, per-emitter spawned counts, and indirect draw args. The backend readback helper maps aligned WGPU ranges and then slices back to the requested word window, so packed counter payloads can be followed by indirect args even when the counter byte length is not itself map-aligned. `SceneRendererAdvancedPluginOutputs` stores this payload in the shared plugin renderer output mailbox and can take the particle slot without clearing VG/HGI slots.
- `ParticleRuntimeFeedback` and `ParticleGpuFeedback` are neutral runtime feedback carriers in `zircon_runtime`. Runtime submission now drains the renderer particle mailbox and merges prepared sideband particle outputs into this feedback packet, updates particle GPU feedback stats, and leaves concrete state application to the particles plugin manager or host. `ParticlesManager::apply_gpu_feedback` stores the last non-empty neutral particle readback packet for diagnostics/parity without mutating the CPU simulation snapshot; empty feedback means no new GPU packet and does not erase the prior diagnostic packet.
- Runtime 15 F12 `runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked` removed the stale dead-code suppression from `take_last_particle_gpu_readback_outputs.rs`. The accessor is production-live because `collect_runtime_feedback.rs::collect_particle_feedback(...)` drains `renderer.take_last_particle_gpu_readback_outputs()`, merges it with prepared sideband particle readback outputs, and creates `ParticleGpuFeedback` for non-empty merged payloads.
- Runtime 15 F12 advanced plugin output test accessor cleanup (`runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked`) removed stale dead-code suppressions from `advanced_plugin_outputs/output_access.rs`. `has_virtual_geometry_gpu_readback(...)`, `plugin_renderer_outputs(...)`, and `has_particle_gpu_readback(...)` are now test-only observation helpers, while production plugin-output draining remains on the `take_*_readback_outputs(...)` methods.
- The editor authoring surface is registered from `zircon_plugins/particles/editor/src/authoring.rs`. It contributes the `particles.authoring` and `particles.preview` views, concrete `.ui.toml` templates for authoring/preview/component drawer surfaces, a `ParticleSystemComponent` drawer, a `particles.system` asset editor, a CPU sprite particle-system creation template, and descriptor-level operations for create, add component, open, add emitter, add module, edit curve, validate, preview play, pause, stop, rewind, and warmup.
- The CPU sprite creation template points at `zircon_plugins/particles/templates/cpu_sprite_system.toml`, a starter TOML document for a local-space CPU sprite emitter. Until concrete editor operation handlers are added, non-view particles authoring menu rows are registered disabled and the corresponding operations are not callable from remote/CLI. This keeps schema/template discovery visible without presenting an enabled click path that would fail with an unhandled operation.

The runtime package options now satisfy the shared manifest validator directly. `particles.backend` is an enum with `cpu` and `gpu` values and a `cpu` default; `particles.fixed_preview_dt` is a finite `number` option instead of the older non-standard `scalar` spelling. Capability-gated options still use boolean defaults and remain gated by their physics or animation capability rows.

## Data Flow

1. Runtime plugin registration installs the embedded `particles.runtime` descriptor, then contributes the particle component descriptor, particle options, dynamic event catalog, optional physics/animation/GPU feature manifests, particle render feature, and a runtime-prepare collector built from the same shared `ParticlesManager` exposed by the module service.
2. Editor plugin registration contributes capability-gated authoring descriptors. The root `lib.rs` delegates to the `authoring` module so the crate entry remains structural while the descriptor batch owns authoring operations and asset-template wiring.
3. A host or editor tool creates a `ParticleSystemComponent` and calls `ParticlesManager::instantiate`.
4. `ParticlesManager::tick` advances all playing instances. Spawn rate and burst emission are accumulated per emitter; random sampling uses a deterministic local RNG seeded from the asset seed and handle.
5. Optional animation events enter through `ParticlesManager::apply_animation_event`. The manager verifies the animation-control capability before applying spawn-once or timed emission state changes.
6. `ParticlesManager::snapshot` reports emitter state, live sprites, and diagnostics. `ParticlesManager::build_extract` converts that snapshot into `ParticleExtract` for the render framework, including sorted sprites, bounds, material/texture handles, and the sort camera metadata.
7. The runtime particle renderer builds rotated billboards from `ParticleExtract.sprites`; the shader now preserves vertex alpha and the pipeline uses standard transparent blending.
8. GPU-capable render hosts can compile `ParticleGpuProgram`, use `ParticleGpuFramePlanner` for per-frame spawn requests, and call `ParticleGpuBackend::execute_frame` with their `wgpu::Queue` and command encoder. The generated indirect args buffer uses non-indexed draw layout: vertex count 6, instance count equal to compacted alive particles, first vertex 0, first instance 0. The runtime plugin path now drives this through `ParticleGpuRuntimeOwner` during runtime prepare when the shared manager contains concrete GPU instances; multiple playing GPU systems are collapsed into one aggregate backend so the render graph still receives one authoritative `particles.gpu.*` binding set.
9. The renderer-owned compiled graph path executes particle pass descriptors at the `Transparent` stage while the scene command encoder is live. Shared runtime imports `scene-color` and `scene-depth` into `RenderGraphExecutionResources`; particle executors refer to those names and particle buffer names through graph resource contracts rather than direct runtime/plugin coupling. The GPU transparent path uses the same scene bind group layout, target format, depth format, command encoder, and queue as the renderer-owned graph context, so the particles plugin can lazily build its transparent pipeline without depending on runtime concrete renderer types.
10. When counter readback is requested, `ParticleGpuBackend::read_render_outputs_readback` decodes counter words plus indirect draw args into `RenderParticleGpuReadbackOutputs`. During staged graph execution, `particle.gpu.indirect-args` can also project `ParticleExtract.gpu_frame` into the neutral `RenderPluginRendererOutputs.particles` sink. The scene-renderer advanced plugin output mailbox can store that neutral payload and expose it through `take_last_particle_gpu_readback_outputs` while leaving VG/HGI readbacks intact.
11. During runtime prepare, `particles.runtime-prepare` first queries the shared manager for concrete GPU instances. If present, it executes the plugin-owned `ParticleGpuRuntimeOwner`, registers the backend's active `particles.gpu.*` buffers as runtime-prepare external backings, and lets the plugin graph binder consume them before fallback buffer synthesis. The `particle.transparent` executor receives a clone of the same owner handle, so it can consume the just-executed `particles-b`, alive-index, and indirect-args buffers for the transparent draw. If no concrete GPU instance exists but `ParticleExtract.gpu_frame` is present, the collector creates the neutral summary-derived buffer set instead. If neither source exists, the collector is a no-op and the renderer-side plugin fallback binder remains the materialization safety net.
12. Runtime submission collects particle GPU readback from both renderer mailbox output and prepared runtime sideband output. Renderer-produced readback is treated as authority when both exist; otherwise the sideband packet is used. Non-empty readback becomes `ParticleRuntimeFeedback`, contributes `RenderStats` particle GPU counters, and can be handed to `ParticlesManager::apply_gpu_feedback` by a host/plugin runtime owner. The manager records only non-empty packets, so frames without particle readback do not clear the last diagnostic/parity packet.

## Design and Rationale

The module follows the current plugin architecture rather than adding particle-specific runtime ownership. The asset and CPU simulation live in `zircon_plugins/particles`, while render DTOs remain neutral. This matches the independent-plugin plan and leaves room for GPU simulation without forcing `zircon_runtime` to depend on plugin concrete types.

The CPU pool stores `alive`, `age`, `lifetime`, `position`, `previous_position`, `velocity`, `size`, `color`, `rotation`, `angular_velocity`, `seed`, and `emitter_index` as separate arrays. The GPU path extends that channel list with `initial_size` and `start_color` because color/size-over-life evaluation needs stable spawn-time values.

Reference evidence used for the GPU direction:

- Unreal Niagara: `NiagaraComputeExecutionContext.h`, `NiagaraDataSet.h`, `NiagaraDataInterface.h`, and `NiagaraDataInterfaceRW.h` show GPU compute dispatch, double-buffered data buffers, simulation stage hooks, and indirect dispatch argument generation.
- Unity VFX Graph: `VFXDataParticle.cs`, `VFXGraphCompiledData.cs`, and `VFXCodeGenerator.cs` show attribute layout compilation, indirect buffer allocation, generated compute shader code, active indirection, and compute bounds handling.
- Bevy render: `bevy_render/src/batching/gpu_preprocessing.rs` shows Rust/wgpu-oriented indirect parameter buffers, compute preprocessing, CPU metadata, and debug-copy settings.

The intentional Zircon divergence is that particle assets stay in `zircon_plugins/particles` and the GPU executor is an explicit renderer-owned object. The shared runtime graph sees only external resource names and pass descriptors; it does not gain a plugin-specific dependency.

Particles M6 uses the same neutral renderer graph surface as other advanced plugins. `zircon_runtime` owns stage metadata, `RenderPassExecutionContext`, `RenderGraphExecutionResources`, `RenderGraphExecutionRecord`, and the renderer command-encoder lifetime. The particles plugin owns only descriptor contracts, executor objects, GPU layout/program/readback logic, transparent render pipeline creation, and particle DTO projection. The transparent graph slice tries the shared plugin-owned GPU backend first and keeps the CPU billboard path as a deterministic fallback when no backend was executed for the frame.

The feedback continuation keeps the same ownership split. `zircon_runtime` does not own particle simulation and does not create a particle provider registry in this slice; it only exposes neutral `ParticleGpuFeedback` / `ParticleRuntimeFeedback` packets and records render-framework stats from them. `zircon_plugins/particles` consumes that neutral feedback through `ParticlesManager::apply_gpu_feedback`, which records the last non-empty GPU readback for diagnostics and parity tooling without changing CPU fallback particle state.

## Edge Cases and Constraints

- Delta time must be finite and non-negative.
- Assets must contain at least one emitter.
- Non-finite scalar, vector, shape, physics, color, curve, burst-time, and animation-binding settings are rejected at instantiate time.
- Particle bounds are generated from sprite size and position after CPU extraction. Empty extracts carry no bounds.
- `Gpu` backend requests through `ParticlesManager` remain CPU-compatible for headless/editor snapshots, but the runtime plugin can now execute concrete GPU instances during runtime prepare when a WGPU device and encoder are available.
- The current particles runtime-prepare collector supports one authoritative GPU system binding set per frame. It can bind real backend-owned `particles.gpu.*` WGPU buffers from `ParticleGpuRuntimeOwner`, or fall back to neutral GPU-frame-derived buffers when only `ParticleExtract.gpu_frame` exists. Scene-authored dynamic particle payloads can now feed that neutral fallback by writing a visible `gpu_frame` object into `render.particle_sprites` or `gameplay.particle_sprites`; this does not move concrete simulation ownership into `zircon_runtime`. For real backend buffers, `particles.gpu.particles-a` is the previous/input alias and `particles.gpu.particles-b` is the current/output alias over the active ping-pong pair. The transparent draw path consumes that active backend through the shared owner handle; focused offscreen readback now confirms the indirect draw writes visible pixels, while product-scene image parity and RenderDoc resource/marker confirmation remain follow-ups.
- GPU capacity is clamped to `PARTICLE_GPU_MAX_PARTICLES`. Per-emitter capacities are assigned in emitter order, so overflow emitters receive zero capacity and a compile diagnostic records the clamp.
- GPU v1 evaluates color and size curves as first-to-last linear endpoints. Assets with more than two keys compile with a warning diagnostic instead of silently pretending to have full curve parity.
- Physics and animation support are capability-gated helper surfaces only in this milestone; they do not create hard dependencies on the physics or animation plugins. Their unavailable paths are explicit diagnostics rather than silent behavior changes.
- A neutral particle GPU readback payload is considered present only when count/debug/per-emitter/indirect fields are non-default. Taking the particle payload drains only the particles slot and keeps other plugin renderer outputs available to their own consumers.
- Empty/default particle runtime feedback is treated as no new packet instead of a reset signal. Hosts that need explicit diagnostic clearing must add a separate lifecycle decision rather than relying on absent GPU readback for clearing.
- Staged graph execution currently validates named particle resources and emits neutral readback DTOs; it does not yet issue the full concrete particle compute/draw dispatch from `RenderGraphExecutionResources` alone. Missing `scene-color` or `scene-depth` in the renderer resource registry is reported as a graph resource binding error.

## Test Coverage

`zircon_plugins/particles/runtime/src/tests/mod.rs` is a structural test entry point. Its child modules cover plugin registration, runtime-prepare collector registration, manager resolution, CPU spawn rate, deterministic seed behavior, lifetime death, free-list reuse, pause/stop/preview rewind, extract sorting, material/texture/rotation extraction, bounds and sort metadata, stable sprite keys, previous-sprite DTO initialization, non-finite asset rejection including burst times and animation binding progress, capability-gated physics diagnostics, late physics capability propagation, external force application, capability-gated animation diagnostics and event control, GPU layout/fallback, GPU pass order, WGSL parse coverage, GPU frame spawn planning, neutral GPU frame extract projection, particle graph executor resource-contract validation, capacity clamp diagnostics, neutral GPU feedback recording, and optional physics/animation helper behavior. Shared test helpers live in `tests/support.rs`, while the root test module stays navigational only.

Inline tests in `zircon_plugins/particles/runtime/src/render/runtime_prepare.rs` cover the neutral runtime-prepare buffer sizing for GPU readback payloads, minimum non-zero buffer allocation for empty neutral frames, and the stable `particles.runtime-prepare` collector id. `manager_resolution.rs::particles_runtime_plugin_module_and_runtime_prepare_share_manager` covers the shared manager wiring between plugin registration, module service resolution, and runtime-prepare ownership. The focused GPU owner tests in `tests/gpu.rs` cover concrete WGPU execution through `ParticleGpuRuntimeOwner`, active backend buffer exposure, multi-system aggregation, empty-backend transparent skip behavior, and transparent draw recording from an executed backend with real offscreen color/depth attachments. The executed-backend transparent test also reads back the `Rgba8Unorm` target and asserts that the indirect draw produced non-transparent RGB pixels. `render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args` compares a burst-only CPU fallback extract against concrete GPU counter readback, then verifies indirect args word1 matches the CPU live count. `zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers` covers the scene-authored neutral GPU-frame auto-collection path that feeds the runtime-prepare fallback buffers.

`zircon_plugins/particles/editor/src/tests.rs` covers editor views, templates, asset editor registration, CPU sprite asset creation template registration, disabled descriptor-level authoring operations, preview operations, capability gating, the particle system component drawer, compile-time include guards for the starter particle asset template, and compile-time include guards for the concrete authoring, preview, and component drawer `.ui.toml` documents.

`zircon_runtime/src/graphics/tests/m4_behavior_layers.rs` keeps a render-side guard that the particle shader preserves alpha instead of forcing opaque output.

Inline runtime tests in `zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs`, `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_storage.rs`, and `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/collect_into_outputs.rs` cover neutral particle readback empty detection, mailbox storage, collection, and particle-only take semantics. Runtime graph tests in `compiled_render_pipeline.rs`, `render_pipeline_asset/compile.rs`, `render_graph_execution_record.rs`, `render_graph_execution_resources.rs`, `render_pass_execution_context.rs`, and `render_pass_executor_registry.rs` cover stage preservation, named resource errors, GPU-context requirement errors, staged execution record counts, and object-backed executor invocation.

Runtime submission feedback tests in `prepared_runtime_submission.rs`, `submit/collect_runtime_feedback.rs`, and `submit/submit_runtime_frame.rs` cover particle sideband preservation, particle readback merge authority, renderer-vs-sideband selection, and prepared sideband projection on the direct runtime-frame submit path. Particles runtime tests cover `ParticlesManager::apply_gpu_feedback` storing neutral readback while preserving CPU snapshot state and preserving the prior non-empty packet across empty feedback.

2026-05-04 scoped validation for the M6 graph refactor used `target\codex-shared-a` with `--locked --offline`. Runtime test targets compiled, runtime graph execution tests passed 16/16 filtered tests, the pipeline stage-preservation regression passed, particles runtime passed 20/20 tests, and particles editor passed 1/1 test. These are scoped gates for the particles/render-graph lane; full workspace validation was not run from this dirty checkout.

2026-05-04 scoped validation for the particle feedback continuation used `target\codex-shared-a` with `--locked --offline` after an offline `zircon_plugins/Cargo.lock` refresh added the dependency edges Cargo required. Runtime and particle test targets compiled, targeted particle feedback merge/sideband/direct-submit tests passed, the manager feedback ingest regression passed, scoped `rustfmt --check` passed, and scoped `git diff --check` found no whitespace errors beyond LF-to-CRLF warnings. Full workspace validation was still not run from this dirty checkout.

2026-05-31 metadata parity validation used `D:\cargo-targets\zircon-particles-runtime-metadata` with `--locked --offline`. The linked particles registration test first failed because `runtime.plugin.particles` lacked a partial status row in the linked package manifest, then passed after the descriptor gained explicit runtime/experimental/partial metadata. The static runtime manifest test first failed because `zircon_plugins/particles/plugin.toml` defaulted to `uncategorized`, then passed after static TOML and the built-in catalog exposed the same three owner optional features and dependency rows. Existing runtime warnings were left unchanged.

2026-06-01 M6 manifest validation reran `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime --locked --jobs 1 --message-format short --color never` after the full plugin workspace gate rejected option rows that used enum defaults without `enum_values` and the non-standard `scalar` type. The focused particles runtime command passed with 21 tests and 0 failures after `particles.backend` declared `cpu`/`gpu` enum values and `particles.fixed_preview_dt` moved to `number`.

2026-06-18 RG-M1 particle GPU owner validation used `D:\cargo-targets\zircon-particles-gpu-owner-0618`. `cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed with the existing `zircon_runtime` warning set. `cargo test -p zircon_plugin_particles_runtime particles_runtime_plugin_module_and_runtime_prepare_share_manager --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 1 focused test. `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` first exposed a WGSL return-path validation issue in `find_emitter` and a WGPU compact-pass storage usage conflict, then passed after those lower-level backend fixes.

2026-06-18 RG-M1 particle transparent GPU draw validation reused `D:\cargo-targets\zircon-particles-gpu-owner-0618`. `cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed with the existing `zircon_runtime` warning set. `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 4 focused owner tests, including `particle_gpu_runtime_owner_records_transparent_draw_from_executed_backend` and `particle_gpu_runtime_owner_skips_transparent_draw_without_executed_backend`. `cargo test -p zircon_runtime render_pass_executor_registry --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-render-context-0618 --message-format short --color never -- --nocapture` passed 41 focused runtime graph-executor tests after fixing lower-level registry test fixtures that no longer declared cull roots.

2026-06-18 RG-M1 particle transparent offscreen visual readback reused `D:\cargo-targets\zircon-particles-gpu-owner-0618`. `rustfmt --edition 2021 zircon_plugins/particles/runtime/src/tests/gpu.rs` passed. `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 4 focused owner tests with the existing `zircon_runtime` warning set; the executed-backend transparent test now submits the transparent pass, copies the 32x32 `Rgba8Unorm` target to a mapped buffer, and asserts at least one visible non-transparent pixel.

2026-06-18 RG-M1 particle CPU/GPU count parity validation reused `D:\cargo-targets\zircon-particles-gpu-owner-0618`. `render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args` first failed in `Buffer::map_async` because the indirect-args readback attempted to map byte offset 20, which is not a multiple of wgpu's map alignment. `read_buffer_u32s_at(...)` now maps an aligned range and slices out the requested word window after mapping. The next run exposed a real counter semantic mismatch: per-emitter readback reported claim count `[16]` while spawned total and CPU expected `[5]`; `particle_build_indirect_args` now normalizes each per-emitter spawned counter to `min(claimed, emitter.spawn_count)` and rewrites spawned total from those normalized rows. After these support fixes, `cargo test -p zircon_plugin_particles_runtime render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 1 focused parity test, and `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` still passed 4 focused owner tests.

2026-06-18 scene-authored neutral particle GPU-frame validation used `D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618`. `cargo check -q -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618` passed with the existing warning set. `cargo test -p zircon_runtime --lib render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618 --message-format short --color never -- --nocapture` passed 1 focused scene test, proving visible dynamic particle `gpu_frame` payloads aggregate into `ParticleExtract.gpu_frame` and hidden-layer payloads are excluded.

## Open Issues

- GPU transparent rendering now consumes the shared plugin-owned backend when runtime prepare executed concrete GPU instances for the frame, and the focused WGPU owner lane confirms the indirect transparent pass writes visible offscreen pixels. Remaining work is product-scene parity and RenderDoc confirmation; CPU billboard fallback still handles frames without an executed backend.
- Full CPU/GPU parity for multi-key curves, material/texture metadata, bounds, rotation, angular velocity, and image comparison is not complete. CPU extraction now carries those fields; GPU v1 covers spawn/update, gravity, drag, first-to-last color/size interpolation, alive compaction, indirect args, neutral counter/indirect readback projection, and a focused burst-only CPU/GPU count parity check.
- A neutral particles runtime feedback packet now exists and runtime submission routes particle GPU readbacks into stats/feedback. The remaining feedback gap is end-to-end provider lifecycle integration: the runtime plugin can execute its shared-manager GPU backend during runtime prepare, aggregate playing GPU systems into one graph binding set, render transparent GPU particles from that backend, and consume scene-authored neutral `gpu_frame` payloads, but product-level feedback parity is not complete.
- Runtime scene auto-collection is wired for neutral dynamic `gpu_frame` payloads in `render.particle_sprites` / `gameplay.particle_sprites`. Full scene integration for concrete `ParticleSystemComponent` values is still pending: hosts can instantiate those values through the manager, and future scene integration should collect dynamic components into the manager without moving simulation into `zircon_runtime`.
