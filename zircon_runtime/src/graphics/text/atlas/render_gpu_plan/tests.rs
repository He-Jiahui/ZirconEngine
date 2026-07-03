use super::super::render_batch::glyph_atlas_draw_batch_plan;
use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect};
use super::*;

#[test]
fn render_text_atlas_gpu_viewport_transform_uses_pixel_edge_ndc_contract() {
    let transform = glyph_atlas_gpu_viewport_transform(UVec2::new(100, 50));

    assert_eq!(transform.viewport_size, UVec2::new(100, 50));
    assert_eq!(
        transform.pixel_coordinate_convention,
        GlyphAtlasGpuPixelCoordinateConvention::PixelEdges
    );
    assert_ndc_near(transform.position_ndc([0.0, 0.0]), [-1.0, 1.0]);
    assert_ndc_near(transform.position_ndc([50.0, 25.0]), [0.0, 0.0]);
    assert_ndc_near(transform.position_ndc([100.0, 50.0]), [1.0, -1.0]);
}

#[test]
fn render_text_atlas_gpu_viewport_transform_uses_unit_extent_for_empty_viewports() {
    let transform = glyph_atlas_gpu_viewport_transform(UVec2::new(0, 0));

    assert_eq!(transform.viewport_size, UVec2::new(0, 0));
    assert_ndc_near(transform.position_ndc([0.0, 0.0]), [-1.0, 1.0]);
    assert_ndc_near(transform.position_ndc([1.0, 1.0]), [1.0, -1.0]);
}

#[test]
fn render_text_atlas_gpu_vertex_layout_matches_cpu_vertex_shape() {
    let layout = glyph_atlas_gpu_vertex_buffer_layout();

    assert_eq!(
        layout.stride_bytes,
        std::mem::size_of::<GlyphAtlasGpuVertex>() as u64
    );
    assert_eq!(layout.stride_bytes, 52);
    assert_eq!(layout.attributes.len(), 5);
    assert_eq!(
        layout.attributes,
        [
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::PositionNdc,
                shader_location: 0,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x2,
                offset_bytes: 0,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::Uv,
                shader_location: 1,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x2,
                offset_bytes: 8,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::ForegroundColor,
                shader_location: 2,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x4,
                offset_bytes: 16,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::BackgroundColor,
                shader_location: 3,
                format: GlyphAtlasGpuVertexAttributeFormat::Float32x4,
                offset_bytes: 32,
            },
            GlyphAtlasGpuVertexAttribute {
                semantic: GlyphAtlasGpuVertexAttributeSemantic::PageIndex,
                shader_location: 4,
                format: GlyphAtlasGpuVertexAttributeFormat::Uint32,
                offset_bytes: 48,
            },
        ]
    );
}

#[test]
fn render_text_atlas_gpu_bind_group_layout_uses_texture_array_sampler_contract() {
    let layout = glyph_atlas_gpu_bind_group_layout();

    assert_eq!(layout.atlas_texture.group, 0);
    assert_eq!(layout.atlas_texture.binding, 0);
    assert_eq!(
        layout.atlas_texture.sample_type,
        GlyphAtlasGpuTextureSampleType::FloatFilterable
    );
    assert_eq!(
        layout.atlas_texture.view_dimension,
        GlyphAtlasGpuTextureViewDimension::D2Array
    );
    assert!(!layout.atlas_texture.multisampled);
    assert_eq!(layout.atlas_sampler.group, 0);
    assert_eq!(layout.atlas_sampler.binding, 1);
    assert_eq!(
        layout.atlas_sampler.binding_type,
        GlyphAtlasGpuSamplerBindingType::Filtering
    );
}

#[test]
fn render_text_atlas_gpu_draw_commands_follow_batch_quad_ranges() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(4.0, 4.0, 10.0, 8.0),
            ),
            glyph(
                GlyphAtlasFormat::Color,
                3,
                GlyphAtlasScreenRect::new(20.0, 4.0, 10.0, 8.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(36.0, 4.0, 10.0, 8.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(80, 32));

    assert_eq!(gpu_plan.batches.len(), 2);
    assert_eq!(gpu_plan.draw_commands.len(), 2);
    assert_eq!(gpu_plan.draw_commands[0].key, gpu_plan.batches[0].key);
    assert_eq!(gpu_plan.draw_commands[0].vertex_start, 0);
    assert_eq!(gpu_plan.draw_commands[0].vertex_count, 12);
    assert_eq!(gpu_plan.draw_commands[0].quad_count(), 2);
    assert_eq!(gpu_plan.draw_commands[0].triangle_count(), 4);
    assert_eq!(gpu_plan.draw_commands[0].atlas_layer, 0);
    assert_eq!(
        gpu_plan.draw_commands[0].primitive_topology,
        GlyphAtlasGpuPrimitiveTopology::TriangleList
    );
    assert_eq!(
        gpu_plan.draw_commands[0].pipeline_key,
        GlyphAtlasGpuPipelineKey {
            render_contract: gpu_plan.batches[0].key.render_contract,
            primitive_topology: GlyphAtlasGpuPrimitiveTopology::TriangleList,
        }
    );
    assert_eq!(
        gpu_plan.draw_commands[0].render_contract,
        gpu_plan.batches[0].key.render_contract
    );
    assert!(gpu_plan.draw_commands[0].is_quad_aligned());

    assert_eq!(gpu_plan.draw_commands[1].key, gpu_plan.batches[1].key);
    assert_eq!(gpu_plan.draw_commands[1].vertex_start, 12);
    assert_eq!(gpu_plan.draw_commands[1].vertex_count, 6);
    assert_eq!(gpu_plan.draw_commands[1].quad_count(), 1);
    assert_eq!(gpu_plan.draw_commands[1].triangle_count(), 2);
    assert_eq!(gpu_plan.draw_commands[1].atlas_layer, 3);
    assert!(gpu_plan.draw_commands[1].is_quad_aligned());
}

#[test]
fn render_text_atlas_gpu_pipeline_contracts_follow_unique_render_contracts() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(4.0, 4.0, 10.0, 8.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                1,
                GlyphAtlasScreenRect::new(18.0, 4.0, 10.0, 8.0),
            ),
            glyph(
                GlyphAtlasFormat::SubpixelMask,
                0,
                GlyphAtlasScreenRect::new(32.0, 4.0, 10.0, 8.0),
            ),
            glyph(
                GlyphAtlasFormat::Color,
                0,
                GlyphAtlasScreenRect::new(46.0, 4.0, 10.0, 8.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(80, 32));

    assert_eq!(gpu_plan.draw_commands.len(), 4);
    assert_eq!(gpu_plan.pipeline_contracts.len(), 3);
    assert!(gpu_plan.pipeline_contracts.iter().all(|contract| {
        contract.vertex_layout == glyph_atlas_gpu_vertex_buffer_layout()
            && contract.bind_group_layout == glyph_atlas_gpu_bind_group_layout()
            && contract.shader_entry_points == contract.key.render_contract.shader_entry_points()
    }));
    assert!(gpu_plan.pipeline_contracts.iter().any(|contract| contract
        .key
        .render_contract
        .shader_decode
        == super::super::render_contract::GlyphAtlasShaderDecode::AlphaCoverage));
    assert!(gpu_plan.pipeline_contracts.iter().any(|contract| contract
        .key
        .render_contract
        .shader_decode
        == super::super::render_contract::GlyphAtlasShaderDecode::SubpixelRgbCoverage));
    assert!(gpu_plan.pipeline_contracts.iter().any(|contract| contract
        .key
        .render_contract
        .shader_decode
        == super::super::render_contract::GlyphAtlasShaderDecode::ColorRgba));
}

#[test]
fn render_text_atlas_gpu_plan_flattens_batches_into_ndc_vertices() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(10.0, 20.0, 20.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::Color,
                1,
                GlyphAtlasScreenRect::new(50.0, 40.0, 10.0, 20.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 100.0, 100.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(100, 100));

    assert_eq!(
        gpu_plan.vertex_layout,
        glyph_atlas_gpu_vertex_buffer_layout()
    );
    assert_eq!(
        gpu_plan.bind_group_layout,
        glyph_atlas_gpu_bind_group_layout()
    );
    assert_eq!(
        gpu_plan.viewport_transform,
        glyph_atlas_gpu_viewport_transform(UVec2::new(100, 100))
    );
    assert_eq!(gpu_plan.visible_glyph_count, 2);
    assert_eq!(gpu_plan.skipped_glyph_count, 0);
    assert_eq!(gpu_plan.vertex_count(), 12);
    assert_eq!(gpu_plan.batches.len(), 2);
    assert_eq!(gpu_plan.draw_commands.len(), 2);
    assert_eq!(gpu_plan.pipeline_contracts.len(), 2);
    assert!(gpu_plan.pipeline_contracts.iter().all(|contract| {
        contract.shader_entry_points.vertex == "vs_main"
            && contract.shader_entry_points.fragment.starts_with("fs_")
    }));
    assert_eq!(gpu_plan.batches[0].vertex_start, 0);
    assert_eq!(gpu_plan.batches[0].vertex_count, 6);
    assert_eq!(gpu_plan.batches[1].vertex_start, 6);
    assert_eq!(gpu_plan.batches[1].vertex_count, 6);
    assert_eq!(gpu_plan.draw_commands[0].vertex_start, 0);
    assert_eq!(gpu_plan.draw_commands[0].vertex_count, 6);
    assert_eq!(gpu_plan.draw_commands[1].vertex_start, 6);
    assert_eq!(gpu_plan.draw_commands[1].vertex_count, 6);
    assert_near(gpu_plan.vertices[0].position_ndc[0], -0.8);
    assert_near(gpu_plan.vertices[0].position_ndc[1], 0.6);
    assert_near(gpu_plan.vertices[2].position_ndc[0], -0.4);
    assert_near(gpu_plan.vertices[2].position_ndc[1], 0.4);
    assert_eq!(gpu_plan.vertices[6].page_index, 1);
}

#[test]
fn render_text_atlas_gpu_plan_keeps_subpixel_and_color_batches_separate() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::SubpixelMask,
                0,
                GlyphAtlasScreenRect::new(10.0, 20.0, 20.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::Color,
                0,
                GlyphAtlasScreenRect::new(40.0, 20.0, 20.0, 10.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 100.0, 100.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(100, 100));

    assert_eq!(gpu_plan.batches.len(), 2);
    assert!(gpu_plan.requires_background_composite);
    assert_eq!(
        glyph_atlas_gpu_batch_contract(gpu_plan.batches[0]).blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert_eq!(
        glyph_atlas_gpu_batch_contract(gpu_plan.batches[1]).blend_mode,
        GlyphAtlasBlendMode::SourceRgba
    );
}

#[test]
fn render_text_atlas_gpu_plan_preserves_vertex_colors() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [glyph_with_colors(
            GlyphAtlasFormat::SubpixelMask,
            0,
            GlyphAtlasScreenRect::new(8.0, 8.0, 12.0, 12.0),
            [0.9, 0.8, 0.7, 1.0],
            [0.1, 0.2, 0.3, 1.0],
        )],
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 64.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(64, 64));

    assert!(gpu_plan.vertices.iter().all(|vertex| {
        vertex.foreground_color == [0.9, 0.8, 0.7, 1.0]
            && vertex.background_color == [0.1, 0.2, 0.3, 1.0]
    }));
}

#[test]
fn render_text_atlas_gpu_plan_keeps_skipped_glyph_counts_without_empty_batches() {
    let draw_plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(4.0, 4.0, 10.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(80.0, 4.0, 10.0, 10.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 32.0, 32.0),
    );

    let gpu_plan = glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(32, 32));

    assert_eq!(gpu_plan.visible_glyph_count, 1);
    assert_eq!(gpu_plan.skipped_glyph_count, 1);
    assert_eq!(gpu_plan.vertex_count(), 6);
    assert_eq!(gpu_plan.batches.len(), 1);
}

fn glyph(
    format: GlyphAtlasFormat,
    page_index: u32,
    screen_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasDrawGlyph {
    glyph_with_colors(
        format,
        page_index,
        screen_rect,
        [0.92, 0.94, 0.96, 1.0],
        [0.08, 0.1, 0.12, 1.0],
    )
}

fn glyph_with_colors(
    format: GlyphAtlasFormat,
    page_index: u32,
    screen_rect: GlyphAtlasScreenRect,
    foreground_color: [f32; 4],
    background_color: [f32; 4],
) -> GlyphAtlasDrawGlyph {
    GlyphAtlasDrawGlyph {
        page_key: GlyphAtlasPageKey::new(format, page_index),
        atlas_size: UVec2::new(128, 64),
        atlas_rect: GlyphAtlasRect {
            x: 16,
            y: 8,
            width: 24,
            height: 18,
        },
        content_size: UVec2::new(20, 12),
        screen_rect,
        foreground_color,
        background_color,
    }
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be near {expected}"
    );
}

fn assert_ndc_near(actual: [f32; 2], expected: [f32; 2]) {
    assert_near(actual[0], expected[0]);
    assert_near(actual[1], expected[1]);
}
