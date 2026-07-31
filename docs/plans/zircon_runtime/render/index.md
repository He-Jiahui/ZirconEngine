---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/mod.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_geometry_source_descriptor.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/template/mod.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/pass_specialization.rs
  - zircon_runtime/src/graphics/shader/template/validation.rs
  - zircon_runtime/src/graphics/shader/template/tests.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_surface_types.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_scene_runtime.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_static.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_geometry_skinned_morphed.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_gbuffer.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_depth_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_shadow_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_velocity_alpha.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_gbuffer_encode_standard_pbr.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/paths.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/permutation_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/revision.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_shader_permutation_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/metadata.rs
  - zircon_runtime/src/plugin/extension_registry/access/metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/package_contributions/manifest_metadata.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/core/framework/render/plugin_renderer_outputs.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/runtime_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/create_buffers/scene_prepare_depth_samples.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/create_buffers/scene_prepare_trace_tiles.rs
  - zircon_runtime/src/graphics/material/shading_models/include_sources.rs
  - zircon_runtime/src/graphics/material/shading_models/registry.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_shading_models.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_plugins/virtual_geometry/runtime/src/plugin.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_revision.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_permutation_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_permutation_registry_auto_export.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_shading_model_descriptor.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_runtime_staged_cache_hit.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_product_staged_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/project_plugin_registry_staged_cache.rs
  - tools/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/tests/test_zircon_build_shader_prewarm_cache_contract.py
  - tools/zircon_build_shader_prewarm.py
  - tools/zircon_build_shader_resource_registry.py
  - tools/zircon_build.py
  - tools/tests/test_zircon_build_shader_prewarm.py
  - tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py
  - tools/tests/test_zircon_build_plugin_carriers.py
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache/tests.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/compiled_graph_cache_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/registry_contracts.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/postprocess_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests/renderer_context_guards.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/allocation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/cache_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/owner.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_gpu_scene_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/core_contracts.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests/external_compute_guards.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/default_pipelines.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/dynamic_resolution.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/compile_options.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_core.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/validation_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pipeline_compile_monolith_tests.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/stats.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/history.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/pipeline_profiles.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/neural_compute.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/advanced_providers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_framework_bridge_tests.rs
  - zircon_runtime/src/core/framework/render/material/management/tests.rs
  - zircon_runtime/src/core/framework/render/material/management/tests/record_views.rs
  - zircon_runtime/src/core/framework/render/material/management/tests/query_execution.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_material_management_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/material_runtime_pbr_projection_tests.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_frame_extract_geometry.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/material_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/rebuild_batch.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_mesh_build_draws_build.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_mesh_build_draws_skinning_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests/virtual_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/postprocess.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/queue_override.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/m4_behavior_postprocess_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_submit/profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_product_submit_profiles_tests.rs
  - zircon_runtime/src/graphics/tests/project_render.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_runtime/src/graphics/tests/project_render/render_quality.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_project_scene_products_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/composite.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/ordering.rs
  - zircon_runtime/src/graphics/tests/render_product_camera_targets/custom_target/viewport.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRenderer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
plan_sources:
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/Runtime 渲染风险清单与 RenderDoc 调试支持计划.md
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
---

# Zircon 渲染管线 Unreal/Unity 对齐总体架构计划

本目录是 `zircon_runtime` wgpu 渲染管线向 `dev/UnrealEngine` 渲染器架构与 `dev/Graphics`(Unity SRP/URP/HDRP)管线设计对齐的总计划。它承接 `.codex/plans` 中已有的 SRP/RHI、GI、VG 等计划,分两层组织:

- **骨架层(计划 01–08)**:现有计划没有覆盖、且是 UE 渲染器性能与正确性来源的中间层 —— RDG 资源图、MeshDrawCommand 管线、GPUScene、可见性剔除、light grid、时域管线、后处理链定稿、shader permutation 与光照模型。
- **能力层(计划 09–16)**:面向用户的渲染能力族 —— 相机与渲染顺序体系(Unity 语义)、渲染器组件族、环境光照、特效与粒子、纹理体系、2D 栈、地形植被、compute 与神经网络。

参考引擎分工(对齐 zr-reference-engine-routing 技能):UE 主导引擎规模系统的内部结构(RDG/MeshPass/GPUScene/Nanite/Lumen/Landscape/Niagara);Unity Graphics 主导管线资产化、Volume 容器、相机栈、排序体系、URP 量级的简洁实现与 2D renderer;bevy/Fyrox 提供 Rust/wgpu 落地形态;godot 提供 tilemap 等通用设施;slint 提供 UI/wgpu 文本渲染参照。

Plan 08 精确锚点补记 2026-07-01：render index 镜像 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs`、`graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs`、`graphics/shader/wgsl/zr_template_velocity_alpha.wgsl`、`shading_model_gbuffer_include_for` 与 `zr_skinned_joint_matrix(v.joints.x)`，用于锁定 Velocity/TAA source-cache、alpha velocity template、GBuffer include resolver 与 skinned geometry template 的跨文档锚点；完成判定仍以对应 Plan 08 子计划和 Runtime 15 结构守卫为准。

## 1. 现状评审结论

当前管线(入口 `WgpuRenderFramework::submit_frame_extract`,见
`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs`)
已经具备 Extract → Prepare → Queue/Sort → Execute → Present 的骨架、
RenderFeatureDescriptor 驱动的 pass 编排、Forward+/Deferred 双路径、阴影/后处理/SSAO/SSR 等执行器。
框架层次(framework 契约 / graphics 实现 / plugins 扩展)与 UE 的 Engine / Renderer / RenderCore / RHI 分层方向一致。

与 UE 渲染器对照,差距集中在以下骨架层,这也是"不及预期"的根因:

| # | 差距 | 现状表现 | UE 对应物 |
|---|------|---------|----------|
| 1 | RenderGraph 偏统计层 | 资源生命周期固定、无 transient 复用、无 pass culling、feature 图每帧重新解析编译 | FRDGBuilder / transient allocator / pass culling |
| 2 | 无 MeshDrawCommand 缓存管线 | `MeshDraw` 每帧全量重建,draw call 逐条提交,无静态命令跨帧缓存与状态去重 | FMeshPassProcessor / FMeshDrawCommand / cached commands |
| 3 | 无 GPUScene | 逐 draw model uniform,instancing 仅有候选统计,无 indirect 提交 | GPUScene SOA buffers / instance culling / indirect draw |
| 4 | 可见性单薄 | 仅 BVH frustum 剔除,无 relevance、无 HZB occlusion、无并行任务化 | InitViews / SceneVisibility / HZB |
| 5 | 灯光数量受 uniform 限制 | cluster grid 已建但无 per-cluster light list,灯光走场景 uniform | LightGridInjection (froxel light grid) |
| 6 | 时域管线断链 | 有 motion vector 基础与 history 槽位,但无 jitter、无 TAA resolve、history ghosting 是 P0 风险 | VelocityRendering / TemporalAA |
| 7 | 后处理链未定稿 | effect stack DAG 存在,但顺序、HDR 色彩空间、exposure、tonemap 无权威定义 | FPostProcessing 链 |
| 8 | shader 排列无管理 | fallback shader 拼接 skinning,GPU skinning 不适用自定义材质,无 permutation 缓存 | VertexFactory / MaterialShaderMap |

## 2. 目标分层映射

固定映射,所有子计划共享,不再新增 crate:

| UE 层 | Zircon 归属 | 说明 |
|-------|------------|------|
| RHI / RHICore | `zircon_runtime` RHI + wgpu backend(`graphics/backend/`) | wgpu 即 RHI;descriptor 不携带场景语义 |
| RenderCore(RDG、VertexFactory、GlobalShader) | `zircon_runtime/src/render_graph/` + `graphics/shader`、`graphics/pipeline` | RDG 升级见计划 01;VertexFactory 等价物见计划 08 |
| Renderer(SceneRenderer、MeshPass、GPUScene、Visibility、Lights、Shadows、PostProcess) | `zircon_runtime/src/graphics/scene/scene_renderer/` + `graphics/visibility/` | 计划 02–07 的主战场 |
| Engine(SceneProxy、LightSceneInfo) | `zircon_runtime::core::framework::render` extract 契约(`frame_extract.rs`、`scene_extract.rs`、`light/`) | extract 即 proxy 快照;公共 facade 固定于此 |
| 插件(Nanite/Lumen 类比) | `zircon_plugins/`(virtual_geometry、hybrid_gi、rendering) | 经 RenderFeature descriptor 接入 graph |

Unity SRP 概念到 Zircon 的补充映射:`RenderPipelineAsset/ScriptableRenderer` ↔ pipeline asset + compiled pipeline(`graphics/pipeline/`);`ScriptableRenderPass/RendererFeature` ↔ RenderFeature descriptor + pass executor;`VolumeManager/VolumeComponent` ↔ 计划 07 的 Volume 容器框架;`RTHandle` ↔ 计划 01 资源池 + 计划 07 动态分辨率;URP `ForwardLights`(zbin+tile) ↔ 计划 05 light grid。

## 3. 子计划地图与执行顺序

骨架层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 01 RenderGraph RDG 化 | `01-render-graph-rdg-alignment.md` | 无(最先) |
| 02 MeshDrawCommand 管线 | `02-mesh-draw-command-pipeline.md` | 01(执行接口) |
| 03 GPUScene 与 GPU-driven | `03-gpu-scene-gpu-driven.md` | 02(command ABI) |
| 04 可见性与剔除 | `04-visibility-culling.md` | 01(HZB pass)、03(indirect 部分) |
| 05 光照与阴影 | `05-lighting-shadows.md` | 01、03(light buffer) |
| 06 时域管线(velocity/jitter/TAA) | `06-temporal-pipeline.md` | 01(持久资源)、03(prev transform) |
| 07 后处理、色彩与 Volume 容器 | `07-postprocess-color-pipeline.md` | 01;与 06 协调顺序 |
| 08 材质、光照模型与 permutation | `08-material-shader-permutation.md` | 02、03(ABI 定稿后);可与 05–07 并行 |

能力层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 09 相机与渲染顺序体系 | `09-camera-render-ordering.md` | 01、02(排序键);可与阶段 C/D 并行 |
| 10 渲染器组件族 | `10-renderer-family.md` | 02、03、04;LOD 过渡依赖 08 |
| 11 环境光照 | `11-environment-lighting.md` | 05、08、13(cubemap) |
| 12 特效与粒子 | `12-effects-particles.md` | 03(indirect)、04(HZB)、10(注册表) |
| 13 纹理体系 | `13-texture-pipeline.md` | 01(资源池);SVT 依赖 16 readback 队列 |
| 14 2D 栈 | `14-2d-stack.md` | 09(排序)、10(注册表) |
| 15 地形与植被 | `15-terrain-vegetation.md` | 03、04、08、10、13(最后启动) |
| 16 compute 与神经网络 | `16-compute-neural.md` | 01;框架部分可提前到阶段 B 并行 |

横切层:

| 计划 | 文档 | 依赖 |
|------|------|------|
| 17 性能体系与优化 | `17-performance-and-profiling.md` | PF-M1(观测底座)无依赖、最先启动,是各计划 stats 验收的前置;PF-M2(CPU 并行)依赖 01/02;PF-M3(预算降级)接 01/07/13;PF-M4(编译治理与防回归)接 08 |

扩展层(每项机制独立 feature、可单独启停,在其依赖计划的里程碑完成后即可逐项启动,不占用阶段序):

| 计划 | 文档 | 依赖 |
|------|------|------|
| 18 进阶光照与透明特性 | `18-advanced-lighting-features.md` | 体积雾/体积光(05/07)、light cookies(05/13)、clearcoat/anisotropy/transmission(08)、OIT(09 排序之上可选)、局部 irradiance volumes(11 姊妹项)、planar reflection(09 RT 相机)、Burley SSS(07/08) |
| 19 GPU 能力利用与带宽优化 | `19-gpu-capability-optimizations.md` | GC-M1 能力 gate 修复建议随阶段 A;bindless(03/08)、multi_draw_indirect_count(03/04)、subgroup 归约(04/07/13/16)、pipeline statistics(17)、静态阴影缓存(05)、半分辨率透明(07/09/12)、mip 流送(13,与 SVT 分工)、GPU 排序(12/16)、specular AA(08/13) |
| 20 统一光栅/光追 RHI 与跨平台能力门控 | `20-hybrid-raster-raytracing-rhi.md` | RT-M1 能力 resolver 依赖 19/17;RHI/AS 合同依赖 01/08;共享 BLAS/TLAS 依赖 03/09/13;混合 graph 依赖 01/06;消费者只回接 05/07/18、Hybrid GI 与 Solari |

阶段划分:

- 阶段 A(地基):01 + 02。先把"图"和"命令"两条骨架立起来,后续一切 pass 与 draw 都在其上表达。16 的 compute 框架切片(CN-M1)可与阶段 A 末尾并行;17 的观测底座(PF-M1:GPU 计时/分层 stats/抓帧钩子)与阶段 A 同步启动,为全部后续计划提供量化验收手段。
- 阶段 B(GPU 场景):03 → 04。数据上 GPU,剔除走 GPU,打开 indirect 提交。
- 阶段 C(光照阴影):05。light grid 与 shadow atlas 在 GPUScene 之上落地。09(相机/排序)可在本阶段并行启动。
- 阶段 D(时域与后处理):06 → 07。velocity/jitter/TAA 解链后定稿后处理顺序、色彩空间与 Volume 容器。
- 阶段 E(材质收敛):08。几何源、光照模型与材质排列正交化,GPU skinning 全材质可用。
- 阶段 F(能力铺开):10 → {11、12、13、14 任意并行} → 15;16 的 NN 插件部分随需启动。能力层各计划共享骨架层产出的注册表、排序键、instancing 与资源池,不允许另起旁路。
- 扩展层(18/19/20)不占用阶段序:每项机制在其依赖计划的里程碑完成后即可独立启动;19 的 GC-M1(通用能力探测/请求断链修复)与 20 的 RT-M1(设备能力/项目策略/feature 需求三层 resolver)建议随阶段 A 一并完成。20 的 RT-M2 之后才允许创建 BLAS/TLAS 或 RT graph 节点。

阶段 B 之后,既有的 Hybrid GI(Lumen-style)与 Virtual Geometry(Nanite-like)计划可以切换到这套基础设施继续推进:VG 的 N3/N4(GPU 剔除、indirect)直接复用 03/04;HGI 的多灯型与 grid 复用 05。

## 4. 能力覆盖矩阵(需求 → 承接计划)

| 需求项 | 承接计划 | 备注 |
|--------|---------|------|
| 前向 / 延迟渲染 | 既有双管线 + 05 | grid 化后两管线共用光照数据 |
| 多相机、相机栈、RT 相机 | 09 | Base/Overlay、viewport rect、clear 策略 |
| render layer / 多 layer 过滤 | 09(+04/05/07 消费) | 相机/灯光/volume/渲染器同一 mask |
| render queue / order in layer / depth / ui z-index | 09 | Unity queue 数值段;统一 sort_key |
| mesh / skinned mesh / sprite / ui renderer 定制裁剪 | 10 | RendererCommon 基座 + 注册表 |
| LOD(组、过渡)| 10 | dither cross-fade 走 08 变体 |
| 静态合批 / 动态合批 / GPU instancing | 10(策略)+ 03(机制) | 互斥优先级固定、stats 可解释 |
| early-z | 02 + 04 | depth prepass 既有,HZB 补遮挡 |
| 光照模型(unlit / blinn-phong / PBR / 自定义) | 08 | ShadingModelDescriptor 注册 |
| shader / material / renderer 管理 | 08 + 10 | 变体缓存、模板拼接、注册表 |
| compute shader 框架 | 16 | descriptor 化、indirect、readback 队列 |
| 神经网络支持 | 16 | NN 插件:算子库 + 图执行器 + NN 后处理 |
| 后处理全家桶(LUT/bloom/blur/grading/DoF/SSR/dither/vignette/grain/CA) | 07 | uber pass 合并轻效果 |
| 局部容器组件化 / 全局容器 | 07 | Unity Volume 框架对齐 |
| 雾效 | 11(解析雾/高度雾)+ 07(屏幕空间) | Volume 可覆写 |
| 抗锯齿 | 06(TAA)+ 07(FXAA/SMAA)+ 既有 MSAA | 互斥/共存策略在 07 定稿 |
| 环境光遮蔽 | 既有 SSAO feature + 04(HZB 共享) | 不另立计划 |
| HDR 支持 | 07 | linear 全链、HDR 中间格式、输出转换 |
| 反射探针 | 11 | box/sphere、box projection、混合 |
| 光照烘焙(lightmap / light probe) | 11 | runtime 消费契约;烘焙器归插件 |
| skybox / cubemap | 11 + 13 | IBL 预滤波(mip 链 + SH) |
| 稀疏纹理(SVT) | 13 | feedback 驱动页加载,feature gate |
| texture2dArray / normal map / mipmap / 色彩空间 | 13 | 元数据权威化;07 互为表里 |
| 粒子(CPU/GPU) | 12 + particles 插件 | GPU 模拟写 indirect args |
| halo / lens flare / trail / billboard / projector | 12 | projector 与 decals 收敛同源 |
| 2D 文本渲染与排版 | 14 | shaping/字形图集下沉共享,UI 切换消费 |
| 图像渲染 / 九宫切片 | 14 | 拉伸/平铺/填充模式 |
| tilemap(矩形/六边形/等距、画笔、图集) | 14 | chunk 化 + 增量重建,godot 对照 |
| 动态分辨率 | 07 | render scale + 链尾 upscale |
| terrain | 15 | 插件包族;四叉树 LOD + splat |
| tree(speedtree 风格)/ grass | 15 | LOD 链 + imposter + 风动画 |
| 虚拟几何(Nanite-like)/ 动态 GI(Lumen-like) | 既有 VG/HGI 计划 | 阶段 B 后切换到新底座 |
| UI 渲染(screen-space) | 既有闭环 | 本计划不改动 |
| 性能观测(GPU 计时 / 分层 stats / RenderDoc 抓帧)与防回归 | 17 | PF-M1 最先启动;`render_perf_*` 计数断言进测试 |
| 多线程渲染(extract 双缓冲 / 并行 prepare / 并行录制) | 17 | PF-M2,依赖 01/02 |
| 内存与带宽预算、超预算降级阶梯 | 17(+01/07/13 消费) | render scale→mip bias→关 feature 顺序定稿 |
| pipeline 异步编译与首帧卡顿治理 | 17 + 08 | 变体磁盘缓存预热衔接 |
| 体积雾 / 体积光(froxel)、light cookies、平面反射 | 18 | 消费 05 light grid、07 Volume、09 RT 相机 |
| clearcoat / anisotropy / transmission / SSS、OIT、局部 irradiance volumes | 18(+08/11) | shading 扩展位 + 独立 pass,feature 可关 |
| bindless、indirect_count、subgroup、pipeline statistics | 19 | 能力 gate + 回退路径双轨,产物逐像素一致 |
| 静态阴影缓存、半分辨率透明、纹理 mip 流送、GPU 排序、specular AA | 19 | 带宽/缓存类优化,逐项接 05/07/12/13 |
| 硬件光追能力开关(设备支持 / 项目策略 / feature 需求) | 20 | `Capabilities + Policy + CapabilityPlan -> Selection`;Disabled/Auto/Required,缺失能力与策略阻断均可诊断 |
| BLAS/TLAS、inline RayQuery、完整 ray pipeline、SBT | 20 | 三类能力独立报告与启停;光栅/光追共用 mesh buffer;无能力时不创建资源 |
| 混合 raster/compute/RT 调度与确定性降级 | 20(+01/05/07/18/HGI) | feature 声明有序候选;上层禁止按 DX12/Vulkan/Metal/WebGPU 名称分支 |

## 5. 与既有 .codex/plans 计划的关系

- 承接:`Zircon SRP_RHI Rendering Architecture Roadmap.md` 已落地的 RHI v1、RenderGraph 资源图、feature descriptor、executor registry 是本计划的起点;本计划相当于该路线图的下一大段。
- 吸收:`Runtime 渲染风险清单与 RenderDoc 调试支持计划.md` 的 P0(history ghosting、缺 motion vector 重投影)由计划 06 正面解决;P1(RenderGraph 偏统计、资源生命周期固定)由计划 01 解决;RenderDoc/debug marker 接口在各计划的诊断切片中复用。
- 协同:`Hybrid GI Lumen-Style V1 三阶段计划.md` 与 `M5 Nanite-Like Virtual Geometry 全链收束计划.md` 保持独立推进,但其 GPU 数据面与剔除面应在阶段 B 完成后切换到 GPUScene/可见性新底座,避免插件各自维护私有场景缓冲。
- 光追边界:`Hybrid GI Lumen-Style V1 三阶段计划.md` 的 Optional Hardware RT、Solari 与未来 ray-traced shadow/reflection 统一消费计划 20 的 capability resolver、RayTracingScene 和 RHI;消费者不得私建 BLAS/TLAS。当前 WGPU HGI 的 SDF/Voxel 基线不因计划 20 改变。
- 调研映射:`.codex/plans/Hybrid GI 计算机图形学合集工程映射.md` 维护用户指定微信合集的逐篇工程归属。文章只提供问题线索;API、扩展可用性和性能结论必须回到本地实现、参考引擎与官方规范复核。
- 细化替代:`ZirconEngine Bevy-Level Rendering Completion Plan.md` 中 post/AA 相关条目由计划 06/07 按 UE 对齐口径细化;冲突时以本目录为准。
- 能力层协同:`UI SDF 字体真实 Bake 收束计划.md` 的 SDF 产物由计划 14 的共享文本服务消费;`ZirconEngine Particles 插件完善计划.md` 的模拟面与计划 12 的渲染契约对接;`ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md` 的纹理条目并入计划 13 执行;`Rendering 插件选项补齐计划.md` 的 8 个 feature 选项分别由 05(contact shadow)、07(post 效果)、11(探针/烘焙)、12(decals/vfx)承接深化。
- 不触碰:Editor/Runtime UI 渲染链(GPU Command Stream、direct-screen、damage cache)已闭环,本计划不改其路径,仅要求 UI pass 继续作为 graph 末端 executor 存在。
- 编辑器 UI 链路定位(与 `editor_layout` / `editor_ui` / runtime UI 的关系):编辑器界面遵 `docs/plans/zircon_editor/editor_layout` 的规范层(设计语言 + 声明接口 + 约束/样式/输入/导航/**提交**契约),其中 `editor_layout/21`(GPU 提交与绘制管线:批次合并键含裁剪、scissor/stencil 裁剪栈、动态图集、顶点吸附、dirty-region)是**本目录渲染侧的 UI 提交契约**——计划 14(2D/UI stack)的字形图集/SDF/批次组织应满足该契约;`editor_ui/` 是运行时能力层、runtime `ui/surface/render` 产出 extract,本目录只负责把 UI pass 作为 graph 末端 executor 上屏。提交契约语义以 `editor_layout/21` 为准,本目录不另立 UI 提交规范。勾稽:`editor_layout/index §6.1` ↔ `editor_ui/index §3.1` ↔ `zircon_runtime/runtime/09` ↔ `render/14`。

## 6. 全局边界约束(各子计划必须遵守)

来自 `Runtime 吸收层与 Editor_Scene 边界收束计划.md`、`全系统重构方案.md` 与 SRP/RHI 路线图:

1. 渲染公共 facade 固定为 `zircon_runtime::core::framework::render`;不新增渲染 crate,不使用非网络语义的 `server` 命名。
2. `zircon_editor` / `zircon_app` / framework 契约层不得直接 import `wgpu`;RHI descriptor 不出现 Mesh/Material/Light/Scene 场景语义。
3. 每个实际 pass 必须有 RenderGraph 节点、资源 IO 声明与 executor id;不允许绕过 graph 的旁路提交。
4. 插件能力(GI/VG/SSAO/decals 等)只经 RenderFeature descriptor 接入;feature 关闭时 compiled graph 不含对应 pass。
5. 硬切换:新路径落地的同一变更内迁移调用方并删除旧路径,不保留兼容 re-export 或双路径。
6. 渲染模块只消费 `RenderFrameExtract`,不直接访问 ECS World;extract 仅由 runtime 生成。
7. `wgpu` 是当前唯一必须实现的主后端;DX12/Vulkan/Metal 只通过计划 20 定义未来 adapter 映射,不得提前创建原生后端或把平台类型暴露给 framework/feature。
8. 能力选择只由“设备实际启用能力 ∩ 项目策略 ∩ feature 候选需求”决定;`backend_name` 只用于诊断。硬件光追缺失时必须选择已声明的 raster/compute fallback、禁用 feature 或返回 strict mismatch,不得静默假支持。

## 7. 全局验收与测试基线

按 [`milestone-validation-policy.md`](../../milestone-validation-policy.md) 执行：实现切片期间只做格式、结构守卫与源码检查；每个里程碑末才进入一次批量 Cargo 验证阶段。

- 里程碑验证阶段：一条 `cargo check -p zircon_runtime --lib --tests --locked` 覆盖该里程碑的所有 render 切片；随后运行一次按计划模块过滤词组合的 `cargo test -p zircon_runtime --lib --locked`。
- 渲染产物对拍:`render_product_*` 系列测试 + `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧人工比对(对照 UE 同场景行为)
- 插件接缝：仅在该里程碑触及插件边界时，将受影响插件合并为一次 `cargo test --manifest-path zircon_plugins/Cargo.toml -p <受影响插件> --locked` 批次。
- 工作区级验证留给依赖波次收口；每个里程碑完成后,按源码镜像路径更新 `docs/zircon_runtime/**` 模块文档,并保持本目录子计划中的状态标记最新。

## 8. 全局工程约定(各子计划"工程落地细化"章节共享)

跨计划的实现级约定在此唯一定义,子计划不得重定义、只能引用:

1. **bind group 槽位**:`group0` = frame/view 级(相机矩阵、时间、曝光、jitter);`group1` = pass 级输入(light grid、shadow map、HZB、attachment 采样);`group2` = material 级(材质 uniform 与纹理);`group3` = object/instance 级(GPUScene instance index / instance buffer)。所有新 pass 与 shader 模板按此布局,不得私设。
2. **GPU 数据布局**:跨帧/大块数据一律 storage buffer(std430,显式 padding 注释偏移);仅 frame 级小块用 uniform;矩阵列主序;基元 f32/u32,fp16 走能力检测。
3. **WGSL 共享 include**:统一 `zr_` 前缀(如 `zr_gpu_scene.wgsl`、`zr_light_grid.wgsl`、`zr_shadow.wgsl`、`zr_fog.wgsl`、`zr_wind.wgsl`),由计划 08 的模板拼接消费;include 只暴露函数与 struct,不含 entry point。
4. **RenderQueueValue 数值段**(对齐 Unity):Background=1000、Geometry=2000、AlphaTest=2450、Transparent=3000、Overlay=4000;材质可覆写 ±100 内偏移。
5. **统一排序键** `sort_key: u64` 的位段布局唯一由计划 09 定义;其余计划(02 的命令排序、10 的合批切分、14 的 2D 排序)只消费该布局,不得另造位段。
6. **测试命名**:`render_<topic>_*` 单测、`render_product_*` 产物对拍、`render_perf_*` 性能计数断言(确定性计数:draw 数/状态切换/上传字节/瞬态峰值,归计划 17;时间类指标只观测不断言);各子计划"工程落地细化"章节给出函数级测试清单。
7. **实施权威**:每份子计划的"## 工程落地细化"章节是该计划的实施权威 —— 文件落点、类型签名、GPU 布局、切片步骤、测试清单以该章节为准;与正文概述冲突时以细化章节为新。
8. **参考对照纪律(防凭空实现)**:每个新机制动手前必须先读对应子计划"参考代码"表列出的文件 —— UE/Unity 提供设计与算法样板,`dev/bevy`/`dev/Fyrox` 提供 Rust/wgpu 落地形态(API 形态、所有权组织、wgpu 资源管理),两类都要读,不得只凭记忆或常识实现;计划中显式标注"无 Rust 同类参照"的机制(如 SVT、NN 算子),实现时必须对拍测试先行、逐切片抓帧验证。
9. **能力开关**:通用 capability 由计划 19 维护,硬件光追的分层开关和候选解析唯一归计划 20。设备探测、device feature 请求、项目 Disabled/Auto/Required 策略、RenderFeature execution candidates 与 compiled selection 必须是不同对象;禁止用单个 bool 同时表达四层状态。

## 代码结构规范

graphics/render 代码同样遵守引擎级 [`engine-code-structure-convention.md`](../../engine-code-structure-convention.md)：

- `graphics/**` 的大文件热点(如 `graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs`(1510)、`core/framework/render/post_process/stack.rs`(1683)、`submit_frame_extract/update_stats/base_stats.rs`)纳入 [Runtime 15](../runtime/15-code-structure-and-module-conventions.md) 的 `module_convention_gate` 与 `large_file_ownership_gate` 共同治理,按 ownership 拆 owner 叶子,root 留薄 façade。
- `runtime_*` 前缀模块(`hybrid_gi_runtime_provider/`、`virtual_geometry_runtime_provider/` 内部)按规范 §2 去冗余前缀。
- WGSL 共享 include 的 `zr_` 前缀(本文 §8.3 已定)与渲染资源描述放置遵循规范 §5(资源/描述文件归属)。
- 渲染相关测试沿用规范 §4 单一规则与 `render_*` 命名;render 子计划触及上述大文件时按 Runtime 15 的 owner 边界落地,不在旧巨型文件上叠加。

2026-06 代码审查登记的渲染侧待补项继续由渲染与 Runtime 子计划承接；具体切片状态、验证证据与历史补记已迁入 Render 08 产出目录。

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 迁入记录：[`08/2026-07-09-index-output-records.md`](08/2026-07-09-index-output-records.md)

## 9. 当前状态总览

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Render 总索引中的跨计划状态与产品验证明细已迁入 Render 08 产出目录。

- 迁入记录：[`08/2026-07-09-index-output-records.md`](08/2026-07-09-index-output-records.md)
