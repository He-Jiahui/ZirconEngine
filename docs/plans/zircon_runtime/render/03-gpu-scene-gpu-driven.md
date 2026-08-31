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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_scene_instance_data
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_executes_skinned_joint_palette_behind_draw_flag
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/processors/tests.rs::render_mesh_draw_processor_shadow_excludes_non_casters_and_picks_alpha_mask_variant
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs::mesh_batch_ref_emits_gpu_scene_instance_command
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs::prepared_queue_stats_carry_gpu_scene_counts
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs::render_product_diagnostics_record_gpu_scene_upload_stats
status: in_progress
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

### 2026-08-26 persistent-scene hard-cut amendment

本修订覆盖下方历史 `GpuSceneIdAllocator`、`resource_streamer.ensure` primitive 登记和
frame-extract 直接驱动 GPUScene 的描述。旧代码在产品切换完成前仍可能物理存在，但不再是目标
architecture authority：

- `RenderScenePrimitiveHandle(slot, slot_generation)` 是唯一 persistent primitive identity；
  GPUScene 直接投影同一 slot，不再分配第二套 primitive ID 或 stable-key map；
- `RenderSceneChangeJournal` 是增量数据面，addition/update/removal 携带 immutable primitive，
  update 同时保留 before/after `Arc`；
- journal 一次封存 base/all-LOD model/mesh/material、primitive binding、material override 与
  skeleton 的 deterministic net typed-resource reference delta，供 09D 唯一 residency authority
  消费；RenderScene/GPUScene 不持有 residency ticket/cache；
- `ResourceStreamer::ensure_scene_resources` 的同步 load/clone/WGPU-create 路径是待硬删迁移源，
  禁止再承担 primitive 注册或 RenderScene resolver；
- 稳态复杂度按 changed frontier `C`，identity lookup/second allocator 为 0；全量排序/重投影只允许
  initial/full-resync/device-recovery 路径。

代码与证据见
[`03/2026-08-26-persistent-render-scene-generation-architecture.md`](03/2026-08-26-persistent-render-scene-generation-architecture.md)
和
[`03/2026-08-26-render-scene-resource-dependency-delta-review.md`](03/2026-08-26-render-scene-resource-dependency-delta-review.md)。

核心类型:

- `GpuScene`:`primitive_data: StorageBuffer`(transform、prev transform、bounds、flags、lightmap/payload 槽)、`instance_data`、`light_data`(计划 05 消费)。SOA 打包对齐 16 字节；primitive row 直接使用 persistent RenderScene slot/generation，旧 `GpuSceneIdAllocator` 仅作为待清退迁移实现。
- `GpuSceneJournalConsumer`:消费 `RenderSceneChangeJournal`，按 direct slot 生成去重 full/dirty/retirement work；大批量 initial/recovery 走整段上传，稳态仅处理 journal touched rows。
- shader ABI:bind group 布局新增 scene data group(`pipeline_layout.rs` 收口);顶点着色器以 `instance_index`(来自 draw args first_instance 或 instance step 顶点输入)取 transform。计划 02 的 `DrawInstanceSource` 在此切换为 instance index 路径并删除 uniform 路径。
- `IndirectDrawBatcher`:同 (pipeline, geometry, material set) 的命令聚为一个 indirect batch;CPU 填充 args 为基线,计划 04 的剔除 compute 直接改写 args 中 instance_count。
- capability gate:`RenderCapabilityClass` 增加 gpu_driven 档位;不支持 multi-draw indirect 时 batcher 退化为逐条 `draw_indexed`(仍用 instance index ABI,只是提交方式不同),不维护两套 shader ABI。

插件衔接:VG/HGI 的 provider 改为注册"场景数据消费者",直接读 `GpuScene` 缓冲;私有重复缓冲在各自计划内清退。

## 里程碑

### GS-M1 GpuScene 数据面

实施切片:
1. `RenderScene` persistent slot/generation、immutable journal 与 GPUScene direct-slot consumer。
2. journal 发布完整 typed resource-reference delta；09D 唯一 residency manager 以 generation-bound
   non-blocking ticket 解析 all-LOD geometry，再进入 WGPU capacity/upload staging。
3. 硬删 `resource_streamer.ensure` primitive 登记、GPUScene 第二 ID allocator/stable-key map 与旧
   pending-draw ownership。

测试阶段:
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter gpu_scene`（布局/分配/回收单测）
- 验收证据:缓冲内容与 extract 对拍一致(readback 测试);id 回收无串台。

### GS-M2 instance index 着色路径

实施切片:
1. scene data bind group 接入 SRP 布局;内建 mesh shader(含 fallback)改为 instance index 取变换。
2. 计划 02 的 `DrawInstanceSource` 切换并删除 uniform 路径;skinning palette 走同一 scene-data ABI 的 binding 3/4,真实 per-skinned-draw palette bind group 在本里程碑内补齐。

当前进度(2026-06-12):`GpuScene` 已拥有 primitive/instance/light storage bind group layout 与 bind group;生产 `build_mesh_draws` 会在同步 GPUScene 后保留每个 stable instance key 对应的 `GpuSceneEntry`,并把 `(first_instance_index, instance_count)` 回挂到 `MeshDraw`。`MeshDraw::mesh_pass_batch_ref` 继续传递该 span,`MeshBatchRef::command` 只生成 `DrawInstanceSource::GpuSceneInstance` 和带 first-instance 的 direct draw args。GPUScene 命令不再携带 fallback object bind,`MeshDrawCommandReplayer` 的 model-bind helper 已删除,四条内建 WGSL 已不再声明 `model_data` 作为变换来源。

后续 GS-M2 切片已把同一个 `GpuScene::scene_bind_group_layout()` 注入 forward mesh pipeline cache、normal prepass、shadow map 与 deferred geometry pipeline layout,并完成最终物理槽位收束:group0 为 scene,group1 为 forward shadow receiver(仅需要的 pass 安装),group2 为 material set,group3 为 GPUScene scene-data。`render_compiled_scene` 克隆当前 GPUScene bind group 并放入 `RenderPassMeshCommandLists`,prepass/base/shadow/deferred/velocity pass 通过 `MeshDrawCommandReplayer::bind_gpu_scene_if_needed` 在 draw replay 中按需绑定 group3;legacy `render_scene` 路径也把同一 handle 传入 overlay mesh recording。GPUScene layout 按计划 ABI 暴露 binding 0/1/2 primitive/instance/light storage,以及 binding 3/4 current/previous skinned joint palette uniform。`GpuScene::new` 接收 renderer 现有空 palette fallback buffer,非 skinned draw 的共享 group3 bind group 绑定 fallback palette,skinned draw 可通过 `GpuScene::create_scene_bind_group_for_palettes(...)` 创建 command-local scene-data bind group,复用同一批 storage buffers 并只覆盖真实 current/previous palette buffer。`mesh/shaders/zr_gpu_scene.wgsl` 定义 `ZrGpuPrimitiveData` / `ZrGpuInstanceData` 与 transform、primitive 参数、current/previous palette helper,并拼接进 forward fallback、normal prepass、shadow map、deferred geometry 和内置 PBR shader source。四条内建 mesh shader 的顶点入口通过 `@builtin(instance_index)` 读取 GPUScene transform,velocity 入口同时读取 previous transform;primitive tint、shadow params 与 motion params 从 `GpuPrimitiveData` 读取并经 vertex output 传入 fragment 阶段。`ModelUniform`/`model_data`、旧 pass-local `SkinnedJointPaletteUniform`/`@group(1)` palette 声明、Rust 侧 `ModelUniformCache`、`ModelUniform`、`MeshDraw::{model_buffer, model_bind_group}`、pending model cache key、fallback object bind 与 model-bind replay helper 均已删除。

最终槽位切片还把 material textures/samplers 与 `material_properties` 合并为 group2 material set(binding 0-9 为贴图/采样器,binding 10 为材质 uniform),删除旧 material texture 单独绑定路径,并把 material/custom shader ABI 诊断迁移为 group2 material + group3 GPUScene 校验。项目渲染测试生成的自定义 WGSL 已改为拼接 `zr_gpu_scene.wgsl`,通过 `@builtin(instance_index)` 读取 transform/tint,并声明 group2 binding10 material uniform。旧 material-uniform-only bind group owner 也已删除,material uniform 资源现在只保留 uniform buffer,由 group2 material set bind group 引用。已运行的验证包括此前重新编译后的 `shader_is_valid_wgsl`(4 项)、直接 lib-test 二进制 `skinned_joint_palette`(5 项)与 `shader_declares_gpu_scene_group`(4 项)、`fallback_mesh_shader`、`reads_gpu_scene_instance_data`、command span focused test、rustfmt,以及 scene_renderer 静态扫描无剩余 model-uniform/cache/replay、group4/group5、旧 material texture bind helper 或 mesh-compat 引用。最新 scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` 通过并报告 89 个 warning。最终槽位后的 `cargo test -p zircon_runtime --lib shader_declares_gpu_scene_group ...` 由于其它会话并行链接 lib-test 二进制而在编译阶段超时,未返回测试结果。GS-M3 V1 direct-write 增量上传已完成;GS-M4 已完成 capability gate、CPU indirect batch planning 和诊断统计接线。尚未完成的是 final layout 的 focused shader/ABI lib-test 重跑、GS-M4 WGPU multi-draw replay/args-buffer 执行、real-adapter WGPU pipeline 创建与 render-product 验证。

测试阶段:
- 分别运行 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter mesh` 与同脚本的 `-TestFilter render_product`。
- 验收证据:渲染产物逐像素不变;draw 间 bind group 切换计数显著下降(统计断言)。

### GS-M3 增量上传

实施切片:
1. 脏队列合并与 scatter 上传;变换静态对象零上传。
2. 上传字节数进 RenderStats。

测试阶段:
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter gpu_scene`（静态场景第二帧上传字节为 0；单对象移动只传该条目）
- 验收证据:大场景稳态上传量统计记入文档。

### GS-M4 indirect 提交(capability gate)

实施切片:
1. `RenderCapabilitySummary` / RHI / wgpu backend 增加 `supports_multi_draw_indirect` 与 `supports_indirect_first_instance`,并用 `gpu_driven_submission_supported()` 统一 gate。
2. `IndirectDrawBatcher` 做 CPU-side batch planning:gate 关闭时逐 draw fallback,gate 开启时把 eligible direct indexed commands 转成 `IndexedIndirectArgs` 并按相邻 phase/pipeline/geometry/material/GPUScene bind identity 聚批。
3. `render_compiled_scene` 把 frame capability summary 传入 `MeshPassCommandBuffers::stats_with_indirect_batches(...)`;`PreparedMeshQueueStats`、`RenderStats.last_indirect_*` 和 `render.mesh.queue.indirect_*` diagnostics 记录 batch plan。
4. `MeshPassIndirectDrawExecutions` 为 depth-prepass、shadow、opaque、alpha-mask、transparent、velocity phase 分配 phase-local WGPU indirect args buffer;built-in mesh pass replay 通过 `MeshDrawCommandStream` 在 eligible batches 上调用 `multi_draw_indexed_indirect`,并保留逐 draw fallback。
5. 后续验证切片:同几何多实例 render-product 场景、回落路径产物一致性、RenderDoc 抓帧确认 multi-draw 调用与真实 adapter coverage。

测试阶段:
- 分别运行 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter mesh` 与同脚本的 `-TestFilter gpu_scene`。
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
| `zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs` | 迁移期 legacy instance-span allocator；primitive identity 硬切 direct RenderScene slot 后删除其 primitive 职责 |
| `zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs` | 迁移期 instance/morph 辅助 dirty queue；primitive row 由 `RenderSceneChangeJournal` direct-slot work 驱动 |
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
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs` | 迁移 hard cut：删除同步 ensure 的 primitive 登记；journal typed-resource delta 改由 09D manager 在提交锁域外消费 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/create_mesh_draw.rs` | 删除 model uniform/bind group 构建,改写为依赖 GPUScene entry span;skinned draw 创建 command-local GPUScene palette bind group |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs` | `MeshDraw` 删除 `model_buffer` / `model_bind_group` 字段,保留 GPUScene instance span 与可选 command-local GPUScene bind group |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs` | 已从 frame path 删除;command replay 直接以 GPUScene span 的 first_instance 提交 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs`、`prepass/normal_prepass_pipeline/new.rs`、`shadow/shadow_map_renderer.rs`、`deferred/geometry_pipeline/create.rs` | pipeline layout 使用最终槽位:group0 scene、group1 shadow receiver(需要时)、group2 material set、group3 GPUScene |
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

graphics 实现层(`graphics/scene/gpu_scene/`,可用 wgpu)。下方代码块记录旧迁移实现形状；
primitive `HashMap`/allocator 部分已被本节 hard-cut amendment 覆盖，不得作为新增实现依据：

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

`GpuPrimitiveData`(stride 96 字节):

| 字段 | WGSL 类型 | 字节偏移 | 说明 |
|------|-----------|---------|------|
| `local_bounds_center` | `vec3<f32>` | 0 | primitive local-space 保守包围球心，由 instance transform 唯一投影 |
| `local_bounds_radius` | `f32` | 12 | 填入 vec3 的 padding lane,无空洞 |
| `tint` | `vec4<f32>` | 16 | 迁自 `ModelUniform.tint` |
| `shadow_params` | `vec4<f32>` | 32 | lane 语义与现 `ModelUniform.shadow_params` 一致(alpha_mask、cutoff、receive_shadows、0) |
| `motion_params` | `vec4<f32>` | 48 | lane 语义与现 `ModelUniform.motion_params` 一致 |
| `flags` | `u32` | 64 | bit0 visible、bit1 cast_shadows、bit2 has_prev_transform、bit3 force_hzb_visible |
| `first_instance_index` | `u32` | 68 | 指向 instance_data 起始槽 |
| `instance_count` | `u32` | 72 | |
| `payload_slot` | `u32` | 76 | primitive/VG payload 槽，无效值为 0xFFFFFFFF |
| `material_payload_slot` | `u32` | 80 | bindless material payload 槽，无效值为 0xFFFFFFFF |
| `material_payload_padding` | `vec3<u32>` 的三个标量 lane | 84 | 显式 storage-array stride 尾部，Rust 使用 `[u32; 3]` |

`GpuInstanceData`(stride 176 字节):

| 字段 | WGSL 类型 | 字节偏移 | 说明 |
|------|-----------|---------|------|
| `world_from_local` | `mat4x4<f32>` | 0 | |
| `prev_world_from_local` | `mat4x4<f32>` | 64 | prev transform 槽:本计划随上传写入(保持现有 motion vector 行为),双缓冲/jitter 精化归计划 06 的 `TaaResolveExecutor` |
| `primitive_index` | `u32` | 128 | 反向索引 primitive_data |
| `flags` | `u32` | 132 | affine normal transform、负行列式、退化与 shear 分类位 |
| `payload_slot` | `u32` | 136 | instance/VG payload 槽，无效值为 0xFFFFFFFF |
| `morph_payload_slot` | `u32` | 140 | morph payload header 槽，无效值为 0xFFFFFFFF |
| `lightmap_uv_rect` | `vec4<f32>` | 144 | lightmap atlas UV scale/offset |
| `lightmap_params` | `vec4<u32>` | 160 | atlas page、有效位与 light-set generation 高低位 |

`light_data`:`array<GpuLightData>`,本计划只负责缓冲创建、id 分配与 binding 槽;`GpuLightData` 条目布局与写入由计划 05 定义。

group3 binding 编号(group0–2 语义见 §8 第 1 条;2026-07-03 SH02-SH04 已完成槽位归位:标准贴图/采样器位于 group2 binding 0..9，material property uniform 位于 group2 binding 10，shadow receiver 采样位于 group1):

| binding | 资源 | 类型 | 说明 |
|---------|------|------|------|
| 0 | `zr_primitive_data` | `var<storage, read>` | `array<GpuPrimitiveData>` |
| 1 | `zr_instance_data` | `var<storage, read>` | `array<GpuInstanceData>` |
| 2 | `zr_light_data` | `var<storage, read>` | packed light rows，空场景绑定 fallback storage |
| 3 | `zr_skinned_joint_palette` | `var<storage, read>` | current skinned palette;非 skinned 绑 fallback |
| 4 | `zr_previous_skinned_joint_palette` | `var<storage, read>` | previous skinned palette;非 skinned 绑 fallback |
| 5 | `zr_visible_instance_remap` | `var<storage, read>` | GPU culling 后的可见 instance 重映射；direct 路径绑定 fallback |
| 6 | `zr_visible_instance_remap_params` | `var<uniform>` | remap gate 与 light/VG 有效 row count |
| 7 | `zr_morph_deltas` | `var<storage, read>` | morph delta rows |
| 8 | `zr_morph_weights` | `var<storage, read>` | morph weight rows |
| 9 | `zr_virtual_geometry_pages` | `var<storage, read>` | VG page table rows |
| 10 | `zr_virtual_geometry_clusters` | `var<storage, read>` | VG cluster payload rows |
| 11 | `zr_morph_payloads` | `var<storage, read>` | morph payload headers |

非 skinned draw 共享 `GpuScene` 持有的唯一 group3 bind group;skinned draw 以相同 layout 创建携带真实 palette 的 per-draw bind group(skinned 本就走 dynamic 列表,不破坏静态命令缓存)。

`zr_gpu_scene.wgsl`(只含 struct 与函数,无 entry point):

```wgsl
struct ZrGpuPrimitiveData {
    local_bounds_center: vec3<f32>, local_bounds_radius: f32,
    tint: vec4<f32>, shadow_params: vec4<f32>, motion_params: vec4<f32>,
    flags: u32, first_instance_index: u32, instance_count: u32, payload_slot: u32,
    material_payload_slot: u32,
    material_payload_padding_0: u32,
    material_payload_padding_1: u32,
    material_payload_padding_2: u32,
}
struct ZrGpuInstanceData {
    world_from_local: mat4x4<f32>, prev_world_from_local: mat4x4<f32>,
    primitive_index: u32, flags: u32, payload_slot: u32, morph_payload_slot: u32,
    lightmap_uv_rect: vec4<f32>, lightmap_params: vec4<u32>,
}
@group(3) @binding(0) var<storage, read> zr_primitive_data: array<ZrGpuPrimitiveData>;
@group(3) @binding(1) var<storage, read> zr_instance_data: array<ZrGpuInstanceData>;

fn zr_gpu_scene_instance(instance_index: u32) -> ZrGpuInstanceData { return zr_instance_data[zr_gpu_scene_resolve_instance_index(instance_index)]; }
fn zr_gpu_scene_primitive(instance: ZrGpuInstanceData) -> ZrGpuPrimitiveData { return zr_primitive_data[instance.primitive_index]; }
fn zr_world_from_local(instance_index: u32) -> mat4x4<f32> { return zr_gpu_scene_instance(instance_index).world_from_local; }
fn zr_previous_world_from_local(instance_index: u32) -> mat4x4<f32> { return zr_gpu_scene_instance(instance_index).prev_world_from_local; }
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
| 1.1 | `gpu_scene/{mod,layout,id_allocator}.rs`、`graphics/scene/mod.rs` | Pod 镜像 struct + offset 常量;span free list 分配器 | `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest` 通过；layout 常量与本章表格一致 |
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
| `render_gpu_scene_layout_matches_wgsl_offsets` | `std::mem::size_of::<GpuPrimitiveData>() == 96`、`std::mem::size_of::<GpuInstanceData>() == 176`、`offset_of!` 与本章偏移表逐项相等 | `gpu_scene/layout.rs` |
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

里程碑测试命令：分别运行 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter <filter>`，其中 `<filter>` 为 `gpu_scene`、`mesh`。

`RenderStats` 新字段(均在 `backend_types.rs`,GS-M3/M4 断言用):`last_gpu_scene_primitive_count`、`last_gpu_scene_instance_count`、`last_gpu_scene_dirty_entry_count`、`last_gpu_scene_uploaded_bytes`、`last_gpu_scene_free_span_count`、`last_indirect_batch_count`、`last_indirect_batched_draw_count`、`last_indirect_fallback_draw_count`、`last_indirect_args_count`。

`render_product_*` 场景:GS-M2 后全量回归逐像素不变(opaque/alpha-mask/transparent/shadow/motion vector 既有场景);GS-M4 新增"同几何 64 实例"场景断言 `last_indirect_batch_count == 1` 且回落路径产物 hash 与 indirect 路径一致;`ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧确认 `gpu_scene_upload` marker 与 multi-draw 调用。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`03/2026-07-09-gpu-scene-gpu-driven-output-records.md`](03/2026-07-09-gpu-scene-gpu-driven-output-records.md)
- 2026-08-26 P0-1 局部进展：已按 UE `FScene`/`FPrimitiveSceneInfo`/`FScenePrimitiveUpdates`/`FGPUScene` 责任分工新增独立 CPU `RenderScene` owner。复核 extract 全链后确认 camera LOD 会替换完整 model/mesh/material/primitive source，故 primitive 已修正为 component-level identity，持有 base + 全部 LOD 的 camera-neutral source，view 侧 O(log L) 选择，不再仅排除 `mesh_lod` 标记；同时包含与 static-mesh eligibility 解耦的显式 revision、稳定代际 handle、密集 payload、O(1) swap-remove relocation、确定性增量 delta 与不可变 change journal。dirty-domain 已进一步收敛：LOD threshold-only 仅失效 view selection，bounds-only 不再误触 geometry resolution，geometry-only 不再上传未变 bounds，mobility/cast-shadow/alpha phase 正确失效 view relevance，而同一 Mask phase 的 cutoff 修改保持 material-only；mask cutoff 构造边界也与资产层统一为 `[0,1]`，非有限 material-property override 在进入 persistent/GPU owner 前拒绝。共享 `RenderSceneJournalCursor` 已实现预检不推进、成功后显式 commit、same-world exact replay no-op 及 cross-world/stale/gap/inverted/non-adjacent/overlap/superseded typed error，world identity 从 `RenderScene` 贯穿 read view、journal 与 preflight token；journal stats 在分类主循环内封存七类 dirty-domain entry count 以及 live/high-water/reusable-hole/exhausted-slot/fragmentation storage count，不为 diagnostics 二次扫描 updates。RenderScene 子树共 42 个 focused tests 已编写，scoped rustfmt/结构扫描通过；GPUScene journal consumer 的 identity/transaction/retirement/counter 设计已完成，但 Runtime04 world-owned dirty-entity producer、consumer 产品接线、managed Cargo、产品 WGPU/RenderDoc/PNG 与性能基线仍未完成，故 GS-M1 与总计划保持 `in_progress`。证据见 [`03/2026-08-26-persistent-render-scene-generation-architecture.md`](03/2026-08-26-persistent-render-scene-generation-architecture.md)与[`03/2026-08-26-render-scene-gpu-scene-journal-consumer-review.md`](03/2026-08-26-render-scene-gpu-scene-journal-consumer-review.md)。
- 2026-08-26 P0-3 局部进展：GPUScene/HZB 已硬切为“primitive local bounds + instance transform”单次变换 ABI，非法/skin/变化 morph/shear 路径保守 fail-open；CPU visibility 共享 bounds authority、managed WGPU/Naga、RenderDoc PNG 与性能基线仍未完成，故计划保持 `in_progress`。证据见 [`03/2026-08-26-gpu-scene-local-bounds-hzb-abi.md`](03/2026-08-26-gpu-scene-local-bounds-hzb-abi.md)。

- 2026-08-26 P0-1 补充进展：primitive 构造现强制提供 base 与每个 LOD 的 local bounds，数量不匹配或非法 AABB typed reject，并只持久化保守 union；LOD threshold-only 与 geometry dirty 已分离；新增中立 skeleton/current pose 输入并直接复用 Runtime04 sealed `AnimationPoseHandle`，pose 变化仅发布 `DEFORMATION | BOUNDS`，非有限骨骼 transform typed reject。journal/read-view 新增 O(1) live/high-water/reusable-hole/exhausted-slot/fragmentation 统计；cursor token 已不可伪造且 commit 二次校验 range，拒绝跨代与宽重叠 journal。RenderScene 子树现有 42 个 focused tests；统一 residency authority、Runtime04/GPUScene 产品接线、managed Cargo、WGPU/RenderDoc/PNG 与性能基线仍未完成，状态保持 `in_progress`。
- 2026-08-26 P0-2 消费事务进展：新增 `gpu_scene/journal_consumer.rs` 独立 CPU 驻留 owner，直接以 persistent RenderScene slot/generation 验证 additions/updates/removals，使用仅覆盖本 journal 驻留变化槽的有序稀疏投影，复杂度为 `O(C log C)`、临时状态 `O(C)`，不建立第二份 stable-key map 或 primitive allocator；remove+add 同 journal 复用、exact replay、错 key 与 stale plan 均保持 typed/atomic。apply plan 现同时投影 slot-ordered full/dirty resident writes 与旧 generation retirements：addition 全域写、update 保留 exact dirty flags、same-slot reuse 同时保留旧退役和新写入，primitive 直接借用 immutable journal；direct-slot validation、projected resident/high-water、full/dirty/retirement 和 stable-key lookup=0 计数均可直接读取。产品入口已收敛为唯一 `apply_with_staging` 事务门，内部 preflight/commit 不再暴露给 sibling owner；staging 失败不推进 cursor/residency，成功返回 typed staging output，exact replay 不调用 staging。设备恢复另有 typed full reprojection：只接受与 consumer 一致的 world/generation/high-water/resident slots，按 persistent slot 输出全量写且不重置 CPU generation；`O(N log N)`/`O(N)` 仅发生在恢复路径，stable-key lookup=0。8 个 folder-backed focused tests 已编写；WGPU asset/capacity staging、device-generation 接线、提交完成回收、previous-state roll、Runtime04 producer、旧 pending-draw ownership 硬切和真实性能/截图验收仍未完成，状态保持 `in_progress`。
- 2026-08-26 P0-1 跨层接线进展：Runtime04 稀疏 component artifact 已硬切到 `core/framework/render/frame_extract/scene_changes/` 中立 owner；scene wrapper 在发布时一次投影为资源句柄、精确 `Mat4`、`bool`、`u32` 与 core `Mobility`，base/all-LOD/morph payload 使用 immutable `Arc` slice。`GeometryExtract::scene_changes` 对 active/inactive camera 均携带同一个 world-owned `Arc`，稳定和多 viewport 提取不重建 delta；Render03 `RenderSceneComponentProjector::project_frame` 已从真实 frame 消费、校验外部 world lineage，并覆盖 exact replay/cross-world reject。旧 scene-owned artifact/mask 已删除，graphics 不再依赖 `scene::world` DTO。当前仍未把该入口调度进 `SceneRenderer`：同步 `ResourceStreamer` 不满足统一 09D all-LOD residency ticket、typed pending/fail-open 与 no-third-cache 约束，故 WGPU staging、旧 pending-draw hard cut、managed Cargo、RenderDoc/PNG、性能和功耗验收仍未完成，GS-M1 与总计划继续 `in_progress`。
- 2026-08-26 P0-4 residency 前置进展：复核当前 `ResourceStreamer` 与 UE `FScenePreUpdateChangeSet`/dynamic render-asset remove-before-insert 生命周期后，`RenderSceneUpdatedPrimitive` 已保留 exact before/current primitive `Arc`；journal 一次生成 deterministic net typed-resource reference delta，覆盖 base/all-LOD model/mesh/material、primitive binding、material override 与 skeleton，单 primitive 去重且同 journal acquire/release 抵消。实现使用两个复用 scratch `Vec` 与一个连续 observation buffer，transform/no-op 不扫描依赖，不新增 per-primitive shadow cache、residency state 或 WGPU owner。RenderScene 子树现有 60 个、既定 CPU 范围共 81 个 authored focused tests；scoped static checks 通过，managed Cargo 仍无 terminal result。09D residency ticket/manager、产品接线、RenderDoc/PNG 与性能/功耗基线未完成，状态保持 `in_progress`。证据见 [`03/2026-08-26-render-scene-resource-dependency-delta-review.md`](03/2026-08-26-render-scene-resource-dependency-delta-review.md)。

## 2026-08-27 PFO-4d1k Palette Arena Supersession

PFO-4d1k 已替代本计划早期的 per-draw/per-instance palette resource 路径。GPUScene 现在拥有两个 grow-only global palette arena buffer；192-byte `GpuInstanceData` 用 current/previous matrix base+joint count 间接寻址，binding 3/4 为 `array<mat4x4<f32>>`。frame sync 把 active matrices 紧凑打包为一个连续 upload并附着到现有 `GpuScenePreparedUpload`，scene success 后才滚动 staged slot/span。旧 `create_scene_bind_group_for_palettes`、逐实例双 buffer、每 palette 两次 queue write、MeshDraw buffer保活和 skinned command-local bind-group override已硬删除。历史段落中的 per-draw palette 描述只代表当时落地状态，不再是当前合同；indirect visible-remap 仍可拥有命令/phase级 GPUScene override。源码实施与静态证据见 `docs/plans/optimize/zircon_runtime/90/2026-08-27-pfo-4d1k-skinned-palette-arena-hard-cut-plan.md`，动态 WGPU/RenderDoc/PNG/profile/功耗仍 pending。

## 2026-08-27 PFO-4d1s Sideband Upload Transaction

Morph payload 与 VirtualGeometry resident rows 不再在 mesh build 中取得 raw Queue 或提前更新 CPU shadow。两个 owner 现在准备 immutable upload batch 与 move-only commit token；本帧 VG page/cluster counts 显式进入 core scene-count 参数，随后 sideband 一起附着到唯一 `GpuScenePreparedUpload`。scene submission 成功后才统一提交 shadow/counts。grow-only buffer 在失败帧替换物理资源后保留 full-upload intent，因此下一帧回到旧内容也不会把尚未初始化的新 buffer 误判为稳定。每类 sideband 同时只允许一个未决 preparation，reservation 由组合帧持有到 commit/drop。core 与 sideband preparation 同时保留不可伪造的 scene identity；attachment 不接受 caller-supplied scene，batch 离开本地所有权前与 commit 时都校验目标。因此旧 frame 无法在新物理 buffer 准备后越序发布 shadow，A 场景 sideband 也无法拼接到 B 场景 core frame。dirty-row 算法保持最大连续区间的单次 `O(n)` 扫描。源码实施与静态证据见 `docs/plans/optimize/zircon_runtime/90/2026-08-27-pfo-4d1s-gpu-scene-sideband-upload-transaction-plan.md`；动态 WGPU、PNG、RenderDoc、profile、VRAM与功耗仍 pending。

## 性能审阅交接

- 2026-07-18 static batch交接：per-mesh render-layer Vec key与scene空/真实override双build已由borrowed key、一次性constructor止损；稳定scene仍每camera/frame重建BTreeMap、mesh-index/entity Vec。GS owner需把compiled batch membership放进scene/static generation，camera只投影visibility/phase refs，override/layer dirty精确失效；见PERF-MVP-340。
- 2026-07-18 Virtual Geometry sideband交接：runtime-prepare为注册page-request external buffer而深clone整份prepared readback、随后与原frame sideband重复merge的问题已止损，frame sideband现为唯一feedback owner。GS owner仍须把page-request GPU buffer与capacity纳入generation持久资源，stable frame create=0，并用producer/owner tag与duplicate counter证明每item反馈一次；见PERF-MVP-347及`docs/plans/performance/01/2026-07-18-runtime-core-framework-render-sideband-relevance-static-review.md`。
- 2026-07-18 lightmap slot 性能交接：`LightmapConsumeContract::slot_for_instance` 线性扫描 slots，GPU Scene 对每个 static pending draw、Hybrid GI 对每个 mesh 各自调用，形成 meshes×slots。Render03 联动 Render11/17在 validate/publish 边界构建唯一 immutable dense/hash id→slot index并由所有消费者共享；记录 probes/builds/CPU，稳定 generation build=0、changed generation build=1；见 PERF-MVP-353。
- 2026-07-18 shader variant 性能交接：mesh pass当前对每个batch/pass重新resolve owned variant key并为固定platform分配String。Render03联动Render08/17把compiled `MeshPipelineVariantId`纳入material/static-batch generation artifact，queue只引用dense id；稳定generation resolve/key alloc=0，changed每唯一variant≤1；见PERF-MVP-355。
- 2026-07-18 material override性能交接：mesh pending draw当前为稳定override重复clone/hash payload并per-draw创建uniform buffer，override存在还禁用static command cache。Render03联动Render08/17让prepared override handle进入GPU Scene/static batch generation，全部primitive/camera共享，stable encode/hash/create/upload=0、changed每唯一generation≤1；见PERF-MVP-359。
- 2026-07-18 indirect execution workspace交接：compiled-scene每帧为draw order建两份index Vec、新建execution args buffer并逐draw copy/Arc clone，各phase又重建args/compaction/visible/draw-count buffers。Render03联动Render04/17按phase+command generation持久复用capacity与bindings，stable create/index build=0、changed只更新dirty ranges；见PERF-MVP-376及compiled-scene render静态证据。
- 2026-07-18 plugin runtime-prepare调度交接：VG/GPU-scene相关collector当前在render submission线程串行运行并返回owned payload；Render03联动Plugins01/Render12/18把重CPU准备发布为generation-owned immutable artifact，render线程只record/apply ready delta并复用binding/output capacity。stable heavy prepare/payload copy=0、changed prepare≤1/generation、队列有界；见PERF-MVP-379。
- 2026-07-18 mesh/VG stats融合交接：pending draw stats第二遍查询GPU-scene并重建batch key，VG execution stats又逐draw构造DTO与segment/page HashSet。Render03联动Render02/17让主命令/indirect generation artifact携带sealed counters；stable extra draw/GPU-entry visits、DTO/set=0，diagnostics off unique work=0。见PERF-MVP-381。
- 2026-07-18 prepared-batch handle交接：每draw进入cache前仍clone GPU-scene bind group、mesh/previous/indirect Arc和material handles到临时batch。Render03联动Render02/17发布generation-owned dense prepared-batch identity，static cache hit只读revision/identity且handle clone=0，dynamic command arena不做per-draw分配。见PERF-MVP-382。
- 2026-07-18 indirect plan唯一权威交接：command stats当前先build/discard每phase `IndirectDrawBatcher`，execution随后重复key/args/batch/compaction并创建GPU buffers。Render03联动Render02/17让command generation artifact持有唯一CPU indirect plan与sealed stats，GPU capacity由PERF-MVP-376复用；stable batcher build=0、changed≤1/generation。见PERF-MVP-383。
- 2026-07-18 draw binding owner交接：每draw当前除两套material bind group外，skinned palette pair还单独创建GPU-scene bind group。Render03联动Render08/02把palette并入共享scene binding+slot/dynamic offset，或按buffer-pair generation缓存；stable palette bind create=0，changed≤1/unique pair。见PERF-MVP-384。
- 2026-07-18 morph/skin/VG resident owner交接：Render03须让静态morph delta、current/previous weight slots、skin palette slots、VG page/cluster/segment/draw-ref/args及dynamic mesh resident handle进入device+content generation持久owner。stable generation的CPU flatten/primitive clone、GPU object create和upload均为0；changed只做dirty range/scatter并复用capacity。联动Runtime04、Plugins04、Render02/04与PERF-MVP-385/386/388/389。
- 2026-07-18 skin palette传输补充交接：当前`SkinnedMeshJointPaletteStorage`固定约16 KiB并全块`write_buffer`，64 active bones也上传256容量；1k实例current+previous约32.8 MiB。Render03须用persistent palette slot/ring与active-prefix dirty upload，ABI最大joint数只决定capacity，不决定每帧bytes；stable=0、changed近active joints。见PERF-MVP-386。
- 2026-07-18 lighting唯一投影交接：GPUScene lights与light-grid当前分别pack同一`LightingExtract`并重复cookie plan。Render03联动Render05/18发布按lighting/cookie/volumetric generation唯一的`PackedLightingFrameArtifact`，GPUScene只消费dense packed range，stable generation pack/cookie build=0、changed≤1。见PERF-MVP-393及lighting静态证据。
- 2026-07-18 legacy particle hard-cut交接：旧particle renderer每frame CPU展开6 world vertices并创建颜色/速度vertex buffers；Render03联动Render12让`ParticleSimOutput`/`BillboardInstanceData`持唯一persistent instance/storage/indirect artifact，vertex shader展开角点，CPU/GPU sim共享dense handle。stable create/upload=0、CPU world quad bytes=0。见PERF-MVP-396及particle静态证据。
- 2026-07-18 GPUScene owner全目录交接：15/15文件确认稳定核心upload已为0，但CPU仍全draw/history扫描，span first-fit+free全排序，morph/VG长度抖动精确重建，palette逐实例双最大buffer。GS-M3 refinement须以`CompiledGpuSceneDelta`、dense history epoch、deferred ordered allocator、grow-only suballocated morph/VG/palette arena和direct/scatter阈值完成终态。本轮stable morph/VG write与full-upload dirty临时分配已止损；见PERF-MVP-405及GPUScene静态证据。
- 2026-07-18 Virtual Geometry diagnostics交接：page inspection已用page/resident索引止住二次方查找；Render03仍须让真实prepare/cull/execution发布唯一`PreparedVirtualGeometryFrameReport`和compiled page/cluster ordinal/instance/node dense indices。debug off不建snapshot，on时不CPU重演cull或selected sort，Runtime/Editor只借用/分页投影；见PERF-MVP-416。
- 2026-07-18 Virtual Geometry runtime-provider补充：neutral assignment/replacement投影已按输入容量预留并锁定overflow skip；automatic extract仍按camera把全mesh slice交给provider，并允许同步`load_model_asset`。Render03在PERF-MVP-379/414下让scene/model generation发布唯一prepared VG extract与resident model handles，多camera只借用，stable mesh scan/asset load/extract build=0；readback按PERF-MVP-415进入sealed ticket。
- 2026-07-18 production VG visibility-plan交接：当前ordinal/count在visible DTO与draw segment内对同entity clusters反复scan/sort/dedup，frontier每split全排序，requested page/lineage又做多重candidate×ancestry并每walk分配set。Render03按PERF-MVP-421把entity range/ordinal/count/parent/depth/page dense index编译进asset generation，以bounded GPU/parallel hierarchy work queue发布唯一plan/request/feedback；CPU fallback复用scratch且近O(candidates+edges)。
