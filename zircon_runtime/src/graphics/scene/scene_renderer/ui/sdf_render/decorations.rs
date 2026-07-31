use crate::core::math::UVec2;
use crate::text::font::{TextDecorationKind, TextDecorationMetrics, text_decoration_frame};
use crate::text::sdf::SdfRunCpuPreparation;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;

use super::super::render::ScreenSpaceUiTextBatch;
use super::vertices::{ScreenSpaceUiSdfVertex, push_clipped_solid_quad, transform_sdf_vertices};

pub(super) fn build_text_decoration_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    cpu_runs: &[SdfRunCpuPreparation],
    viewport_size: UVec2,
) -> Vec<ScreenSpaceUiSdfVertex> {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let mut vertices = Vec::new();
    for (text, cpu_run) in texts
        .iter()
        .zip(cpu_runs)
        .filter(|(text, _)| text.text_decorations.underline || text.text_decorations.strikethrough)
    {
        let start = vertices.len();
        push_text_decorations_for_metrics(
            &mut vertices,
            text,
            cpu_run.decoration_metrics,
            viewport,
        );
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
        let frame: UiFrame = text_decoration_frame(
            text.frame.into(),
            baseline,
            text.writing_mode.into(),
            metrics,
            TextDecorationKind::Underline,
        )
        .into();
        push_clipped_solid_quad(
            vertices,
            frame,
            clip,
            viewport,
            text.text_decorations.underline_color,
        );
    }
    if text.text_decorations.strikethrough {
        let frame: UiFrame = text_decoration_frame(
            text.frame.into(),
            baseline,
            text.writing_mode.into(),
            metrics,
            TextDecorationKind::Strikethrough,
        )
        .into();
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
