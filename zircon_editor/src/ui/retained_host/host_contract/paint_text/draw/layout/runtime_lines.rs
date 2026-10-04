use zircon_runtime::ui::surface::{
    layout_text, resolved_text_glyph_artifact_line, UiResolvedTextGlyphArtifactLine,
};
use zircon_runtime_interface::ui::surface::{UiTextOverflow, UiTextWrap};

use super::super::super::super::data::FrameRect;
use super::super::super::font::{runtime_text_style_for_face, HostTextFontFace};
use super::super::metrics::runtime_text_layout_frame;

/// One Runtime-owned visual line. Editor keeps only placement DTOs and the immutable artifact lease.
pub(super) struct RuntimeTextLine {
    pub(super) text: String,
    pub(super) frame_x: f32,
    pub(super) frame_y: f32,
    pub(super) frame_width: f32,
    pub(super) frame_height: f32,
    pub(super) artifact_line: Option<UiResolvedTextGlyphArtifactLine>,
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
    let layout = layout_text(
        text,
        &style,
        runtime_text_layout_frame(rect, layout_height),
        None,
    );
    let lines = layout
        .lines
        .iter()
        .enumerate()
        .map(|(line_index, line)| RuntimeTextLine {
            text: line.text.clone(),
            frame_x: line.frame.x,
            frame_y: line.frame.y,
            frame_width: line.frame.width,
            frame_height: line.frame.height,
            artifact_line: resolved_text_glyph_artifact_line(&layout, line_index),
        })
        .collect::<Vec<_>>();
    let artifact_line_count = lines
        .iter()
        .filter(|line| line.artifact_line.is_some())
        .count();
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_artifact_candidate_line_count",
        artifact_line_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_artifact_missing_line_count",
        lines.len().saturating_sub(artifact_line_count)
    );
    lines
}

fn empty_runtime_text_line() -> RuntimeTextLine {
    RuntimeTextLine {
        text: String::new(),
        frame_x: 0.0,
        frame_y: 0.0,
        frame_width: 0.0,
        frame_height: 0.0,
        artifact_line: None,
    }
}
