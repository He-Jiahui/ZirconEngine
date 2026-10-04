use std::num::NonZeroU64;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderPluginRendererOutputs, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::ViewportRenderFrame;
use crate::render_graph::{
    QueueLane, RenderGraphBufferRange, RenderGraphBuilder, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphShaderStages,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};
use crate::scene::world::World;

use super::RenderPassGpuExecutionContext;

#[test]
fn public_gpu_resource_lookup_requires_compiled_pass_declaration_access() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let mut builder = RenderGraphBuilder::new("gpu-buffer-public-lookup");
    let scene_depth = builder.create_texture(
        TextureDesc::new(
            "scene-depth",
            16,
            16,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_sample_count(4),
    );
    let hybrid_gi_scene = builder.create_buffer(BufferDesc::new(
        "hybrid-gi-scene",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    ));
    let hzb = builder.create_texture(
        TextureDesc::new(
            "hzb-furthest",
            8,
            8,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(4),
    );
    let depth_prepass = builder.add_pass("depth-prepass", QueueLane::Graphics);
    builder.write_texture(depth_prepass, scene_depth).unwrap();
    let hzb_build = builder.add_pass("hzb-build", QueueLane::AsyncCompute);
    builder.read_texture(hzb_build, scene_depth).unwrap();
    builder.write_texture(hzb_build, hzb).unwrap();
    let pass = builder.add_pass("hybrid-gi-scene-prepare", QueueLane::Graphics);
    builder.read_texture(pass, scene_depth).unwrap();
    builder.read_texture(pass, hzb).unwrap();
    builder
        .access_buffer(
            pass,
            hybrid_gi_scene,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(64, Some(128)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .unwrap();
    let output = builder.import_present_external_resource("viewport-output");
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.read_buffer(present, hybrid_gi_scene).unwrap();
    builder.write_external(present, output).unwrap();
    let graph = builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "hybrid-gi-scene-prepare")
        .unwrap();
    let mut resources = RenderGraphExecutionResources::new();
    let mut transient_pool = TransientResourcePool::default();
    transient_pool.begin_frame(backend.device_profile());
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            &graph,
            &mut transient_pool,
        )
        .unwrap();
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("gpu-buffer-public-lookup-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gpu-buffer-public-lookup-empty-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu-buffer-public-lookup-empty-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), UVec2::new(16, 16));
    let mut screen_space_ui_renderer = ScreenSpaceUiRenderer::new_for_test(
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
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap_or_default()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id)
        .with_gpu(gpu);

    assert!(context
        .resource_resolver()
        .is_some_and(|resolver| resolver.has_physical_resources()));
    assert!(context
        .gpu()
        .and_then(RenderPassGpuExecutionContext::resource_resolver)
        .is_some_and(|resolver| resolver.has_physical_resources()));
    context
        .gpu()
        .unwrap()
        .require_texture_desc("scene-depth", RenderGraphResourceAccessKind::Read)
        .map(|desc| assert_eq!(desc.sample_count, 4))
        .expect("declared read texture descriptor should expose MSAA sample count");
    let binding = context
        .gpu()
        .unwrap()
        .require_buffer_binding("hybrid-gi-scene", RenderGraphResourceAccessKind::Write)
        .expect("declared write buffer should resolve through the exact GPU binding facade");
    assert_eq!(binding.offset, 64);
    assert_eq!(binding.size.map(NonZeroU64::get), Some(128));
    context
        .gpu()
        .unwrap()
        .require_owned_texture_full_mip_view("hzb-furthest", RenderGraphResourceAccessKind::Read)
        .expect("declared transient HZB read should expose its full mip chain");
    let error = context
        .gpu()
        .unwrap()
        .require_buffer_binding("hybrid-gi-scene", RenderGraphResourceAccessKind::Read)
        .unwrap_err();

    assert!(
        error.contains("did not declare Read access for resource `hybrid-gi-scene`"),
        "{error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
