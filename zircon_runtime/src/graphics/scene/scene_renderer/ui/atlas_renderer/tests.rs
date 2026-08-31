use crate::core::math::UVec2;
use crate::text::atlas::render_batch::GlyphAtlasDrawBatchKey;
use crate::text::atlas::render_contract::{GlyphAtlasBlendMode, GlyphAtlasRenderContract};
use crate::text::atlas::render_gpu_plan::{
    GlyphAtlasGpuBatch, GlyphAtlasGpuDrawPlan, GlyphAtlasGpuInstance, GlyphAtlasGpuPipelineKey,
    GlyphAtlasGpuPrimitiveTopology, GlyphAtlasGpuViewportTransform,
    glyph_atlas_gpu_bind_group_layout, glyph_atlas_gpu_draw_command,
    glyph_atlas_gpu_instance_buffer_layout, glyph_atlas_gpu_pipeline_contract,
};
use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasBitmapFaceValidity,
    GlyphAtlasBitmapPageUploadStaging, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapStagedUpload, GlyphAtlasBitmapStagedUploadPlan,
    GlyphAtlasBitmapUploadStagingPlan, GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec,
    GlyphAtlasRect, GlyphAtlasSamplingSemantics, GlyphAtlasSet, GlyphAtlasStorageFormat,
    GlyphAtlasUploadMode, glyph_atlas_upload_command,
};

use super::super::atlas_texture_upload::GlyphAtlasBitmapTextureUploadFrameReport;
use super::GlyphAtlasBitmapRendererPrepareReport;
use super::instance::glyph_atlas_wgpu_instance_buffer_layout;
use super::instance_buffer::{
    glyph_atlas_bitmap_renderer_instance_buffer_capacity,
    glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation,
    glyph_atlas_bitmap_renderer_instance_buffer_write_required,
};
use super::pipeline::{glyph_atlas_wgpu_blend_state, glyph_atlas_wgpu_primitive_state};
use super::renderer::{
    glyph_atlas_bitmap_renderer_atlas_layer_capacity,
    glyph_atlas_bitmap_renderer_idle_prepare_report,
    glyph_atlas_bitmap_renderer_ordered_draw_segment_count,
    glyph_atlas_bitmap_renderer_prepare_report,
    glyph_atlas_bitmap_renderer_prepare_report_with_face_invalidation,
    glyph_atlas_bitmap_renderer_prepare_report_with_upload_report,
    glyph_atlas_bitmap_renderer_release_idle_draw_passes,
    glyph_atlas_bitmap_renderer_texture_upload_frame_plan,
    glyph_atlas_bitmap_renderer_viewport_uniform_write_required,
};
use super::resources::{
    glyph_atlas_bitmap_sampler_descriptor, glyph_atlas_wgpu_bind_group_layout_entries,
};
use super::state::GlyphAtlasBitmapRendererDrawPass;

#[test]
fn glyph_atlas_bitmap_instance_layout_matches_gpu_plan_contract() {
    let contract_layout = glyph_atlas_gpu_instance_buffer_layout();
    let layout = glyph_atlas_wgpu_instance_buffer_layout(contract_layout);

    assert_eq!(layout.array_stride, contract_layout.stride_bytes);
    assert_eq!(layout.step_mode, wgpu::VertexStepMode::Instance);
    assert_eq!(layout.attributes.len(), contract_layout.attributes.len());
    for (actual, expected) in layout
        .attributes
        .iter()
        .zip(contract_layout.attributes.iter())
    {
        assert_eq!(actual.offset, expected.offset_bytes);
        assert_eq!(actual.shader_location, expected.shader_location);
    }
    assert_eq!(layout.attributes[0].format, wgpu::VertexFormat::Float32x4);
    assert_eq!(layout.attributes[2].format, wgpu::VertexFormat::Float32x4);
    assert_eq!(layout.attributes[4].format, wgpu::VertexFormat::Uint32);
}

#[test]
fn glyph_atlas_bitmap_instance_bytes_are_castable_for_gpu_upload() {
    let instances = [
        GlyphAtlasGpuInstance {
            screen_rect_px: [0.0, 0.0, 12.0, 14.0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            foreground_color: [1.0, 1.0, 1.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            page_index: 0,
        },
        GlyphAtlasGpuInstance {
            screen_rect_px: [16.0, 4.0, 10.0, 18.0],
            uv_rect: [0.25, 0.25, 0.5, 0.5],
            foreground_color: [0.5, 0.5, 0.5, 1.0],
            background_color: [0.1, 0.2, 0.3, 1.0],
            page_index: 2,
        },
    ];

    let bytes: &[u8] = bytemuck::cast_slice(instances.as_slice());

    assert_eq!(bytes.len(), std::mem::size_of_val(instances.as_slice()));
    assert_eq!(std::mem::size_of::<GlyphAtlasGpuInstance>(), 68);
}

#[test]
fn glyph_atlas_bitmap_instance_buffer_writes_only_for_new_payloads_or_reallocation() {
    let payload = [7; 32];
    assert!(!glyph_atlas_bitmap_renderer_instance_buffer_write_required(
        false,
        Some(payload),
        payload
    ));
    assert!(glyph_atlas_bitmap_renderer_instance_buffer_write_required(
        false,
        Some(payload),
        [8; 32]
    ));
    assert!(glyph_atlas_bitmap_renderer_instance_buffer_write_required(
        true,
        Some(payload),
        payload
    ));
    assert!(glyph_atlas_bitmap_renderer_instance_buffer_write_required(
        false, None, payload
    ));
}

#[test]
fn glyph_atlas_bitmap_viewport_uniform_writes_only_when_transform_changes() {
    let viewport = GlyphAtlasGpuViewportTransform::new(UVec2::new(1280, 720));
    assert!(glyph_atlas_bitmap_renderer_viewport_uniform_write_required(
        None, viewport
    ));
    assert!(!glyph_atlas_bitmap_renderer_viewport_uniform_write_required(Some(viewport), viewport));
    assert!(glyph_atlas_bitmap_renderer_viewport_uniform_write_required(
        Some(viewport),
        GlyphAtlasGpuViewportTransform::new(UVec2::new(1920, 1080)),
    ));
}

#[test]
fn glyph_atlas_bitmap_bind_group_entries_match_texture_array_contract() {
    let layout = glyph_atlas_gpu_bind_group_layout();
    let entries = glyph_atlas_wgpu_bind_group_layout_entries(layout);

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].binding, layout.atlas_texture.binding);
    assert_eq!(entries[0].visibility, wgpu::ShaderStages::FRAGMENT);
    match &entries[0].ty {
        wgpu::BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        } => {
            assert_eq!(
                *sample_type,
                wgpu::TextureSampleType::Float { filterable: true }
            );
            assert_eq!(*view_dimension, wgpu::TextureViewDimension::D2Array);
            assert!(!*multisampled);
        }
        _ => panic!("atlas texture binding must be a sampled texture array"),
    }
    assert_eq!(entries[1].binding, layout.atlas_sampler.binding);
    assert_eq!(
        entries[1].ty,
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    );
    assert_eq!(entries[2].binding, layout.viewport_uniform.binding);
    assert_eq!(entries[2].visibility, wgpu::ShaderStages::VERTEX);
    assert!(matches!(
        &entries[2].ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            ..
        }
    ));
}

#[test]
fn glyph_atlas_bitmap_sampler_uses_padded_bilinear_device_pixel_sampling() {
    let descriptor = glyph_atlas_bitmap_sampler_descriptor();

    assert_eq!(descriptor.mag_filter, wgpu::FilterMode::Linear);
    assert_eq!(descriptor.min_filter, wgpu::FilterMode::Linear);
    assert_eq!(descriptor.mipmap_filter, wgpu::MipmapFilterMode::Nearest);
    assert_eq!(descriptor.lod_min_clamp, 0.0);
    assert_eq!(descriptor.lod_max_clamp, 0.0);
}

#[test]
fn glyph_atlas_bitmap_pipeline_state_tracks_render_contract_blend_modes() {
    assert_eq!(
        glyph_atlas_wgpu_blend_state(GlyphAtlasBlendMode::StandardAlpha),
        wgpu::BlendState::ALPHA_BLENDING
    );
    assert_eq!(
        glyph_atlas_wgpu_blend_state(GlyphAtlasBlendMode::SourceRgba),
        wgpu::BlendState::ALPHA_BLENDING
    );

    let subpixel = glyph_atlas_wgpu_blend_state(GlyphAtlasBlendMode::SubpixelBackgroundComposite);

    assert_eq!(subpixel.color.src_factor, wgpu::BlendFactor::One);
    assert_eq!(subpixel.color.dst_factor, wgpu::BlendFactor::Zero);
    assert_eq!(subpixel.alpha.src_factor, wgpu::BlendFactor::One);
    assert_eq!(subpixel.alpha.dst_factor, wgpu::BlendFactor::Zero);
}

#[test]
fn glyph_atlas_bitmap_pipeline_uses_triangle_list_commands() {
    let state = glyph_atlas_wgpu_primitive_state(GlyphAtlasGpuPrimitiveTopology::TriangleList);

    assert_eq!(state.topology, wgpu::PrimitiveTopology::TriangleList);
    assert_eq!(state.cull_mode, None);
    assert_eq!(state.polygon_mode, wgpu::PolygonMode::Fill);
}

#[test]
fn glyph_atlas_bitmap_shader_expands_instance_rects_with_a_viewport_uniform() {
    let shader = crate::text::atlas::render_contract::GLYPH_ATLAS_TEXT_SHADER;

    assert!(shader.contains("@builtin(vertex_index)"));
    assert!(shader.contains("glyph_atlas_viewport"));
    assert!(shader.contains("screen_rect_px"));
    assert!(shader.contains("uv_rect"));
    assert!(!shader.contains("position_ndc"));
}

#[test]
fn glyph_atlas_bitmap_prepare_report_counts_real_draw_resources() {
    let contract = GlyphAtlasRenderContract::for_sampling_semantics(
        GlyphAtlasSamplingSemantics::AlphaCoverage,
    );
    let key = GlyphAtlasDrawBatchKey {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
        render_contract: contract,
    };
    let batch = GlyphAtlasGpuBatch {
        key,
        instance_start: 0,
        instance_count: 1,
    };
    let command = glyph_atlas_gpu_draw_command(batch);
    let pipeline_contract = glyph_atlas_gpu_pipeline_contract(GlyphAtlasGpuPipelineKey {
        render_contract: contract,
        primitive_topology: GlyphAtlasGpuPrimitiveTopology::TriangleList,
    });
    let mut plan = GlyphAtlasGpuDrawPlan {
        pipeline_contracts: vec![pipeline_contract],
        draw_commands: vec![command],
        visible_glyph_count: 1,
        instances: vec![GlyphAtlasGpuInstance {
            screen_rect_px: [0.0, 0.0, 10.0, 12.0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            foreground_color: [1.0, 1.0, 1.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            page_index: 0,
        }],
        ..GlyphAtlasGpuDrawPlan::default()
    };
    plan.requires_background_composite = true;

    let report = glyph_atlas_bitmap_renderer_prepare_report(
        &plan,
        1,
        UVec2::new(256, 128),
        0,
        GlyphAtlasStorageFormat::R8Unorm,
        true,
    );

    assert_eq!(report.atlas_size, UVec2::new(256, 128));
    assert_eq!(report.atlas_layer_count, 1);
    assert_eq!(report.storage_pass_count, 1);
    assert_eq!(report.storage_pass_visible_glyph_count, 1);
    assert!(!report.mixed_atlas_storage_format);
    assert_eq!(report.vertex_count, 6);
    assert_eq!(report.vertex_buffer_byte_len, 68);
    assert_eq!(report.draw_command_count, 1);
    assert_eq!(report.pipeline_count, 1);
    assert!(report.requires_background_composite);
    assert!(report.atlas_resized);
}

#[test]
fn glyph_atlas_bitmap_ordered_draw_segments_preserve_interleaved_resource_order() {
    let alpha_key = GlyphAtlasDrawBatchKey {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
        render_contract: GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::AlphaCoverage,
        ),
    };
    let color_key = GlyphAtlasDrawBatchKey {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
        render_contract: GlyphAtlasRenderContract::for_sampling_semantics(
            GlyphAtlasSamplingSemantics::ColorRgba,
        ),
    };
    let command = |key, instance_start| {
        glyph_atlas_gpu_draw_command(GlyphAtlasGpuBatch {
            key,
            instance_start,
            instance_count: 1,
        })
    };
    let plan = GlyphAtlasGpuDrawPlan {
        draw_commands: vec![
            command(alpha_key, 0),
            command(color_key, 1),
            command(alpha_key, 2),
        ],
        ..GlyphAtlasGpuDrawPlan::default()
    };

    assert_eq!(
        glyph_atlas_bitmap_renderer_ordered_draw_segment_count(&plan),
        3
    );
}

#[test]
fn glyph_atlas_bitmap_idle_prepare_report_keeps_resources_without_frame_draw_work() {
    let report = glyph_atlas_bitmap_renderer_idle_prepare_report(
        [
            (UVec2::new(512, 512), 2, GlyphAtlasFormat::AlphaMask),
            (UVec2::new(512, 512), 3, GlyphAtlasFormat::Color),
        ],
        4,
    );

    assert_eq!(report.atlas_size, UVec2::new(512, 512));
    assert_eq!(report.atlas_layer_count, 3);
    assert_eq!(
        report.atlas_storage_format,
        GlyphAtlasStorageFormat::R8Unorm
    );
    assert_eq!(report.storage_pass_count, 0);
    assert_eq!(report.storage_resource_count, 2);
    assert_eq!(report.ordered_draw_segment_count, 0);
    assert!(report.mixed_atlas_storage_format);
    assert_eq!(report.pipeline_count, 4);
    assert_eq!(report.vertex_count, 0);
    assert_eq!(report.draw_command_count, 0);
    assert_eq!(report.upload_request_count, 0);
    assert_eq!(report.upload_byte_len, 0);
    assert!(!report.atlas_resized);
}

#[test]
fn glyph_atlas_bitmap_renderer_reserves_the_configured_page_capacity_per_format() {
    let configured_capacity =
        u32::try_from(GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT).expect("page cap fits u32");

    assert_eq!(
        glyph_atlas_bitmap_renderer_atlas_layer_capacity(1),
        configured_capacity
    );
    assert_eq!(
        glyph_atlas_bitmap_renderer_atlas_layer_capacity(configured_capacity),
        configured_capacity
    );
    assert_eq!(
        glyph_atlas_bitmap_renderer_atlas_layer_capacity(configured_capacity.saturating_add(1)),
        configured_capacity.saturating_add(1)
    );
}

#[test]
fn glyph_atlas_bitmap_instance_buffer_grows_only_when_its_capacity_is_exhausted() {
    assert_eq!(glyph_atlas_bitmap_renderer_instance_buffer_capacity(0), 0);
    assert_eq!(
        glyph_atlas_bitmap_renderer_instance_buffer_capacity(68),
        4096
    );
    assert_eq!(
        glyph_atlas_bitmap_renderer_instance_buffer_capacity(4096),
        4096
    );
    assert_eq!(
        glyph_atlas_bitmap_renderer_instance_buffer_capacity(4097),
        8192
    );

    assert!(glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(0, 68));
    assert!(!glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(4096, 68));
    assert!(!glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(4096, 4096));
    assert!(glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(4096, 4097));
}

#[test]
fn glyph_atlas_bitmap_instance_buffer_capacity_is_stable_at_scale() {
    let mut previous_capacity = 0;
    for glyph_count in [1_usize, 100, 1_000, 10_000] {
        let required_byte_len = glyph_count * std::mem::size_of::<GlyphAtlasGpuInstance>();
        let requires_growth = glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(
            previous_capacity,
            required_byte_len,
        );
        let capacity = if requires_growth {
            glyph_atlas_bitmap_renderer_instance_buffer_capacity(required_byte_len)
        } else {
            previous_capacity
        };

        assert!(capacity >= required_byte_len as u64);
        assert!(capacity.is_power_of_two());
        assert!(
            !glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(
                capacity,
                required_byte_len,
            )
        );
        previous_capacity = capacity;
    }
}

#[test]
fn glyph_atlas_bitmap_idle_releases_all_retained_instance_buffers() {
    let mut draw_passes = vec![
        GlyphAtlasBitmapRendererDrawPass::new(),
        GlyphAtlasBitmapRendererDrawPass::new(),
    ];
    draw_passes[0].instance_buffer_capacity_bytes = 4096;
    draw_passes[0].instance_buffer_payload_hash = Some([7; 32]);
    draw_passes[1].instance_buffer_capacity_bytes = 8192;

    glyph_atlas_bitmap_renderer_release_idle_draw_passes(&mut draw_passes);

    assert_eq!(draw_passes.len(), 1);
    assert!(draw_passes[0].instance_buffer.is_none());
    assert_eq!(draw_passes[0].instance_buffer_capacity_bytes, 0);
    assert_eq!(draw_passes[0].instance_buffer_payload_hash, None);
    assert!(draw_passes[0].draw_commands.is_empty());
}

#[test]
fn glyph_atlas_bitmap_prepare_report_projects_upload_requeue_frame_report() {
    let report = glyph_atlas_bitmap_renderer_prepare_report_with_upload_report(
        GlyphAtlasBitmapRendererPrepareReport::default(),
        GlyphAtlasBitmapTextureUploadFrameReport {
            request_count: 0,
            binding_count: 0,
            binding_failure_count: 1,
            staging_failure_count: 0,
            staged_upload_failure_count: 0,
            skipped_staged_upload_failure_count: 0,
            requeued_upload_count: 2,
            missing_page_requeue_count: 1,
            page_generation_mismatch_requeue_count: 0,
            stale_page_generation_count: 1,
            face_invalidated_count: 1,
            upload_byte_len: 0,
            ready_to_write_texture: false,
        },
    );

    assert_eq!(report.upload_request_count, 0);
    assert_eq!(report.upload_requeued_count, 2);
    assert_eq!(report.upload_missing_page_requeue_count, 1);
    assert_eq!(report.upload_page_generation_mismatch_requeue_count, 0);
    assert_eq!(report.upload_face_invalidated_count, 1);
    assert_eq!(report.upload_failure_count, 3);
    assert!(!report.upload_ready_to_write_texture);
}

#[test]
fn glyph_atlas_bitmap_renderer_upload_plan_requeues_invalidated_face_uploads() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let page = GlyphAtlasPageSpec::new(page_key, UVec2::new(4, 4));
    let command = glyph_atlas_upload_command(&page, GlyphAtlasUploadMode::FullPage, None, 16)
        .expect("test page upload command");
    let prepared_upload = GlyphAtlasBitmapPreparedUploadPlan {
        staging: GlyphAtlasBitmapUploadStagingPlan {
            pages: vec![GlyphAtlasBitmapPageUploadStaging {
                page_key,
                page_generation: page.generation,
                target_rect: GlyphAtlasRect {
                    x: 0,
                    y: 0,
                    width: 4,
                    height: 4,
                },
                bytes_per_row: 4,
                bytes: vec![255; 16].into(),
            }],
            failures: Vec::new(),
        },
        staged_uploads: GlyphAtlasBitmapStagedUploadPlan {
            uploads: vec![GlyphAtlasBitmapStagedUpload {
                staging_page_index: 0,
                command,
                staging_page_byte_len: 16,
            }],
            failures: Vec::new(),
        },
    };
    let atlas = GlyphAtlasSet::from_page(page);

    let report = glyph_atlas_bitmap_renderer_texture_upload_frame_plan(
        &prepared_upload,
        &atlas,
        GlyphAtlasBitmapFaceValidity::Invalidated,
    )
    .report();

    assert_eq!(report.request_count, 0);
    assert_eq!(report.requeued_upload_count, 1);
    assert_eq!(report.face_invalidated_count, 1);
    assert!(!report.ready_to_write_texture);
}

#[test]
fn glyph_atlas_bitmap_prepare_report_records_face_invalidated_storage_passes() {
    let report = glyph_atlas_bitmap_renderer_prepare_report_with_face_invalidation(
        GlyphAtlasBitmapRendererPrepareReport::default(),
        2,
    );

    assert_eq!(report.invalidated_storage_pass_count, 2);
    assert_eq!(report.storage_pass_visible_glyph_count, 0);
}
