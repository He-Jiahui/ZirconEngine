use super::super::render_batch::glyph_atlas_draw_batch_plan;
use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::render_plan::{GlyphAtlasDrawGlyph, GlyphAtlasScreenRect};
use super::super::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasRect};
use super::*;

#[test]
fn render_text_atlas_gpu_viewport_transform_serializes_a_shader_uniform() {
    let transform = glyph_atlas_gpu_viewport_transform(UVec2::new(100, 50));

    assert_eq!(transform.viewport_size, UVec2::new(100, 50));
    assert_eq!(
        transform.pixel_coordinate_convention,
        GlyphAtlasGpuPixelCoordinateConvention::PixelEdges
    );
    assert_eq!(transform.uniform_bytes(), [100.0, 50.0, 0.0, 0.0]);
}

#[test]
fn render_text_atlas_gpu_viewport_uniform_uses_unit_extent_for_empty_viewports() {
    assert_eq!(
        glyph_atlas_gpu_viewport_transform(UVec2::new(0, 0)).uniform_bytes(),
        [1.0, 1.0, 0.0, 0.0]
    );
}

#[test]
fn render_text_atlas_gpu_instance_layout_matches_cpu_instance_shape() {
    let layout = glyph_atlas_gpu_instance_buffer_layout();

    assert_eq!(
        layout.stride_bytes,
        std::mem::size_of::<GlyphAtlasGpuInstance>() as u64
    );
    assert_eq!(layout.stride_bytes, 68);
    assert_eq!(layout.attributes.len(), 5);
    assert_eq!(
        layout.attributes,
        [
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::ScreenRectPx,
                shader_location: 0,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: 0,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::UvRect,
                shader_location: 1,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: 16,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::ForegroundColor,
                shader_location: 2,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: 32,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::BackgroundColor,
                shader_location: 3,
                format: GlyphAtlasGpuInstanceAttributeFormat::Float32x4,
                offset_bytes: 48,
            },
            GlyphAtlasGpuInstanceAttribute {
                semantic: GlyphAtlasGpuInstanceAttributeSemantic::PageIndex,
                shader_location: 4,
                format: GlyphAtlasGpuInstanceAttributeFormat::Uint32,
                offset_bytes: 64,
            },
        ]
    );
}

#[test]
fn render_text_atlas_gpu_bind_group_layout_includes_the_vertex_viewport_uniform() {
    let layout = glyph_atlas_gpu_bind_group_layout();

    assert_eq!(layout.atlas_texture.group, 0);
    assert_eq!(layout.atlas_texture.binding, 0);
    assert_eq!(layout.atlas_sampler.group, 0);
    assert_eq!(layout.atlas_sampler.binding, 1);
    assert_eq!(layout.viewport_uniform.group, 0);
    assert_eq!(layout.viewport_uniform.binding, 2);
}

#[test]
fn render_text_atlas_gpu_draw_commands_follow_batch_instance_ranges() {
    let gpu_plan = plan_for_two_batches();

    assert_eq!(gpu_plan.batches.len(), 3);
    assert_eq!(gpu_plan.draw_commands.len(), 3);
    assert_eq!(gpu_plan.draw_commands[0].instance_start, 0);
    assert_eq!(gpu_plan.draw_commands[0].instance_count, 1);
    assert_eq!(gpu_plan.draw_commands[0].quad_count(), 1);
    assert_eq!(gpu_plan.draw_commands[0].triangle_count(), 2);
    assert_eq!(gpu_plan.draw_commands[0].atlas_layer, 0);
    assert_eq!(gpu_plan.draw_commands[1].instance_start, 1);
    assert_eq!(gpu_plan.draw_commands[1].instance_count, 1);
    assert_eq!(gpu_plan.draw_commands[1].quad_count(), 1);
    assert_eq!(gpu_plan.draw_commands[1].triangle_count(), 2);
    assert_eq!(gpu_plan.draw_commands[1].atlas_layer, 3);
    assert_eq!(gpu_plan.draw_commands[2].instance_start, 2);
    assert_eq!(gpu_plan.draw_commands[2].instance_count, 1);
    assert_eq!(gpu_plan.draw_commands[2].atlas_layer, 0);
    assert!(
        gpu_plan
            .draw_commands
            .iter()
            .all(|command| command.is_quad_aligned())
    );
}

#[test]
fn render_text_atlas_gpu_plan_flattens_batches_into_instances_without_cpu_ndc_vertices() {
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
        gpu_plan.instance_layout,
        glyph_atlas_gpu_instance_buffer_layout()
    );
    assert_eq!(
        gpu_plan.bind_group_layout,
        glyph_atlas_gpu_bind_group_layout()
    );
    assert_eq!(
        gpu_plan.viewport_transform.uniform_bytes(),
        [100.0, 100.0, 0.0, 0.0]
    );
    assert_eq!(gpu_plan.visible_glyph_count, 2);
    assert_eq!(gpu_plan.vertex_count(), 12);
    assert_eq!(gpu_plan.instances.len(), 2);
    assert_eq!(
        gpu_plan.instances[0].screen_rect_px,
        [10.0, 20.0, 20.0, 10.0]
    );
    assert_eq!(
        gpu_plan.instances[0].uv_rect,
        [0.125, 0.125, 0.28125, 0.3125]
    );
    assert_eq!(gpu_plan.instances[1].page_index, 1);
    assert_eq!(gpu_plan.draw_commands[0].instance_start, 0);
    assert_eq!(gpu_plan.draw_commands[1].instance_start, 1);
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
fn render_text_atlas_gpu_plan_preserves_instance_colors() {
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

    assert_eq!(gpu_plan.instances[0].foreground_color, [0.9, 0.8, 0.7, 1.0]);
    assert_eq!(gpu_plan.instances[0].background_color, [0.1, 0.2, 0.3, 1.0]);
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
    assert_eq!(gpu_plan.instances.len(), 1);
    assert_eq!(gpu_plan.vertex_count(), 6);
}

#[test]
fn render_text_atlas_gpu_plan_preallocates_known_output_sizes() {
    let source = include_str!("../render_gpu_plan.rs");

    assert!(source.contains("plan.instances.reserve(draw_plan.instance_count)"));
    assert!(source.contains("plan.batches.reserve(draw_plan.batches.len())"));
    assert!(source.contains("plan.draw_commands.reserve(draw_plan.batches.len())"));
}

#[test]
fn render_text_atlas_gpu_plan_emits_one_instance_per_visible_glyph() {
    let gpu_plan = plan_for_two_batches();

    assert_eq!(gpu_plan.instances.len(), 3);
    assert_eq!(gpu_plan.draw_commands[0].instance_start, 0);
    assert_eq!(gpu_plan.draw_commands[0].instance_count, 1);
    assert_eq!(gpu_plan.draw_commands[1].instance_start, 1);
    assert_eq!(gpu_plan.draw_commands[1].instance_count, 1);
    assert_eq!(gpu_plan.draw_commands[2].instance_start, 2);
    assert_eq!(gpu_plan.draw_commands[2].instance_count, 1);
    assert!(
        std::mem::size_of::<GlyphAtlasGpuInstance>()
            < 6 * (2 * std::mem::size_of::<f32>()
                + 2 * std::mem::size_of::<f32>()
                + 2 * 4 * std::mem::size_of::<f32>()
                + std::mem::size_of::<u32>()),
        "the GPU payload must not materialize six full CPU vertices per glyph",
    );
}

fn plan_for_two_batches() -> GlyphAtlasGpuDrawPlan {
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
    glyph_atlas_gpu_draw_plan(&draw_plan, UVec2::new(80, 32))
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
