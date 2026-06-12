---
related_code:
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_motion_vector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_motion_vector.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
  - zircon_runtime/src/graphics/backend/render_backend/request_device.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
plan_sources:
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs::tests::render_gpu_scene_layout_matches_wgsl_offsets
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_reuses_freed_spans_without_aliasing
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_coalesces_adjacent_free_spans
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs::tests::render_gpu_scene_update_queue_merges_adjacent_dirty_ranges
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs::tests::render_gpu_scene_static_scene_second_frame_uploads_zero_bytes
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs::tests::render_gpu_scene_single_moving_entity_uploads_only_its_entry
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_shader_source.rs::tests::shadow_map_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs::tests::prepared_queue_stats_carry_gpu_scene_counts
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_gpu_scene_upload_stats
  - cargo test -p zircon_runtime --lib render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib fallback_mesh_shader --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib shader_declares_gpu_scene_group --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib prepared_queue_stats_carry_gpu_scene_counts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib shader_is_valid_wgsl --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe fallback_mesh_shader --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe reads_gpu_scene_instance_data --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe shader_is_valid_wgsl --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe shader_declares_gpu_scene_group --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe skinned_joint_palette --test-threads=1 --nocapture
  - E:\cargo-targets\zircon-render-main-chain\debug\deps\zircon_runtime-de6f737e1b69a0f9.exe mesh_batch_ref_emits_gpu_scene_instance_command --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_gpu_scene_static_scene_second_frame_uploads_zero_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_gpu_scene_single_moving_entity_uploads_only_its_entry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# GPUScene Data Module

## Purpose

`graphics::scene::gpu_scene` is the data-plane landing point for plan 03. It defines the CPU-side storage-buffer ABI, stable id allocation, dirty-range collection, and the first WGPU storage-buffer owner that later scene-data bind groups and GPU-driven indirect submission will consume.

This module deliberately lives under `zircon_runtime/src/graphics/scene/` rather than `core::framework::render`: the framework layer owns renderer-neutral extract contracts, while GPUScene owns shader-visible data layout and eventually WGPU buffers.

## Related Files

- `mod.rs` mounts the subsystem and re-exports only the small set of types that later renderer code should consume.
- `binding.rs` defines the read-only storage bind group layout for primitive, instance, and light buffers.
- `gpu_scene.rs` owns primitive, instance, and fallback light storage buffers, shared and per-palette scene-data bind groups, CPU shadow vectors, and stable-key entries.
- `layout.rs` defines `GpuPrimitiveData` and `GpuInstanceData` as `#[repr(C, align(16))]` Pod structs with explicit stride and offset constants.
- `id_allocator.rs` defines `GpuSceneIdAllocator`, a first-fit span allocator with deferred free reuse.
- `update_queue.rs` collects primitive and instance dirty ranges and drains them as merged byte ranges for upload.
- `upload.rs` contains the current direct `queue.write_buffer` upload helpers.
- `scene_extract.rs` and `scene/world/render.rs` generate the stable mesh identity and transform revision that GPUScene uses as registration input.
- `SceneRendererCore` owns one `GpuScene`, and both compiled-scene and legacy render paths pass it into mesh draw building.
- `mesh/build_mesh_draws/build.rs` mirrors prepared pending mesh draws into GPUScene storage records before command recording.
- `mesh/build_mesh_draws/create_mesh_draw.rs` no longer builds model uniforms; skinned draws that need real current or previous palettes create command-local GPUScene scene-data bind groups over the same storage buffers.
- `mesh_pipeline_cache/new.rs`, `normal_prepass_pipeline/new.rs`, `shadow_map_renderer.rs`, and `deferred/geometry_pipeline/create.rs` now use the target mesh layout slots: group0 scene, group1 forward shadow receiver where needed, group2 material set, and group3 GPUScene scene data.
- `RenderPassMeshCommandLists` carries the frame's `MeshSceneDataBindHandle`, and prepass/base/shadow/deferred/motion-vector recorders bind it through the mesh command replayer unless a command-local skinned-palette GPUScene bind group overrides it.
- `mesh/shaders/zr_gpu_scene.wgsl` defines the shared WGSL storage ABI and helper functions; the forward fallback, normal prepass, shadow map, and deferred geometry shader source modules prepend it and now read transform, primitive data, and current/previous skinning palettes from GPUScene bindings.
- `prepared_queue.rs`, `backend_types.rs`, `update_stats/base_stats.rs`, and `render_stats_store/product.rs` carry GPUScene counts, upload bytes, upload range counts, the current upload path, and the first GS-M4 indirect batch planning counters into public `RenderStats` and diagnostics.

## Behavior Model

`GpuPrimitiveData` contains bounds, tint/material-derived values, motion/shadow parameters, flags, and the instance span that belongs to the primitive. Its stride is 80 bytes. `GpuInstanceData` contains current and previous transforms plus primitive and payload references. Its stride is 144 bytes.

The layout constants are manual ABI constants, not computed aliases. Unit tests compare them against `std::mem::offset_of!` and `size_of` so a Rust field reorder or padding change fails before WGSL is wired in.

`GpuSceneIdAllocator` allocates single ids and contiguous spans from the same free-list structure. Released spans enter `pending_free_spans` first and only become reusable after `commit_pending_frees()`. That frame-boundary step prevents a newly registered primitive or instance span from reusing an id that an in-flight command buffer can still reference.

`GpuSceneUpdateQueue` accepts dirty primitive ids and dirty instance spans. Draining sorts ranges, coalesces overlaps, and merges gaps of at most eight entries. The result carries both element-space ranges and byte-space ranges so the upload layer can choose direct `queue.write_buffer` or staged copy without redoing range math.

`GpuScene` maps a stable instance key to `GpuSceneEntry { primitive_index, first_instance_index, instance_count, last_transform_revision }`. Registration allocates one primitive id and one contiguous instance span, writes default CPU shadow records, marks both ranges dirty, and grows WGPU buffers by powers of two when high-water ids exceed the current capacity.

`flush_updates()` is the current upload entry point. On initial creation or buffer growth it writes the active high-water prefix as a full upload. Otherwise it drains merged primitive and instance ranges and writes only those byte ranges. `GpuSceneUploadReport` explicitly records `GpuSceneUploadPath::DirectQueueWrite`; this is the V1 policy for persistent GPUScene storage buffers while staging-ring or render-graph upload ownership remains a later optimization rather than an implicit missing step. `write_primitive()` and `write_instances()` compare incoming data against the CPU shadow first, so a full-frame extract can be replayed every frame without re-marking stable primitive or instance entries dirty. Free spans are committed only after the flush, preserving the same-frame no-aliasing invariant from the allocator.

Frame integration now feeds the shader-visible path for built-in mesh shaders. `build_mesh_draws` expands real mesh draws first, then registers each pending draw with a stable key derived from source entity plus draw ordinal. `PendingMeshDraw` carries the transform revision from extract, while GPUScene writes primitive data from the same tint, shadow, motion, and transform inputs used by the existing mesh path and one instance record. Unchanged primitive/instance payloads are skipped at the CPU shadow comparison point, so the second submission of the same static scene returns a zero-byte upload report. The builder retains only live frame keys and flushes dirty GPUScene ranges before command recording. It records the resulting `GpuSceneEntry` span on the production `MeshDraw`, and every `MeshBatchRef` now requires that span before it can emit a command. `DrawInstanceSource` only carries GPUScene instance ranges, so built-in draws always use first-instance draw args plus `@builtin(instance_index)`.

`GpuScene` owns a scene-data bind group layout and a shared frame bind group. Bindings 0, 1, and 2 expose primitive, instance, and light buffers as read-only storage to vertex, fragment, and compute stages. Bindings 3 and 4 expose current and previous skinned joint palette uniforms. The shared bind group fills both palette slots with the renderer's empty skinned-palette fallback buffer, while `create_scene_bind_group_for_palettes(...)` creates command-local bind groups that reuse the same storage buffers and override only the palette buffers for skinned draws. If primitive or instance buffers grow, `ensure_capacity()` recreates the shared bind group after replacing the WGPU buffer so later shader consumption cannot keep a stale buffer handle.

The WGPU pipeline bridge now uses the plan's final physical mesh slots. `SceneRendererCore::new_with_icon_source` creates the empty skinned-palette fallback buffer first, passes it into `GpuScene::new`, then creates mesh pipeline resources so the same GPUScene layout is injected into forward mesh, normal prepass, shadow, and deferred geometry pipeline layouts at group3. Group1 is no longer a mesh compatibility placeholder; forward passes use it for the shadow receiver resources, while passes that do not need group1 install no layout there. During compiled-scene rendering, `render_compiled_scene` clones the active GPUScene bind group into a `MeshSceneDataBindHandle`; `RenderPassMeshCommandLists` forwards that handle to graph execution, and `MeshDrawCommandReplayer` binds a command-local GPUScene bind group when present or falls back to the shared frame handle. The legacy `render_scene` path builds the same handle and passes it through overlay mesh recording so the non-compiled path does not keep a separate mesh-bind behavior.

The shader bridge performs the GS-M2 transform and palette reads for the built-in mesh shaders. `zr_gpu_scene.wgsl` defines `ZrGpuPrimitiveData`, `ZrGpuInstanceData`, group3 storage bindings 0-2, group3 palette bindings 3-4, and helper functions such as `zr_world_from_local`, `zr_previous_world_from_local`, primitive tint/shadow/motion accessors, `zr_skinned_joint_matrix`, and `zr_previous_skinned_joint_matrix`. The forward fallback, normal prepass, shadow map, and deferred geometry shader source modules prepend that include, use `@builtin(instance_index)` in their vertex entries, and pass primitive parameters through vertex outputs where fragment stages need them. `ModelUniform`/`model_data` declarations and pass-local `SkinnedJointPaletteUniform`/`@group(1)` palette declarations are no longer present in those four WGSL sources. Rust-side `ModelUniform`, `ModelUniformCache`, model buffers, model bind groups, and the old mesh compatibility bind slot have been removed from `scene_renderer`. The material set now combines material textures/samplers and `material_properties` at group2 binding10, so built-in and custom material shader ABI checks share the same group2/material plus group3/GPUScene contract.

GS-M4 has started at the command-stat planning layer. WGPU device creation now requests `MULTI_DRAW_INDIRECT_COUNT` and `INDIRECT_FIRST_INSTANCE` when available, RHI/backend capability summaries expose those flags, and `RenderCapabilitySummary::gpu_driven_submission_supported()` gates indirect batching. `IndirectDrawBatcher` consumes sorted mesh commands, converts eligible direct indexed commands into CPU `IndexedIndirectArgs`, groups adjacent commands that share the same phase/pipeline/geometry/material/GPUScene bind identity, and reports fallback counts when the gate is closed or a command already owns an indirect args buffer. `render_compiled_scene` passes the frame capability summary through to command-buffer stats so `RenderStats.last_indirect_*` and `render.mesh.queue.indirect_*` diagnostics describe the batch plan for the current backend. The WGPU pass replay still submits via the existing draw paths; actual `multi_draw_indexed_indirect` execution and args-buffer ownership remain the next GS-M4 slice.

## Design and Rationale

Plan 03 follows Unreal's GPUScene shape at a smaller scale: persistent primitive and instance storage buffers with stable indices, but without UE's float4 SOA tiling or GPU-write delegate surface in V1. Zircon uses typed WGSL storage-buffer structs because the current wgpu shader path benefits from simple AoS mirrors and direct Pod upload.

The allocator uses first-fit plus adjacent-span coalescing instead of a grow-only policy. This keeps static-scene churn from permanently increasing buffer high-water marks while still preserving same-frame no-aliasing through deferred free commits.

The dirty queue is intentionally semantic-light. Extract and renderer diff code will decide what is dirty; the queue only batches indices into upload ranges. This keeps transform/material revision policy out of the low-level upload utility.

## Control Flow

The current GS-M1 frame flow is:

1. Extract produces renderer-neutral mesh snapshots, stable keys, and transform revisions.
2. `SceneRendererCore` passes its `GpuScene` owner into mesh draw construction.
3. `build_mesh_draws` registers or updates the GPUScene entry for each real pending draw, using CPU shadow comparison to mark only changed primitive or instance records dirty, and unregisters entries missing from the live frame set.
4. `build_mesh_draws` attaches each entry's first-instance span to the corresponding `MeshDraw`; skinned draws may also attach a command-local GPUScene bind group with real current/previous palette buffers.
5. `flush_updates()` drains dirty ranges and writes primitive/instance storage buffers through direct `queue.write_buffer` calls.
6. `render_compiled_scene` carries the active GPUScene bind group through `RenderPassMeshCommandLists` so built-in mesh passes can bind the scene-data storage group during command replay.
7. `CompiledSceneDraws` returns `GpuSceneUploadReport`, and `RenderStats` receives primitive count, instance count, dirty count, upload bytes, upload path, free-span count, upload range counts, and GPUScene command instance counts.

The current GS-M2 shader flow uses that same data owner and command span to read current transforms, previous transforms, tint, shadow params, motion params, and skinning palettes from GPUScene storage/uniform bindings. GPUScene commands no longer carry an object-bind compatibility handle, the model-uniform cache/build path is gone, and material/custom shader validation now expects group2 material bindings plus group3 GPUScene bindings. The remaining validation work is real-adapter pipeline creation and render-product coverage for the final slot layout.

## Edge Cases and Constraints

All spans must have nonzero length. Zero-length frees or dirty marks are ignored for queueing, while zero-length allocations are rejected. Span end calculations use checked arithmetic to expose index-space overflow.

`GPU_SCENE_INVALID_PAYLOAD_SLOT` is `u32::MAX`. It reserves payload/lightmap indirection for later plans without forcing a second payload buffer in this slice.

The dirty-range merge gap is fixed at eight entries. This intentionally trades a small amount of extra upload bandwidth for fewer copy commands when edits are near each other.

## Test Coverage

Current inline unit tests cover:

- primitive and instance stride/offset ABI parity with the plan table,
- deferred free reuse so same-frame allocations do not alias pending frees,
- adjacent free-span coalescing while preserving high-water capacity,
- dirty-range sorting, duplicate collapse, gap merging, and byte-range output,
- static-scene second-frame zero upload and single-moving-entity one-instance-stride upload on a headless WGPU device,
- GPUScene bind group layout binding order, read-only storage types, and palette uniform slots,
- group3 GPUScene shader ABI declarations in forward fallback, normal prepass, shadow, and deferred geometry shader source,
- GPUScene instance-data shader consumption in forward fallback, normal prepass, shadow, and deferred geometry vertex/fragment paths,
- GPUScene palette helper consumption in forward fallback, normal prepass, shadow, and deferred geometry shader sources, including absence of the old pass-local group1 palette bindings,
- material/custom shader ABI diagnostics that require group2 texture/sampler bindings, group2 binding10 material uniform, and group3 GPUScene bindings,
- Naga WGSL parsing/validation for all four built-in shader sources after `zr_gpu_scene.wgsl` is prepended,
- mesh batch conversion of a GPUScene entry span into a GPUScene instance command,
- prepared queue propagation of GPUScene count, upload range, upload byte, and upload path statistics,
- CPU indirect batcher grouping/fallback behavior and mesh pass aggregation of indirect batch counts under the GPU-driven capability gate,
- WGPU indirect execution source coverage for phase-local `INDIRECT` args buffers and `multi_draw_indexed_indirect` replay,
- render-product diagnostics for GPUScene primitive/instance counts, dirty entry count, uploaded bytes, direct queue-write path, free spans, and upload range counts.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` passed on 2026-06-12 after the final slot convergence, material set merge, built-in PBR shader update, custom project test WGSL ABI update, material shader layout diagnostic migration, removal of the old material-uniform-only bind group owner, GS-M3 CPU-shadow diff upload, explicit direct-write upload-path diagnostics, GS-M4 CPU indirect batch planning, and GS-M4 WGPU multi-draw replay wiring; it reports 89 existing warnings. Static scans found no remaining production `ModelUniform`/`model_data` shader resources, group4/group5 mesh pass bindings, `MATERIAL_TEXTURE_BIND_GROUP_SLOT`, `bind_material_textures_if_needed`, or mesh compatibility bind-group references under `scene_renderer`.

Focused `cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7 on 2026-06-12 after compiling the lib-test binary. This covers the layout, allocator, dirty range, bind group layout, static second-frame zero-byte upload, and one moving entry uploading exactly `GPU_INSTANCE_DATA_STRIDE` bytes, with `GpuSceneUploadPath::DirectQueueWrite` asserted by the two upload tests. Focused `cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 and proves the public diagnostics path records GPUScene counts, bytes, ranges, and direct-write upload policy. The runtime diagnostics aggregation fixture now registers the fake render framework under `crate::graphics::GRAPHICS_MODULE_NAME` so its `GraphicsModule.Manager.RenderFramework` service name satisfies runtime owner validation; the `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` rerun is pending because active plugin-session test files currently stop the `zircon_runtime` lib-test target from compiling. Focused `cargo test -p zircon_runtime --lib mesh_batch_ref_emits_gpu_scene_instance_command --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` passed 1/1 after the Rust-side model-uniform removal. Earlier focused runs on 2026-06-12 covered shader instance-data consumption, `shader_is_valid_wgsl`, `shader_declares_gpu_scene_group`, skinned palette helpers, prepared queue GPUScene stats, and fallback mesh shader validity. Fresh `cargo test -p zircon_runtime --lib render_gpu_scene_indirect_batcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` attempts after GS-M4 CPU batch planning timed out while compiling the lib-test binary after 120 seconds and 300 seconds; `cargo test -p zircon_runtime --lib mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` also timed out after 304 seconds after GS-M4 replay wiring. Process hygiene was checked afterward; remaining cargo/rustc processes belonged to other target dirs/sessions. The milestone testing stage still needs completed focused shader/ABI/lib-test runs plus real-adapter WGPU pipeline/render-product coverage for the final group2/group3 layout and GS-M4 multi-draw execution.

## Plan Sources

This module implements the first data-plane slices from `docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`, the GS-M2 built-in shader consumption/final slot step, the GS-M3 CPU-shadow diff upload plus explicit V1 direct-write upload policy, and the GS-M4 CPU indirect batch planning/telemetry plus WGPU multi-draw replay slice. It still prepares for later staging/graph upload refinement and real-adapter multi-draw validation.

## Open Issues

Staging-ring upload, render-graph upload node integration, GPU-generated draw counts, and real-adapter render-product validation remain later slices. The current frame-path integration reaches mesh commands, binds group3 GPUScene scene data, uses command-local group3 overrides for real skinned palettes, uses explicit direct queue writes for V1 uploads, produces capability-gated CPU indirect batch telemetry, builds phase-local indirect args buffers, and replays eligible batches through `multi_draw_indexed_indirect`. The next runtime validation should create the affected WGPU pipelines on the real adapter and confirm the final group1 shadow receiver, group2 material set, group3 GPUScene layout, instance-index/palette helper shader consumption, and indirect execution path.
