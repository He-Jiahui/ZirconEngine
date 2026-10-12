use crate::core::framework::render::{
    RenderPipelineHandle, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::pipeline::{
    CompiledRenderPipelineParts, RenderGraphExecutionPassMetadata, RenderPassStage,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, TransientResourcePool,
};
use crate::graphics::CompiledRenderPipeline;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{
    BufferDesc, BufferUsage, SubmissionStatus, SubmissionTicket, TextureDesc, TextureFormat,
    TextureUsage,
};
use std::time::{Duration, Instant};

/// This module is included beside the production lowering helper in the
/// compiled-scene submission tests. It exercises the real lowering and
/// submission owners rather than constructing a receipt by hand.
#[test]
fn compiled_scene_submission_lowers_real_bindings_into_ticketed_gpu_receipt() {
    let backend = RenderBackend::new_offscreen().unwrap_or_else(|error| {
        panic!("managed Windows GPU gate unavailable: {error}");
    });

    let mut builder = RenderGraphBuilder::new("compiled-scene-gpu-receipt");
    let buffer = builder.create_buffer(BufferDesc::new(
        "scene-storage",
        128,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let texture_desc = TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    );
    let scene_color = builder.create_texture(texture_desc.clone());
    let owned_texture_desc = TextureDesc::new(
        "scene-transient",
        8,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    );
    let scene_transient = builder.create_texture(owned_texture_desc.clone());
    let pass = builder.add_pass("scene-pass", QueueLane::Graphics);
    builder
        .access_buffer(
            pass,
            buffer,
            RenderGraphResourceAccessKind::Write,
            crate::render_graph::RenderGraphBufferRange::new(0, Some(64)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .expect("scene buffer access must compile");
    builder
        .access_texture(
            pass,
            scene_color,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::ColorAttachment,
            None,
        )
        .expect("scene texture access must compile");
    builder
        .access_texture(
            pass,
            scene_transient,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::ColorAttachment,
            None,
        )
        .expect("owned transient texture access must compile");
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("the scene pass must remain live");
    let graph = builder.compile().expect("compiled scene graph");
    let pipeline = CompiledRenderPipeline::from_parts(CompiledRenderPipelineParts {
        handle: RenderPipelineHandle::new(901),
        name: "compiled-scene-gpu-receipt".to_owned(),
        renderer_name: "compiled-scene-gpu-receipt".to_owned(),
        execution_pass_metadata: vec![RenderGraphExecutionPassMetadata::new(
            pass,
            RenderPassStage::Opaque3d,
        )],
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        environment_ibl_bake_request: None,
        ambient_occlusion_profile: None,
        half_resolution_transparency_depth_sigma: DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
        graph,
    })
    .expect("compiled scene pipeline");

    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    let mut resources = RenderGraphExecutionResources::new();
    let imported_usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let imported_texture = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("compiled-scene-imported-view"),
        size: wgpu::Extent3d {
            width: texture_desc.width,
            height: texture_desc.height,
            depth_or_array_layers: texture_desc.depth_or_array_layers(),
        },
        mip_level_count: texture_desc.mip_levels,
        sample_count: texture_desc.sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: imported_usage,
        view_formats: &[],
    });
    let imported_view = imported_texture.create_view(&wgpu::TextureViewDescriptor::default());
    resources.import_borrowed_texture_view_with_physical_desc(
        "scene-color",
        &imported_view,
        texture_desc.clone(),
    );
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            pipeline.graph(),
            &mut pool,
        )
        .expect("compiled buffer and texture leases must materialize on the backend device");
    assert_eq!(
        texture_desc.usage,
        TextureUsage::RENDER_ATTACHMENT,
        "the imported logical scene-color usage must match its color-attachment graph access"
    );
    assert_eq!(
        imported_usage,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
        "the imported native view usage must match its color-attachment graph access"
    );
    assert_eq!(
        owned_texture_desc.usage,
        TextureUsage::RENDER_ATTACHMENT,
        "the transient logical usage must match its color-attachment graph access"
    );
    assert!(
        resources.physical_texture_desc("scene-color").is_some(),
        "the borrowed scene-color view must retain its physical descriptor"
    );
    assert!(
        resources.owned_texture("scene-transient").is_some(),
        "the normal materializer must allocate the pool-owned transient texture"
    );
    assert_eq!(
        resources
            .owned_texture_desc("scene-transient")
            .map(|desc| desc.usage),
        Some(TextureUsage::RENDER_ATTACHMENT),
        "the owned physical texture usage must match the graph declaration"
    );

    let receipt = super::super::build_graph_execution_receipt(&backend, &pipeline, &resources, 11)
        .expect("normal compiled-scene lowering must produce a device-qualified receipt");
    assert_eq!(receipt.passes().len(), 1);
    assert_eq!(
        receipt.physical_lease_count(),
        3,
        "the receipt must retain the buffer, borrowed view, and pool-owned texture leases"
    );

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compiled-scene-gpu-receipt"),
        });
    encoder.insert_debug_marker("compiled-scene-gpu-receipt");
    let ticket: SubmissionTicket = backend
        .submit_graphics_command_buffers_with_graph_receipt_and_frame_diagnostics_and_surface(
            vec![encoder.finish()],
            receipt,
            None,
            None,
            None,
        )
        .expect("the production submission service must admit the receipt with the command buffer");
    assert!(ticket.sequence() > 0);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = backend
            .submission_status(ticket)
            .expect("submission status must remain queryable");
        if status == SubmissionStatus::Completed {
            break;
        }
        assert!(
            !status.is_terminal(),
            "receipt ticket ended abnormally before completion: {status:?}"
        );
        assert!(
            Instant::now() < deadline,
            "receipt ticket did not complete within the bounded GPU gate"
        );
        backend
            .poll_submission_completions()
            .expect("submission completion poll must succeed");
        std::thread::yield_now();
    }
    resources.retire_transient_backings_after_submission(&mut pool, ticket);
    pool.collect_completed_submissions(|completed| backend.submission_status(completed));
    pool.end_frame();
    let report = pool.last_frame_report();
    assert_eq!(report.pending_retire_buffer_count, 0);
    assert_eq!(report.pending_retire_texture_count, 0);
    assert!(report.completion_reclaimed_buffer_count >= 1);
    assert!(report.completion_reclaimed_texture_count >= 1);
}
