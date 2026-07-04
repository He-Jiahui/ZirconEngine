use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_batch::GlyphAtlasDrawBatchKey;
use crate::graphics::text::atlas::render_contract::{
    GlyphAtlasBlendMode, GlyphAtlasRenderContract,
};
use crate::graphics::text::atlas::render_gpu_plan::{
    glyph_atlas_gpu_bind_group_layout, glyph_atlas_gpu_draw_command,
    glyph_atlas_gpu_pipeline_contract, glyph_atlas_gpu_vertex_buffer_layout, GlyphAtlasGpuBatch,
    GlyphAtlasGpuDrawPlan, GlyphAtlasGpuPipelineKey, GlyphAtlasGpuPrimitiveTopology,
    GlyphAtlasGpuVertex,
};
use crate::graphics::text::atlas::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasSamplingSemantics, GlyphAtlasStorageFormat,
};

use super::pipeline::{glyph_atlas_wgpu_blend_state, glyph_atlas_wgpu_primitive_state};
use super::renderer::{
    glyph_atlas_bitmap_renderer_prepare_report,
    glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes,
    GlyphAtlasBitmapRendererPrepareReport,
};
use super::resources::glyph_atlas_wgpu_bind_group_layout_entries;
use super::vertex::glyph_atlas_wgpu_vertex_buffer_layout;

#[test]
fn glyph_atlas_bitmap_vertex_layout_matches_gpu_plan_contract() {
    let contract_layout = glyph_atlas_gpu_vertex_buffer_layout();
    let layout = glyph_atlas_wgpu_vertex_buffer_layout(contract_layout);

    assert_eq!(layout.array_stride, contract_layout.stride_bytes);
    assert_eq!(layout.step_mode, wgpu::VertexStepMode::Vertex);
    assert_eq!(layout.attributes.len(), contract_layout.attributes.len());
    for (actual, expected) in layout
        .attributes
        .iter()
        .zip(contract_layout.attributes.iter())
    {
        assert_eq!(actual.offset, expected.offset_bytes);
        assert_eq!(actual.shader_location, expected.shader_location);
    }
    assert_eq!(layout.attributes[0].format, wgpu::VertexFormat::Float32x2);
    assert_eq!(layout.attributes[2].format, wgpu::VertexFormat::Float32x4);
    assert_eq!(layout.attributes[4].format, wgpu::VertexFormat::Uint32);
}

#[test]
fn glyph_atlas_bitmap_vertex_bytes_are_castable_for_gpu_upload() {
    let vertices = [
        GlyphAtlasGpuVertex {
            position_ndc: [-1.0, 1.0],
            uv: [0.0, 0.0],
            foreground_color: [1.0, 1.0, 1.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            page_index: 0,
        },
        GlyphAtlasGpuVertex {
            position_ndc: [1.0, -1.0],
            uv: [1.0, 1.0],
            foreground_color: [0.5, 0.5, 0.5, 1.0],
            background_color: [0.1, 0.2, 0.3, 1.0],
            page_index: 2,
        },
    ];

    let bytes: &[u8] = bytemuck::cast_slice(vertices.as_slice());

    assert_eq!(bytes.len(), std::mem::size_of_val(vertices.as_slice()));
    assert_eq!(std::mem::size_of::<GlyphAtlasGpuVertex>(), 52);
}

#[test]
fn glyph_atlas_bitmap_bind_group_entries_match_texture_array_contract() {
    let layout = glyph_atlas_gpu_bind_group_layout();
    let entries = glyph_atlas_wgpu_bind_group_layout_entries(layout);

    assert_eq!(entries.len(), 2);
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
    assert_eq!(
        subpixel.color.dst_factor,
        wgpu::BlendFactor::OneMinusSrcAlpha
    );
    assert_eq!(subpixel.alpha.src_factor, wgpu::BlendFactor::One);
}

#[test]
fn glyph_atlas_bitmap_pipeline_uses_triangle_list_commands() {
    let state = glyph_atlas_wgpu_primitive_state(GlyphAtlasGpuPrimitiveTopology::TriangleList);

    assert_eq!(state.topology, wgpu::PrimitiveTopology::TriangleList);
    assert_eq!(state.cull_mode, None);
    assert_eq!(state.polygon_mode, wgpu::PolygonMode::Fill);
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
        vertex_start: 0,
        vertex_count: 6,
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
        vertices: vec![GlyphAtlasGpuVertex {
            position_ndc: [0.0, 0.0],
            uv: [0.0, 0.0],
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
    assert_eq!(report.vertex_count, 1);
    assert_eq!(report.vertex_buffer_byte_len, 52);
    assert_eq!(report.draw_command_count, 1);
    assert_eq!(report.pipeline_count, 1);
    assert!(report.requires_background_composite);
    assert!(report.atlas_resized);
}

#[test]
fn glyph_atlas_bitmap_prepare_report_aggregates_mixed_storage_passes() {
    let alpha = GlyphAtlasBitmapRendererPrepareReport {
        atlas_size: UVec2::new(64, 64),
        atlas_layer_count: 1,
        atlas_storage_format: GlyphAtlasStorageFormat::R8Unorm,
        storage_pass_count: 1,
        storage_pass_visible_glyph_count: 1,
        mixed_atlas_storage_format: false,
        atlas_resized: false,
        vertex_count: 6,
        vertex_buffer_byte_len: 312,
        draw_command_count: 1,
        pipeline_count: 1,
        requires_background_composite: false,
        upload_request_count: 1,
        upload_byte_len: 64,
        upload_ready_to_write_texture: true,
        upload_failure_count: 0,
    };
    let color = GlyphAtlasBitmapRendererPrepareReport {
        atlas_storage_format: GlyphAtlasStorageFormat::Rgba8Unorm,
        storage_pass_visible_glyph_count: 2,
        vertex_count: 12,
        vertex_buffer_byte_len: 624,
        draw_command_count: 2,
        upload_request_count: 2,
        upload_byte_len: 256,
        atlas_resized: true,
        ..alpha.clone()
    };

    let report = glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(&[alpha, color], 3);

    assert_eq!(report.atlas_size, UVec2::new(64, 64));
    assert_eq!(report.atlas_layer_count, 2);
    assert_eq!(
        report.atlas_storage_format,
        GlyphAtlasStorageFormat::R8Unorm
    );
    assert_eq!(report.storage_pass_count, 2);
    assert_eq!(report.storage_pass_visible_glyph_count, 3);
    assert!(report.mixed_atlas_storage_format);
    assert!(report.atlas_resized);
    assert_eq!(report.vertex_count, 18);
    assert_eq!(report.vertex_buffer_byte_len, 936);
    assert_eq!(report.draw_command_count, 3);
    assert_eq!(report.pipeline_count, 3);
    assert_eq!(report.upload_request_count, 3);
    assert_eq!(report.upload_byte_len, 320);
    assert!(report.upload_ready_to_write_texture);
    assert_eq!(report.upload_failure_count, 0);
}
