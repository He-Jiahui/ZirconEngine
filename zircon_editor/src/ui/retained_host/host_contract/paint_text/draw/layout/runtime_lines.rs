use zircon_runtime::{
    text::ShapedGlyph,
    ui::surface::{layout_text, shape_text_line},
};
use zircon_runtime_interface::ui::surface::{UiTextOverflow, UiTextWrap};

use super::super::super::super::data::FrameRect;
use super::super::super::font::{runtime_text_style_for_face, HostTextFontFace};
use super::super::metrics::runtime_text_layout_frame;
use super::metrics::empty_runtime_line_frame_x;
use super::runtime_shaped_glyph_advances_from_run;

/// Runtime layout output retained until either exact artifact projection or host fallback consumes it.
pub(super) struct RuntimeTextLine {
    pub(super) text: String,
    pub(super) frame_x: f32,
    pub(super) frame_y: f32,
    pub(super) glyph_advances: Vec<f32>,
    pub(super) shaped_glyphs: Vec<ShapedGlyph>,
    pub(super) artifact_line: Option<zircon_runtime::ui::surface::UiResolvedTextGlyphArtifactLine>,
}

pub(super) fn runtime_single_line_text(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
) -> RuntimeTextLine {
    runtime_text_lines(
        rect,
        text,
        font_size,
        line_height,
        font_face,
        UiTextWrap::None,
        line_height,
    )
    .into_iter()
    .next()
    .unwrap_or_else(empty_runtime_text_line)
}

pub(super) fn runtime_word_wrapped_text(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
) -> Vec<RuntimeTextLine> {
    runtime_text_lines(
        rect,
        text,
        font_size,
        line_height,
        font_face,
        UiTextWrap::Word,
        rect.height,
    )
}

fn runtime_text_lines(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    wrap: UiTextWrap,
    layout_height: f32,
) -> Vec<RuntimeTextLine> {
    let style = runtime_text_style_for_face(
        font_face,
        font_size,
        line_height,
        wrap,
        UiTextOverflow::Ellipsis,
    );
    let frame = runtime_text_layout_frame(rect, layout_height);
    let layout = layout_text(text, &style, frame, None);
    let artifact_lines = layout
        .lines
        .iter()
        .enumerate()
        .map(|(line_index, _)| {
            zircon_runtime::ui::surface::resolved_text_glyph_artifact_line(&layout, line_index)
        })
        .collect::<Option<Vec<_>>>();
    if let Some(artifact_lines) = artifact_lines {
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_candidate_line_count",
            artifact_lines.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_candidate_glyph_count",
            artifact_lines
                .iter()
                .filter_map(|artifact_line| artifact_line.glyphs())
                .map(|glyphs| glyphs.len())
                .sum::<usize>()
        );
        zircon_runtime::profile_counter!("editor", "retained_text_surface_shape_line_count", 0);
        zircon_runtime::profile_counter!("editor", "retained_text_shaped_glyph_copy_count", 0);
        zircon_runtime::profile_counter!("editor", "retained_text_shaped_glyph_copy_line_count", 0);
        return layout
            .lines
            .iter()
            .zip(artifact_lines)
            .map(|(line, artifact_line)| RuntimeTextLine {
                text: line.text.clone(),
                frame_x: line.frame.x,
                frame_y: line.frame.y,
                glyph_advances: line.glyph_advances.clone(),
                shaped_glyphs: Vec::new(),
                artifact_line: Some(artifact_line),
            })
            .collect();
    }
    let mut shaped_glyph_copy_count = 0_usize;
    let mut shaped_glyph_copy_line_count = 0_usize;
    let fallback_lines = layout
        .lines
        .iter()
        .map(|line| {
            let shaped = shape_text_line(line.text.as_str(), &style);
            let glyph_advances = runtime_shaped_glyph_advances_from_run(
                line.text.as_str(),
                &shaped,
                &line.glyph_advances,
            );
            let shaped_glyphs = shaped
                .lines
                .first()
                .map(|shaped_line| shaped_line.glyphs.clone())
                .unwrap_or_default();
            shaped_glyph_copy_count = shaped_glyph_copy_count.saturating_add(shaped_glyphs.len());
            shaped_glyph_copy_line_count = shaped_glyph_copy_line_count
                .saturating_add(if shaped_glyphs.is_empty() { 0 } else { 1 });
            RuntimeTextLine {
                text: line.text.clone(),
                frame_x: line.frame.x,
                frame_y: line.frame.y,
                glyph_advances,
                shaped_glyphs,
                artifact_line: None,
            }
        })
        .collect();
    zircon_runtime::profile_counter!("editor", "retained_text_artifact_candidate_line_count", 0);
    zircon_runtime::profile_counter!("editor", "retained_text_artifact_candidate_glyph_count", 0);
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_surface_shape_line_count",
        fallback_lines.len()
    );
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_shaped_glyph_copy_count",
        shaped_glyph_copy_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_shaped_glyph_copy_line_count",
        shaped_glyph_copy_line_count
    );
    fallback_lines
}

fn empty_runtime_text_line() -> RuntimeTextLine {
    RuntimeTextLine {
        text: String::new(),
        frame_x: empty_runtime_line_frame_x(),
        frame_y: 0.0,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        artifact_line: None,
    }
}
