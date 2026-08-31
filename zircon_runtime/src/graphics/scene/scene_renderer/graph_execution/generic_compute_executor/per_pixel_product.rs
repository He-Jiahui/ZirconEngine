use std::path::PathBuf;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderPluginRendererOutputs, RenderWorldSnapshotHandle,
};
use crate::core::math::UVec2;
use crate::graphics::ViewportRenderFrame;
use crate::graphics::backend::{RenderBackend, read_texture_rgba};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassExecutorRegistry, RenderPassGpuExecutionContext,
};
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBuilder,
    RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind, RenderGraphResourceAccessRange,
    RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};
use crate::scene::world::World;

use super::COMPUTE_GENERIC_EXECUTOR_ID;

const PRODUCT_PNG_NAME: &str = "plan16_compute_generic_per_pixel_wgpu_20260808.png";

struct PerPixelProductCapture {
    output_size: UVec2,
    dispatch_groups: [u32; 3],
    rgba: Vec<u8>,
}

#[test]
fn generic_executor_writes_per_pixel_storage_texture() {
    let Some(capture) = render_per_pixel_storage_texture() else {
        return;
    };

    assert_eq!(capture.dispatch_groups, [2, 2, 1]);
    assert_eq!(&capture.rgba[..4], &[0, 0, 64, 255]);
    assert_eq!(
        &capture.rgba[capture.rgba.len() - 4..],
        &[255, 255, 64, 255]
    );
}

#[test]
#[ignore = "writes Plan 16 generic compute WGPU framebuffer evidence under docs/tests/runtime/render"]
fn export_generic_executor_per_pixel_product_png() {
    let Some(capture) = render_per_pixel_storage_texture() else {
        return;
    };

    assert_eq!(capture.dispatch_groups, [2, 2, 1]);
    assert_eq!(&capture.rgba[..4], &[0, 0, 64, 255]);
    assert_eq!(
        &capture.rgba[capture.rgba.len() - 4..],
        &[255, 255, 64, 255]
    );
    let output = product_png_path();
    assert!(
        output
            .components()
            .all(|component| component.as_os_str() != "target")
    );
    std::fs::create_dir_all(
        output
            .parent()
            .expect("generic compute proof output parent"),
    )
    .expect("create generic compute proof output directory");
    image::save_buffer(
        &output,
        &capture.rgba,
        capture.output_size.x,
        capture.output_size.y,
        image::ColorType::Rgba8,
    )
    .expect("save generic compute product framebuffer");
    assert!(output.is_file());
    eprintln!("Plan 16 generic compute framebuffer={}", output.display());
}

fn render_per_pixel_storage_texture() -> Option<PerPixelProductCapture> {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return None;
    };
    let output_size = UVec2::new(13, 9);
    let mut graph_builder = RenderGraphBuilder::new("generic-compute-per-pixel-dispatch");
    let output = graph_builder.import_present_external_texture_with_binding(
        "per-pixel-output",
        TextureDesc::new(
            "generic-compute-per-pixel-output",
            output_size.x,
            output_size.y,
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        ),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass_id = graph_builder.add_pass_with_executor(
        "generic-per-pixel",
        QueueLane::AsyncCompute,
        Some(COMPUTE_GENERIC_EXECUTOR_ID),
    );
    graph_builder
        .access_external(
            pass_id,
            output,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full()),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    graph_builder
        .set_pass_flags(
            pass_id,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph_builder
        .set_compute_workload(
            pass_id,
            RenderGraphComputeWorkload::per_pixel(
                "generic-per-pixel",
                [8, 8, 1],
                "per-pixel-output",
                [8, 8],
            ),
        )
        .unwrap();
    graph_builder
        .set_compute_pass_metadata(
            pass_id,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl(
                    "generic-per-pixel",
                    "@group(1) @binding(0) var output: texture_storage_2d<rgba8unorm, write>;\n@compute @workgroup_size(8, 8, 1) fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) { let dimensions = textureDimensions(output); if (invocation_id.x >= dimensions.x || invocation_id.y >= dimensions.y) { return; } let coordinate = vec2<i32>(invocation_id.xy); textureStore(output, coordinate, vec4<f32>(f32(invocation_id.x) / 12.0, f32(invocation_id.y) / 8.0, 0.25, 1.0)); }",
                ),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "per-pixel-output",
                    ComputeBindingKind::StorageTextureWrite,
                )],
            ),
        )
        .unwrap();
    let graph = graph_builder.compile().unwrap();
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == "generic-per-pixel")
        .unwrap();

    let output_texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("generic-compute-per-pixel-output"),
        size: wgpu::Extent3d {
            width: output_size.x,
            height: output_size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut resources = RenderGraphExecutionResources::new();
    resources.import_borrowed_texture(
        "per-pixel-output",
        &output_texture,
        &output_view,
        TextureDesc::new(
            "generic-compute-per-pixel-output",
            output_size.x,
            output_size.y,
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        ),
    );
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("generic-compute-per-pixel-test"),
        });
    let scene_bind_group_layout =
        backend
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("generic-compute-empty-scene-layout"),
                entries: &[],
            });
    let scene_bind_group = backend
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("generic-compute-empty-scene-bind-group"),
            layout: &scene_bind_group_layout,
            entries: &[],
        });
    let frame = ViewportRenderFrame::from_extract(test_extract(), output_size);
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
        &resources,
        &mut plugin_outputs,
        &mut screen_space_ui_renderer,
    );
    let mut context =
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
        .with_compute_workload(pass.compute_workload.as_ref())
        .with_compute_pass_metadata(pass.compute_pass_metadata.as_ref())
        .with_compute_binding_access_packet(graph.compute_binding_access_packet(pass.id))
        .with_compute_dispatch_access_packet(graph.compute_dispatch_access_packet(pass.id))
        .with_gpu(gpu);

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .execute(&mut context)
        .unwrap();
    let dispatches = context.gpu_mut().unwrap().take_compute_dispatches();
    assert_eq!(dispatches.len(), 1);
    let dispatch_groups = dispatches[0].dispatch_groups;
    drop(context);
    backend.queue.submit([encoder.finish()]);

    let rgba = read_texture_rgba(
        &backend.device,
        &backend.queue,
        &output_texture,
        output_size,
    )
    .expect("per-pixel compute texture should be readable after submission");
    Some(PerPixelProductCapture {
        output_size,
        dispatch_groups,
        rgba,
    })
}

fn product_png_path() -> PathBuf {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .map(|parent| parent.to_path_buf())
        .unwrap_or(crate_root);
    workspace_root
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join(PRODUCT_PNG_NAME)
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
