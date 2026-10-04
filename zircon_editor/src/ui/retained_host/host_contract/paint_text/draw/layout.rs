use std::sync::Arc;

use zircon_runtime::ui::surface::UiTextGlyphArtifactRasterFace;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::super::font::{font_face_for_paint_style, HostTextFontFace};
use super::super::layout_policy::HostTextLayoutPolicy;

mod artifact;
mod cache;
mod metrics;
mod runtime_lines;

pub(super) use metrics::centered_line_y;

use self::artifact::positioned_artifact_glyphs;
use self::cache::cached_paint_text_layout;
use self::runtime_lines::{runtime_single_line_text, runtime_word_wrapped_text, RuntimeTextLine};

pub(super) struct PaintTextLayout {
    pub(super) display_text: String,
    pub(super) glyphs: Vec<RuntimeTextGlyph>,
    pub(super) artifact_raster_faces: Vec<UiTextGlyphArtifactRasterFace>,
    pub(super) measured_lines: Vec<PaintTextLine>,
}

pub(super) struct PaintTextLine {
    pub(super) text: String,
    pub(super) frame_x: f32,
    pub(super) frame_y: f32,
    pub(super) frame_width: f32,
    pub(super) frame_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RuntimeTextGlyph {
    pub(super) glyph_id: u32,
    pub(super) physical_ppem: u32,
    pub(super) origin_x: f32,
    pub(super) baseline_y: f32,
    pub(super) raster_face_index: usize,
}

pub(super) fn layout_text_run(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) -> Arc<PaintTextLayout> {
    layout_text_run_with_layout_policy(
        rect,
        text,
        font_size,
        line_height,
        style,
        HostTextLayoutPolicy::SingleLineEllipsis,
    )
}

pub(super) fn layout_text_run_with_layout_policy(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
    layout_policy: HostTextLayoutPolicy,
) -> Arc<PaintTextLayout> {
    layout_text_run_with_layout_policy_and_smoothing(
        rect,
        text,
        font_size,
        line_height,
        font_face_for_paint_style(style),
        current_host_text_preferences().smoothing,
        layout_policy,
    )
}

#[cfg(test)]
fn layout_text_run_with_smoothing(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    smoothing: HostTextSmoothing,
) -> Arc<PaintTextLayout> {
    layout_text_run_with_layout_policy_and_smoothing(
        rect,
        text,
        font_size,
        line_height,
        font_face,
        smoothing,
        HostTextLayoutPolicy::SingleLineEllipsis,
    )
}

fn layout_text_run_with_layout_policy_and_smoothing(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
) -> Arc<PaintTextLayout> {
    cached_paint_text_layout(
        rect,
        text,
        font_size,
        line_height,
        font_face,
        smoothing,
        layout_policy,
        || {
            layout_text_run_uncached(
                rect,
                text,
                font_size,
                line_height,
                font_face,
                smoothing,
                layout_policy,
            )
        },
    )
}

fn layout_text_run_uncached(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    font_face: HostTextFontFace,
    smoothing: HostTextSmoothing,
    layout_policy: HostTextLayoutPolicy,
) -> PaintTextLayout {
    zircon_runtime::profile_scope!("editor", "host_painter", "text_layout_cache_miss");
    let lines = match layout_policy {
        HostTextLayoutPolicy::SingleLineEllipsis => vec![runtime_single_line_text(
            rect,
            text,
            font_size,
            line_height,
            font_face,
        )],
        HostTextLayoutPolicy::WordWrap => {
            runtime_word_wrapped_text(rect, text, font_size, line_height, font_face)
        }
    };
    let display_text = display_text_from_lines(&lines);
    let measured_lines = lines
        .iter()
        .map(|line| PaintTextLine {
            text: line.text.clone(),
            frame_x: line.frame_x,
            frame_y: line.frame_y,
            frame_width: line.frame_width,
            frame_height: line.frame_height,
        })
        .collect();
    let artifact = positioned_artifact_glyphs(
        &lines,
        rect,
        font_size,
        line_height,
        smoothing,
        layout_policy,
    );
    let (glyphs, artifact_raster_faces) = match artifact {
        Some(artifact) => {
            zircon_runtime::profile_counter!(
                "editor",
                "retained_text_artifact_projection_layout_hit_count",
                1
            );
            zircon_runtime::profile_counter!(
                "editor",
                "retained_text_artifact_projected_glyph_count",
                artifact.glyphs.len()
            );
            (artifact.glyphs, artifact.raster_faces)
        }
        None => {
            zircon_runtime::profile_counter!(
                "editor",
                "retained_text_artifact_projection_layout_miss_count",
                1
            );
            (Vec::new(), Vec::new())
        }
    };

    PaintTextLayout {
        display_text,
        glyphs,
        artifact_raster_faces,
        measured_lines,
    }
}

fn display_text_from_lines(lines: &[RuntimeTextLine]) -> String {
    let Some((first, rest)) = lines.split_first() else {
        return String::new();
    };
    let additional_capacity = rest
        .iter()
        .map(|line| 1_usize.saturating_add(line.text.len()))
        .sum::<usize>();
    let mut display_text = String::with_capacity(first.text.len() + additional_capacity);
    display_text.push_str(&first.text);
    for line in rest {
        display_text.push('\n');
        display_text.push_str(&line.text);
    }
    display_text
}

#[cfg(test)]
#[path = "layout/tests/cases.rs"]
mod tests;
