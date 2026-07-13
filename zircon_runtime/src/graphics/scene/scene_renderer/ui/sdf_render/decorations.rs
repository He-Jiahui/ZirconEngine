use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::graphics::text::font::{
    text_decoration_frame, FontDatabase, TextDecorationKind, TextDecorationMetrics,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;

use super::super::render::ScreenSpaceUiTextBatch;
use super::super::sdf_font_bake::SdfFontBakeCache;
use super::vertices::{push_clipped_solid_quad, transform_sdf_vertices, ScreenSpaceUiSdfVertex};

pub(super) fn build_text_decoration_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    font_bake: &mut SdfFontBakeCache,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
    viewport_size: UVec2,
) -> Vec<ScreenSpaceUiSdfVertex> {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let mut vertices = Vec::new();
    for text in texts
        .iter()
        .filter(|text| text.text_decorations.underline || text.text_decorations.strikethrough)
    {
        let start = vertices.len();
        let metrics = font_bake.text_decoration_metrics(text, font_database, asset_manager);
        push_text_decorations_for_metrics(&mut vertices, text, metrics, viewport);
        if let Some(transform) = text.clip_transform {
            transform_sdf_vertices(&mut vertices[start..], transform);
        }
    }
    vertices
}

pub(super) fn push_text_decorations_for_metrics(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    text: &ScreenSpaceUiTextBatch,
    metrics: TextDecorationMetrics,
    viewport: UiFrame,
) {
    let clip = text
        .clip_frame
        .and_then(|clip| clip.intersection(viewport))
        .unwrap_or(viewport);
    let baseline = text_decoration_baseline(text, metrics);
    if text.text_decorations.underline {
        let frame = text_decoration_frame(
            text.frame,
            baseline,
            text.writing_mode,
            metrics,
            TextDecorationKind::Underline,
        );
        push_clipped_solid_quad(
            vertices,
            frame,
            clip,
            viewport,
            text.text_decorations.underline_color,
        );
    }
    if text.text_decorations.strikethrough {
        let frame = text_decoration_frame(
            text.frame,
            baseline,
            text.writing_mode,
            metrics,
            TextDecorationKind::Strikethrough,
        );
        push_clipped_solid_quad(
            vertices,
            frame,
            clip,
            viewport,
            text.text_decorations.strikethrough_color,
        );
    }
}

fn text_decoration_baseline(text: &ScreenSpaceUiTextBatch, metrics: TextDecorationMetrics) -> f32 {
    if let Some(baseline) = text.text_decoration_baseline {
        return baseline;
    }
    if matches!(text.writing_mode, UiTextWritingMode::VerticalRl) {
        return text.frame.x + text.frame.width * 0.5;
    }
    let leading = (text.line_height.max(text.font_size) - text.font_size).max(0.0);
    text.frame.y + leading * 0.5 + metrics.ascender_px
}
