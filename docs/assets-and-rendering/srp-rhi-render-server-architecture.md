---
related_code:
  - zircon_rhi/src/lib.rs
  - zircon_rhi/src/capabilities.rs
  - zircon_rhi/src/descriptors.rs
  - zircon_rhi/src/device.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_rhi_wgpu/src/capabilities.rs
  - zircon_rhi_wgpu/src/device.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_graph/src/builder.rs
  - zircon_render_graph/src/graph.rs
  - zircon_render_graph/src/types.rs
  - zircon_render_server/src/lib.rs
  - zircon_render_server/src/handle.rs
  - zircon_render_server/src/server.rs
  - zircon_render_server/src/types.rs
  - zircon_resource/src/id.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/render.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/host/module_host.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/history.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/deferred.rs
  - zircon_graphics/src/scene/scene_renderer/history.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/hybrid_gi_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/editing/viewport/controller/mod.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editor_event/runtime.rs
  - zircon_editor/src/host/slint_host/viewport.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_entry/src/lib.rs
  - zircon_entry/src/runtime_presenter.rs
implementation_files:
  - zircon_rhi/src/lib.rs
  - zircon_rhi/src/capabilities.rs
  - zircon_rhi/src/descriptors.rs
  - zircon_rhi/src/device.rs
  - zircon_rhi_wgpu/src/lib.rs
  - zircon_rhi_wgpu/src/capabilities.rs
  - zircon_rhi_wgpu/src/device.rs
  - zircon_render_graph/src/lib.rs
  - zircon_render_graph/src/builder.rs
  - zircon_render_graph/src/graph.rs
  - zircon_render_graph/src/types.rs
  - zircon_render_server/src/lib.rs
  - zircon_render_server/src/handle.rs
  - zircon_render_server/src/server.rs
  - zircon_render_server/src/types.rs
  - zircon_resource/src/id.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/render.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/backend/render_backend/mod.rs
  - zircon_graphics/src/host/module_host.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/history.rs
  - zircon_graphics/src/runtime/offline_bake.rs
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/backend/render_backend/read_buffer_u32s.rs
  - zircon_graphics/src/runtime/server/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport.rs
  - zircon_graphics/src/runtime/server/viewport_record.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/pipeline/mod.rs
  - zircon_graphics/src/feature/mod.rs
  - zircon_graphics/src/material/mod.rs
  - zircon_graphics/src/shader/mod.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/culling/mod.rs
  - zircon_graphics/src/visibility/culling/is_mesh_visible.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan.rs
  - zircon_graphics/src/extract/mod.rs
  - zircon_graphics/src/extract/history.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/runtime_features.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_hybrid_gi.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/mod.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_readback.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/deferred.rs
  - zircon_graphics/src/scene/scene_renderer/history.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/particle.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/bloom_params.rs
  - zircon_graphics/src/scene/scene_renderer/prepass.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/reflection_probe_gpu.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_bloom.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/execute_post_process.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/resources/new.rs
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/bloom.wgsl
  - zircon_graphics/src/scene/scene_renderer/post_process/shaders/post_process.wgsl
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/m4_behavior_layers.rs
  - zircon_graphics/src/tests/m5_flagship_slots.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/editing/viewport/controller/mod.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/editor_event/runtime.rs
  - zircon_editor/src/host/slint_host/viewport.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_entry/src/lib.rs
  - zircon_entry/src/runtime_presenter.rs
plan_sources:
  - user: 2026-04-16 implement Zircon SRP/RHI Rendering Architecture Roadmap
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history.md
  - docs/superpowers/plans/2026-04-16-m4-clustered-lighting-ssao-history-remaining.md
  - docs/superpowers/plans/2026-04-16-m4-runtime-shader-resource-paths.md
  - docs/superpowers/plans/2026-04-16-m4-deferred-runtime-execution.md
  - docs/superpowers/plans/2026-04-16-m4-remaining-behavior-layers.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-preprocess.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-cluster-refine.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
tests:
  - zircon_rhi/src/tests/capabilities.rs
  - zircon_rhi/src/tests/descriptors.rs
  - zircon_rhi_wgpu/src/tests.rs
  - zircon_render_graph/src/tests/ordering.rs
  - zircon_render_graph/src/tests/cycles.rs
  - zircon_render_server/src/tests.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_graphics/src/tests/pipeline_compile.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/host/slint_host/viewport.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer.rs
  - zircon_editor/src/tests/host/render_server_boundary.rs
  - zircon_entry/src/runtime_presenter.rs
  - cargo test -p zircon_rhi --lib --tests
  - cargo test -p zircon_rhi_wgpu --lib --tests
  - cargo test -p zircon_render_graph --lib --tests
  - cargo test -p zircon_render_server --lib --tests
  - cargo test -p zircon_scene --test render_frame_extract --locked
  - cargo test -p zircon_scene --locked
  - cargo test -p zircon_graphics pipeline_compile --locked
  - cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked
  - cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_consumes_feedback_and_promotes_requested_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_leaves_requests_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics virtual_geometry_gpu --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked
  - cargo test -p zircon_graphics visibility_context_builds_hybrid_gi_probe_and_trace_plan --locked
  - cargo test -p zircon_graphics visibility_context_with_history_tracks_hybrid_gi_requested_probes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_tracks_page_table_and_request_sink --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics history_resolve_blends_previous_scene_color_when_enabled --locked
  - cargo test -p zircon_graphics ssao_quality_profile_darkens_scene_when_enabled --locked
  - cargo test -p zircon_graphics clustered_lighting_quality_profile_applies_runtime_tile_lighting --locked
  - cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked
  - cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked
  - cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked
  - cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_graphics render_server_tracks_viewports_and_accepts_frame_extract_submission
  - cargo test -p zircon_editor render_frame_extract_matches_legacy_render_snapshot_projection
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor slint_viewport_toolbar_pointer --locked
  - cargo test -p zircon_editor host::slint_host::viewport
  - cargo test -p zircon_editor tests::host::render_server_boundary
  - cargo test -p zircon_entry runtime_presenter
  - cargo test -p zircon_entry --lib --locked
  - cargo test -p zircon_entry runtime_sources_route_preview_through_render_server_without_wgpu_surface_bindings
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1
doc_type: module-detail
---

# SRP RHI Render Server Architecture

## Purpose

这份文档记录本轮已经真正落地的渲染基础边界，而不是路线图里的全部长期目标。

当前交付集中在两个目标：

- 把渲染基础边界从单一 `zircon_graphics` 里切开，形成 `zircon_rhi`、`zircon_rhi_wgpu`、`zircon_render_graph`、`zircon_render_server`
- 把场景渲染输入从旧 `RenderSceneSnapshot` 提升到新的 `RenderFrameExtract`，同时保留一个明确的兼容桥给现有 viewport 路径

## Landed Crate Roles

### `zircon_rhi`

当前承载无场景语义的底层图形接口类型：

- `RenderBackendCaps`
- `RenderQueueClass`
- `AccelerationStructureCaps`
- `BufferDesc` / `TextureDesc` / `SamplerDesc`
- `PipelineDesc` / `SwapchainDesc`
- `RenderDevice` / `CommandList`

这里故意不出现 mesh、material、light、particle、scene 这些上层概念。

### `zircon_rhi_wgpu`

当前不是完整设备后端，而是 `wgpu` 基线能力映射层：

- `wgpu_backend_caps(...)` 负责把当前 `wgpu` 基线映射到 `zircon_rhi::RenderBackendCaps`
- `WgpuRenderDevice` / `WgpuCommandList` 作为后续真正设备接入前的稳定包装

本轮明确保持 RT/AS 相关能力关闭，确保高级特性只能走 capability gate，而不是偷偷从 `wgpu` 类型向上泄漏。

### `zircon_render_graph`

当前落地的是可编译的 RenderGraph 骨架：

- `RenderGraphBuilder`
- `RenderPassId`
- `QueueLane`
- `PassFlags`
- `TransientTexture` / `TransientBuffer` / `ExternalResource`
- `CompiledRenderGraph`

它现在负责 pass 拓扑、依赖排序和 cycle rejection，还没有承担真正的命令录制与资源别名优化执行器。

### `zircon_render_server`

这是新的稳定渲染 façade crate，当前提供：

- `RenderServer`
- `RenderViewportHandle`
- `RenderPipelineHandle`
- `RenderViewportDescriptor`
- `RenderQualityProfile`
- `RenderStats`
- `CapturedFrame`
- `RenderCommand` / `RenderQuery`
- `RenderServerHandle` / `resolve_render_server(...)`

`RenderStats` 现在不仅承载 frame/view/pipeline 计数，还额外带一份 `RenderCapabilitySummary`：

- `backend_name`
- `queue_classes`
- `supports_surface` / `supports_offscreen`
- `supports_async_compute` / `supports_async_copy`
- `supports_pipeline_cache`
- `acceleration_structures_supported`
- `inline_ray_query`
- `ray_tracing_pipeline`
- `virtual_geometry_supported`
- `hybrid_global_illumination_supported`

这份摘要由 `zircon_graphics::runtime::WgpuRenderServer` 从 `zircon_rhi_wgpu::WgpuRenderDevice` 基线能力映射出来，用来给后续 RT/GI/Virtual Geometry feature 做 façade 侧 capability gate，但不会把 `zircon_rhi` 或 `wgpu` 原生类型直接推给 editor/runtime/script。

`RenderQualityProfile` 当前也不再只是名字字符串。它已经支持：

- `pipeline_override`
- `RenderFeatureQualitySettings.clustered_lighting`
- `RenderFeatureQualitySettings.screen_space_ambient_occlusion`
- `RenderFeatureQualitySettings.history_resolve`
- `RenderFeatureQualitySettings.bloom`
- `RenderFeatureQualitySettings.color_grading`
- `RenderFeatureQualitySettings.reflection_probes`
- `RenderFeatureQualitySettings.baked_lighting`
- `RenderFeatureQualitySettings.particle_rendering`
- `RenderFeatureQualitySettings.virtual_geometry`
- `RenderFeatureQualitySettings.hybrid_global_illumination`
- `RenderFeatureQualitySettings.allow_async_compute`

这意味着 viewport 在没有显式 `set_pipeline_asset(...)` 时，既可以通过 quality profile 选择默认 built-in pipeline，也可以直接控制当前 M4 行为层里 `clustered lighting / SSAO / history / bloom / color grading / reflection probes / baked lighting / particle rendering` 的启闭与 async-compute 偏好；同时还可以对 `virtual geometry / hybrid global illumination` 发出 opt-in 请求，但这些旗舰路径只会在 backend capability 满足时进入有效编译结果。

`RenderStats` 当前还会带上 `last_frame_history`。`FrameHistoryHandle` 定义在 `zircon_render_server` 的稳定 handle 层，并由 `zircon_graphics` 在 extract 子域重导出给 renderer/SRP 侧继续使用。这样 viewport history 生命周期既能被 façade 侧观测，又不会把 backend 私有资源类型推给上层。

为了让 behavior-layer 编译结果对 façade 可见，`RenderStats` 当前还会暴露：

- `last_effective_features`
- `last_async_compute_pass_count`
- `last_virtual_geometry_page_table_entry_count`
- `last_virtual_geometry_resident_page_count`
- `last_virtual_geometry_pending_request_count`
- `last_hybrid_gi_cache_entry_count`
- `last_hybrid_gi_resident_probe_count`
- `last_hybrid_gi_pending_update_count`
- `last_hybrid_gi_scheduled_trace_region_count`

前两者用于验证 quality/capability 之后真正还留下了哪些内建 feature，以及 async-compute pass 是否已经 cleanly 降级到 graphics queue；中间三者用于观测 Virtual Geometry runtime host 当前持有的 page-table、resident page 与 pending request 队列规模；最后四者用于观测 Hybrid GI runtime host 当前维护的 probe cache、resident probe、pending probe update 与 trace schedule 规模。

## Scene Extract Transition

`zircon_scene` 新增了 `render_extract.rs`，把新的提取面固定为：

- `RenderWorldSnapshotHandle`
- `RenderExtractContext`
- `RenderExtractProducer`
- `RenderViewExtract`
- `GeometryExtract`
- `LightingExtract`
- `PostProcessExtract`
- `DebugOverlayExtract`
- `ParticleExtract`
- `VisibilityInput`
- `RenderFrameExtract`

当前 `RenderFrameExtract` 仍然通过 `RenderFrameExtract::from_snapshot(...)` 和 `to_legacy_snapshot()` 与旧 `RenderSceneSnapshot` 双向适配。也就是说：

- 新边界已经存在，新的 render server 和后续 SRP 功能可以围绕它继续扩展
- 旧 `SceneRenderer`、`RenderService`、`RuntimePreviewRenderer` 还没有被删除，它们仍然通过 `EditorOrRuntimeFrame` 中的兼容 `scene` 字段工作

这条双写桥是刻意保留的过渡层，不应被视为长期设计。

在 `M4` 的后半段，这个 extract 面已经继续长出真实行为层数据，而不是只保留空壳 section：

- `PostProcessExtract` 现在额外携带 `RenderBloomSettings` 与 `RenderColorGradingSettings`
- `LightingExtract` 现在额外携带 `reflection_probes` 与 `baked_lighting`
- `ParticleExtract` 现在额外携带 billboard 级 `sprites`
- `GeometryExtract` 现在额外预埋 `virtual_geometry: Option<RenderVirtualGeometryExtract>`
- `LightingExtract` 现在额外预埋 `hybrid_global_illumination: Option<RenderHybridGiExtract>`
- `zircon_scene::components` 公开了 `RenderReflectionProbeSnapshot`、`RenderBakedLightingExtract`、`RenderParticleSpriteSnapshot` 这组新的跨 crate snapshot 契约

这意味着后续 behavior layer 已经不需要偷偷回读旧 `RenderSceneSnapshot` 才能工作；新增后处理、烘焙和粒子路径都已经可以只消费 `RenderFrameExtract`。

## `zircon_graphics` Current Shape

`zircon_graphics` 现在开始向高层 SRP/Renderer crate 收束，新增了固定子域：

- `compat/`
- `extract/`
- `feature/`
- `material/`
- `pipeline/`
- `runtime/`
- `shader/`
- `visibility/`

本轮真正有行为的新增核心是 `runtime::WgpuRenderServer`：

- 通过 `RenderServer` trait 暴露创建 viewport、提交 `RenderFrameExtract`、设置 pipeline/quality、统计与 capture 的稳定接口
- 内部暂时仍复用现有 `SceneRenderer` 做离屏渲染
- 在 `module_descriptor()` 里额外注册了 `GraphicsModule.Manager.RenderServer`

同时保留了旧兼容面：

- `RenderService`
- `SharedTextureRenderService`
- `RuntimePreviewRenderer`

这三者现在主要是 `zircon_graphics` 内部兼容能力，不再是 editor/runtime 的主消费路径。

## Consumer State

当前 consumer 已经切到 `RenderServer` façade：

- `zircon_editor::host::slint_host::viewport::SlintViewportController` 在初始化时直接 `resolve_render_server(...)`
- `EditorState` / `EditorEventRuntime` 现在直接暴露 `render_frame_extract()`，把旧 `RenderSceneSnapshot` 适配压回状态层
- editor viewport 不再走 shared texture + `wgpu` 导入，而是提交 `RenderFrameExtract`，然后把 `CapturedFrame.rgba` 转成 `slint::Image`
- `zircon_entry::runtime_presenter::RenderServerRuntimeBridge` 负责 runtime viewport handle 生命周期、frame submit 与 capture
- runtime 入口自身通过 `World::to_render_frame_extract()` 直接生成 extract，不再在 `lib.rs` 里手写 snapshot 兼容适配
- runtime window 不再依赖 `RuntimePreviewRenderer` 的 `wgpu` surface path，而是使用 `SoftbufferRuntimePresenter` 把 `RenderServer` 输出的 RGBA 帧 blit 到窗口
- `runtime_bootstrap_excludes_editor_module` 现在显式验证 runtime bootstrap 后可以 `resolve_render_server(&core)`
- editor/runtime 额外有源码边界守卫测试，防止后续重新引入 `wgpu`、shared-texture preview renderer 或旧 preview façade

当前仍未完成的迁移包括：

- `zircon_manager::RenderingManager` 退化成纯兼容桥
- `zircon_graphics` compat 层对旧 `RenderService` / `RuntimePreviewRenderer` 的最终收束与删除
- shader/material hot reload、真正的 feature 实例注册和 GPU-driven visibility 前处理

## Default SRP Compile Skeleton

`zircon_graphics::pipeline` 现在已经不再只靠 `stage -> pass name` 的硬编码表编译默认 Forward+。

当前固定的 M2 编译骨架是：

- `BuiltinRenderFeature` 负责声明 feature 级 extract 依赖和 stage pass 贡献
- `RenderPipelineAsset::compile(...)` 负责验证 `RendererAsset`，收集启用 feature 的 descriptor，然后按 renderer stage 顺序把 pass 编译进 `CompiledRenderGraph`
- `runtime::WgpuRenderServer` 只消费编译结果，不在 submit 时重新推断 stage/pass 结构

当前内建的默认 Forward+ feature 组合包括：

- `VirtualGeometry`
- `Mesh`
- `Shadows`
- `ScreenSpaceAmbientOcclusion`
- `ClusteredLighting`
- `GlobalIllumination`
- `Particle`
- `Bloom`
- `ReflectionProbes`
- `BakedLighting`
- `PostProcess`
- `ColorGrading`
- `HistoryResolve`
- `DebugOverlay`

其中 `VirtualGeometry` 与 `GlobalIllumination` 当前属于 “声明在 built-in renderer 里，但默认不进入有效编译结果” 的旗舰槽位。它们只有在 compile options 显式 opt-in 时才会真正贡献 pass 和 history binding。

这带来三个关键边界改进：

- overlay/gizmo 不再通过 `PostProcess` 间接声明 `debug` extract 依赖，而是由独立 `DebugOverlay` feature 负责
- pipeline compile 会显式拒绝 duplicate stage 和 duplicate feature 配置，而不是把不确定顺序静默吞掉
- `VirtualGeometry / GlobalIllumination / RayTracing` 这类旗舰 feature 现在可以沿同一套 descriptor 系统挂进 pipeline，但不会反向污染基础 renderer 默认结果

当前 pipeline compile 也已经进入 M4 的“可配置编译”阶段。`RenderPipelineAsset` 除了默认 `compile(...)`，现在还支持 `compile_with_options(...)`，并消费 `RenderPipelineCompileOptions`：

- `enabled_features`
- `disabled_features`
- `allow_async_compute`

这让 built-in pipeline 可以在不改动资产结构的前提下，根据 quality profile 和 capability 生成不同的有效 feature 集合与 queue-lane 分布；同时也让 M5 的旗舰路径第一次拥有显式 opt-in 编译入口，而不是默认随 built-in pipeline 一起混入。

对当前默认 pipeline，编译后的固定阶段仍然保持为：

- `DepthPrepass`
- `Shadow`
- `Opaque`
- `Transparent`
- `PostProcess`
- `Overlay`

但 graph pass 来源已经迁移到 feature descriptor 层，后续继续扩展 `Particle`、`Deferred`、`GI`、`Virtual Geometry` 时，不需要再把特殊分支塞回中心硬编码点。

## M4 Deferred Pipeline Runtime

`M4` 的第二条内建 renderer/pipeline 不再只是 compile skeleton。当前 `RenderPipelineAsset`、`RenderServer`、`SceneRenderer` 已经共同落下一条真实 deferred runtime：opaque 几何不再复用 forward project shader，而是固定改走 GBuffer 材质解码和 fullscreen deferred lighting。

当前已经固定下来的 deferred pipeline 边界是：

- `RenderPassStage` 新增 `GBuffer` 与 `Lighting`
- `BuiltinRenderFeature` 新增 `DeferredGeometry` 与 `DeferredLighting`
- `RenderPipelineAsset::default_deferred()` 现在固定占用 built-in handle `2`
- `RenderPipelineAsset::builtin(...)` 现在同时能解析 built-in Forward+ 与 built-in Deferred
- `runtime::WgpuRenderServer` 的 built-in pipeline registry 现在同时注册两条内建 pipeline，因此 viewport 可以显式切到 deferred handle 再走完整 submit/capture 路径
- `RenderQualityProfile::pipeline_override` 现在也能把 deferred pipeline 作为 viewport 的默认 renderer 选择源，而不要求 consumer 先手动 `set_pipeline_asset(...)`

当前 deferred pipeline 的固定阶段顺序是：

- `DepthPrepass`
- `Shadow`
- `GBuffer`
- `AmbientOcclusion`
- `Lighting`
- `Transparent`
- `PostProcess`
- `Overlay`

当前 deferred pipeline 的 built-in feature 组合是：

- `DeferredGeometry`
- `Shadows`
- `ScreenSpaceAmbientOcclusion`
- `ClusteredLighting`
- `DeferredLighting`
- `Particle`
- `Bloom`
- `ReflectionProbes`
- `BakedLighting`
- `PostProcess`
- `ColorGrading`
- `HistoryResolve`
- `DebugOverlay`

编译后当前固定 pass 顺序是：

- `depth-prepass`
- `shadow-map`
- `gbuffer-mesh`
- `ssao-evaluate`
- `clustered-light-culling`
- `deferred-lighting`
- `transparent-mesh`
- `particle-render`
- `bloom-extract`
- `reflection-probe-composite`
- `baked-lighting-composite`
- `post-process`
- `color-grade`
- `history-resolve`
- `overlay-gizmo`

这条实现故意保持“先固定 pipeline/feature graph，再补真正 shading path”的边界：

- `SceneRendererCore` 现在会按 `CompiledRenderPipeline.enabled_features` 真实分支 forward 与 deferred runtime，而不是只在 compile graph 层区分
- `OffscreenTarget` 当前新增 `gbuffer_albedo`，并继续复用 `normal / depth / scene_color / final_color / ambient_occlusion / cluster_buffer`
- `ViewportOverlayRenderer` 现在把 scene content 进一步拆成 `record_preview_sky(...)` 与 `record_meshes(...)`，这样 deferred 可以只消费背景与透明补绘，而不会重新掉回整条 forward base-scene pass

### Real Deferred Runtime Path

当前 built-in deferred 的真实执行顺序已经固定为：

- `record_preview_sky(...)` 先把 clear color / preview sky 写入 `final_color`，作为 deferred lighting 的背景输入
- `NormalPrepassPipeline` 只对 opaque draw 写入 `normal + depth`
- `DeferredSceneResources::record_gbuffer_geometry(...)` 把 opaque draw 的 `albedo texture * material tint` 写入 `gbuffer_albedo`
- `ScenePostProcessResources::execute_ssao(...)` 与 `execute_clustered_lighting(...)` 继续跑在共享的 `normal / depth / cluster_buffer` 上
- `DeferredSceneResources::execute_lighting(...)` 读取 `gbuffer_albedo + normal + final_color(background)`，输出 lit opaque scene 到 `scene_color`
- `ViewportOverlayRenderer::record_meshes(...)` 再把 transparent draw 用现有 forward mesh path 叠加进 `scene_color`
- `ScenePostProcessResources::execute_post_process(...)` 继续复用既有 `scene_color + AO + history + cluster_buffer -> final_color` 链
- `ViewportOverlayRenderer::record_overlays(...)` 最后把 gizmo/selection/handle 叠加到 `final_color`

这条实现当前刻意保持以下边界：

- opaque deferred geometry 只做稳定材质解码，不执行项目自定义 fragment shader
- transparent 仍走现有 forward mesh path，避免第一轮 deferred baseline 把透明材质和排序语义一并重写
- clustered lighting / SSAO / history resolve 仍然通过共享 post/runtime 资源链生效，而不是在 deferred 路径里私有复制一套
- deferred 目前的 GBuffer 只先落 `albedo`，normal 继续复用已有 normal prepass，足够支撑当前 baseline 的 material decode + directional/ambient lighting

## M4 Clustered Lighting SSAO History Baseline

当前已经把 `clustered lighting / SSAO / history` 这一批 M4 行为层推进到“真实 shader/resource/runtime path”阶段。它们不再只是 compile skeleton，而是由 `WgpuRenderServer -> SceneRenderer` 真正驱动 GPU 资源、compute/fullscreen pass 和跨帧 history copy。

### Frame History Contract

`zircon_graphics::extract` 当前已经把跨帧资源 contract 固定成：

- `FrameHistorySlot`
  - `AmbientOcclusion`
  - `SceneColor`
- `FrameHistoryAccess`
  - `Read`
  - `Write`
  - `ReadWrite`
- `FrameHistoryBinding`

`RenderFeatureDescriptor` 现在除了 `required_extract_sections`，还会显式声明 `history_bindings`。`RenderPipelineAsset::compile(...)` 会按 slot 聚合 enabled feature 的 history usage，并把结果输出到 `CompiledRenderPipeline.history_bindings`。当前 merge 规则固定为：

- 同 slot 的 `Read + Write` 会折叠成 `ReadWrite`
- 任一侧已是 `ReadWrite` 时保持 `ReadWrite`
- 输出顺序按 `FrameHistorySlot` 稳定排序

### Built-In Feature Wiring

当前新增的 built-in feature 族包括：

- `ClusteredLighting`
- `ScreenSpaceAmbientOcclusion`
- `HistoryResolve`

同时 `RenderPassStage` 新增了 `AmbientOcclusion`，让 built-in pipeline 现在可以显式编译：

- `ssao-evaluate`
- `clustered-light-culling`
- `history-resolve`

Forward+ 当前固定阶段顺序变成：

- `DepthPrepass`
- `Shadow`
- `AmbientOcclusion`
- `Lighting`
- `Opaque`
- `Transparent`
- `PostProcess`
- `Overlay`

Deferred 当前固定阶段顺序则变成：

- `DepthPrepass`
- `Shadow`
- `GBuffer`
- `AmbientOcclusion`
- `Lighting`
- `Transparent`
- `PostProcess`
- `Overlay`

compile 边界现在仍然保持稳定，但这些 feature 不再只是空声明：

- `ClusteredLighting` 只声明 `clustered-light-culling` async compute pass 和它需要的 extract section
- `ScreenSpaceAmbientOcclusion` 只声明 `ssao-evaluate` async compute pass，并把 `AmbientOcclusion` slot 标成 `ReadWrite`
- `HistoryResolve` 只声明 `history-resolve` graphics pass，并把 `SceneColor` slot 标成 `ReadWrite`

在这之上，当前已经补上两条真正的行为层闭环：

- `RenderPipelineCompileOptions` 可以显式禁用这三个 built-in feature
- 当 `allow_async_compute = false` 时，这两条 compute pass 会 cleanly 退化到 graphics queue，而不是继续输出 `AsyncCompute`

### Real Runtime Resource Path

`SceneRenderer` 现在新增了一条 server-only runtime 入口：`render_frame_with_pipeline(...)`。`WgpuRenderServer` 在 submit 时不再只是编译 pipeline 然后继续走旧单 target render，而是把 `CompiledRenderPipeline + FrameHistoryHandle` 直接交给 renderer 执行。

当前真实落地的资源链是：

- `OffscreenTarget.final_color`：最终 readback / capture 的颜色目标
- `OffscreenTarget.scene_color`：base scene 先写入的中间 scene color
- `OffscreenTarget.normal`：normal/depth prepass 输出的法线缓冲
- `OffscreenTarget.ambient_occlusion`：SSAO compute pass 输出的 AO 纹理
- `OffscreenTarget.depth`：现在是可采样 depth texture，而不是只做 render attachment
- `OffscreenTarget.cluster_buffer`：按 tile 写入的 clustered-light runtime buffer

对应的 runtime pass 链现在是：

- `NormalPrepassPipeline`：先把世界法线写进 `normal`，并建立可采样 depth
- `ViewportOverlayRenderer::record_scene_content(...)`：把 preview sky + base mesh scene 写进 `scene_color`
- `ScenePostProcessResources::execute_ssao(...)`：读取 `depth + normal + previous AO history`，写入 `ambient_occlusion`
- `ScenePostProcessResources::execute_clustered_lighting(...)`：把 extract lighting 编码进 GPU light buffer，并按 tile 写进 `cluster_buffer`
- `ScenePostProcessResources::execute_post_process(...)`：读取 `scene_color + ambient_occlusion + previous scene color history + cluster_buffer`，输出 `final_color`
- `ViewportOverlayRenderer::record_overlays(...)`：最后把 wire/selection/gizmo/handle 叠加回 `final_color`

这一条实现故意仍然保持“真正执行 M4 行为层，但不假装已经完成完整 deferred shading”的边界：

- clustered lighting 目前是基于 directional light extract 的 tile lighting buffer，而不是完整 local-light clustered shading
- SSAO 当前是基于 depth/normal 的 compute AO，并且先把 AO 作为 post composite 输入，而不是接入完整材质解码链
- history resolve 当前先落 `scene color` 与 `ambient occlusion` 两条跨帧 history copy，而不是直接宣称 TAA 完成

### Viewport Runtime History Host

`zircon_graphics::runtime` 当前新增了 `ViewportFrameHistory`，由 `WgpuRenderServer` 为每个 viewport 持有：

- `handle`
- `viewport_size`
- `pipeline`
- `generation`
- `bindings`
- `visibility`

`submit_frame_extract(...)` 现在会在 render 前读取上一帧 visibility history，调用 `VisibilityContext::from_extract_with_history(...)` 构建统一前处理的跨帧 diff 输入。render 完成后则按下面的规则决定复用还是轮换 `FrameHistoryHandle`：

- viewport 还没有 history 时分配新 handle
- `viewport_size` 变化时轮换
- `pipeline` 变化时轮换
- `history_bindings` 变化时轮换
- 兼容时只更新 `generation / bindings / visibility`

renderer 侧则由 `SceneFrameHistoryTextures` 真正承载 GPU history 资源：

- `scene_color_history`
- `ambient_occlusion_history`

这些纹理按 `FrameHistoryHandle` 建立和回收。viewport destroy 或 handle 轮换时，`WgpuRenderServer` 会同步调用 renderer 的 `release_history(...)`，而不是让旧 history texture 无限堆积。

这意味着 `clustered lighting / SSAO / history resolve` 现在已经不再只是 pipeline compile-time 元数据，而是正式拥有了 per-viewport runtime 宿主和真实跨帧 texture copy 行为。

`WgpuRenderServer` 当前还会把 `RenderQualityProfile + RenderCapabilitySummary` 映射成 `RenderPipelineCompileOptions`：

- quality profile 可以禁用 `clustered lighting / SSAO / history resolve`
- `allow_async_compute` 需要同时满足 profile 允许和 backend capability 支持
- 当前 headless `wgpu` 基线 `supports_async_compute = false`，因此默认 built-in pipeline 会保留 feature，但把 `ssao-evaluate` 和 `clustered-light-culling` cleanly 降级到 graphics queue
- `RenderStats.last_effective_features` 与 `last_async_compute_pass_count` 会记录这一结果

## M4 Remaining Behavior Layers Baseline

`M4` 剩余的行为层当前已经不再停留在 pass/profile skeleton，而是进入真正的 shader/resource/runtime 数据路径：

- `OffscreenTarget` 现在新增 `bloom` 中间纹理，`scene color -> bloom -> final color` 这条后处理链已经有独立资源位
- `ScenePostProcessResources` 现在额外持有：
  - bloom fullscreen pipeline
  - bloom params uniform
  - reflection probe storage buffer
  - 扩展后的 post-process uniform/bind group layout
- `post_process.wgsl` 不再只做 `AO + clustered + history` composite，而是继续消费：
  - bloom 纹理
  - color grading 参数
  - projected reflection probe buffer
  - baked lighting 参数
- `RenderPipelineCompileOptions` 与 `RenderQualityProfile` 现在会真正影响这些行为层的内建 feature 集，而不只是 façade 侧挂名开关

### Bloom And Color Grading Runtime

当前 bloom 不是 fake CPU 后处理，而是 renderer 内的真实 fullscreen pass：

- `execute_bloom(...)` 先从 `scene_color` 里提取超过阈值的亮部，并做一个软化采样
- 最终 `execute_post_process(...)` 再把 bloom 纹理叠回主颜色
- color grading 使用 extract 提供的 `exposure / contrast / saturation / gamma / tint` 参数，在最终 composite shader 内统一执行

这条边界刻意保持简单：

- 不引入复杂 mip-chain 或多级 downsample ping-pong
- 不把 bloom/color grading 私有化进某个 pipeline 分支
- 仍然继续走 `RenderServer -> CompiledRenderPipeline -> SceneRenderer` 的统一 runtime 路径

### Reflection Probes And Baked Lighting Runtime

reflection probes 与 baked lighting 当前已经有一条 capability/profile gated 的真实 baseline：

- `offline_bake_frame(...)` 会从 `RenderFrameExtract` 的方向光和几何体提取里生成：
  - `RenderBakedLightingExtract`
  - `Vec<RenderReflectionProbeSnapshot>`
- renderer 在运行时会把 probe 数据编码成 projected screen-space influence buffer，而不是在 shader 里重新访问 scene/world
- 最终 composite 会把 probe 贡献和 baked ambient 统一叠进主颜色

这条实现仍然刻意是 baseline，而不是伪装成完整 probe/GI 系统：

- probe 目前是 screen-space projected influence，不是 cubemap capture + parallax correction
- baked lighting 目前是 CPU 端 baseline 输出，不是完整 lightmap/irradiance volume 流程
- 但 scene/extract/runtime/profile 边界已经预埋到位，后续继续换成更强实现时不需要回退到旧 snapshot 或 backend 私有路径

### Particle Transparent Stage Runtime

粒子当前已经有独立的透明阶段 runtime pass，而不是 overlay hack：

- `ParticleRenderer` 会从 `RenderParticleSpriteSnapshot` 组装 CPU 侧 billboard 顶点流
- pass 在 scene color 的 transparent 阶段执行，并使用 additive color blend
- `particle_rendering` quality toggle 会通过 built-in feature disable path 干净关闭整条粒子 pass

这让粒子在架构上继续归属于统一 SRP feature/pipeline/runtime 体系，而不是重新开一条私有 renderer 分支。

## M5 Flagship Capability Slots Baseline

`M5` 当前先落的是 “flagship feature capability slot”，不是 Nanite/Lumen 本体。目标是把 `Virtual Geometry` 与 `Hybrid GI` 变成架构上真实存在、但默认关闭、并且完全 capability-gated 的 `RenderFeature` 家族。

当前已经固定下来的边界是：

- `zircon_scene::GeometryExtract` 新增 `virtual_geometry: Option<RenderVirtualGeometryExtract>`，默认由 legacy snapshot adapter 初始化为 `None`
- `zircon_scene::LightingExtract` 新增 `hybrid_global_illumination: Option<RenderHybridGiExtract>`，默认同样保持 `None`
- `zircon_render_server::RenderFeatureQualitySettings` 新增 `virtual_geometry` 与 `hybrid_global_illumination` 两个 profile 开关，默认值都为 `false`
- `RenderCapabilitySummary` 新增 `virtual_geometry_supported` 与 `hybrid_global_illumination_supported`，作为 façade 可观测的 backend capability 摘要
- `FrameHistorySlot` 新增 `GlobalIllumination`
- `BuiltinRenderFeature::VirtualGeometry` 现在会在 opt-in 时贡献 `virtual-geometry-prepare`
- `BuiltinRenderFeature::GlobalIllumination` 现在会在 opt-in 时贡献 `hybrid-gi-resolve`，并把 `GlobalIllumination` history slot 标记为 `ReadWrite`

这一轮最重要的 compile 规则是：

- 默认 built-in Forward+ / Deferred renderer 仍然保留 `VirtualGeometry` 与 `GlobalIllumination` 的 descriptor 槽位，但 compile 时不会自动启用它们
- `RenderPipelineCompileOptions::with_feature_enabled(...)` 负责显式 opt-in；没有 opt-in 时，默认 pass 顺序与 M4 完全保持不变
- `requires_explicit_opt_in()` 当前把 `VirtualGeometry / GlobalIllumination / RayTracing` 收进同一条旗舰功能门控规则，避免后续高阶路径重新把默认 pipeline 污染回基础层

`WgpuRenderServer` 当前把 façade 层 profile 与 backend capability 映射成有效 compile options：

- `virtual_geometry_supported` 目前要求 `supports_async_compute && supports_pipeline_cache`
- `hybrid_global_illumination_supported` 目前要求 `acceleration_structures_supported && (inline_ray_query || ray_tracing_pipeline)`
- 因为当前 headless `wgpu` 基线不满足这些条件，所以即使 quality profile 显式请求 `virtual_geometry / hybrid_global_illumination`，`last_effective_features` 里也不会出现它们

这条实现刻意保持边界预埋而不是伪装行为完成：

- 已经补上 renderer-local 的最小 GPU uploader/readback 与 trace/update completion source，但还没有 cluster streaming、hierarchy refine、radiance cache lighting resolve 或 Nanite/Lumen-like 真实场景表示
- 但 extract、profile、history、pipeline compile、runtime capability gate、stats 可观测性已经对齐，后续继续实现真实旗舰路径时不需要重新拆 façade 边界

### M5 Virtual Geometry Preprocess Baseline

在 capability-slot 之上，当前又把 `Virtual Geometry` 推进了一层统一前处理 baseline，但仍然刻意停在 “数据规划” 而不是 “真实执行器”。

当前新增的 scene/extract 合同是：

- `RenderVirtualGeometryExtract`
  - `cluster_budget`
  - `page_budget`
  - `clusters`
  - `pages`
- `RenderVirtualGeometryCluster`
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `bounds_center`
  - `bounds_radius`
  - `screen_space_error`
- `RenderVirtualGeometryPage`
  - `page_id`
  - `resident`
  - `size_bytes`

`VisibilityContext` 当前新增三组 Virtual Geometry 前处理输出：

- `virtual_geometry_visible_clusters`
- `virtual_geometry_page_upload_plan`
- `virtual_geometry_feedback`

其中 page/upload/feedback 边界被固定成：

- `VisibilityVirtualGeometryPageUploadPlan.resident_pages`
- `VisibilityVirtualGeometryPageUploadPlan.requested_pages`
- `VisibilityVirtualGeometryPageUploadPlan.dirty_requested_pages`
- `VisibilityVirtualGeometryPageUploadPlan.evictable_pages`
- `VisibilityVirtualGeometryFeedback.visible_cluster_ids`
- `VisibilityVirtualGeometryFeedback.requested_pages`
- `VisibilityVirtualGeometryFeedback.evictable_pages`

这一层的规则当前是：

- Virtual Geometry cluster 不只跟随 entity 级 mesh visibility；它们会再走一次 cluster sphere frustum test
- visible cluster 排序按 `screen_space_error` 优先，然后按 `lod_level`、`cluster_id` 做稳定 tie-break
- `cluster_budget` 会截断本帧真正进入可见集的 cluster
- `page_budget` 会截断本帧真正发起请求的 missing page
- `VisibilityHistorySnapshot` 当前额外保留 `virtual_geometry_requested_pages`，让 `dirty_requested_pages` 可以按跨帧 diff 计算，而不是每帧全量重发
- resident 但本帧没有任何 visible cluster 引用的页会进入 `evictable_pages`，作为后续 residency/page table 的稳定前置信号

render-server façade 当前也开始把这条前处理链的规模暴露到 `RenderStats`：

- `last_virtual_geometry_visible_cluster_count`
- `last_virtual_geometry_requested_page_count`
- `last_virtual_geometry_dirty_page_count`

当前这些计数只有在 `VirtualGeometry` 真正进入有效 compiled pipeline 时才会写入；在 headless `wgpu` 基线上，因为 capability gate 仍然关闭这条旗舰 feature，所以这些值会继续保持 `0`。这正是当前基线想要证明的行为：前处理合同可以先落地，但不会反向假装执行路径已经开放。

### M5 Virtual Geometry Runtime Host Baseline

在 preprocess 之上，当前先补了 CPU 侧 runtime host 来承接 viewport 级的 page table、resident page 与 pending request 状态，随后再把最小 GPU uploader/readback 接进 renderer；目前仍然没有更完整的 streaming residency manager 或 cluster hierarchy。

当前已经固定下来的 runtime host 边界是：

- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 现在作为 viewport 级宿主，持有：
  - resident page -> slot 映射
  - resident page budget
  - page `size_bytes` metadata
  - pending request 队列
  - 当前帧 `evictable_pages`
- `ViewportRecord` 新增 `virtual_geometry_runtime`
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 `BuiltinRenderFeature::VirtualGeometry` 真正进入有效 compiled pipeline 时：
  - 先用 `register_extract(...)` 同步 extract page metadata 与 resident baseline
  - 再用 `ingest_plan(...)` 吞入 visibility 生成的 resident/requested/dirty/evictable page 计划
  - 最后把 runtime host 规模写回 façade stats

这一层的规则当前是：

- resident baseline page 会保留稳定 slot，不会在重复提交时重新编号
- `RenderVirtualGeometryExtract.page_budget` 当前会同时约束 CPU fallback runtime host 的 resident page budget，并且至少覆盖 extract 自身声明的 resident baseline page
- `dirty_requested_pages` 只会转成一次 pending request，不会因重复提交而重复入队
- `apply_evictions(...)` 释放出来的 slot 会被后续 fulfill/reload 按最小空闲 slot 优先复用
- headless `wgpu` 基线依旧不会创建有效 runtime host，因为 capability gate 会让 `VirtualGeometry` feature 继续保持关闭

render-server façade 当前还额外暴露这三项 runtime host 计数：

- `last_virtual_geometry_page_table_entry_count`
- `last_virtual_geometry_resident_page_count`
- `last_virtual_geometry_pending_request_count`

它们与前处理计数一样，只会在 `VirtualGeometry` 真正进入有效 compiled pipeline 时写入；在当前 headless `wgpu` 基线上仍然稳定为 `0`。

### M5 Virtual Geometry Prepare Consumption Baseline

在 runtime host 之上，当前又把 `Virtual Geometry` 推进了一层真正进入 frame/runtime path 的 baseline：`virtual-geometry-prepare` 不再只是 compile-time pass 名字，而是已经会消费 viewport 级 runtime host，生成 frame-local prepare snapshot，并驱动当前 mesh fallback path 的实体过滤。

当前新增的 frame/runtime 合同是：

- `EditorOrRuntimeFrame` 新增内部 `virtual_geometry_prepare` 槽位
- `VirtualGeometryPrepareFrame`
  - `visible_entities`
  - `visible_clusters`
  - `resident_pages`
  - `pending_page_requests`
  - `evictable_pages`
- `VirtualGeometryPrepareCluster`
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `resident_slot`
  - `state: Resident | PendingUpload | Missing`

这一层的规则当前是：

- `VirtualGeometryRuntimeState::build_prepare_frame(...)` 会把 runtime host 当前 resident slot、pending request、evictable page 与本帧 visible cluster 合成 prepare snapshot
- 只有 `Resident` 或 `PendingUpload` cluster 对应的 entity 会进入 `visible_entities`；完全 `Missing` 的 page/cluster 会继续保留在 prepare snapshot 里，但不会进入当前 fallback draw 白名单
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 之前就克隆并更新 viewport 级 runtime host，再把 prepare snapshot 挂到 `EditorOrRuntimeFrame`
- `build_mesh_draws(...)` 现在会在 `VirtualGeometry` feature 显式开启时，使用 prepare snapshot 的 `visible_entities` 过滤当前 mesh fallback draw 集；这让 Virtual Geometry runtime host 第一次真正影响当前离屏输出，而不是只停在 stats 里

这条实现现在已经从纯 CPU fallback 推进到最小 GPU uploader/readback baseline，但仍然没有把更完整的 streaming ownership 做完：

- 当前已有最小 GPU page upload/readback baseline，但还没有 async copy queue、page residency manager 或 cluster-streaming ownership
- 当前也没有 cluster hierarchy refine、split-merge、Nanite raster 或 indirect draw integration
- 但 `virtual-geometry-prepare` 已经拿到了 runtime host 的消费边界，后续 GPU uploader/streaming/refine 可以在不重拆 render path 的前提下继续向下替换

### M5 Virtual Geometry Feedback Streaming Baseline

在 prepare consumption 之后，当前又把此前未消费的 `VisibilityVirtualGeometryFeedback` 接进了 viewport runtime host。这样 pending page request 不再只是停在 request sink 统计里，而会在帧后按 resident budget 与 evictable resident page 列表推进到下一帧 residency。

这一层当前新增的合同是：

- `VirtualGeometryRuntimeState::consume_feedback(&VisibilityVirtualGeometryFeedback)`
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 完成后消费当前帧 feedback，再把更新后的 runtime host 写回 viewport record
- façade stats 使用的是 feedback 消费之后的 runtime snapshot，因此当 capability 未来放开时，`page_table / resident / pending-request` 规模会反映 submit 完成后的宿主状态，而不是 render 前的中间状态

这一层的规则当前是：

- 只有当前 feedback 中仍然处于 pending 的 `requested_pages` 才会被尝试 promote
- resident 数达到 `page_budget` 时，只允许回收当前 feedback 提供的 `evictable_pages`
- 如果本帧没有足够的可回收 budget，剩余 request 会保持 `PendingUpload`，而不是无上限扩 resident cache
- 下一帧 `build_prepare_frame(...)` 会直接观察到这次 feedback 消费后的 `Resident / PendingUpload / Missing` 变化

这条实现仍然不是最终的 GPU streaming 路线：

- 还没有真实 GPU copy/upload backend
- 还没有 DMA/readback 或 page completion 信号源
- 但 `VisibilityVirtualGeometryFeedback` 已经不再是死数据结构，后续真正的 uploader/readback 只需要替换 fulfillment 来源，而不需要再重拆 runtime host 与 render-server façade 的边界

### M5 Hybrid GI Preprocess And Runtime Host Baseline

在 capability-slot 与 `GlobalIllumination` pass skeleton 之上，当前又把 `Hybrid GI` 从“统一前处理 + CPU runtime host”推进到 renderer-local GPU completion source，但仍然刻意停在 scene/probe 表示、request planning、probe cache host 与 completion readback，不伪装真正的 radiance cache shading 或 RT lighting 已经存在。

当前新增的 scene/extract 合同是：

- `RenderHybridGiExtract`
  - `probe_budget`
  - `tracing_budget`
  - `probes`
  - `trace_regions`
- `RenderHybridGiProbe`
  - `entity`
  - `probe_id`
  - `position`
  - `radius`
  - `resident`
  - `ray_budget`
- `RenderHybridGiTraceRegion`
  - `entity`
  - `region_id`
  - `bounds_center`
  - `bounds_radius`
  - `screen_coverage`

`VisibilityContext` 当前新增三组 `Hybrid GI` 前处理输出：

- `hybrid_gi_active_probes`
- `hybrid_gi_update_plan`
- `hybrid_gi_feedback`

其中计划与反馈边界被固定成：

- `VisibilityHybridGiUpdatePlan.resident_probe_ids`
- `VisibilityHybridGiUpdatePlan.requested_probe_ids`
- `VisibilityHybridGiUpdatePlan.dirty_requested_probe_ids`
- `VisibilityHybridGiUpdatePlan.evictable_probe_ids`
- `VisibilityHybridGiUpdatePlan.scheduled_trace_region_ids`
- `VisibilityHybridGiFeedback.active_probe_ids`
- `VisibilityHybridGiFeedback.requested_probe_ids`
- `VisibilityHybridGiFeedback.evictable_probe_ids`

这一层的规则当前是：

- `build_hybrid_gi_plan(...)` 只消费统一 visibility 里的 `visible_entities`
- probe 会再做一次 sphere frustum test；trace region 也会按同一视图做 bounds 过滤
- active probe 排序按 `ray_budget` 优先，再按 `probe_id` 做稳定 tie-break
- trace region 排序按 `screen_coverage` 优先，再按 `region_id` 做稳定 tie-break
- `probe_budget` 会截断本帧真正发起的 non-resident probe request
- `tracing_budget` 会截断本帧真正进入 trace schedule 的 region 数
- `VisibilityHistorySnapshot` 当前额外保留 `hybrid_gi_requested_probes`，让 `dirty_requested_probe_ids` 只记录跨帧新增 request
- resident 但本帧不再 active 的 probe 会进入 `evictable_probe_ids`，作为 viewport cache eviction 的稳定前置信号

在 preprocess 之上，当前又补上了 viewport 级 `HybridGiRuntimeState`，固定承担：

- probe resident budget
- probe `ray_budget` metadata 记账
- resident probe -> slot 映射
- pending probe update 队列
- 本帧 `scheduled_trace_region_ids`
- 本帧 `evictable_probe_ids`

`ViewportRecord` 当前新增 `hybrid_gi_runtime`，`WgpuRenderServer::submit_frame_extract(...)` 现在会在 `BuiltinRenderFeature::GlobalIllumination` 真正进入有效 compiled pipeline 时：

- 先用 `register_extract(...)` 同步 extract 的 probe metadata 与 resident baseline
- 再用 `ingest_plan(...)` 吞入 visibility 生成的 resident/requested/dirty/evictable probe 计划以及 trace schedule
- capability gate 关闭时彻底移除 runtime host，而不是维持一个“伪激活”缓存
- 最后把 runtime host 规模写回 façade stats

这一层的规则当前是：

- `RenderHybridGiExtract.probe_budget` 当前会同时约束 CPU fallback runtime host 的 resident probe budget，并且至少覆盖 extract 自身声明的 resident baseline probe
- extract 中声明 `resident = true` 的 probe 会获得稳定 slot，不会在重复提交时重新编号
- 只有 `dirty_requested_probe_ids` 会转成 pending update，因此重复请求不会反复入队
- `apply_evictions(...)` 释放出来的 slot 会被后续 `fulfill_updates(...)` 优先复用
- 当前 headless `wgpu` 基线依旧不会创建有效 `HybridGiRuntimeState`，因为 `hybrid_global_illumination_supported` 的 capability gate 仍然关闭

render-server façade 当前也开始把这条前处理链与 runtime host 的规模暴露到 `RenderStats`：

- `last_hybrid_gi_active_probe_count`
- `last_hybrid_gi_requested_probe_count`
- `last_hybrid_gi_dirty_probe_count`
- `last_hybrid_gi_cache_entry_count`
- `last_hybrid_gi_resident_probe_count`
- `last_hybrid_gi_pending_update_count`
- `last_hybrid_gi_scheduled_trace_region_count`

这些计数与 Virtual Geometry 一样，只有在 `GlobalIllumination` 真正进入有效 compiled pipeline 时才会写入；在当前 headless `wgpu` 基线上，因为 capability gate 关闭，这些值会稳定保持 `0`。这一点正是当前 baseline 要证明的边界：preprocess/runtime host 合同可以先落地，但不会反向伪装旗舰执行路径已经打开。

### M5 Hybrid GI Feedback Streaming Baseline

在 runtime-host baseline 之后，当前又把此前未消费的 `VisibilityHybridGiFeedback` 接进了 viewport probe-cache 宿主。这样 pending probe update 与 trace schedule 不再只是停留在 `VisibilityContext` 的 request/feedback 输出里，而会在帧后按 runtime budget 推进到下一帧宿主状态。

这一层当前新增的合同是：

- `HybridGiRuntimeState::consume_feedback(&VisibilityHybridGiFeedback)`
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 完成后消费当前帧 feedback，再把更新后的 probe-cache runtime host 留在 viewport record
- façade stats 使用的是 feedback 消费之后的 runtime snapshot，因此当 capability 未来放开时，`cache-entry / resident-probe / pending-update / scheduled-trace` 规模会反映 submit 完成后的宿主状态

这一层的规则当前是：

- feedback 的 `scheduled_trace_region_ids` 会直接写入 runtime host
- 只有当前 feedback 中仍然处于 pending 的 `requested_probe_ids` 才会被尝试 promote
- resident 数达到 `probe_budget` 时，只允许回收当前 feedback 提供的 `evictable_probe_ids`
- 如果本帧没有足够的可回收 budget，剩余 probe update 会保持 `PendingUpdate`，而不是无上限扩 probe cache

这条实现仍然不是最终的 GI 执行路径：

- 还没有真实 radiance-cache lighting resolve
- 还没有 RT tracing backend 或 update completion 信号源
- 但 `VisibilityHybridGiFeedback` 已经不再是死数据结构，后续真正的 tracing/update backend 只需要替换 fulfillment 来源，而不需要重拆 viewport runtime host 与 render-server façade 的边界

## M3 Visibility Preprocess Baseline

这一轮已经从 “只有 batch/instancing 结构” 进入到真正消费视图数据的 M3 基线，但仍然刻意停在统一前处理层，不把后续 draw submission 或 feature 私有逻辑提前耦合进去。

当前已经固定下来的前处理边界是：

- `zircon_scene::RenderMeshSnapshot` 现在显式携带 `mobility` 和 `render_layer_mask`，让 legacy snapshot 兼容桥也能保留 visibility 元数据
- `zircon_scene::VisibilityInput` 新增 `renderables`，元素类型为 `VisibilityRenderableInput`
- `World::build_viewport_render_packet(...)` 会按 `node_id` 稳定排序 mesh/light extract，避免 `HashMap` 迭代顺序把 batch key 和缓存行为变成随机结果
- `SceneViewportExtractRequest` 新增 `viewport_size`，`ViewportCameraSnapshot` 新增 `aspect_ratio`，`RenderFrameExtract` 提供 `apply_viewport_size(...)` / `with_viewport_size(...)`，让 scene extract 自己持有真实视口纵横比
- editor/runtime submit bridge 会在提交前补丁 `RenderFrameExtract` 的 viewport size，作为现阶段 consumer 侧安全网；真正的相机/视图语义仍然归 `zircon_scene`
- camera gizmo frustum overlay 现在直接使用 extract 上的 `aspect_ratio`，不再退回硬编码 `16:9`
- `zircon_graphics::VisibilityContext` 不再只是三组 entity id，而是会基于 mesh extract 生成稳定的 `VisibilityBatchKey` / `VisibilityBatch`
- `VisibilityContext` 现在额外暴露 `visible_entities`、`culled_entities`、`visible_batches`，把 “结构化 batch” 与 “视图裁剪结果” 明确分层
- frustum culling 当前已经进入统一 visibility 前处理：同时支持 perspective / orthographic camera，并使用 view-space sphere test 过滤 mesh batch 成员
- 当前 culling 半径仍然是保守占位实现：`radius = mesh.transform.scale.abs().length() * 0.5`。这只是给后续真实 mesh bounds、BVH、cluster/virtual geometry 接入预留槽位，不是最终几何界限模型
- `gpu_instancing_candidates` 现在只来自 `visible_batches`，因此同一个 structural batch 只要有成员被裁掉，就不会继续把整组实例错误地当成当前 pass 可直接实例化的可见 draw
- `VisibilityContext` 现在还会生成连续的 `visible_instances` 与 `draw_commands`：前者是按稳定 batch 顺序压实后的实例列表，后者只保存 `visible_instance_offset + visible_instance_count + batch key`，作为后续 indirect draw/upload buffer 的中立脚手架，而不是直接绑定某个后端的原生参数结构
- `VisibilityContext::from_extract_with_history(...)` 现在支持把统一前处理扩到 BVH/AS 脏区规划：`bvh_instances` 提供当前场景实例边界，`history_snapshot` 记录上一帧可比较状态，`bvh_update_plan` 输出 `FullRebuild / Incremental` 策略以及 inserted/updated/removed entity 集合
- `VisibilityContext` 现在还会输出 `instance_upload_plan`：把当前实例按 static/dynamic 拆开，并且只把需要重传的 dynamic entity 放进 `dirty_dynamic_entities`，从而把实例上传准备也收回统一前处理层，而不是交给各个 mesh/RT feature 自己重新 diff
- `VisibilityContext` 现在也会消费 `ParticleExtract.emitters`，并输出 `particle_upload_plan`：当前阶段只做 emitter membership 级别的 `dirty_emitters / removed_emitters` 规划，不伪造完整粒子模拟参数，但已经把粒子上传准备接进同一条历史 diff/上传准备边界

这条实现故意保持“只做前处理结构，不做后续 draw submission 特化”的边界：

- 目前没有把 occlusion culling、真实 BVH 节点构建、TLAS/BLAS 上传、RT instance buffer 编码直接塞进 `VisibilityContext`
- 目前也没有让具体 feature 在 pass 里各自重新做一遍分类；统一 batch 结果仍然归 `visibility/` 负责，后续只是在这里继续向下加层
- `RenderPipelineAsset::compile(...)` 仍然保持 asset/feature graph 编译职责，不把 per-frame visibility/batching 混进 pipeline asset 编译期

## Validation Status

当前已验证通过的内容：

- `zircon_rhi` 描述符与 capability 基线
- `zircon_rhi_wgpu` 的 capability fallback
- `zircon_render_graph` 的排序与 cycle rejection
- `zircon_render_server` 的稳定 handle/type 契约
- `RenderServer::query_stats()` 的 capability plumbing：`wgpu` 基线能力现在会通过 `RenderCapabilitySummary` 进入 façade 侧统计快照，供上层按 capability 做 feature gate，而不需要直接接触 `zircon_rhi` / `wgpu`
- `RenderQualityProfile` 的 pipeline override：viewport 在没有显式 pipeline 绑定时，quality profile 现在可以稳定选择 built-in deferred pipeline 作为默认 renderer
- `RenderQualityProfile` 的 M4 feature toggles：当前可以直接控制 `clustered lighting / SSAO / history resolve` 以及 async-compute 偏好，而不需要 consumer 直接接触 renderer 内部类型
- `RenderQualityProfile` 的 M5 flagship toggles：当前可以对 `virtual geometry / hybrid global illumination` 发出 opt-in 请求，同时继续通过 capability gate 保证纯 `wgpu` 基线不会被 profile 强行打开
- `RenderStats.last_frame_history`：render façade 现在会把最新 viewport history handle 暴露到统计快照，便于验证跨帧资源宿主是否稳定工作
- `RenderStats.last_effective_features / last_async_compute_pass_count`：render façade 现在能暴露当前 pipeline 在 quality/capability 处理后的真正 feature 集和 async-compute 退化结果
- `RenderStats.capabilities.virtual_geometry_supported / hybrid_global_illumination_supported`：render façade 现在会把旗舰功能是否具备 backend 支撑显式暴露给上层
- `RenderStats.last_virtual_geometry_visible_cluster_count / last_virtual_geometry_requested_page_count / last_virtual_geometry_dirty_page_count`：render façade 现在已经预埋 Virtual Geometry 前处理计数，但当前 `wgpu` capability gate 仍会把它们保持在 `0`
- `RenderStats.last_virtual_geometry_page_table_entry_count / last_virtual_geometry_resident_page_count / last_virtual_geometry_pending_request_count`：render façade 现在还会暴露 Virtual Geometry runtime host 的 page-table / resident / pending-request 规模；当前 `wgpu` capability gate 关闭时它们同样保持 `0`
- `RenderStats.last_hybrid_gi_active_probe_count / last_hybrid_gi_requested_probe_count / last_hybrid_gi_dirty_probe_count`：render façade 现在已经预埋 Hybrid GI 前处理计数，但当前 `wgpu` capability gate 仍会把它们保持在 `0`
- `RenderStats.last_hybrid_gi_cache_entry_count / last_hybrid_gi_resident_probe_count / last_hybrid_gi_pending_update_count / last_hybrid_gi_scheduled_trace_region_count`：render façade 现在还会暴露 Hybrid GI runtime host 的 probe cache / resident probe / pending update / trace schedule 规模；当前 `wgpu` capability gate 关闭时它们同样保持 `0`
- `zircon_scene` 的 `RenderFrameExtract <-> RenderSceneSnapshot` 适配
- `zircon_graphics::runtime::WgpuRenderServer` 的 viewport 创建、pipeline/profile 设置、frame submit 与 stats 更新
- `zircon_graphics::pipeline::RenderPipelineAsset::compile(...)` 的确定性编译、duplicate stage/feature rejection，以及 `DebugOverlay` 独立 extract 依赖
- `zircon_graphics::pipeline::RenderPipelineAsset::default_deferred()` 的第二条内建 pipeline：固定 deferred stage/pass 顺序、built-in handle lookup，以及 `RenderServer` 侧的 built-in deferred pipeline 选择
- `zircon_graphics::pipeline::RenderPipelineAsset::compile(...)` 的 M4 compile contract：当前会稳定聚合 `history_bindings`，并把 built-in Forward+ / Deferred 编译到 `ssao-evaluate -> clustered-light-culling -> history-resolve` 这一组新的 pass 链
- `zircon_graphics::pipeline::RenderPipelineAsset::compile_with_options(...)`：当前已经支持显式禁用 M4 feature 和 async-compute lane fallback，从而让 quality profile / capability 能真正参与 built-in pipeline 编译
- `zircon_graphics::pipeline::RenderPipelineAsset::compile_with_options(...)` 的 M5 opt-in contract：当前已经支持用 `with_feature_enabled(...)` 显式打开 `VirtualGeometry` 与 `GlobalIllumination`，同时保证默认 Forward+ / Deferred 编译结果不被旗舰槽位污染
- `zircon_scene` 的 visibility 元数据保留：`RenderFrameExtract` 现在会保留 static/dynamic 分区、render layer mask，以及稳定排序后的 mesh/light extract
- `zircon_scene` 的 camera aspect propagation：viewport size 会进入 `SceneViewportExtractRequest`、`ViewportCameraSnapshot`、`RenderFrameExtract`，并同步到 camera gizmo frustum overlay
- `zircon_scene` 的 M5 extract 预埋：legacy snapshot adapter 当前会稳定初始化 `virtual_geometry = None` 与 `hybrid_global_illumination = None`
- `zircon_scene` 的 Virtual Geometry preprocess contract：当前已经公开 `RenderVirtualGeometryCluster`、`RenderVirtualGeometryPage` 与扩展后的 `RenderVirtualGeometryExtract`
- `zircon_scene` 的 Hybrid GI preprocess contract：当前已经公开 `RenderHybridGiProbe`、`RenderHybridGiTraceRegion` 与扩展后的 `RenderHybridGiExtract`
- `zircon_graphics::VisibilityContext` 的 M3 基线：稳定 batch key、deterministic batch ordering、frustum culling、visible/culled 分区，以及只对真正重复且仍然可见的动态 batch 暴露 instancing candidates
- `zircon_graphics::VisibilityContext` 的 GPU-driven 脚手架：`visible_instances` / `draw_commands` 会按稳定 batch 顺序压实可见实例，为后续 indirect draw args 与 instance upload 提供一致入口
- `zircon_graphics::VisibilityContext` 的 BVH dirty/update 框架：无历史时回退 `FullRebuild`，有历史时输出 `Incremental` 的 inserted/updated/removed entity 集合，且继续复用统一的保守 sphere bounds 占位模型
- `zircon_graphics::VisibilityContext` 的实例上传准备：`instance_upload_plan` 会分离 static/dynamic 实例，并且只标记本帧需要重传的 dynamic entity，避免把后续 instance upload policy 分散回 renderer feature
- `zircon_graphics::VisibilityContext` 的粒子上传准备：`particle_upload_plan` 会在没有历史时对全部 emitter 做全量上传，在有历史时只标记新增/移除的 emitter，为未来真正的粒子 GPU buffer/upload policy 预埋统一入口
- `zircon_graphics::VisibilityContext` 的 Virtual Geometry 前处理：当前已经能输出 cluster-level 可见集、resident/requested/dirty/evictable page 计划，以及稳定的 feedback 请求集合
- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 的 prepare snapshot：当前已经能把 resident/pending/evictable page 与 visible cluster 合成为 `VirtualGeometryPrepareFrame`
- `zircon_graphics::VisibilityContext` 的 Hybrid GI 前处理：当前已经能输出 active probe、resident/requested/dirty/evictable probe 计划，以及稳定的 trace schedule / feedback 请求集合
- `zircon_graphics::runtime::WgpuRenderServer` 的 viewport history host：当前已经能在兼容的重复提交间复用 `FrameHistoryHandle`，并在 pipeline 切换时轮换 handle，同时继续复用统一 visibility history 作为跨帧 diff 输入
- `zircon_graphics::runtime::WgpuRenderServer` 的 M4 quality/capability mapping：当前会把 profile/caps 编译成有效 pipeline，headless `wgpu` 会把 async-compute pass cleanly 降级到 graphics queue，并把 effective feature 结果写回 façade stats
- `zircon_graphics::feature::BuiltinRenderFeature` 的 M5 skeleton：当前 opt-in 后会稳定编译出 `virtual-geometry-prepare` 与 `hybrid-gi-resolve`，并把 `GlobalIllumination` history slot 聚合进 `history_bindings`
- `zircon_graphics::runtime::WgpuRenderServer` 的 M5 capability gate：headless `wgpu` 当前会把 `virtual_geometry_supported / hybrid_global_illumination_supported` 暴露为 `false`，并阻止这两条旗舰 feature 进入 `last_effective_features`
- `zircon_graphics::runtime::WgpuRenderServer` 的 Virtual Geometry stats plumbing：当前 submit 路径已经能把 Virtual Geometry 可见 cluster / requested page / dirty page 数，以及 runtime host 的 page-table / resident / pending-request 规模写回 façade stats；capability gate 关闭时会稳定回落到 `0`
- `zircon_graphics::visibility::build_virtual_geometry_plan(...)` 的 hierarchy refine baseline：当前已经支持 `parent_cluster_id` 驱动的 budget-aware refine frontier，children 只会在替换 parent 后仍不超过 `cluster_budget` 时进入最终可见集
- `zircon_graphics::runtime::VirtualGeometryRuntimeState` 的 feedback consumption baseline：当前 pending page request 会在 resident budget 内消费 feedback 并推进为 resident；没有可回收 budget 时则会继续保持 pending
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry GPU completion baseline：当前 renderer 已经会把 resident page table 与 pending request 上传到真实 `wgpu` storage buffer，执行最小 compute uploader，并把 `page_table_entries / completed_page_ids` 通过 readback 返回给 runtime host
- `zircon_graphics::runtime::WgpuRenderServer` 的 Virtual Geometry post-render progression：当前 submit 路径已经会优先消费 renderer GPU readback，再回退到 `VisibilityVirtualGeometryFeedback`，从而让下一帧 prepare snapshot 可以观察到 residency 变化
- `zircon_graphics::scene::SceneRenderer` 的 Virtual Geometry fallback consumption：当前当 `VirtualGeometry` feature 显式开启且 frame 挂有 prepare snapshot 时，mesh fallback path 会 honor `visible_entities` 过滤结果，从而让 `virtual-geometry-prepare` 第一次真正改变当前离屏输出
- `zircon_graphics::runtime::WgpuRenderServer` 的 Hybrid GI stats plumbing：当前 submit 路径已经能把 Hybrid GI active/requested/dirty probe 数，以及 runtime host 的 cache-entry / resident-probe / pending-update / scheduled-trace 规模写回 façade stats；capability gate 关闭时会稳定回落到 `0`
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 feedback consumption baseline：当前 pending probe update 会在 resident budget 内消费 feedback 并推进为 resident；没有可回收 budget 时则会继续保持 pending，同时 trace schedule 会被写回 runtime host
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 renderer prepare snapshot：当前 runtime host 已经能导出 `HybridGiPrepareFrame`，把 resident probe cache、pending update、trace schedule 与 evictable probe 列表显式交给 renderer
- `zircon_graphics::scene::SceneRenderer` 的 Hybrid GI GPU completion baseline：当前 renderer 已经会把 resident probe cache、pending update、scheduled trace region ids 上传到真实 `wgpu` compute/readback 路径，并返回 `completed_probe_ids / completed_trace_region_ids`
- `zircon_graphics::runtime::WgpuRenderServer` 的 Hybrid GI post-render progression：当前 submit 路径已经会优先消费 renderer GPU readback，再回退到 `VisibilityHybridGiFeedback`，从而让下一帧 runtime snapshot 可以观察到 probe residency 与 trace schedule 的变化
- `zircon_graphics::scene::SceneRenderer` 的 Hybrid GI radiance-cache lighting resolve baseline：post-process 现在会直接消费 `EditorOrRuntimeFrame.hybrid_gi_prepare`，把 resident probe 的 `irradiance_rgb` 编成 storage buffer，并在 `post_process.wgsl` 里生成真正影响最终帧的间接光贡献
- `zircon_graphics::runtime::HybridGiRuntimeState` 的 probe irradiance slot：当前 runtime host 已经会在 build-prepare 阶段为 resident probe 填充稳定的默认 irradiance，从而把后续真实 radiance-cache 输出的落点固定在 prepare/runtime 合同里
- `zircon_graphics::scene::SceneRenderer` 的真实 M4 runtime path：当前已经会为 `RenderServer` 路径建立 `final_color / scene_color / bloom / gbuffer_albedo / normal / ambient_occlusion / depth / cluster_buffer` 中间资源，并按 feature 集真实分支 forward 与 deferred；forward 继续执行 mesh shader 直写 scene color，deferred 则执行 preview background、GBuffer、fullscreen deferred lighting、transparent 补绘、particle pass、bloom extract、post composite、history resolve 与 overlay
- `zircon_graphics::scene::DeferredSceneResources`：当前已经真正持有 deferred geometry 和 deferred lighting 两条 GPU pipeline，并且把 opaque 材质解码固定在 renderer 内部，而不是让 deferred 继续执行项目 fragment shader
- `zircon_graphics::runtime::offline_bake_frame(...)`：当前已经能从 extract 的方向光和几何体快照生成 `RenderBakedLightingExtract + Vec<RenderReflectionProbeSnapshot>`，并直接回灌到同一帧 runtime 数据路径
- `zircon_graphics::scene::SceneFrameHistoryTextures`：当前已经真正持有 `scene color` 与 `ambient occlusion` 两条 history texture，并在 viewport history handle 轮换或销毁时由 renderer 回收
- `zircon_graphics` 的 M4 integration renders：当前已经有离屏回归证明 history resolve 会保留上一帧颜色、SSAO 会让同一 scene 变暗、clustered lighting 会给同一 scene 带来可测量的 tile lighting tint、bloom 会扩散高亮邻域像素、color grading 会改变通道偏色、offline bake 输出会改变最终画面、particle billboard 会在 transparent stage 增加可测量热像素，而且 built-in deferred 会稳定改走 `GBuffer material decode -> deferred lighting`，与 forward project shader 路径出现可测量差异
- `zircon_graphics` 的 M5 capability-slot 回归：当前已经有单测证明默认 Forward+ 不会误带入 `VirtualGeometry / GlobalIllumination`，显式 opt-in 时会编译出新 pass 与 GI history slot，而 headless `wgpu` server 仍会把它们 gate 为关闭
- `zircon_graphics` 的 Hybrid GI resolve 离屏回归：当前已经有离屏测试证明 resident probe 会让最终帧变亮，而且不同 `irradiance_rgb` 会把最终帧推向不同颜色通道，证明 `runtime prepare -> GPU resource -> shader resolve` 已经形成真实数据闭环
- `zircon_graphics::visibility` 的 support-layer 编译边界：`culling/` 与 `planning/` 的 helper 现在通过显式模块路径暴露给 `VisibilityContext`，`is_mesh_visible(...)` 也稳定改用 `transform_point3(...)`，从而恢复 `cargo test -p zircon_graphics --lib --locked`
- `zircon_editor` 的 Slint viewport controller 通过 `RenderServer` 创建/重建 viewport，并从 capture 拉回最新帧
- `zircon_editor` 的 shared viewport toolbar pointer route 通过同一 runtime dispatch 路径触发 typed `ViewportCommand`
- `zircon_entry` 的 runtime presenter bridge 通过 `RenderServer` 管理 viewport 生命周期并返回最新 captured frame
- `zircon_editor` 的 shared pointer callback source-location tests 现在接受 `app.rs` 与 `app/callback_wiring.rs` 双路径，从而保持 root entry file 精简和 pointer wiring 模块化并存
- editor/runtime 的源码边界测试会阻止 `wgpu`、`RuntimePreviewRenderer`、`SharedTextureRenderService` 等旧上层消费路径重新回流
- 受影响 crate 当前已通过：
  - `cargo test -p zircon_render_server --locked`
  - `cargo test -p zircon_scene --locked`
  - `cargo test -p zircon_graphics pipeline_compile --locked`
  - `cargo test -p zircon_graphics compile_options_can_opt_in_virtual_geometry_and_hybrid_gi_features --locked`
  - `cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked`
  - `cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked`
  - `cargo test -p zircon_graphics virtual_geometry_gpu --locked`
  - `cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked`
  - `cargo test -p zircon_graphics visibility_context_builds_hybrid_gi_probe_and_trace_plan --locked`
  - `cargo test -p zircon_graphics visibility_context_with_history_tracks_hybrid_gi_requested_probes --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked`
  - `cargo test -p zircon_graphics hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots --locked`
  - `cargo test -p zircon_graphics hybrid_gi --locked`
  - `cargo test -p zircon_graphics hybrid_gi_resolve_uses_prepare_probe_irradiance_colors --locked`
  - `cargo test -p zircon_graphics history_resolve_blends_previous_scene_color_when_enabled --locked`
  - `cargo test -p zircon_graphics ssao_quality_profile_darkens_scene_when_enabled --locked`
  - `cargo test -p zircon_graphics clustered_lighting_quality_profile_applies_runtime_tile_lighting --locked`
  - `cargo test -p zircon_graphics deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --locked`
  - `cargo test -p zircon_graphics bloom_quality_profile_spreads_bright_pixels_when_enabled --locked`
  - `cargo test -p zircon_graphics color_grading_extract_tints_scene_after_post_process --locked`
  - `cargo test -p zircon_graphics offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering --locked`
  - `cargo test -p zircon_graphics particle_rendering_draws_billboard_sprites_in_transparent_stage --locked`
  - `cargo test -p zircon_graphics render_server_bridge --locked`
  - `cargo test -p zircon_graphics visibility --locked`
  - `cargo test -p zircon_graphics --lib --locked`
  - `cargo test -p zircon_render_server --locked`
  - `cargo test -p zircon_entry --lib --locked`
  - `cargo test -p zircon_editor --lib --locked`
  - `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_graphics`

当前还没有完成的验证：

- assetized `RenderPipelineAsset` 真正接入 shader/material/feature 选择
- GPU-driven visibility 的 occlusion、真正 indirect args buffer 编码、真实 BVH 构建
- `Virtual Geometry` 的 cluster streaming / feedback residency manager / 深层 split-merge hierarchy refinement
- `Hybrid GI` 的 radiance cache / probe gather / RT hybrid lighting

