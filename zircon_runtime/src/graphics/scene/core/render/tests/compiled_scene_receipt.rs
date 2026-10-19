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

fn optional_external_receipt_pipeline(
    required: bool,
) -> (CompiledRenderPipeline, TextureDesc, BufferDesc) {
    use crate::render_graph::{
        RenderGraphBufferRange, RenderGraphExternalResourceBinding, RenderGraphResourceAccessRange,
    };
    let mut builder = RenderGraphBuilder::new("typed-optional-external-receipt");
    let texture_desc = TextureDesc::new(
        "history.previous.hzb-furthest",
        16,
        16,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED,
    )
    .with_mip_levels(2);
    let buffer_desc = BufferDesc::new("optional-cull-history", 128, BufferUsage::STORAGE);
    let texture = builder.import_external_texture_with_binding(
        "history.previous.hzb-furthest",
        texture_desc.clone(),
        if required {
            RenderGraphExternalResourceBinding::required_texture()
        } else {
            RenderGraphExternalResourceBinding::report_only_texture()
        },
    );
    let buffer = builder.import_external_buffer_with_binding(
        "optional-cull-history",
        buffer_desc.clone(),
        if required {
            RenderGraphExternalResourceBinding::required_buffer()
        } else {
            RenderGraphExternalResourceBinding::report_only_buffer()
        },
    );
    let pass = builder.add_pass("history-reader", QueueLane::Graphics);
    builder
        .read_external_with_access(
            pass,
            texture,
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(1),
            ),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder
        .read_external_with_access(
            pass,
            buffer,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(32, Some(64))),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();
    let pipeline = CompiledRenderPipeline::from_parts(CompiledRenderPipelineParts {
        handle: RenderPipelineHandle::new(902),
        name: "typed-optional-external-receipt".to_owned(),
        renderer_name: "typed-optional-external-receipt".to_owned(),
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
        graph: builder.compile().unwrap(),
    })
    .unwrap();
    (pipeline, texture_desc, buffer_desc)
}

#[test]
fn compiled_scene_optional_external_receipt_preserves_absent_and_real_native_bindings() {
    use crate::rhi::{RhiGraphAccessRange, RhiGraphResourceBounds};
    let backend = RenderBackend::new_offscreen()
        .unwrap_or_else(|error| panic!("managed Windows GPU gate unavailable: {error}"));
    let (pipeline, texture_desc, buffer_desc) = optional_external_receipt_pipeline(false);
    for present in [false, true] {
        let mut pool = TransientResourcePool::default();
        pool.begin_frame(backend.device_profile());
        let mut resources = RenderGraphExecutionResources::new();
        if present {
            let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("real-optional-history"),
                size: wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 2,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            resources.import_borrowed_texture(
                "history.previous.hzb-furthest",
                &texture,
                &view,
                texture_desc.clone(),
            );
            let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("real-optional-history-buffer"),
                size: 128,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            resources.import_borrowed_buffer_with_physical_desc(
                "optional-cull-history",
                &buffer,
                buffer_desc.clone(),
            );
        }
        resources
            .materialize_transient_resources_with_pool(
                &backend.device,
                backend.device_profile(),
                pipeline.graph(),
                &mut pool,
            )
            .expect("normal frame device epoch and resources");
        resources
            .materialize_external_access_bindings(pipeline.graph())
            .expect("exact optional external contracts must materialize in both states");
        let receipt =
            super::super::build_graph_execution_receipt(&backend, &pipeline, &resources, 12)
                .expect("typed optional absence and present native owners both lower normally");
        let accesses = receipt.passes()[0].accesses();
        assert_eq!(accesses.len(), 2);
        assert_eq!(
            accesses[0].range(),
            RhiGraphAccessRange::texture(1, 2, 0, 1, 1)
        );
        assert_eq!(accesses[1].range(), RhiGraphAccessRange::buffer(32, 64));
        assert_eq!(receipt.physical_lease_count(), if present { 2 } else { 0 });
        assert_eq!(accesses[0].has_physical_binding(), present);
        assert_eq!(accesses[1].has_physical_binding(), present);
        assert_eq!(
            accesses[0].optional_external_absence_bounds(),
            if present {
                None
            } else {
                Some(RhiGraphResourceBounds::texture(2, 1, 1).unwrap())
            }
        );
        assert_eq!(
            accesses[1].optional_external_absence_bounds(),
            if present {
                None
            } else {
                Some(RhiGraphResourceBounds::buffer(128).unwrap())
            }
        );
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("typed-optional-external-receipt"),
            });
        encoder.insert_debug_marker("typed-optional-external-receipt");
        let ticket = backend
            .submit_graphics_command_buffers_with_graph_receipt_and_frame_diagnostics_and_surface(
                vec![encoder.finish()],
                receipt,
                None,
                None,
                None,
            )
            .expect("real backend must admit both receipt states");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let status = backend.submission_status(ticket).unwrap();
            if status == SubmissionStatus::Completed {
                break;
            }
            assert!(
                !status.is_terminal(),
                "optional receipt submission ended abnormally: {status:?}"
            );
            assert!(
                Instant::now() < deadline,
                "optional receipt GPU submission timed out"
            );
            backend.poll_submission_completions().unwrap();
            std::thread::yield_now();
        }
        pool.end_frame();
    }
}

#[test]
fn compiled_scene_optional_external_receipt_rejects_required_foreign_contract_and_invalid_range() {
    use crate::render_graph::{RenderGraphExternalResourceBinding, RenderGraphResourceAccessRange};
    let backend = RenderBackend::new_offscreen()
        .unwrap_or_else(|error| panic!("managed Windows GPU gate unavailable: {error}"));
    let (pipeline, _, _) = optional_external_receipt_pipeline(false);
    let mut pool = TransientResourcePool::default();
    pool.begin_frame(backend.device_profile());
    let mut resources = RenderGraphExecutionResources::new();
    resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            pipeline.graph(),
            &mut pool,
        )
        .unwrap();
    resources
        .materialize_external_access_bindings(pipeline.graph())
        .unwrap();
    let key = pipeline
        .execution_passes_in_graph_order()
        .next()
        .unwrap()
        .device_access_bindings[0]
        .key;
    let declaration = pipeline.graph().resource_declaration(key.resource).unwrap();
    let range = super::super::rhi_access_range(&resources, declaration, key.range).unwrap();
    let mut wrong_contract = declaration.clone();
    wrong_contract.external_binding = RenderGraphExternalResourceBinding::required_texture();
    assert!(
        super::super::optional_external_absence_bounds(&resources, &wrong_contract, key, range)
            .is_err(),
        "optional absence requires equality with the declaration contract"
    );
    let mut wrong_key = key;
    wrong_key.range =
        RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::single_mip(0));
    assert!(
        super::super::optional_external_absence_bounds(&resources, declaration, wrong_key, range)
            .is_err(),
        "the exact materialized key includes the typed range"
    );
    assert!(
        super::super::rhi_access_range(
            &resources,
            declaration,
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(2)
            )
        )
        .is_err(),
        "absence never bypasses the original complete mip-range validation"
    );
    let buffer_key = pipeline
        .execution_passes_in_graph_order()
        .next()
        .unwrap()
        .device_access_bindings[1]
        .key;
    let buffer_decl = pipeline
        .graph()
        .resource_declaration(buffer_key.resource)
        .unwrap();
    assert!(
        super::super::rhi_access_range(
            &resources,
            buffer_decl,
            RenderGraphResourceAccessRange::Buffer(
                crate::render_graph::RenderGraphBufferRange::new(96, Some(64))
            )
        )
        .is_err(),
        "absence never bypasses the original complete buffer-range validation"
    );
    let (foreign_pipeline, _, _) = optional_external_receipt_pipeline(false);
    resources
        .materialize_external_access_bindings(foreign_pipeline.graph())
        .unwrap();
    assert!(
        super::super::build_graph_execution_receipt(&backend, &pipeline, &resources, 12).is_err(),
        "another graph's optional contracts cannot authorize this graph's absent resources"
    );
    let (required_pipeline, _, _) = optional_external_receipt_pipeline(true);
    let mut required_resources = RenderGraphExecutionResources::new();
    required_resources
        .materialize_transient_resources_with_pool(
            &backend.device,
            backend.device_profile(),
            required_pipeline.graph(),
            &mut pool,
        )
        .unwrap();
    let error = required_resources
        .materialize_external_access_bindings(required_pipeline.graph())
        .expect_err("required external history cannot be absent");
    assert!(error.contains("required external"));
    assert!(super::super::build_graph_execution_receipt(
        &backend,
        &required_pipeline,
        &required_resources,
        12
    )
    .is_err());
    pool.end_frame();
}
