use super::*;

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
        vec![RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::SCENE_DEPTH.to_string(),
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
        "depth prepass graph executor for pass `depth-prepass` requires mesh draw context"
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
