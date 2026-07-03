---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_scene_content.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/record_meshes.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/mod.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/GPUSceneWriter.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp
  - dev/bevy/crates/bevy_render/src/batching/mod.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
  - dev/bevy/crates/bevy_render/src/render_resource/gpu_array_buffer.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/bevy/crates/bevy_pbr/src/render/gpu_preprocess.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh_preprocess.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/build_indirect_params.wgsl
  - dev/bevy/crates/bevy_pbr/src/render/mesh_functions.wgsl
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/build_mesh_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
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
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
tests:
  - zircon_runtime/src/graphics/scene/gpu_scene/layout.rs::tests::render_gpu_scene_layout_matches_wgsl_offsets
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_reuses_freed_spans_without_aliasing
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs::tests::render_gpu_scene_id_allocator_coalesces_adjacent_free_spans
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs::tests::render_gpu_scene_update_queue_merges_adjacent_dirty_ranges
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs::tests::render_gpu_scene_bind_group_layout_reserves_storage_and_palette_bindings
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_static_scene_second_frame_uploads_zero_bytes
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_single_moving_entity_uploads_only_its_entry
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs::render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_gpu_scene_tests.rs::runtime_15_gpu_scene_tests_are_child_owner
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_declares_gpu_scene_group
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs::render_mesh_draw_processor_shadow_excludes_non_casters_and_picks_alpha_mask_variant
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
  - cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: milestone-detail
---

# 计划 03:GPUScene 与 GPU-driven 提交

## 目标

建立 UE GPUScene 等价物:primitive/instance/light 数据集中放入 GPU storage buffer,
着色器按 instance index 访问;在此之上打开 compute 剔除 → indirect draw 的 GPU-driven 路径。完成后:

1. 逐 draw 的 model uniform 绑定被 instance index 取代,draw 间不再因变换数据切换 bind group。
2. 场景数据增量上传(只传脏条目),静态大场景稳态帧上传量趋近于零。
3. 具备 capability gate 的 indirect 提交:支持 multi-draw indirect 的后端走 GPU 剔除 + indirect,不支持的回落 CPU 提交,外部接口一致。
4. 为 VG(Nanite-like)、HGI(Lumen-like)、计划 06(prev transform)提供统一场景数据底座。

## 现状与差距

- 变换/材质参数经 `gpu_model` / `gpu_material_uniform` 以逐 draw uniform 绑定,256 矩阵 skinning palette 也走 per-draw uniform ABI,绑定切换是 draw 提交的主要状态成本。
- `prepared_queue.rs` 已统计 `gpu_instancing_candidate_draw_count` 与 static batch 候选,说明合批意图存在,但没有 indirect args buffer 与 GPU 剔除,候选无法兑现。
- 灯光走场景 uniform,数量受限(计划 05 解除依赖本计划的 light buffer)。
- VG/HGI 插件各自维护私有 GPU 缓冲,与主场景数据不共享。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.h/.cpp` | SOA float4 缓冲布局(InstanceSceneData/InstancePayloadData/PrimitiveSceneData/LightData);脏区跟踪与增量上传(`FGPUScene::Update`、upload 任务划分);id 分配与回收 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/GPUSceneWriter.h` | 着色器侧访问 ABI:shader parameter struct 如何暴露缓冲与解包函数 |
| `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp` | 视锥/遮挡剔除 compute 的批组织(load balancer)、剔除结果写 indirect args、多 view 合批剔除 |

次参考:`dev/bevy/crates/bevy_pbr`(`MeshUniform` 的 storage buffer 化与 `GpuArrayBuffer` 抽象,Rust/wgpu 实操);wgpu 能力:`Features::MULTI_DRAW_INDIRECT_COUNT`、`INDIRECT_FIRST_INSTANCE`。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs` | capability gate + `IndirectDrawBatcher` | `GpuPreprocessingSupport`/`GpuPreprocessingMode`(Off/PreprocessingOnly/Culling 三档,按 adapter 能力降级)与 `IndirectParametersBuffers`/`PreprocessWorkItem` 的 indirect 批组织;GS-M4 的 gate 分档直接对照 |
| `dev/bevy/crates/bevy_render/src/batching/mod.rs` | 批次聚合判据 | `GetBatchData`/`GetFullBatchData`:哪些数据决定两 draw 可合批(buffer index 连续性、bin 内相邻);`NoAutomaticBatching` 的逃生口设计 |
| `dev/bevy/crates/bevy_render/src/render_resource/gpu_array_buffer.rs` | storage buffer 回落路径 | `GpuArrayBuffer` 枚举在 storage buffer 与 `BatchedUniformBuffer`(dynamic offset uniform)间按 limits 切换——"同一 ABI、不同提交方式"的能力回落实例 |
| `dev/bevy/crates/bevy_pbr/src/render/mesh.rs` | `GpuScene` 条目与脏维护 | `MeshUniform`(GPU 侧逐实例布局)与 `MeshInputUniform`(CPU 写入面,含 previous transform、flags)分离;`RenderMeshInstanceGpuBuilder` 的增量更新与稳定 index 维护 |
| `dev/bevy/crates/bevy_pbr/src/render/gpu_preprocess.rs` | GPU-driven compute 调度 | `PreprocessPipelines`/`PreprocessPhasePipelines`:reset batch sets → transform/cull preprocess → build indirect params 的 compute 串联与 bind group 组织 |
| `dev/bevy/crates/bevy_pbr/src/render/mesh_preprocess.wgsl` | 剔除 + indirect 改写 compute | 读 `MeshInputUniform` 写 `MeshUniform`、`view_frustum_intersects_obb`、HZB 遮挡测试、原子累加 instance_count——GS-M4/计划 04 VC-M3 的 WGSL 直接样板 |
| `dev/bevy/crates/bevy_pbr/src/render/build_indirect_params.wgsl` | indirect args buffer 布局 | `IndirectParametersIndexed` 由 CPU/GPU metadata 两段拼出 args 的布局与写入时序;`IndexedIndirectArgs` 字段对齐可对照 |
| `dev/bevy/crates/bevy_pbr/src/render/mesh_functions.wgsl` | instance index 着色 ABI | `get_world_from_local(instance_index)`/`get_previous_world_from_local`:顶点着色器按 instance_index 取变换的标准写法(GS-M2 切换后的目标形态) |

Fyrox 无 GPUScene/indirect 同类实现(逐 draw uniform 块分配,见其 `renderer/cache/uniform.rs`),本计划 Rust 参照以 bevy 为准。

## 目标架构

归属:新增 `zircon_runtime/src/graphics/scene/gpu_scene/` 子模块(数据面),剔除 compute 与 indirect 提交在 `scene_renderer/mesh/` 与计划 04 协同;extract 契约扩展在 `core/framework/render/scene_extract.rs`。

核心类型:

- `GpuScene`:`primitive_data: StorageBuffer`(transform、prev transform、bounds、flags、lightmap/payload 槽)、`instance_data`、`light_data`(计划 05 消费)。SOA 打包对齐 16 字节;id 由 `GpuSceneIdAllocator` 分配,free list 回收。
- `GpuSceneUpdateQueue`:extract 携带脏标记 → 帧首合并为去重上传批;大批量首帧走整段上传,稳态走 scatter 上传 compute(或 queue.write_buffer 分段,按尺寸阈值选择)。
- shader ABI:bind group 布局新增 scene data group(`pipeline_layout.rs` 收口);顶点着色器以 `instance_index`(来自 draw args first_instance 或 instance step 顶点输入)取 transform。计划 02 的 `DrawInstanceSource` 在此切换为 instance index 路径并删除 uniform 路径。
- `IndirectDrawBatcher`:同 (pipeline, geometry, material set) 的命令聚为一个 indirect batch;CPU 填充 args 为基线,计划 04 的剔除 compute 直接改写 args 中 instance_count。
- capability gate:`RenderCapabilityClass` 增加 gpu_driven 档位;不支持 multi-draw indirect 时 batcher 退化为逐条 `draw_indexed`(仍用 instance index ABI,只是提交方式不同),不维护两套 shader ABI。

插件衔接:VG/HGI 的 provider 改为注册"场景数据消费者",直接读 `GpuScene` 缓冲;私有重复缓冲在各自计划内清退。

## 里程碑

### GS-M1 GpuScene 数据面

实施切片:
1. `GpuScene` 缓冲布局、id 分配、整段上传;extract 增加 stable instance id 与脏标记。
2. `resource_streamer` 在 ensure 流程中登记 primitive 条目(替代 per-draw model uniform 的数据来源)。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime gpu_scene --locked`(布局/分配/回收单测)
- 验收证据:缓冲内容与 extract 对拍一致(readback 测试);id 回收无串台。

### GS-M2 instance index 着色路径

实施切片:
1. scene data bind group 接入 SRP 布局;内建 mesh shader(含 fallback)改为 instance index 取变换。
2. 计划 02 的 `DrawInstanceSource` 切换并删除 uniform 路径;skinning palette 走同一 scene-data ABI 的 binding 3/4,真实 per-skinned-draw palette bind group 在本里程碑内补齐。

当前进度(2026-06-12):`GpuScene` 已拥有 primitive/instance/light storage bind group layout 与 bind group;生产 `build_mesh_draws` 会在同步 GPUScene 后保留每个 stable instance key 对应的 `GpuSceneEntry`,并把 `(first_instance_index, instance_count)` 回挂到 `MeshDraw`。`MeshDraw::mesh_pass_batch_ref` 继续传递该 span,`MeshBatchRef::command` 只生成 `DrawInstanceSource::GpuSceneInstance` 和带 first-instance 的 direct draw args。GPUScene 命令不再携带 fallback object bind,`MeshDrawCommandReplayer` 的 model-bind helper 已删除,四条内建 WGSL 已不再声明 `model_data` 作为变换来源。

后续 GS-M2 切片已把同一个 `GpuScene::scene_bind_group_layout()` 注入 forward mesh pipeline cache、normal prepass、shadow map 与 deferred geometry pipeline layout,并完成最终物理槽位收束:group0 为 scene,group1 为 forward shadow receiver(仅需要的 pass 安装),group2 为 material set,group3 为 GPUScene scene-data。`render_compiled_scene` 克隆当前 GPUScene bind group 并放入 `RenderPassMeshCommandLists`,prepass/base/shadow/deferred/velocity pass 通过 `MeshDrawCommandReplayer::bind_gpu_scene_if_needed` 在 draw replay 中按需绑定 group3;legacy `render_scene` 路径也把同一 handle 传入 overlay mesh recording。GPUScene layout 按计划 ABI 暴露 binding 0/1/2 primitive/instance/light storage,以及 binding 3/4 current/previous skinned joint palette uniform。`GpuScene::new` 接收 renderer 现有空 palette fallback buffer,非 skinned draw 的共享 group3 bind group 绑定 fallback palette,skinned draw 可通过 `GpuScene::create_scene_bind_group_for_palettes(...)` 创建 command-local scene-data bind group,复用同一批 storage buffers 并只覆盖真实 current/previous palette buffer。`mesh/shaders/zr_gpu_scene.wgsl` 定义 `ZrGpuPrimitiveData` / `ZrGpuInstanceData` 与 transform、primitive 参数、current/previous palette helper,并拼接进 forward fallback、normal prepass、shadow map、deferred geometry 和内置 PBR shader source。四条内建 mesh shader 的顶点入口通过 `@builtin(instance_index)` 读取 GPUScene transform,velocity 入口同时读取 previous transform;primitive tint、shadow params 与 motion params 从 `GpuPrimitiveData` 读取并经 vertex output 传入 fragment 阶段。`ModelUniform`/`model_data`、旧 pass-local `SkinnedJointPaletteUniform`/`@group(1)` palette 声明、Rust 侧 `ModelUniformCache`、`ModelUniform`、`MeshDraw::{model_buffer, model_bind_group}`、pending model cache key、fallback object bind 与 model-bind replay helper 均已删除。

最终槽位切片还把 material textures/samplers 与 `material_properties` 合并为 group2 material set(binding 0-9 为贴图/采样器,binding 10 为材质 uniform),删除旧 material texture 单独绑定路径,并把 material/custom shader ABI 诊断迁移为 group2 material + group3 GPUScene 校验。项目渲染测试生成的自定义 WGSL 已改为拼接 `zr_gpu_scene.wgsl`,通过 `@builtin(instance_index)` 读取 transform/tint,并声明 group2 binding10 material uniform。旧 material-uniform-only bind group owner 也已删除,material uniform 资源现在只保留 uniform buffer,由 group2 material set bind group 引用。已运行的验证包括此前重新编译后的 `shader_is_valid_wgsl`(4 项)、直接 lib-test 二进制 `skinned_joint_palette`(5 项)与 `shader_declares_gpu_scene_group`(4 项)、`fallback_mesh_shader`、`reads_gpu_scene_instance_data`、command span focused test、rustfmt,以及 scene_renderer 静态扫描无剩余 model-uniform/cache/replay、group4/group5、旧 material texture bind helper 或 mesh-compat 引用。最新 scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 通过并报告 89 个 warning。最终槽位后的 `cargo test -p zircon_runtime --lib shader_declares_gpu_scene_group ...` 由于其它会话并行链接 lib-test 二进制而在编译阶段超时,未返回测试结果。GS-M3 V1 direct-write 增量上传已完成;GS-M4 已完成 capability gate、CPU indirect batch planning 和诊断统计接线。尚未完成的是 final layout 的 focused shader/ABI lib-test 重跑、GS-M4 WGPU multi-draw replay/args-buffer 执行、real-adapter WGPU pipeline 创建与 render-product 验证。

测试阶段:
- `cargo test -p zircon_runtime mesh --locked` + `render_product` 全量回归
- 验收证据:渲染产物逐像素不变;draw 间 bind group 切换计数显著下降(统计断言)。

### GS-M3 增量上传

实施切片:
1. 脏队列合并与 scatter 上传;变换静态对象零上传。
2. 上传字节数进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime gpu_scene --locked`(静态场景第二帧上传字节为 0;单对象移动只传该条目)
- 验收证据:大场景稳态上传量统计记入文档。

### GS-M4 indirect 提交(capability gate)

实施切片:
1. `RenderCapabilitySummary` / RHI / wgpu backend 增加 `supports_multi_draw_indirect` 与 `supports_indirect_first_instance`,并用 `gpu_driven_submission_supported()` 统一 gate。
2. `IndirectDrawBatcher` 做 CPU-side batch planning:gate 关闭时逐 draw fallback,gate 开启时把 eligible direct indexed commands 转成 `IndexedIndirectArgs` 并按相邻 phase/pipeline/geometry/material/GPUScene bind identity 聚批。
3. `render_compiled_scene` 把 frame capability summary 传入 `MeshPassCommandBuffers::stats_with_indirect_batches(...)`;`PreparedMeshQueueStats`、`RenderStats.last_indirect_*` 和 `render.mesh.queue.indirect_*` diagnostics 记录 batch plan。
4. `MeshPassIndirectDrawExecutions` 为 depth-prepass、shadow、opaque、alpha-mask、transparent、velocity phase 分配 phase-local WGPU indirect args buffer;built-in mesh pass replay 通过 `MeshDrawCommandStream` 在 eligible batches 上调用 `multi_draw_indexed_indirect`,并保留逐 draw fallback。
5. 后续验证切片:同几何多实例 render-product 场景、回落路径产物一致性、RenderDoc 抓帧确认 multi-draw 调用与真实 adapter coverage。

测试阶段:
- `cargo test -p zircon_runtime mesh --locked` 与 `cargo test -p zircon_runtime gpu_scene --locked`
- 验收证据:同几何多实例场景 draw call 数按 batch 合并(统计断言);回落路径产物一致;RenderDoc 确认 indirect 调用。

## 工程落地细化

本章是计划 03 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、std430、`zr_` include、queue 数值段、`sort_key` 归属等全局约定直接引用 index.md §8,本章不重定义。

### 模块与文件落点

新增文件(根 facade 不变,均在 `zircon_runtime` 内):

| 路径 | 内容 |
|------|------|
| `zircon_runtime/src/graphics/scene/gpu_scene/mod.rs` | 模块声明 + curated re-export(`GpuScene`、`GpuSceneIdAllocator`、`GpuSceneUpdateQueue`、`IndirectDrawBatcher`),保持 thin |
| `zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs` | `GpuScene` 主体:三块 storage buffer、CPU shadow 副本、扩容策略 |
| `zircon_runtime/src/graphics/scene/gpu_scene/layout.rs` | `GpuPrimitiveData` / `GpuInstanceData` 的 `#[repr(C)]` Pod 镜像 struct 与 stride/offset 常量(与 `zr_gpu_scene.wgsl` 逐字段对拍) |
| `zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs` | `GpuSceneIdAllocator`:单槽 free list + 连续 span 分配/合并回收 |
| `zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs` | `GpuSceneUpdateQueue`:脏标记收集、按 index 排序、相邻区间合并 |
| `zircon_runtime/src/graphics/scene/gpu_scene/staging_ring.rs` | `GpuSceneStagingRing`:3 帧轮转的 `MAP_WRITE | COPY_SRC` staging 环 |
| `zircon_runtime/src/graphics/scene/gpu_scene/upload.rs` | `flush_updates`:合并区间 → 直写/staging 二选一 → copy 命令;`gpu_scene_upload` graph 节点 executor |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/indirect_draw_batcher/mod.rs` | 模块声明 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/indirect_draw_batcher/indirect_draw_batcher.rs` | `IndirectDrawBatcher` + `IndirectDrawBatch` + capability gate 提交路径 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl` | 共享 include:镜像 struct、`get_instance_data` / `get_primitive_data` 等函数,无 entry point |

修改文件:

| 路径 | 改动要点 |
|------|---------|
| `zircon_runtime/src/graphics/scene/mod.rs` | 挂载 `gpu_scene` 子模块 |
| `zircon_runtime/src/core/framework/render/scene_extract.rs` | `RenderMeshSnapshot` 增加 `stable_instance_key: u64` 与 `transform_revision: u64` |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderCapabilitySummary` 增加 `supports_multi_draw_indirect` / `supports_indirect_first_instance`;`RenderStats` 增加 `last_gpu_scene_*` / `last_indirect_*` 字段(见测试清单) |
| `zircon_runtime/src/graphics/backend/render_backend/request_device.rs` | 由 `wgpu::Features::MULTI_DRAW_INDIRECT_COUNT`、`INDIRECT_FIRST_INSTANCE` 填充上述两个能力位 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs` | ensure 流程中按 `stable_instance_key` 登记/释放 GpuScene primitive + instance 条目 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs` | 删除 model uniform/bind group 构建,改写为依赖 GPUScene entry span;skinned draw 创建 command-local GPUScene palette bind group |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs` | `MeshDraw` 删除 `model_buffer` / `model_bind_group` 字段,保留 GPUScene instance span 与可选 command-local GPUScene bind group |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs` | 已从 frame path 删除;command replay 直接以 GPUScene span 的 first_instance 提交 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs`、`prepass/normal_prepass_pipeline/new.rs`、`shadow/shadow_map_renderer.rs`、`deferred/geometry_pipeline/create.rs` | pipeline layout 使用最终槽位:group0 scene、group1 shadow receiver(需要时)、group2 material set、group3 GPUScene |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs` | 拼接 `zr_gpu_scene.wgsl` include |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl` | 按 §8 槽位重排;顶点改为 `@builtin(instance_index)` 取变换 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs` | `gpu_instancing_candidate_*` 统计改由 batcher 实际合批结果派生 |
| `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs` | 接入 extract 差分 → `GpuSceneUpdateQueue`;`flush_updates` 时序见集成节 |

硬删除(GS-M2 同一变更内):`scene_renderer/mesh/model_uniform_cache/` 整个目录(`ModelUniformCache`、`ModelUniformCacheKey`)、`scene_renderer/primitives/model_uniform/` 的 `ModelUniform` struct 及其在 `create_mesh_draw.rs` 中的 `model_uniform_from_draw_state` 函数。

### 核心类型与接口

framework 契约层(`core/framework/render/`,不得出现 wgpu):

```rust
// scene_extract.rs(追加字段)
pub struct RenderMeshSnapshot {
    // ...既有字段不动...
    /// runtime 侧生成的稳定实例键(entity + primitive 序号哈希),跨帧不变
    pub stable_instance_key: u64,
    /// 变换修订号;静态对象跨帧保持不变,是增量上传的脏判据
    pub transform_revision: u64,
}

// backend_types.rs(追加字段)
pub struct RenderCapabilitySummary {
    // ...
    pub supports_multi_draw_indirect: bool,
    pub supports_indirect_first_instance: bool,
}
impl RenderCapabilitySummary {
    /// 两者同时具备才走 GPU-driven indirect 提交,否则 batcher 走 CPU 回落
    pub fn gpu_driven_submission_supported(&self) -> bool;
}
```

graphics 实现层(`graphics/scene/gpu_scene/`,可用 wgpu):

```rust
pub(crate) struct GpuScene {
    primitive_data: wgpu::Buffer,      // STORAGE | COPY_DST,array<GpuPrimitiveData>
    instance_data: wgpu::Buffer,       // STORAGE | COPY_DST,array<GpuInstanceData>
    light_data: wgpu::Buffer,          // STORAGE | COPY_DST,array<GpuLightData>(条目布局归计划 05)
    primitive_shadow: Vec<GpuPrimitiveData>,   // CPU 影子副本,diff 与重传依据
    instance_shadow: Vec<GpuInstanceData>,
    primitive_ids: GpuSceneIdAllocator,
    instance_ids: GpuSceneIdAllocator,         // span 模式
    entries: HashMap<u64 /*stable_instance_key*/, GpuSceneEntry>,
    scene_bind_group_layout: wgpu::BindGroupLayout,  // group3 布局,见 WGSL 节
    scene_bind_group: wgpu::BindGroup,               // 非 skinned 共享一份
    stats: GpuSceneStats,
}
pub(crate) struct GpuSceneEntry {
    pub(crate) primitive_index: u32,
    pub(crate) first_instance_index: u32,
    pub(crate) instance_count: u32,
    pub(crate) last_transform_revision: u64,
}
impl GpuScene {
    pub(crate) fn register(&mut self, key: u64, instance_count: u32) -> GpuSceneEntry;
    pub(crate) fn unregister(&mut self, key: u64);
    pub(crate) fn write_primitive(&mut self, entry: &GpuSceneEntry, data: GpuPrimitiveData);
    pub(crate) fn write_instances(&mut self, entry: &GpuSceneEntry, data: &[GpuInstanceData]);
    /// 帧首调用:合并脏区间并产出 copy 命令;返回本帧上传字节数(进 RenderStats)
    pub(crate) fn flush_updates(
        &mut self, device: &wgpu::Device, queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder, ring: &mut GpuSceneStagingRing,
    ) -> u64;
}

pub(crate) struct GpuSceneIdAllocator {
    free_spans: Vec<(u32, u32)>,   // (start, len),按 start 有序,free 时与邻接 span 合并
    next: u32,
    live: u32,
    high_water: u32,
}
impl GpuSceneIdAllocator {
    pub(crate) fn allocate(&mut self) -> u32;                  // 单槽 = allocate_span(1)
    pub(crate) fn allocate_span(&mut self, len: u32) -> u32;   // first-fit;不足时从 next 扩展
    pub(crate) fn free_span(&mut self, start: u32, len: u32);  // 插入 + 双向合并,空洞供复用
    pub(crate) fn high_water(&self) -> u32;                    // 缓冲容量判据
}
```

稳定 index 语义:`primitive_index` / `first_instance_index` 自分配到 `unregister` 期间不变;shader、`MeshDrawCommand` 缓存(计划 02)与 VG/HGI 消费者可跨帧持有。回收的槽位在同一帧内不得重新分配给新条目(`free_span` 进 pending 列表,`flush_updates` 末尾才并入 `free_spans`),防止 in-flight 帧串台。

```rust
pub(crate) struct GpuSceneUpdateQueue {
    dirty_primitives: Vec<u32>,            // primitive_index,去重收集
    dirty_instance_spans: Vec<(u32, u32)>,
}
impl GpuSceneUpdateQueue {
    pub(crate) fn mark_primitive(&mut self, index: u32);
    pub(crate) fn mark_instances(&mut self, start: u32, len: u32);
    /// 排序 + 相邻合并:间隙 <= 8 个条目的区间并为一段,换取更少 copy 命令
    pub(crate) fn drain_merged(&mut self, stride: u32) -> Vec<(u64 /*byte offset*/, u64 /*byte len*/)>;
}
```

`scene_renderer/mesh/indirect_draw_batcher/`:

```rust
pub(crate) struct IndirectDrawBatcher {
    args_cpu: Vec<IndexedIndirectArgs>,    // 复用 build_mesh_draws/indexed_indirect_args.rs 既有 struct
    args_buffer: Option<Arc<wgpu::Buffer>>, // INDIRECT | COPY_DST | STORAGE(STORAGE 供计划 04 剔除 compute 改写 instance_count)
    batches: Vec<IndirectDrawBatch>,
    fallback_draw_count: usize,
}
pub(crate) struct IndirectDrawBatch {
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) geometry_key: (ResourceId, u32, u32), // mesh id + first_index + index_count
    pub(crate) material_key: ResourceId,
    pub(crate) first_args: u32,    // args_buffer 内的起始条目
    pub(crate) args_count: u32,    // multi_draw_indexed_indirect 的 count
    pub(crate) total_instances: u32,
}
impl IndirectDrawBatcher {
    /// 输入为计划 02 排序后的 MeshDrawCommand 流(sort_key 布局归计划 09,这里只消费序);
    /// 相邻且 (pipeline, geometry, material) 相同的命令聚为一个 batch
    pub(crate) fn build(&mut self, device: &wgpu::Device, queue: &wgpu::Queue,
                        commands: &[MeshDrawCommand], caps: &RenderCapabilitySummary);
    pub(crate) fn submit<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>, caps: &RenderCapabilitySummary);
}
```

计划 02 衔接:`MeshDrawCommand` 的 `DrawInstanceSource` 在 GS-M2 切换为唯一的 `DrawInstanceSource::GpuSceneInstanceIndex { first_instance: u32, instance_count: u32 }`,uniform 变体删除,不留双实现。

### GPU 数据布局与 WGSL 约定

数据划分对齐 UE:变换在 instance,包围盒/材质派生参数在 primitive。std430、列主序、显式 padding(§8 第 2 条)。

`GpuPrimitiveData`(stride 80 字节):

| 字段 | WGSL 类型 | 字节偏移 | 说明 |
|------|-----------|---------|------|
| `bounds_center` | `vec3<f32>` | 0 | 世界空间包围球心 |
| `bounds_radius` | `f32` | 12 | 填入 vec3 的 padding lane,无空洞 |
| `tint` | `vec4<f32>` | 16 | 迁自 `ModelUniform.tint` |
| `shadow_params` | `vec4<f32>` | 32 | lane 语义与现 `ModelUniform.shadow_params` 一致(alpha_mask、cutoff、receive_shadows、0) |
| `motion_params` | `vec4<f32>` | 48 | lane 语义与现 `ModelUniform.motion_params` 一致 |
| `flags` | `u32` | 64 | bit0 visible、bit1 cast_shadows、bit2 has_prev_transform;其余保留 |
| `first_instance_index` | `u32` | 68 | 指向 instance_data 起始槽 |
| `instance_count` | `u32` | 72 | |
| `payload_slot` | `u32` | 76 | lightmap/payload 预留槽(计划 11 消费),V1 恒 0xFFFFFFFF |

`GpuInstanceData`(stride 144 字节):

| 字段 | WGSL 类型 | 字节偏移 | 说明 |
|------|-----------|---------|------|
| `world_from_local` | `mat4x4<f32>` | 0 | |
| `prev_world_from_local` | `mat4x4<f32>` | 64 | prev transform 槽:本计划随上传写入(保持现有 motion vector 行为),双缓冲/jitter 精化归计划 06 的 `TaaResolveExecutor` |
| `primitive_index` | `u32` | 128 | 反向索引 primitive_data |
| `flags` | `u32` | 132 | 保留 |
| `payload_slot` | `u32` | 136 | 保留 |
| `_pad0` | `u32` | 140 | 显式 padding,Rust 镜像同名字段 |

`light_data`:`array<GpuLightData>`,本计划只负责缓冲创建、id 分配与 binding 槽;`GpuLightData` 条目布局与写入由计划 05 定义。

group3 binding 编号(group0–2 语义见 §8 第 1 条;2026-07-03 SH02-SH04 已完成槽位归位:material property uniform 位于 group2 binding 0,标准贴图/采样器位于 group2 binding 1..10,shadow receiver 采样位于 group1):

| binding | 资源 | 类型 | 说明 |
|---------|------|------|------|
| 0 | `zr_primitive_data` | `var<storage, read>` | `array<GpuPrimitiveData>` |
| 1 | `zr_instance_data` | `var<storage, read>` | `array<GpuInstanceData>` |
| 2 | `zr_light_data` | `var<storage, read>` | 计划 05 消费;本计划绑 fallback 空缓冲 |
| 3 | `zr_skinned_joint_palette` | `var<uniform>` | current skinned palette;非 skinned 绑 fallback |
| 4 | `zr_previous_skinned_joint_palette` | `var<uniform>` | previous skinned palette;非 skinned 绑 fallback |

非 skinned draw 共享 `GpuScene` 持有的唯一 group3 bind group;skinned draw 以相同 layout 创建携带真实 palette 的 per-draw bind group(skinned 本就走 dynamic 列表,不破坏静态命令缓存)。

`zr_gpu_scene.wgsl`(只含 struct 与函数,无 entry point):

```wgsl
struct GpuPrimitiveData {
    bounds_center: vec3<f32>, bounds_radius: f32,
    tint: vec4<f32>, shadow_params: vec4<f32>, motion_params: vec4<f32>,
    flags: u32, first_instance_index: u32, instance_count: u32, payload_slot: u32,
}
struct GpuInstanceData {
    world_from_local: mat4x4<f32>, prev_world_from_local: mat4x4<f32>,
    primitive_index: u32, flags: u32, payload_slot: u32, _pad0: u32,
}
@group(3) @binding(0) var<storage, read> zr_primitive_data: array<GpuPrimitiveData>;
@group(3) @binding(1) var<storage, read> zr_instance_data: array<GpuInstanceData>;

fn get_instance_data(instance_index: u32) -> GpuInstanceData { return zr_instance_data[instance_index]; }
fn get_primitive_data(primitive_index: u32) -> GpuPrimitiveData { return zr_primitive_data[primitive_index]; }
fn zr_world_from_local(instance_index: u32) -> mat4x4<f32> { return zr_instance_data[instance_index].world_from_local; }
fn zr_prev_world_from_local(instance_index: u32) -> mat4x4<f32> { return zr_instance_data[instance_index].prev_world_from_local; }
```

instance index ABI 结论:采用 **per-draw first_instance + `@builtin(instance_index)`**,不引入 instance step 顶点缓冲。理由:直接提交路径 `pass.draw_indexed(indices, base_vertex, first..first + count)` 的非零 first_instance 是 wgpu/WebGPU core 行为,全后端可用;indirect 路径的非零 `first_instance` 由 `INDIRECT_FIRST_INSTANCE`(与 `MULTI_DRAW_INDIRECT_COUNT` 一起)gate;step buffer 需要额外 vertex layout 排列并污染计划 08 的 VertexFactory 等价物,放弃。`render_pass_bindings.rs::record_indexed_draw` 改写为:

```rust
pub(crate) fn record_indexed_draw<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>) {
    if let Some(args) = &self.indirect_args_buffer {
        pass.draw_indexed_indirect(args, self.indirect_args_offset);
    } else {
        pass.draw_indexed(
            self.first_index..(self.first_index + self.draw_index_count),
            0,
            self.instance_index..(self.instance_index + self.instance_count),
        );
    }
}
```

indirect args buffer 布局:`array<IndexedIndirectArgs>`(等价 `wgpu::util::DrawIndexedIndirectArgs`),条目 20 字节、按 4 字节对齐连续排布,`first_args * 20` 为 `multi_draw_indexed_indirect` 的字节偏移;计划 04 的剔除 compute 以 binding STORAGE 改写其中 `instance_count` 字段。

### 帧时序与集成点

帧内顺序(锚点:`graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs::submit_frame_extract_with_ui` → `prepare_runtime_submission` → `scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs`):

1. **Extract 差分**(`render_scene.rs`,build_mesh_draws 之前):遍历 `RenderFrameExtract` 的 mesh 快照,按 `stable_instance_key` 与 `GpuScene::entries` 对账 —— 新 key 走 `register` + 全量写;`transform_revision` 变化走 `write_instances`;tint/材质派生参数变化走 `write_primitive`;消失的 key 走 `unregister`。`resource_streamer_ensure_scene_resources.rs` 在 ensure 成功后才登记,资产未就绪的条目不进 GpuScene。
2. **上传 flush**:`GpuScene::flush_updates` 在 graph 执行前运行;脏区间合并后,总字节 < 256 KiB 走 `queue.write_buffer` 直写,否则经 `GpuSceneStagingRing`(3 帧轮转,`copy_buffer_to_buffer` 进帧首 encoder)。首帧/扩容帧整段上传。依据 §6 第 3 条,上传以 `gpu_scene_upload` 为 executor id 注册 graph 节点(`render_graph/builder.rs::add_pass_with_executor` + `import_external_resource`/`write_external` 声明三块缓冲的写访问),节点经 `graph_execution/render_pass_executor_registration.rs` 注册,保证 RenderDoc marker 与统计覆盖。
3. **命令构建**:计划 02 的 processor 产出 `MeshDrawCommand`(携带 `DrawInstanceSource::GpuSceneInstanceIndex`);GS-M4 起 `IndirectDrawBatcher::build` 在排序后聚批并填充 args buffer。
4. **执行**:`graph_execution/builtin_scene_executors.rs` 的 opaque/alpha-mask/transparent executor 经 batcher `submit`;`gpu_driven_submission_supported()` 为真走 `multi_draw_indexed_indirect`,否则逐 batch 逐 draw 回落(同一 shader ABI,仅提交方式不同)。
5. **回收**:帧末将 pending free span 并入 free list;`high_water` 超过缓冲容量时下帧扩容(1.5 倍)并整段重传。

硬切换删除项(GS-M2 一次性,验收以 `render_product` 全量对拍守护):

- `mesh/model_uniform_cache/`(整目录)及 `MeshDraw::{model_buffer, model_bind_group}` 字段;
- `primitives/model_uniform/` 的 `ModelUniform` 与 `create_mesh_draw.rs::model_uniform_from_draw_state`;
- `render_pass_bindings.rs::bind_model` 与所有 `set_bind_group(1, model, ..)` 调用方;
- 内置 WGSL 的旧 `@group(1)` model/palette 声明(palette 现在位于 `zr_gpu_scene.wgsl` group3 binding 3/4);
- pipeline layout 中旧 model/mesh-compat bind group layout 的创建与布局数组项。

### 实施切片细化

**GS-M1 GpuScene 数据面**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1.1 | `gpu_scene/{mod,layout,id_allocator}.rs`、`graphics/scene/mod.rs` | Pod 镜像 struct + offset 常量;span free list 分配器 | `cargo check -p zircon_runtime --lib --locked` 过;layout 常量与本章表格一致 |
| 1.2 | `gpu_scene/gpu_scene.rs`、`update_queue.rs`、`staging_ring.rs`、`upload.rs` | 三缓冲创建、register/write/unregister、整段上传、`gpu_scene_upload` graph 节点 | check 过;节点出现在 compiled graph 统计中 |
| 1.3 | `scene_extract.rs`、runtime extract 生成处、`build_mesh_draws/build.rs`、`RenderStats` 透传处 | `stable_instance_key`/`transform_revision` 契约字段;mesh draw 扩展后登记 GPUScene 条目并输出统计 | check 过;extract 与统计单测更新 |

2026-06-12 当前落地状态:GS-M1.1 的模块挂载、Pod 布局镜像、手写 stride/offset ABI 常量和 span allocator 已落地在 `graphics/scene/gpu_scene/`。`update_queue.rs` 也已加入纯数据面脏区间合并工具。GS-M1.2 已新增 `GpuScene` owner、primitive/instance/fallback light 三块 storage buffer、CPU shadow、稳定 key 登记、容量扩容、直接 `queue.write_buffer` flush 和 uploaded-bytes/range-count 报告。GS-M1.3 的 framework 契约字段已落地:`RenderMeshSnapshot` 携带 `stable_instance_key` 与 `transform_revision`,`World::render_mesh_snapshots_for_camera` 会按 entity+primitive ordinal 与 Transform 内容填充。`SceneRendererCore` 现在持有 `GpuScene`;compiled scene 与 legacy scene 的 mesh draw 构建路径都会传入该 owner;`build_mesh_draws` 在扩展真实 pending draw 后以 source entity + draw ordinal 登记 GPUScene 条目、写入 primitive/instance CPU shadow、保留本帧 live key 并 flush 到 WGPU storage buffer。`CompiledSceneDraws` 返回 `GpuSceneUploadReport`,`PreparedMeshQueueStats` 汇总 owner stats 与 upload report,`RenderStats` 暴露 `last_gpu_scene_*` 计数与上传字节。当前接入仍是数据侧通道:shader 和 draw command ABI 尚未改用 instance index,staging ring 与 render graph upload 节点也未接入,因此可观测统计和 buffer 内容,但不改变渲染输出。新增单测覆盖 WGSL offset 对拍、pending free 同帧不复用、邻接 free span 合并、脏区间 gap<=8 合并与 byte range 计算,以及 GPUScene 统计透传。主链 `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 已在 2026-06-12 通过;`cargo test -p zircon_runtime --lib prepared_queue_stats_carry_gpu_scene_counts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` 也已通过。聚焦 `cargo test -p zircon_runtime --lib gpu_scene --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` 在 604 秒后仍停留于 lib-test 编译并超时,未产出测试结果。

**GS-M2 instance index 着色路径**(单一变更完成硬切换)

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 2.1 | `gpu_scene/binding.rs`、`fallback_mesh.wgsl`、`zr_gpu_scene.wgsl`、`fallback_mesh_shader_source.rs`、两个 `create_*_pipeline.rs` | GPUScene storage bind group 基础;§8 槽位重排;顶点用 `@builtin(instance_index)` 经 `zr_world_from_local` 取变换 | check 过;naga 校验通过 |
| 2.2 | `mesh_draw.rs`、`create_mesh_draw.rs`、`render_pass_bindings.rs`、`model_uniform_cache/`(删)、`primitives/model_uniform/`(删)、计划 02 的 `DrawInstanceSource` 定义处 | `MeshDraw` 改携带 instance index;删除 uniform 路径全部残留 | check 过;`grep ModelUniform` 仅剩 git 历史 |

2026-06-12 GS-M2 当前落地状态:`gpu_scene/binding.rs` 提供最终 group3 GPUScene bind group layout,绑定 0/1/2 分别暴露 primitive/instance/light buffer 给 vertex/fragment/compute 阶段,绑定 3/4 暴露 current/previous skinned joint palette uniform 给 vertex 阶段。`GpuScene` 构造时创建共享 frame bind group,primitive 或 instance buffer 扩容后会重建 bind group;非 skinned 共享 bind group 使用现有空 skinned palette fallback buffer 填充 binding 3/4,而 skinned draw 可通过 `GpuScene::create_scene_bind_group_for_palettes(...)` 创建 command-local scene-data bind group,复用同一批 storage buffers 并只覆盖 current/previous palette buffers。compiled scene 与 legacy scene mesh pass 都把 group3 bind group 传到 command replay,record 阶段优先绑定 command-local GPUScene bind group,否则绑定 frame shared handle。`zr_gpu_scene.wgsl` 已拼接进 forward fallback、normal prepass、shadow map、deferred geometry 与内置 PBR shader source,并提供 transform、previous transform、primitive tint/shadow/motion 与 current/previous palette helper。四条内建 WGSL 的顶点入口通过 `@builtin(instance_index)` 读取 GPUScene transform,velocity shader 同时读取 previous transform;primitive 参数经 vertex output 传到 fragment 阶段,蒙皮矩阵读取通过 `zr_skinned_joint_*` / `zr_previous_skinned_joint_*` helper 走 GPUScene group3 binding 3/4。Rust scene_renderer 侧旧 model uniform/cache/replay 已删除,material textures/samplers 与 `material_properties` 已收束为 group2 material set,shadow receiver 资源移到 group1,旧 material-uniform-only bind group owner、旧 group4/group5 与 mesh-compat layout 不再存在于生产 scene_renderer。最新 scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 通过并报告 88 个 warning;最终槽位后的 focused shader test 重跑因并行编译负载在测试编译阶段超时,仍需在验证阶段补跑。

**GS-M3 增量上传**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 3.1 | `update_queue.rs`、`upload.rs`、`render_scene.rs` | revision 差分只标脏变更条目;区间合并参数(间隙 8)定稿 | check 过;静态场景帧日志 0 上传 |
| 3.2 | `backend_types.rs`、`gpu_scene.rs` | `last_gpu_scene_uploaded_bytes` 等字段进 `RenderStats` | check 过;stats 流转到 framework 层 |

2026-06-12 GS-M3 当前落地状态:`PendingMeshDraw` 已携带 extract 的 `transform_revision`,`GpuScene::write_primitive` 与 `GpuScene::write_instances` 会先对比 CPU shadow,只有 primitive/instance 数据真实变化才标脏。compiled scene 的 full-frame pending draw 同步仍可每帧运行,但同一静态条目第二帧不会进入 dirty range,`flush_updates` 返回 0 上传字节;单个移动条目只上传一个 `GpuInstanceData` stride。`GpuSceneUploadReport` 现在显式携带 `GpuSceneUploadPath::DirectQueueWrite`,并映射到 `RenderGpuSceneUploadPath::DirectQueueWrite`;`RenderStats.last_gpu_scene_*`、primitive/instance upload range count 与 upload path 已沿 `GpuSceneUploadReport` -> `PreparedMeshQueueStats` -> framework stats -> render-product diagnostics 流转。当前 GS-M3 V1 策略是保留直接 `queue.write_buffer` 写入 persistent GPUScene storage buffer,把 staging ring / render graph upload node 留作后续优化,而不是在本阶段留下隐式缺口。验证:`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 通过并报告 88 个 warning;`cargo test -p zircon_runtime --lib render_gpu_scene_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` 通过 7/7;`cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` 通过 1/1。运行时聚合诊断夹具已改为用 `crate::graphics::GRAPHICS_MODULE_NAME` 注册 fake render framework,以匹配 `GraphicsModule.Manager.RenderFramework` 的服务所有权;完整 `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` 重跑当前被插件会话的 `extension_registry_bridge.rs` / `runtime_extension_registry.rs` 编译错误阻塞,不是 GPUScene 诊断代码失败。剩余 GS-M3 refinement:用真实 render-product 帧日志记录静态稳态上传量,并在后续上传优化阶段再决定 staging ring / render graph upload node 的切入点。

2026-06-14 计划 06 TP-M1 previous-transform 数据面已接回 GPUScene:`gpu_scene/prev_transform.rs`
通过 `roll_prev_transforms_after_success()` 在成功提交后把当前 `GpuInstanceData.world_from_local` 滚动到 `prev_world_from_local`,
`GpuSceneEntry.has_rolled_previous_transform` 防止首帧对象误报 previous 可用,并在 previous 发生变化时保留
下一帧 instance dirty range。compiled-scene 与 legacy `render_scene` 路径都在 `queue.submit(...)` 后触发 roll。
后续 CPU object-history 硬切已经删除 `ViewportMotionVectorObjectHistory` submit/viewport/frame DTO 路径;
`build_mesh_draws` 同步 pending draw 时只读取 GPUScene rolled previous,并把有效 previous 传播到 primitive flags、
motion params 与 MeshDraw 的 velocity eligibility。随后 `gpu_scene/prev_skinned_palette.rs` 把 previous skinned palette
也迁入 GPUScene 持久面:mesh draw 同步阶段按 stable key 暂存 current palette state,successful submit 后滚动到
previous 面;draw 侧在 skeleton signature/joint count 匹配时启用 previous palette,CPU-morphed source 还要求
current/previous morph-shape signature 匹配。稳定 morph weights 可进入 object velocity,changing morph weights 的
previous-shape buffer/velocity writer 仍归计划 06 后续切片。

**GS-M4 indirect 提交(capability gate)**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 4.1 | `request_device.rs`、`backend_types.rs` | 两个能力位 + `gpu_driven_submission_supported()` | check 过 |
| 4.2 | `mesh_pass/indirect_draw_batcher.rs`、`build_mesh_draws/indexed_indirect_args.rs`(struct 迁出复用) | CPU-side batch planning 与 fallback 统计 | check 过;batcher 单测已写入 |
| 4.3 | `prepared_queue.rs`、`render.rs`、`frame_submission_context.rs`、`base_stats.rs`、`render_stats_store/product.rs` | batch plan stats 从 frame capability 派生并进入 `RenderStats` / diagnostics | check 过;统计语义文档化 |
| 4.4 | `mesh_pass/replay.rs`、built-in pass recorders、args-buffer owner | 实际 WGPU multi-draw replay 与逐 draw fallback 产物对齐 | 待实施 |

### 测试与验收清单

单测(命名遵循 §8 第 6 条;落点为对应实现文件的 `#[cfg(test)]` 模块):

| 测试函数 | 断言 | 位置 |
|---------|------|------|
| `render_gpu_scene_layout_matches_wgsl_offsets` | `std::mem::size_of::<GpuPrimitiveData>() == 80`、`offset_of!` 与本章偏移表逐项相等 | `gpu_scene/layout.rs` |
| `render_gpu_scene_id_allocator_reuses_freed_spans_without_aliasing` | free 后再 allocate 复用空洞;在 flush 前的同帧内不复用 | `gpu_scene/id_allocator.rs` |
| `render_gpu_scene_id_allocator_coalesces_adjacent_free_spans` | 相邻 span 释放后合并为一段;`high_water` 不回退 | `gpu_scene/id_allocator.rs` |
| `render_gpu_scene_update_queue_merges_adjacent_dirty_ranges` | 间隙 ≤8 条目的脏区间合并;输出字节区间正确 | `gpu_scene/update_queue.rs` |
| `render_gpu_scene_static_scene_second_frame_uploads_zero_bytes` | 同一 extract 连提两帧,第二帧 `flush_updates` 返回 0 | `gpu_scene/gpu_scene/tests.rs`(headless device) |
| `render_gpu_scene_single_moving_entity_uploads_only_its_entry` | 仅 bump 一个 `transform_revision`,上传字节 == 该条目 stride 合并区间 | `gpu_scene/gpu_scene/tests.rs`(headless device) |
| `render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads` | light storage buffer 扩容后首帧上传 lights,重复写入同一 light set 不再产生上传 | `gpu_scene/gpu_scene/tests.rs`(headless device) |
| `render_gpu_scene_rolls_current_transform_into_previous_after_success` | 提交成功后的 roll 将 current 写入 previous,标记下一帧 instance upload,并让 entry previous 状态有效 | `gpu_scene/prev_transform.rs` |
| `render_gpu_scene_roll_marks_previous_valid_without_dirty_upload_when_unchanged` | current/previous 已一致时不产生脏上传,但下一帧可读取有效 previous | `gpu_scene/prev_transform.rs` |
| `render_gpu_scene_buffer_readback_matches_extract` | readback 三缓冲与 CPU shadow 逐字节一致 | `gpu_scene/gpu_scene.rs` |
| `fallback_mesh_shader_reads_gpu_scene_instance_data` | forward fallback vertex/velocity 入口使用 `@builtin(instance_index)` 读取 current/previous transform 与 primitive 参数 | `fallback_mesh_shader_source.rs` |
| `normal_prepass_shader_reads_gpu_scene_instance_data` | normal prepass 顶点入口从 GPUScene 读取 transform 与 motion params,fragment normal map gate 使用传入参数 | `normal_prepass_shader_source.rs` |
| `mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked` | shadow template source 通过 GPUScene/runtime template path 生成 opaque depth 与 alpha-mask variants | `shader_source.rs` |
| `deferred_geometry_shader_reads_gpu_scene_instance_data` | deferred geometry path 从 GPUScene 读取 transform、tint 与 motion params | `deferred/geometry_pipeline/shader_source.rs` |
| `*_shader_is_valid_wgsl` | forward fallback、normal prepass、shadow map、deferred geometry 拼接 `zr_gpu_scene.wgsl` 后通过 Naga WGSL 解析/验证 | 对应 shader source tests |
| `render_gpu_scene_indirect_batcher_groups_by_pipeline_geometry_material` | 同键相邻命令聚为一个 batch;`total_instances` 守恒 | `indirect_draw_batcher.rs` |
| `render_gpu_scene_indirect_batcher_falls_back_per_draw_without_multi_draw` | gate 关闭时 `fallback_draw_count == 命令数` 且 args buffer 不创建 | `indirect_draw_batcher.rs` |
| `mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer` | phase-local execution owner 创建 `INDIRECT` args buffer,packed args 走 `bytemuck::cast_slice` | `indirect_draw_execution.rs` |
| `mesh_draw_command_replayer_records_multi_draw_indexed_indirect_batches` | replayer 对 batch 使用 `multi_draw_indexed_indirect`、`first_args` offset 与 `args_count` | `replay.rs` |
| `mesh_pass_command_buffers_report_indirect_batch_stats_when_gpu_driven_supported` | 多个 phase list 汇总 capability-gated indirect batch stats;default gate 报 fallback | `mesh_draw_command_list.rs` |
| `render_product_diagnostics_record_mesh_indirect_batch_stats` | `render.mesh.queue.indirect_*` 四个统计路径投影到 `DiagnosticStore` | `render_stats_store/product.rs` |
| `render_gpu_scene_skinned_draws_keep_per_draw_palette_binding` | skinned draw 的 command-local scene-data bind group 覆盖真实 palette,非 skinned 共享 frame GPUScene bind group | `create_mesh_draw.rs` tests |

里程碑测试命令:`cargo test -p zircon_runtime gpu_scene --locked`、`cargo test -p zircon_runtime mesh --locked`。

`RenderStats` 新字段(均在 `backend_types.rs`,GS-M3/M4 断言用):`last_gpu_scene_primitive_count`、`last_gpu_scene_instance_count`、`last_gpu_scene_dirty_entry_count`、`last_gpu_scene_uploaded_bytes`、`last_gpu_scene_free_span_count`、`last_indirect_batch_count`、`last_indirect_batched_draw_count`、`last_indirect_fallback_draw_count`、`last_indirect_args_count`。

`render_product_*` 场景:GS-M2 后全量回归逐像素不变(opaque/alpha-mask/transparent/shadow/motion vector 既有场景);GS-M4 新增"同几何 64 实例"场景断言 `last_indirect_batch_count == 1` 且回落路径产物 hash 与 indirect 路径一致;`ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧确认 `gpu_scene_upload` marker 与 multi-draw 调用。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-24 | GS-M1/GS-M3 GPUScene tests owner split | render_plan03_gpu_scene_tests_owner_split_static_passed_cargo_deferred_active_compile_lane | `graphics/scene/gpu_scene/gpu_scene.rs` 从 815 行降到 588 行,只保留 GPUScene storage buffer/data-plane owner、stable registration、primitive/instance/light shadow writes、diff upload 和 scene-data bind group rebuild;新增 150 行 `graphics/scene/gpu_scene/gpu_scene/tests.rs` 承接 static-scene zero upload、single moving entity upload 和 light-buffer growth/unchanged upload headless WGPU tests。结构守卫 `runtime_15_gpu_scene_tests_are_child_owner` 锁定测试不回流、父/子 800 行预算和 docs/status 锚点。 | scoped rustfmt/static owner scans、line-count scan、docs-anchor scan、touched-file whitespace scan 和 scoped diff-check 通过;Cargo/WGPU/RenderDoc 因 active compile lanes 暂缓且不计通过。 | Cargo lane 空闲后补跑 gpu_scene focused guard、headless WGPU upload tests、Plan 03 GPUScene product diagnostics 和 RenderDoc multi-draw 验收。 |
| 2026-06-24 | GS-M1/GS-M4 GPUScene root façade suppression cleanup / F12 | render_plan03_gpu_scene_root_facade_suppression_cleanup_static_passed_cargo_timeout_active_compile | `gpu_scene/mod.rs` 删除整棵子树级 `#![allow(dead_code, unused_imports)]`,并把根 façade 收窄到外部渲染路径实际消费的 `GpuScene`/entry/stats/upload report、Pod layout struct/stride/flag、previous skinned state。bind group helper、allocator、dirty queue、capacity 常量、layout offset 常量和 roll-report 类型回到各自 child owner,不再靠根 re-export 维持脚手架。 | `rustfmt --edition 2021 --check zircon_runtime\src\graphics\scene\gpu_scene\mod.rs` 通过;GPUScene 子树 `allow(dead_code)`/`allow(unused_imports)`/`#![allow]` 扫描零命中;外部用法扫描确认 binding helper、allocator/update queue 与 offset 常量没有跨 GPUScene 边界消费者。`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 在 180s 工具窗口超时,且后续仍有 cargo/rustc 进程活跃,不计 Cargo/WGPU/RenderDoc 通过。 | 若后台 Cargo 后续暴露新的 dead-code/visibility 警告,按 F12 子 owner 继续处理;GS-M4 仍需 focused gpu_scene/mesh lib-test、real-adapter WGPU pipeline、render-product 和 RenderDoc multi-draw 验收。 |
| 2026-06-23 | Render index 当前状态总览拆分 | GS-M1~M3 已完成,GS-M4 部分完成 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 03 GPUScene/GPU-driven 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：GPU-decided draw count 与更高阶 submit 留到计划 19；验收缺口：需要 real-adapter WGPU pipeline、render-product 逐像素回归、RenderDoc multi-draw 确认 |
| 2026-06-15 | GS-M1 GpuScene data plane | 已完成(数据面接入;shader ABI 当时仍待 GS-M2) | `graphics/scene/gpu_scene/` 模块、Pod 布局镜像、span allocator、dirty range 合并、`GpuScene` owner、primitive/instance/light storage buffers、CPU shadow、stable key 登记、容量扩容、direct `queue.write_buffer` flush 与 upload report 已接入;`RenderMeshSnapshot` 携带 `stable_instance_key`/`transform_revision`,compiled scene 与 legacy mesh draw 构建路径均登记 GPUScene 条目并透传统计。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 通过;`prepared_queue_stats_carry_gpu_scene_counts` 过滤测试通过;`gpu_scene` 宽过滤测试在 604s shared lib-test 编译阶段超时,未计为通过。 | staging ring / render graph upload node 保留为后续优化;shader/draw ABI 已由 GS-M2 硬切。 |
| 2026-06-15 | GS-M2 instance-index shader ABI hard cut | 已完成(运行时 ABI 已切换;focused shader 重跑待补) | 最终 group3 GPUScene bind group layout 落地:binding 0/1/2 为 primitive/instance/light storage,3/4 为 current/previous skinned palette uniform;内建 forward fallback、normal prepass、shadow map、deferred geometry 和 PBR shader 拼接 `zr_gpu_scene.wgsl`,顶点入口改用 `@builtin(instance_index)` 读取 GPUScene transform/primitive/material 参数;旧 `ModelUniform`、model uniform cache、model bind replay、旧 group4/group5 与 material-uniform-only bind group owner 删除,material set 收束到 group2。 | scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 通过(88 个既有 warning);此前 `shader_is_valid_wgsl`、`skinned_joint_palette`、`shader_declares_gpu_scene_group`、fallback shader ABI 与 command span focused tests 已通过;最终槽位后的 focused shader test 重跑在并行编译负载下超时。 | 仍需 real-adapter WGPU pipeline 创建、render-product 逐像素回归与插件 shader 适配验收。 |
| 2026-06-15 | GS-M3 direct-write diff upload and diagnostics | 已完成(V1 direct queue-write 策略明确) | `PendingMeshDraw` 使用 extract `transform_revision`;`GpuScene::write_primitive`/`write_instances` 仅在 CPU shadow 真实变化时标脏;静态第二帧上传 0 字节,单个移动条目只上传一个 instance stride;`GpuSceneUploadPath::DirectQueueWrite`、upload range/bytes、dirty/free stats 通过 `PreparedMeshQueueStats`、`RenderStats` 与 render-product diagnostics 透出。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` 通过(88 个 warning);`render_gpu_scene_` 过滤测试通过 7/7;`render_product_diagnostics_record_gpu_scene_upload_stats` 通过 1/1;runtime diagnostics owner fixture 已改用 `crate::graphics::GRAPHICS_MODULE_NAME`,但完整 diagnostics 重跑被活跃 plugin session 编译漂移阻塞。 | 后续上传优化阶段再决定 staging ring / render graph upload node;真实 render-product 帧日志稳定性仍待更宽验收。 |
| 2026-06-15 | GS-M4 indirect submission capability and WGPU replay | 部分完成: CPU planning + WGPU fixed-count multi-draw replay 已接入,产品/RenderDoc 验收待后续 | RHI/wgpu capability flags、`RenderCapabilitySummary::gpu_driven_submission_supported()`、`IndirectDrawBatcher`、phase-local WGPU indirect args buffers、`MeshDrawCommandStream` 与 `MeshDrawCommandReplayer::replay_command_stream(...)` 已接入;depth-prepass/base/shadow/deferred-gbuffer/velocity recorders 保留 fallback draw order,对 eligible CPU-planned batches 使用 `multi_draw_indexed_indirect`;indirect batch stats 写入 `RenderStats` 与 `render.mesh.queue.indirect_*` diagnostics。 | `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 多次通过(89 个既有 warning);`rustfmt --edition 2021 --check`、`git diff --check` 与 touched-file trailing-whitespace scan 通过;`render_gpu_scene_indirect_batcher` 与 `mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer` 过滤测试在 shared lib-test 编译阶段超时,未返回结果。 | 计划 04 VC-M3 已消费该 args buffer 能力做 HZB compact replay;GS-M4 仍需 real-adapter/product 对拍与 RenderDoc multi-draw 调用确认。 |

### 参考实现精读笔记

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.h`:

- `FGPUScene` 持有四块持久缓冲:`PrimitiveBuffer`、`InstanceSceneDataBuffer`、`InstancePayloadDataBuffer`、`LightmapDataBuffer`,各配一个 `FRDGAsyncScatterUploadBuffer`;光照走独立 `TPersistentByteAddressBuffer<FLightSceneData> LightDataBuffer`。Zircon 对应物:`GpuScene` 三缓冲(payload/lightmap 合并为 `payload_slot` 预留位,V1 不建独立缓冲——场景规模未到,砍掉一层间接)。
- id 分配用 `FSpanAllocator InstanceSceneDataAllocator`(连续 span,支持回收)与 `GrowOnlySpanAllocator` 可选策略(cvar `GPUSceneUseGrowOnlyAllocationPolicy`);`FGPUSceneInstanceRange` 记录 (primitive, offset, count)。Zircon:`GpuSceneIdAllocator` 单类型同时覆盖单槽与 span,first-fit + 邻接合并,不做 grow-only 策略开关。
- 脏跟踪:`AddPrimitiveToUpdate(index, EPrimitiveDirtyState)` 标记可累积(`dirty-flags are cumulative`),`TArray<EPrimitiveDirtyState> PrimitiveDirtyState` 与 `PrimitivesToUpdate` 分离存储;`Removed` 会清掉 `Added`。Zircon:`GpuSceneUpdateQueue` 仅收 index + 区间,语义级 dirty(transform/params)在 `render_scene.rs` 差分时就地解析,不留状态机——extract 是全量快照,revision 比对比 UE 的回调式标脏更简单。
- 动态 primitive 走 `FGPUScenePrimitiveCollector::Add/Commit` 帧内追加分配(`DynamicPrimitivesOffset`)。Zircon V1 不区分动态段:所有条目同一 allocator,skinned/morph 每帧重写自己的 instance 槽。
- `bUseTiledInstanceDataLayout` + `InstanceDataTileSizeLog2`:instance 数据按 tile 重排以优化 scatter。Zircon 放弃 tile 与 float4-SOA(见下),AoS 直拍。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp`:

- `FUploadDataSourceAdapterScenePrimitives`(L439)抽象上传源:`NumPrimitivesToUpload` / `GetPrimitiveInfoHeader` / `GetPrimitiveShaderData` / `GetInstanceInfo`,使场景 primitive 与动态 primitive 共用 `UploadGeneral`(L1221)。`UploadGeneral` 先汇总各类上传计数、`Begin/BeginPreSized` 预分配 scatter uploader,再 `ParallelForTemplate` 并行填充,每 instance 按 `CalcInstanceDataIndex(slot, RefIndex)` 算 SOA scatter 偏移。要点:上传与录制解耦、scatter 以"条目"为粒度。Zircon 取舍:V1 不做 scatter compute,合并区间 + `copy_buffer_to_buffer`/`write_buffer` 已满足"静态稳态零上传";区间粒度比条目粒度命令数少,代价是脏条目稀疏时多传间隙字节(间隙阈值 8 控制)。adapter 抽象不引入——只有 extract 一个数据源。
- `GetInstanceInfo` 区分有 `InstanceSceneDataBuffers` 的实例化 primitive 与"old path"单实例 primitive(`NumInstances = 1`,直接用 `PrimitiveToWorld`)。Zircon 同构:非实例化 mesh 注册 `instance_count == 1`,变换写 instance 槽。
- `FInstanceSceneShaderData::GetDataStrideInFloat4s()`/`SupportsCompressedTransforms()`:UE instance 数据是 float4 SOA 数组并可压缩变换。Zircon:WGSL 类型化 storage buffer 不便做 SOA float4 视图,采用 AoS struct;放弃压缩变换(fp32 mat4 直存),fp16 留待能力检测成熟后再议(§8 第 2 条)。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/GPUSceneWriter.h`:

- `FGPUSceneWriterParameters` 暴露 `RWStructuredBuffer<float4>` 的 `GPUSceneInstanceSceneDataRW` 等 UAV + `GPUSceneInstanceSceneDataSOAStride`/`GPUSceneNumAllocatedInstances`,供 compute 在 GPU 侧直写场景数据;`FGPUSceneWriteDelegate` 把写回调挂到指定 `EGPUSceneGPUWritePass` 延迟执行。Zircon 对应物:V1 不开放 GPU 写 ABI;计划 04 的剔除 compute 只改写 indirect args 的 `instance_count`,不写 GpuScene 本体;VG/HGI 以只读消费者身份读 group3 缓冲。取舍:少一套 RW binding 与同步面,GPU 粒子/程序化实例需要时再按本文件形态补 `zr_gpu_scene_writer.wgsl`。

`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/InstanceCulling/InstanceCullingManager.cpp`:

- `FInstanceCullingManager::BeginDeferredCulling` 在 `GPUScene.GetNumInstances() == 0` 或无 culling view 时直接跳过;`AllowBatchedBuildRenderingCommands` 同时检查 `GPUScene.IsEnabled()` 与 immediate mode —— 剔除批处理整体受 GPUScene 可用性 gate。`GetBinIndex` 按 view 的 prev-HZB 分 bin(bin 0 = `EBatchProcessingMode::UnCulled`)。对 Zircon 的意义:`IndirectDrawBatcher` 的 gate 判定(`gpu_driven_submission_supported()`)要在 batcher 构建前短路,而非提交时;HZB bin 组织归计划 04,本计划的 args buffer 以 STORAGE 用途预留其改写入口即可。

跨计划冲突点备注:GS-M2 的 §8 槽位重排已把 GPUScene 落到 group3,并把 shadow 采样移动到 group1；2026-07-03 SH02-SH04 进一步把 material property uniform 归位到 group2 binding 0,标准贴图/采样器使用 group2 binding 1..10。计划 08 的 shader 模板拼接与计划 05 的 shadow 绑定应在这些既定槽位内继续填充。`sort_key` 位段与 light grid 布局本章未定义,分别归计划 09 与 05。

## 风险与回退

- shader ABI 大改波及全部内建 WGSL 与插件 shader:GS-M2 一次性切换,以 `render_product` 全量对拍守护;插件 shader 在同里程碑适配。
- wgpu 对 multi-draw indirect 仅 native 后端支持:gate 设计已覆盖;WebGPU 目标走回落路径,不作为本计划验收项。
- prev transform 槽位与 guarded previous skinned-palette state 已由计划 06 TP-M1 接入 GPUScene 帧末滚动;stable CPU-morphed shape velocity 已由 morph-shape signature gate 覆盖;剩余风险转为 changing morph-weight previous-shape buffer、particle velocity、TAA/history 与 jitter 后续切片。
