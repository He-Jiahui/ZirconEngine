use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasSettings, PostProcessGraphResourceNames, RenderDepthOfFieldSettings,
    RenderFrameExtract, RenderMotionBlurSettings, RenderPipelineHandle,
    RenderPluginRendererOutputs, RenderPostProcessEffectStackSettings,
    RenderScreenSpaceReflectionSettings,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::graphics::{CompiledRenderPipeline, RenderPipelineAsset, RenderPipelineCompileOptions};
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder,
    RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
    RenderPassId,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::super::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassGpuExecutionContext,
};
use super::RenderPassExecutorRegistry;
use support::{
    import_test_buffer, import_test_texture, test_extract, test_ui_extract, ContextMutatingExecutor,
};

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
            RenderPassExecutorId::new("lighting.light-grid"),
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
        "temporal.velocity-camera",
        "temporal.velocity-object",
        "temporal.taa-reactive-mask-clear",
        "temporal.taa-reactive-mask-mesh",
        "temporal.taa-resolve",
        "post.motion-vector-tile-max",
        "post.motion-vector-tile-max-coarse",
        "post.motion-vector-neighbor-max",
        "post.motion-blur",
        "post.blur",
        "post.depth-of-field",
        "post.depth-of-field-prepare",
        "post.scene-composite",
        "post.exposure.histogram",
        "post.exposure.resolve",
        "post.screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-resolve",
        "post.screen-space-reflection-specular-occlusion",
        "visibility.hzb-build",
        "visibility.hzb-occlusion-cull",
        "particle.transparent",
        "post.color-lut-bake",
        "post.uber",
        "post.upscale",
        "post.output-transfer",
        "post.fxaa",
        "post.smaa",
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
fn taa_reactive_mask_clear_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-reactive-mask-clear",
        "temporal.taa-reactive-mask-clear",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA reactive mask clear graph executor for pass `taa-reactive-mask-clear` requires post-process stack context"
    );
}

#[test]
fn taa_reactive_mask_mesh_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-reactive-mask-mesh",
        "temporal.taa-reactive-mask-mesh",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA reactive mask mesh graph executor for pass `taa-reactive-mask-mesh` requires mesh draw context"
    );
}

#[test]
fn taa_resolve_executor_requires_graph_resources_instead_of_nooping() {
    let mut extract = test_extract();
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
        .post_process
        .rebuild_graph_with_anti_alias(true, true, &AntiAliasSettings::taa());

    let error = execute_gpu_executor_without_specialized_context_for_extract(
        "taa-resolve",
        "temporal.taa-resolve",
        extract,
    )
    .unwrap_err();

    assert_eq!(
        error,
        "TAA resolve graph executor for pass `taa-resolve` requires post-process stack context"
    );
}

#[test]
fn uber_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context("uber", "post.uber");

    assert_eq!(
        error,
        "post-process stack graph executor for pass `uber` requires post-process stack context"
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
    let error =
        execute_gpu_executor_without_specialized_context("light-grid-build", "lighting.light-grid");

    assert_eq!(
        error,
        "light grid graph executor for pass `light-grid-build` requires post-process stack context"
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
fn velocity_camera_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "velocity-camera",
        "temporal.velocity-camera",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "velocity camera graph executor for pass `velocity-camera` requires post-process stack context"
    );
}

#[test]
fn velocity_object_executor_requires_graph_target_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "velocity-object",
        "temporal.velocity-object",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "render graph execution texture resource `scene-velocity` is not bound"
    );
}

#[test]
fn motion_vector_tile_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-tile-max",
        "post.motion-vector-tile-max",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max` requires post-process stack context"
    );
}

#[test]
fn motion_vector_tile_max_coarse_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-tile-max-coarse",
        "post.motion-vector-tile-max-coarse",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector tile-max graph executor for pass `motion-vector-tile-max-coarse` requires post-process stack context"
    );
}

#[test]
fn motion_vector_neighbor_max_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "motion-vector-neighbor-max",
        "post.motion-vector-neighbor-max",
        effect_stack_with_motion_vectors(),
    );

    assert_eq!(
        error,
        "motion-vector neighbor-max graph executor for pass `motion-vector-neighbor-max` requires post-process stack context"
    );
}

#[test]
fn depth_of_field_prepare_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "depth-of-field-prepare",
        "post.depth-of-field-prepare",
        effect_stack_with_depth_of_field(),
    );

    assert_eq!(
        error,
        "depth-of-field prepare graph executor for pass `depth-of-field-prepare` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_resolve_executor_requires_post_process_context_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-resolve",
        "post.screen-space-reflection-resolve",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection resolve graph executor for pass `screen-space-reflection-resolve` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_executor_requires_post_process_context_instead_of_nooping(
) {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid graph executor for pass `screen-space-reflection-reflection-pyramid` requires post-process stack context"
    );
}

#[test]
fn screen_space_reflection_reflection_pyramid_coarse_executor_requires_post_process_context_instead_of_nooping(
) {
    let error = execute_gpu_executor_without_specialized_context_with_effect_stack(
        "screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-reflection-pyramid-coarse",
        effect_stack_with_screen_space_reflection(),
    );

    assert_eq!(
        error,
        "screen-space reflection reflection-pyramid coarse graph executor for pass `screen-space-reflection-reflection-pyramid-coarse` requires post-process stack context"
    );
}

#[test]
fn optional_postprocess_executors_skip_resource_work_when_effects_are_disabled() {
    for (pass_name, executor_id) in [
        ("velocity-object", "temporal.velocity-object"),
        ("velocity-camera", "temporal.velocity-camera"),
        (
            "taa-reactive-mask-clear",
            "temporal.taa-reactive-mask-clear",
        ),
        ("taa-reactive-mask-mesh", "temporal.taa-reactive-mask-mesh"),
        ("taa-resolve", "temporal.taa-resolve"),
        ("motion-vector-tile-max", "post.motion-vector-tile-max"),
        (
            "motion-vector-tile-max-coarse",
            "post.motion-vector-tile-max-coarse",
        ),
        (
            "motion-vector-neighbor-max",
            "post.motion-vector-neighbor-max",
        ),
        ("depth-of-field-prepare", "post.depth-of-field-prepare"),
        (
            "screen-space-reflection-reflection-pyramid",
            "post.screen-space-reflection-reflection-pyramid",
        ),
        (
            "screen-space-reflection-reflection-pyramid-coarse",
            "post.screen-space-reflection-reflection-pyramid-coarse",
        ),
        (
            "screen-space-reflection-resolve",
            "post.screen-space-reflection-resolve",
        ),
        (
            "screen-space-reflection-specular-occlusion",
            "post.screen-space-reflection-specular-occlusion",
        ),
    ] {
        execute_gpu_executor_without_specialized_context_for_extract(
            pass_name,
            executor_id,
            test_extract(),
        )
        .unwrap_or_else(|error| {
            panic!(
                "disabled optional post-process executor `{executor_id}` should skip before resource work; error={error}"
            )
        });
    }
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
    execute_gpu_executor_without_specialized_context_for_extract(
        pass_name,
        executor_id,
        test_extract(),
    )
    .unwrap_err()
}

fn execute_gpu_executor_without_specialized_context_with_effect_stack(
    pass_name: &str,
    executor_id: &str,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> String {
    let mut extract = test_extract();
    extract.post_process.effect_stack = effect_stack;
    execute_gpu_executor_without_specialized_context_for_extract(pass_name, executor_id, extract)
        .unwrap_err()
}

fn execute_gpu_executor_without_specialized_context_for_extract(
    pass_name: &str,
    executor_id: &str,
    extract: RenderFrameExtract,
) -> Result<(), String> {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(16, 16));
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context =
        RenderPassExecutionContext::new(pass_name, RenderPassExecutorId::new(executor_id))
            .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors().execute(&mut context)
}

fn effect_stack_with_motion_vectors() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        motion_blur: RenderMotionBlurSettings {
            shutter_angle: 90.0,
            samples: 8,
        },
        ..Default::default()
    }
}

fn effect_stack_with_depth_of_field() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        depth_of_field: RenderDepthOfFieldSettings {
            aperture: 0.75,
            max_blur_radius: 3.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn effect_stack_with_screen_space_reflection() -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 0.5,
            max_steps: 24,
            ..Default::default()
        },
        ..Default::default()
    }
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
fn shadow_atlas_executor_requires_graph_shadow_atlas_resource_instead_of_nooping() {
    let error = execute_gpu_executor_without_specialized_context("shadow-atlas", "shadow.atlas");

    assert_eq!(
        error,
        "render graph execution texture resource `shadow-atlas` is not bound"
    );
}

#[test]
fn shadow_atlas_executor_records_depth_only_pass_when_graph_resource_is_bound() {
    let backend = RenderBackend::new_offscreen().unwrap();
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("shadow-atlas-executor-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow-atlas-executor-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow-atlas-executor-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let mut resources = RenderGraphExecutionResources::new();
    import_shadow_atlas_texture(&mut resources, &backend.device);
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
        &scene_bind_group,
        &mut resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context = RenderPassExecutionContext::with_graph_metadata_and_resources(
        "shadow-atlas",
        RenderPassExecutorId::new("shadow.atlas"),
        QueueLane::Graphics,
        PassFlags::default(),
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::SHADOW_ATLAS.to_string(),
            kind: RenderGraphResourceKind::External,
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
        PostProcessGraphResourceNames::SCENE_DEPTH,
        PostProcessGraphResourceNames::SHADOW_ATLAS,
    ] {
        import_test_texture(&mut resources, &backend.device, resource);
    }
    for resource in [
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
    ] {
        import_test_buffer(&mut resources, &backend.device, resource, 256);
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
        &scene_bind_group_layout,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Depth32Float,
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
    let custom_pass =
        graph.add_pass_with_executor("custom-pass", QueueLane::Graphics, Some("custom.executor"));
    let output = graph.import_external_resource("custom-output");
    graph.write_external(custom_pass, output).unwrap();
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
    let custom_pass = graph.add_pass("custom-pass", QueueLane::Graphics);
    let output = graph.import_external_resource("custom-output");
    graph.write_external(custom_pass, output).unwrap();
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
    let root = graph.add_pass_with_executor(
        "root-pass",
        QueueLane::Graphics,
        Some("lighting.baked-composite"),
    );
    let output = graph.import_external_resource("custom-output");
    graph.write_external(root, output).unwrap();
    let unused = graph.create_texture(TextureDesc::new(
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

fn import_shadow_atlas_texture(
    resources: &mut RenderGraphExecutionResources,
    device: &wgpu::Device,
) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow-atlas"),
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
        PostProcessGraphResourceNames::SHADOW_ATLAS,
        texture.create_view(&wgpu::TextureViewDescriptor::default()),
    );
}
