---
related_code:
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs
implementation_files:
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs::tests::collector_context_registers_external_buffer_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs::tests::runtime_prepare_collectors_can_register_external_buffer_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs::tests::advanced_plugin_readbacks_hold_runtime_prepare_external_buffer_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs::tests::plugin_external_fallback_buffers_satisfy_materialization_report
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs::tests::plugin_external_binder_skips_unknown_and_untyped_externals
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs::tests::plugin_external_binder_prefers_runtime_prepare_buffers_over_fallback
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs::tests::plugin_external_binder_accepts_registered_non_fallback_plugin_names
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_neutral_frame_sizes_cover_readback_payload
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_neutral_frame_uses_minimum_nonzero_buffers
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_registration_id_is_stable
  - zircon_plugins/particles/runtime/src/tests/manager_resolution.rs::particles_runtime_plugin_module_and_runtime_prepare_share_manager
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::particle_gpu_runtime_owner_records_transparent_draw_from_executed_backend
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::particle_gpu_runtime_owner_skips_transparent_draw_without_executed_backend
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args
  - zircon_plugins/particles/runtime/src/tests/registration.rs::particles_plugin_registration_contributes_runtime_module_render_feature_and_component
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs::tests::virtual_geometry_feedback_binding_names_stay_stable
  - cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-runtime-prepare-0618 --message-format short --color never
  - cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-plugin-real-buffer-handoff-0618 --message-format short --color never
doc_type: module-detail
---

# Plugin Graph Resources

`bind_plugin_graph_resources.rs` owns the production binding model for plugin-owned, non-frame external buffer resources in the compiled-scene render path. It runs after transient materialization and HZB execution-owned binding, before `RenderGraphExecutionResources::validate_materialized_graph_resources(...)`.

The binder is graph-lifetime-aware. It only considers live `CompiledRenderGraph::resource_lifetime_by_name(...)` rows whose external binding is typed as `Buffer`, then binds those names into the same execution resource table used by frame, history, HZB, and transient resources. Runtime prepare collectors can now register actual WGPU buffers through `RuntimePrepareCollectorContext::register_external_buffer_binding(...)`; those bindings are carried by `SceneRendererAdvancedPluginReadbacks` and are bound before fallback buffers are considered.

## Bound Resources

The current binding set covers:

- Particle GPU simulation buffers: `particles.gpu.particles-a`, `particles.gpu.emitter-params`, `particles.gpu.particles-b`, `particles.gpu.counters`, `particles.gpu.alive-indices`, `particles.gpu.indirect-draw-args`, and `particles.gpu.debug-readback`.
- Virtual Geometry feedback: `virtual-geometry-feedback`.

When a runtime prepare collector registers a buffer for any live typed plugin external, the binder records that runtime-prepare backing and does not allocate a fallback for the same logical name. If no registered backing exists, the current first-party names above still receive deterministic minimum fallback buffers and logical-to-physical aliases with the `:plugin-external-fallback` suffix. This keeps typed report-only plugin externals visible to materialization validation and alias reports while first-party plugin backends are incrementally taught to register their real buffers.

Virtual Geometry now has a producer-side registration. Its runtime-prepare collector mirrors the neutral prepared NodeAndClusterCull page-request sideband and, when page requests are present, creates a per-frame WGPU storage buffer registered as `virtual-geometry-feedback` with the `virtual-geometry-feedback:runtime-prepare-page-requests` backing name. The later `virtual-geometry.page-feedback` graph executor is still a contract validator rather than a buffer-writing compute/copy implementation, so this registration represents the current prepared feedback payload, not full graph-pass feedback population.

Particles now have a plugin-side concrete buffer producer and consumer path. The particles runtime plugin registers `particles.runtime-prepare` with the same shared `ParticlesManager` exposed by its module service. When that manager contains concrete GPU instances, the collector executes `ParticleGpuRuntimeOwner`, aggregates all playing GPU instances into one real `ParticleGpuBackend`, and registers backend WGPU buffers for the declared `particles.gpu.*` names before this binder considers fallbacks. The particle storage names are graph-facing semantic aliases: `particles.gpu.particles-a` binds the previous/input side of the backend ping-pong pair for the last executed frame, while `particles.gpu.particles-b` binds the current/output side consumed by compact/indirect and transparent descriptors. The object-backed `particle.transparent` executor receives a clone of the same owner handle and records the backend transparent draw from `particles-b`, alive indices, and indirect args when runtime prepare executed a concrete backend for the frame. Focused offscreen validation now confirms that draw writes visible RGBA8 pixels after the indirect pass is submitted, and focused CPU/GPU count parity validation confirms the small-scene GPU counter readback agrees with CPU fallback live/spawned counts and indirect instance count. If no concrete GPU instance exists but `ParticleExtract.gpu_frame` is available, the collector still creates neutral summary-derived buffers for the same graph names. That neutral frame can now be produced by scene extraction from visible dynamic particle payloads containing a `gpu_frame` object, keeping `World` free of WGPU and particles-plugin dependencies while still feeding the graph-facing buffer set. Frames without either source keep the deterministic fallback backings and the transparent executor falls back to CPU billboards.

## Boundaries

This module does not change plugin descriptors from report-only to required. Missing or unknown plugin externals remain report-only diagnostics unless a descriptor explicitly opts into a required external binding. A registered runtime-prepare binding may bind a non-fallback plugin name when that name is a live typed external buffer; unknown names without a registered backing are still not synthesized. The binder also does not bind plugin external textures; Hybrid GI's `history-global-illumination` alias is a history-owned texture and is handled by `bind_history_graph_resources.rs`.

The producer-to-consumer handoff is now connected for the particles plugin-owned backend path and the scene-authored neutral GPU-frame path. Remaining evidence gaps are product-scene parity, RenderDoc resource/marker confirmation, wider plugin package validation after the plugin lockfile drift is resolved, and concrete scene-to-manager `ParticleSystemComponent` integration.

## Validation State

Source-contract tests build small graphs with the current first-party plugin buffer names and assert that the binder closes the materialization report and emits buffer alias rows. Additional tests cover runtime-prepare registration, registered buffers taking priority over fallbacks, and registered non-fallback plugin names binding only when the compiled graph declares them as typed external buffers.

`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-plugin-real-buffer-handoff-0618 --message-format short --color never` passed on 2026-06-18 with the existing 141-warning set. A focused lib-test command for `runtime_prepare_collectors_can_register_external_buffer_bindings` was attempted after the new tests were added, but the `zircon_runtime` lib-test crate still fails before running filtered tests because of unrelated root-surface/test-crate drift: root `crate::BuiltinRenderFeature`/`RenderPassStage` style imports are missing in multiple graphics tests, `RenderPhaseSortComponents` is missing in mesh-pass tests, and one compile-test path reaches a private `scene_renderer` module.

`cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-runtime-prepare-0618 --message-format short --color never` passed on 2026-06-18 after the particles runtime-prepare slice synced CPU extract DTO construction with `ParticleExtract.previous_sprites` and the current `RenderParticleSpriteSnapshot` stable-key/aspect/billboard/depth fields. The first run reached the particles crate and exposed only that DTO drift after the existing `zircon_runtime` warning set.

`cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed on 2026-06-18 with the existing `zircon_runtime` warning set. Focused tests `particles_runtime_plugin_module_and_runtime_prepare_share_manager` and `particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers` also passed in the same target dir; the GPU owner test first exposed a WGSL return-path validation issue and a compact-pass WGPU storage usage conflict, then passed after the shader/backend fixes.

The same `zircon-particles-gpu-owner-0618` target dir was reused on 2026-06-18 for the ping-pong graph-binding alias slice. `cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed, and `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed with the focused assertion that `particles_a` and `particles_b` are distinct WGPU buffer handles.

The same target dir was reused again on 2026-06-18 for the multi-system aggregation slice. `cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed after fixing the owner refactor type errors, and `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed with 2 focused GPU owner tests, including `particle_gpu_runtime_owner_aggregates_playing_gpu_instances`.

The same target dir was reused again on 2026-06-18 for the transparent GPU draw consumption slice. `cargo check -p zircon_plugin_particles_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never` passed with the existing `zircon_runtime` warning set. `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 4 focused owner tests, including `particle_gpu_runtime_owner_records_transparent_draw_from_executed_backend` and `particle_gpu_runtime_owner_skips_transparent_draw_without_executed_backend`. `cargo test -p zircon_runtime render_pass_executor_registry --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-render-context-0618 --message-format short --color never -- --nocapture` passed 41 focused runtime graph-executor tests; the first runtime attempt exposed cull-root fixture drift and deferred-lighting missing test resources, which were repaired at the shared fixture layer before rerunning.

The same target dir was reused again on 2026-06-18 for the transparent GPU draw offscreen visual readback slice. `rustfmt --edition 2021 zircon_plugins/particles/runtime/src/tests/gpu.rs` passed. `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` passed 4 focused owner tests with the existing `zircon_runtime` warning set; the executed-backend transparent test now copies the 32x32 `Rgba8Unorm` color target to a mapped buffer and asserts non-transparent RGB output after the indirect draw.

The same target dir was reused again on 2026-06-18 for focused particle CPU/GPU count parity. `cargo test -p zircon_plugin_particles_runtime render_particle_cpu_gpu_parity_small_scene_matches_counts_and_indirect_args --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` first exposed that the packed counter-readback helper tried to map indirect args from byte offset 20, which violates WGPU's map-alignment rule; `read_buffer_u32s_at(...)` now maps an aligned range and slices the requested word window. The next run exposed that per-emitter counters reported slot claims instead of successful spawns; `particle_build_indirect_args` now normalizes each per-emitter counter to `min(claimed, emitter.spawn_count)` and rewrites spawned total from those normalized rows. The final focused parity test passed 1 test with the existing warning set, and the follow-up `cargo test -p zircon_plugin_particles_runtime particle_gpu_runtime_owner --locked --jobs 1 --target-dir D:\cargo-targets\zircon-particles-gpu-owner-0618 --message-format short --color never -- --nocapture` still passed 4 owner tests.

The scene neutral GPU-frame auto-collection slice used `D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618` on 2026-06-18. `cargo check -q -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618` passed with the existing warning set. `cargo test -p zircon_runtime --lib render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-scene-particle-gpu-frame-0618 --message-format short --color never -- --nocapture` passed 1 focused test, proving layer-visible dynamic `gpu_frame` payloads reach `ParticleExtract.gpu_frame` while hidden payloads do not.
