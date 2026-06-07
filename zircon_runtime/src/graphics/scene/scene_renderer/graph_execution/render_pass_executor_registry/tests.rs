use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderPipelineHandle, RenderPluginRendererOutputs,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
    RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    RenderPassId,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};
use crate::{CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions};

use super::super::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassExecutorRegistration, RenderPassGpuExecutionContext,
};
use super::RenderPassExecutorRegistry;
use support::{import_test_texture, test_extract, test_ui_extract, ContextMutatingExecutor};

#[path = "plugin_executor_policy.rs"]
mod plugin_executor_policy;
#[path = "support.rs"]
mod support;

#[test]
fn registry_rejects_unregistered_executor_ids() {
    let registry = RenderPassExecutorRegistry::default();
    let error = registry
        .execute(&mut RenderPassExecutionContext::new(
            "custom-pass",
            RenderPassExecutorId::new("custom.executor"),
        ))
        .unwrap_err();

    assert_eq!(
        error,
        "render pass executor `custom.executor` is not registered"
    );
}

#[test]
fn execution_context_records_graph_queue_and_pass_flags() {
    let context = RenderPassExecutionContext::with_graph_metadata(
        "async-virtual-geometry-cull",
        RenderPassExecutorId::new("virtual-geometry.node-cluster-cull"),
        QueueLane::AsyncCompute,
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    );

    assert_eq!(context.declared_queue, QueueLane::AsyncCompute);
    assert_eq!(context.queue, QueueLane::AsyncCompute);
    assert_eq!(
        context.flags,
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        }
    );
    assert!(context.resources.is_empty());

    let resources = vec![RenderGraphPassResourceAccess {
        name: "virtual-geometry-visible-clusters".to_string(),
        kind: RenderGraphResourceKind::TransientBuffer,
        access: RenderGraphResourceAccessKind::Read,
        attachment_ops: None,
    }];
    let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "async-virtual-geometry-visbuffer",
        RenderPassExecutorId::new("virtual-geometry.visbuffer"),
        QueueLane::Graphics,
        PassFlags::default(),
        resources.clone(),
    );

    assert_eq!(context.resources, resources);
    assert!(context.dependencies.is_empty());
    assert!(!context.uses_queue_fallback());

    let context = RenderPassExecutionContext::with_declared_graph_metadata(
        "fallback-ssao",
        RenderPassExecutorId::new("ao.ssao-evaluate"),
        QueueLane::Graphics,
        QueueLane::AsyncCompute,
        PassFlags::default(),
    );
    assert_eq!(context.queue, QueueLane::Graphics);
    assert_eq!(context.declared_queue, QueueLane::AsyncCompute);
    assert!(context.uses_queue_fallback());

    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            "lighting",
            RenderPassExecutorId::new("lighting.clustered-cull"),
            QueueLane::Graphics,
            QueueLane::Graphics,
            PassFlags::default(),
            vec![RenderPassId(1), RenderPassId(3)],
            Vec::new(),
        );
    assert_eq!(context.dependencies, vec![RenderPassId(1), RenderPassId(3)]);
}

#[test]
fn builtin_registry_covers_compiled_pipeline_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
    let extract = test_extract();
    let forward = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
        .unwrap();
    let deferred = RenderPipelineAsset::default_deferred()
        .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
        .unwrap();

    for pipeline in [&forward, &deferred] {
        registry
            .validate_compiled_pipeline(pipeline)
            .expect("builtin registry should cover all compiled executor ids");
        for pass in pipeline.graph.passes() {
            let executor_id = pass
                .executor_id
                .as_ref()
                .expect("compiled SRP passes should carry executor ids");
            assert!(
                registry.contains(&RenderPassExecutorId::new(executor_id.clone())),
                "builtin registry should cover executor `{executor_id}` for pass `{}`",
                pass.name
            );
        }
    }
}

#[test]
fn builtin_registry_excludes_pluginized_advanced_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in [
        "virtual-geometry.prepare",
        "virtual-geometry.node-cluster-cull",
        "virtual-geometry.page-feedback",
        "virtual-geometry.visbuffer",
        "virtual-geometry.debug-overlay",
        "hybrid-gi.scene-prepare",
        "hybrid-gi.trace-schedule",
        "hybrid-gi.resolve",
        "hybrid-gi.history",
        "particle.transparent",
    ] {
        assert!(
            !registry.contains(&RenderPassExecutorId::new(executor_id)),
            "core built-in registry should not carry pluginized executor `{executor_id}`"
        );
    }
}

#[test]
fn builtin_registry_covers_product_postprocess_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in [
        "post.bloom",
        "post.bloom-extract",
        "post.motion-vector-clear",
        "post.motion-vector-camera",
        "post.motion-vector-tile-max",
        "post.motion-vector-tile-max-coarse",
        "post.motion-vector-neighbor-max",
        "post.depth-of-field-prepare",
        "post.screen-space-reflection-depth-pyramid",
        "post.screen-space-reflection-depth-pyramid-coarse",
        "post.screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-resolve",
        "post.screen-space-reflection-specular-occlusion",
        "post.color-grade",
        "post.stack",
        "history.scene-color",
        "post.history-resolve",
        "post.effect-stack",
        "post.final-composite",
        "post.fxaa",
    ] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "product postprocess executor `{executor_id}` should be registered"
        );
    }
}

#[test]
fn builtin_registry_covers_preview_sky_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in ["sky.preview-scene-color", "sky.preview-final-color"] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "preview sky executor `{executor_id}` should be registered"
        );
    }
}

#[test]
fn builtin_registry_covers_runtime_ui_executor_id() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in ["ui.screen-space", "overlay.gizmo"] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "`{executor_id}` should be registered as a graph-owned built-in"
        );
    }
}

#[test]
fn product_postprocess_executor_rejects_missing_gpu_resources() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let mut extract = test_extract();
    extract.post_process.rebuild_graph(true, true);
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("postprocess-product-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("postprocess-product-test-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("postprocess-product-test-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_COLOR,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::new(
        "history-resolve",
        RenderPassExecutorId::new("post.history-resolve"),
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "render graph execution texture resource `history.previous.scene-color` is not bound"
    );
}

#[test]
fn post_stack_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context("post-process", "post.stack");

    assert_eq!(
        error,
        "post-process stack graph executor for pass `post-process` requires post-process stack context"
    );
}

#[test]
fn ssao_executor_requires_post_process_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("ssao-evaluate", "ao.ssao-evaluate");

    assert_eq!(
        error,
        "SSAO graph executor for pass `ssao-evaluate` requires post-process stack context"
    );
}

#[test]
fn clustered_lighting_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "clustered-light-culling",
        "lighting.clustered-cull",
    );

    assert_eq!(
        error,
        "clustered lighting graph executor for pass `clustered-light-culling` requires post-process stack context"
    );
}

#[test]
fn bloom_extract_executor_requires_post_process_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("bloom-extract", "post.bloom-extract");

    assert_eq!(
        error,
        "bloom graph executor for pass `bloom-extract` requires post-process stack context"
    );
}

#[test]
fn motion_vector_clear_executor_requires_graph_target_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "motion-vector-clear",
        "post.motion-vector-clear",
    );

    assert_eq!(
        error,
        "render graph execution texture resource `scene-motion-vector` is not bound"
    );
}

#[test]
fn motion_vector_camera_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "motion-vector-camera",
        "post.motion-vector-camera",
    );

    assert_eq!(
        error,
        "motion-vector camera graph executor for pass `motion-vector-camera` requires post-process stack context"
    );
}

#[test]
fn motion_vector_tile_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "motion-vector-tile-max",
        "post.motion-vector-tile-max",
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max` requires post-process stack context"
    );
}

#[test]
fn motion_vector_tile_max_coarse_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "motion-vector-tile-max-coarse",
        "post.motion-vector-tile-max-coarse",
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max-coarse` requires post-process stack context"
    );
}

#[test]
fn motion_vector_neighbor_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "motion-vector-neighbor-max",
        "post.motion-vector-neighbor-max",
    );

    assert_eq!(
        error,
        "motion-vector neighbor-max graph executor for pass `motion-vector-neighbor-max` requires post-process stack context"
    );
}

#[test]
fn depth_of_field_prepare_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "depth-of-field-prepare",
        "post.depth-of-field-prepare",
    );

    assert_eq!(
        error,
        "depth-of-field prepare graph executor for pass `depth-of-field-prepare` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_resolve_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context(
        "screen-space-reflection-resolve",
        "post.screen-space-reflection-resolve",
    );

    assert_eq!(
        error,
        "screen-space reflection resolve graph executor for pass `screen-space-reflection-resolve` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_depth_pyramid_executor_requires_post_process_context_instead_of_nooping()
{
    let error = execute_gpu_executor_without_specialized_context(
        "screen-space-reflection-depth-pyramid",
        "post.screen-space-reflection-depth-pyramid",
    );

    assert_eq!(
        error,
        "screen-space reflection depth-pyramid graph executor for pass `screen-space-reflection-depth-pyramid` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_depth_pyramid_coarse_executor_requires_post_process_context_instead_of_nooping(
) {
    let error = execute_gpu_executor_without_specialized_context(
        "screen-space-reflection-depth-pyramid-coarse",
        "post.screen-space-reflection-depth-pyramid-coarse",
    );

    assert_eq!(
        error,
        "screen-space reflection depth-pyramid coarse graph executor for pass `screen-space-reflection-depth-pyramid-coarse` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_executor_requires_post_process_context_instead_of_nooping(
) {
    let error = execute_gpu_executor_without_specialized_context(
        "screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid",
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid graph executor for pass `screen-space-reflection-reflection-pyramid` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_coarse_executor_requires_post_process_context_instead_of_nooping(
) {
    let error = execute_gpu_executor_without_specialized_context(
        "screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-reflection-pyramid-coarse",
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid coarse graph executor for pass `screen-space-reflection-reflection-pyramid-coarse` requires post-process stack context"
    );
}

#[test]
fn preview_sky_executor_requires_preview_renderer_context_instead_of_nooping() {
    let error =
        execute_gpu_executor_without_specialized_context("preview-sky", "sky.preview-scene-color");

    assert_eq!(
        error,
        "preview sky graph executor for pass `preview-sky` requires preview sky renderer context"
    );
}

fn execute_gpu_executor_without_specialized_context(pass_name: &str, executor_id: &str) -> String {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("post-process-context-missing-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("post-process-context-missing-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post-process-context-missing-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context =
        RenderPassExecutionContext::new(pass_name, RenderPassExecutorId::new(executor_id))
            .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err()
}

#[test]
fn screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16))
        .with_ui(Some(test_ui_extract()));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("screen-space-ui-attachment-ops-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen-space-ui-attachment-ops-test-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen-space-ui-attachment-ops-test-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "runtime-ui",
        RenderPassExecutorId::new("ui.screen-space"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::VIEWPORT_OUTPUT.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap();

    assert_eq!(
        context
            .gpu()
            .unwrap()
            .screen_space_ui_renderer
            .last_attachment_ops(),
        RenderGraphAttachmentOps::clear_store()
    );
}

#[test]
fn overlay_executor_requires_overlay_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("overlay-missing-context-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("overlay-missing-context-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay-missing-context-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "overlay-gizmo",
        RenderPassExecutorId::new("overlay.gizmo"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::VIEWPORT_OUTPUT.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::load_store()),
        }],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "overlay graph executor for pass `overlay-gizmo` requires overlay renderer context"
    );
}

#[test]
fn sprite_executor_requires_renderer_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sprite-executor-missing-renderer-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite-executor-missing-renderer-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite-executor-missing-renderer-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_COLOR,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "sprite-transparent",
        RenderPassExecutorId::new("sprite.transparent"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "sprite graph executor for stage `Transparent2d` requires sprite renderer context"
    );
}

#[test]
fn mesh_executor_requires_mesh_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mesh-executor-missing-context-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mesh-executor-missing-context-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh-executor-missing-context-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_COLOR,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "opaque-mesh",
        RenderPassExecutorId::new("mesh.opaque"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "mesh graph executor for stage `Opaque3d` requires mesh draw context"
    );
}

#[test]
fn depth_prepass_executor_requires_prepass_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("depth-prepass-missing-context-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("depth-prepass-missing-context-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth-prepass-missing-context-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "depth-prepass",
        RenderPassExecutorId::new("mesh.depth-prepass"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![
            RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string(),
                kind: RenderGraphResourceKind::External,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
            },
            RenderGraphPassResourceAccess {
                name: PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
                kind: RenderGraphResourceKind::External,
                access: RenderGraphResourceAccessKind::Write,
                attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
            },
        ],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "depth prepass graph executor for pass `depth-prepass` requires normal prepass context"
    );
}

#[test]
fn shadow_map_executor_requires_graph_shadow_map_resource_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context("shadow-map", "shadow.map");

    assert_eq!(
        error,
        "render graph execution texture resource `shadow-map` is not bound"
    );
}

#[test]
fn shadow_map_executor_records_depth_only_pass_when_graph_resource_is_bound() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("shadow-map-executor-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow-map-executor-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow-map-executor-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_shadow_map_texture(&mut resources, &backend.device);
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "shadow-map",
        RenderPassExecutorId::new("shadow.map"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: "shadow-map".to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap();
}

#[test]
fn deferred_gbuffer_executor_requires_renderer_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("deferred-gbuffer-missing-context-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("deferred-gbuffer-missing-context-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred-gbuffer-missing-context-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
    );
    import_test_texture(
        &mut resources,
        &backend.device,
        PostProcessGraphResourceNames::SCENE_DEPTH,
    );
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "gbuffer-mesh",
        RenderPassExecutorId::new("deferred.gbuffer"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::GBUFFER_ALBEDO.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "deferred graph executor for pass `gbuffer-mesh` requires deferred renderer context"
    );
}

#[test]
fn deferred_lighting_executor_requires_renderer_context_instead_of_nooping() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("deferred-lighting-missing-context-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("deferred-lighting-missing-context-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred-lighting-missing-context-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    for resource in [
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        PostProcessGraphResourceNames::FINAL_COLOR,
        PostProcessGraphResourceNames::SCENE_COLOR,
    ] {
        import_test_texture(&mut resources, &backend.device, resource);
    }
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new(
        Arc::new(ProjectAssetManager::default()),
        &backend.device,
        &backend.queue,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let gpu = RenderPassGpuExecutionContext::new_for_test(
        &backend.device,
        &backend.queue,
        &mut encoder,
        &frame,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "deferred-lighting",
        RenderPassExecutorId::new("lighting.deferred"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
            kind: RenderGraphResourceKind::External,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: Some(RenderGraphAttachmentOps::clear_store()),
        }],
    )
    .with_gpu(gpu);

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap_err();

    assert_eq!(
        error,
        "deferred graph executor for pass `deferred-lighting` requires deferred renderer context"
    );
}

#[test]
fn registry_invokes_object_backed_executor_with_mutable_context() {
    let mut registry = RenderPassExecutorRegistry::default();
    registry.register_executor(
        RenderPassExecutorId::new("object.executor"),
        Arc::new(ContextMutatingExecutor),
    );
    let mut context = RenderPassExecutionContext::new(
        "object-pass",
        RenderPassExecutorId::new("object.executor"),
    );

    registry.execute(&mut context).unwrap();

    assert_eq!(context.pass_name, "object-pass:executed");
}

#[test]
fn registry_rejects_compiled_pipeline_with_unknown_executor_id() {
    let mut graph = RenderGraphBuilder::new("custom-pipeline");
    graph.add_pass_with_executor("custom-pass", QueueLane::Graphics, Some("custom.executor"));
    let pipeline = CompiledRenderPipeline {
        handle: RenderPipelineHandle::new(42),
        name: "custom pipeline".to_string(),
        renderer_name: "custom renderer".to_string(),
        stages: Vec::new(),
        pass_stages: Vec::new(),
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        graph: graph.compile().unwrap(),
    };

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .validate_compiled_pipeline(&pipeline)
        .unwrap_err();

    assert_eq!(
        error,
        "render pass `custom-pass` references unregistered executor `custom.executor`"
    );
}

#[test]
fn registry_rejects_executable_compiled_pipeline_pass_without_executor_id() {
    let mut graph = RenderGraphBuilder::new("custom-pipeline");
    graph.add_pass("custom-pass", QueueLane::Graphics);
    let pipeline = CompiledRenderPipeline {
        handle: RenderPipelineHandle::new(44),
        name: "custom pipeline".to_string(),
        renderer_name: "custom renderer".to_string(),
        stages: Vec::new(),
        pass_stages: Vec::new(),
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        graph: graph.compile().unwrap(),
    };

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .validate_compiled_pipeline(&pipeline)
        .unwrap_err();

    assert_eq!(error, "render pass `custom-pass` has no executor id");
}

#[test]
fn registry_ignores_culled_pass_with_unknown_executor_id() {
    let mut graph = RenderGraphBuilder::new("custom-pipeline");
    let unused = graph.create_transient_texture(TextureDesc::new(
        "unused-target",
        1,
        1,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let pass =
        graph.add_pass_with_executor("culled-pass", QueueLane::Graphics, Some("custom.culled"));
    graph.write_texture(pass, unused).unwrap();
    let compiled_graph = graph.compile().unwrap();
    assert!(
        compiled_graph
            .passes()
            .iter()
            .any(|pass| pass.name == "culled-pass" && pass.culled),
        "test fixture should produce a culled pass"
    );
    let pipeline = CompiledRenderPipeline {
        handle: RenderPipelineHandle::new(43),
        name: "custom pipeline".to_string(),
        renderer_name: "custom renderer".to_string(),
        stages: Vec::new(),
        pass_stages: Vec::new(),
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        graph: compiled_graph,
    };

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .validate_compiled_pipeline(&pipeline)
        .expect("culled passes should not require executor registration");
}

fn import_shadow_map_texture(resources: &mut RenderGraphExecutionResources, device: &wgpu::Device) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow-map"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    resources.import_texture_view(
        "shadow-map",
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );
}
