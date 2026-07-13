use super::*;
use crate::asset::{FontAssetFaceMetrics, FontAssetLineMetrics};
use crate::graphics::scene::scene_renderer::ui::render::text_decorations::ScreenSpaceUiTextDecorations;
use crate::graphics::text::font::TextDecorationMetrics;

#[test]
fn render_text_decorations_emit_solid_quads_from_face_metrics() {
    let mut text = text_batch("Decorated", UiFrame::new(10.0, 20.0, 100.0, 30.0));
    text.font_size = 20.0;
    text.line_height = 20.0;
    text.text_decorations = ScreenSpaceUiTextDecorations {
        underline: true,
        strikethrough: true,
        underline_color: [0.8, 0.1, 0.2, 0.7],
        strikethrough_color: [0.1, 0.8, 0.2, 0.6],
    };
    let metrics = TextDecorationMetrics::from_font_units(
        FontAssetFaceMetrics {
            units_per_em: 1_000,
            ascender: 800,
            underline: Some(FontAssetLineMetrics {
                position: -100,
                thickness: 50,
            }),
            strikeout: Some(FontAssetLineMetrics {
                position: 300,
                thickness: 40,
            }),
            ..FontAssetFaceMetrics::default()
        },
        text.font_size,
    );
    let mut vertices = Vec::new();

    super::super::decorations::push_text_decorations_for_metrics(
        &mut vertices,
        &text,
        metrics,
        UiFrame::new(0.0, 0.0, 200.0, 100.0),
    );

    assert_eq!(vertices.len(), 12);
    assert!(vertices
        .iter()
        .all(|vertex| vertex.primitive_kind == super::super::vertices::SDF_TEXT_PRIMITIVE_SOLID));
    assert!(vertices[..6]
        .iter()
        .all(|vertex| vertex.color == [0.8, 0.1, 0.2, 0.7]));
    assert!(vertices[6..]
        .iter()
        .all(|vertex| vertex.color == [0.1, 0.8, 0.2, 0.6]));
    let underline_top_px = (1.0 - vertices[0].position[1]) * 50.0;
    let strikeout_top_px = (1.0 - vertices[6].position[1]) * 50.0;
    assert!((underline_top_px - 37.5).abs() < 0.0001);
    assert!((strikeout_top_px - 29.5).abs() < 0.0001);
}

#[test]
fn render_text_decoration_shader_bypasses_atlas_for_solid_quads() {
    assert!(SDF_TEXT_SHADER.contains("const SOLID_PRIMITIVE"));
    assert!(SDF_TEXT_SHADER.contains("input.primitive_kind == SOLID_PRIMITIVE"));
    naga::front::wgsl::parse_str(SDF_TEXT_SHADER)
        .expect("solid text-decoration branch should keep the SDF shader valid");
}
