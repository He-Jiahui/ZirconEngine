use fontdue::layout::{CoordinateSystem, GlyphPosition, Layout, LayoutSettings, TextStyle};
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime::text::ShapedGlyph;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::FrameRect;
use super::super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::super::font::{
    font_face_for_paint_style, host_font_snapshot_for_face, HostTextFontFace,
};
use super::super::layout_policy::HostTextLayoutPolicy;
use super::placement::{
    retained_glyph_left_offset_px, retained_glyph_placements_share_bin_for_smoothing,
    retained_text_origin_for_smoothing,
};

mod artifact;
mod cache;
mod metrics;
mod runtime_lines;

use self::artifact::positioned_artifact_glyphs;
use self::cache::cached_paint_text_layout;
use self::metrics::{
    advances_include_positive_width, centered_line_y, empty_grapheme_advance_px,
    glyph_origin_matches_without_visible_drift, glyph_origin_preserves_monotonic_order,
    grapheme_advances_match, missing_glyph_left_offset_px, missing_host_advance,
    non_negative_advance, total_advances_match,
};
use self::runtime_lines::{runtime_single_line_text, runtime_word_wrapped_text};

pub(super) struct PaintTextLayout {
    pub(super) display_text: String,
    pub(super) font_face: HostTextFontFace,
    pub(super) glyphs: Vec<RuntimeTextGlyph>,
    pub(super) artifact_raster_fonts: Vec<super::super::font::HostTextFontSnapshot>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RuntimeTextGlyph {
    pub(super) glyph_index: u16,
    pub(super) px: f32,
    // Host-layout bitmap-left fallback; raster bearings stay authoritative when the pen origin is valid.
    pub(super) x: f32,
    // Pen origin used for raster subpixel phase; glyph bearings must not choose the phase.
    pub(super) origin_x: f32,
    pub(super) y: f32,
    // `None` uses the retained host face. An index selects an exact runtime-shaped face.
    pub(super) raster_font_index: Option<usize>,
}

pub(super) fn layout_text_run(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) -> Arc<PaintTextLayout> {
    let font_face = font_face_for_paint_style(style);
    let smoothing = current_host_text_preferences().smoothing;
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

pub(super) fn layout_text_run_with_layout_policy(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
    layout_policy: HostTextLayoutPolicy,
) -> Arc<PaintTextLayout> {
    let font_face = font_face_for_paint_style(style);
    let smoothing = current_host_text_preferences().smoothing;
    layout_text_run_with_layout_policy_and_smoothing(
        rect,
        text,
        font_size,
        line_height,
        font_face,
        smoothing,
        layout_policy,
    )
}

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
        HostTextLayoutPolicy::SingleLineEllipsis => {
            vec![runtime_single_line_text(
                rect,
                text,
                font_size,
                line_height,
                font_face,
            )]
        }
        HostTextLayoutPolicy::WordWrap => {
            runtime_word_wrapped_text(rect, text, font_size, line_height, font_face)
        }
    };
    let display_text = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let artifact_glyphs = positioned_artifact_glyphs(
        &lines,
        rect,
        font_size,
        line_height,
        smoothing,
        layout_policy,
    );
    let (glyphs, artifact_raster_fonts) = if let Some(artifact) = artifact_glyphs {
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projection_layout_hit_count",
            1
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projection_layout_miss_count",
            0
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projected_glyph_count",
            artifact.glyphs.len()
        );
        (artifact.glyphs, artifact.raster_fonts)
    } else {
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projection_layout_hit_count",
            0
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projection_layout_miss_count",
            1
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_artifact_projected_glyph_count",
            0
        );
        let glyphs = lines
            .iter()
            .flat_map(|line| {
                let text_y = match layout_policy {
                    HostTextLayoutPolicy::SingleLineEllipsis => {
                        centered_line_y(rect.y, rect.height, line_height)
                    }
                    HostTextLayoutPolicy::WordWrap => rect.y + line.frame_y,
                };
                let text_x = retained_text_origin_for_smoothing(rect.x + line.frame_x, smoothing);
                runtime_positioned_glyphs(
                    line.text.as_str(),
                    &line.glyph_advances,
                    &line.shaped_glyphs,
                    font_face,
                    font_size,
                    text_x,
                    text_y,
                    smoothing,
                )
            })
            .collect();
        (glyphs, Vec::new())
    };
    PaintTextLayout {
        display_text,
        font_face,
        glyphs,
        artifact_raster_fonts,
    }
}

fn fontdue_glyph_layout(
    display_text: &str,
    font_face: HostTextFontFace,
    font_size: f32,
    x: f32,
    y: f32,
) -> Vec<GlyphPosition> {
    let font = host_font_snapshot_for_face(font_face);
    let Some(font) = font.font() else {
        return Vec::new();
    };
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x,
        y,
        ..LayoutSettings::default()
    });
    layout.append(&[font], &TextStyle::new(display_text, font_size, 0));
    layout.glyphs().clone()
}

fn runtime_positioned_glyphs(
    display_text: &str,
    glyph_advances: &[f32],
    shaped_glyphs: &[ShapedGlyph],
    font_face: HostTextFontFace,
    font_size: f32,
    x: f32,
    y: f32,
    smoothing: HostTextSmoothing,
) -> Vec<RuntimeTextGlyph> {
    let glyphs = fontdue_glyph_layout(display_text, font_face, font_size, x, y);
    if glyphs.is_empty() {
        return Vec::new();
    }

    let graphemes = display_text.grapheme_indices(true).collect::<Vec<_>>();
    if graphemes.is_empty() || graphemes.len() != glyph_advances.len() {
        return glyphs
            .into_iter()
            .map(|glyph| runtime_text_glyph_from_host_glyph(glyph, font_face))
            .collect();
    }

    if let Some(positioned) = runtime_positioned_glyphs_from_shaped_positions(
        display_text,
        &glyphs,
        shaped_glyphs,
        font_face,
        x,
        smoothing,
    ) {
        return positioned;
    }

    if !runtime_advances_match_host_layout(&glyphs, &graphemes, glyph_advances, font_face)
        || !runtime_advances_preserve_retained_raster_bins(
            &glyphs,
            &graphemes,
            glyph_advances,
            font_face,
            x,
            smoothing,
        )
    {
        return glyphs
            .into_iter()
            .map(|glyph| runtime_text_glyph_from_host_glyph(glyph, font_face))
            .collect();
    }

    let grapheme_positions = grapheme_positions(&glyphs, &graphemes, glyph_advances, font_face, x);
    glyphs
        .into_iter()
        .map(|glyph| {
            let origin_x = runtime_glyph_origin_x(&glyph, &grapheme_positions, font_face);
            let x = origin_x + glyph_left_offset(&glyph, font_face);
            RuntimeTextGlyph {
                glyph_index: glyph.key.glyph_index,
                px: glyph.key.px,
                x,
                origin_x,
                y: glyph.y,
                raster_font_index: None,
            }
        })
        .collect()
}

fn runtime_positioned_glyphs_from_shaped_positions(
    display_text: &str,
    glyphs: &[GlyphPosition],
    shaped_glyphs: &[ShapedGlyph],
    font_face: HostTextFontFace,
    start_x: f32,
    smoothing: HostTextSmoothing,
) -> Option<Vec<RuntimeTextGlyph>> {
    if glyphs.is_empty() || glyphs.len() != shaped_glyphs.len() {
        return None;
    }

    let host_width = host_glyph_run_width(glyphs, font_face)?;
    let shaped_width = shaped_glyph_run_width(shaped_glyphs)?;
    if !total_advances_match(shaped_width, host_width) {
        return None;
    }

    for (host, shaped) in glyphs.iter().zip(shaped_glyphs) {
        shaped_position_matches_host_glyph(display_text, host, shaped)?;
    }
    if !shaped_positions_match_host_advances(glyphs, shaped_glyphs, font_face) {
        return None;
    }
    if !shaped_positions_preserve_retained_raster_bins(
        glyphs,
        shaped_glyphs,
        font_face,
        start_x,
        smoothing,
    ) {
        return None;
    }

    let mut previous_origin = None;
    glyphs
        .iter()
        .zip(shaped_glyphs)
        .map(|(host, shaped)| {
            let origin_x = start_x + shaped.x + shaped.offset_x;
            if !glyph_origin_preserves_monotonic_order(origin_x, previous_origin) {
                return None;
            }
            previous_origin = Some(origin_x);
            Some(RuntimeTextGlyph {
                glyph_index: host.key.glyph_index,
                px: host.key.px,
                x: origin_x + glyph_left_offset(host, font_face),
                origin_x,
                y: host.y,
                raster_font_index: None,
            })
        })
        .collect()
}

fn shaped_positions_preserve_retained_raster_bins(
    glyphs: &[GlyphPosition],
    shaped_glyphs: &[ShapedGlyph],
    font_face: HostTextFontFace,
    start_x: f32,
    smoothing: HostTextSmoothing,
) -> bool {
    if glyphs.len() != shaped_glyphs.len() {
        return false;
    }

    glyphs.iter().zip(shaped_glyphs).all(|(host, shaped)| {
        let host_origin = glyph_cursor_x(host, font_face);
        let shaped_origin = start_x + shaped.x + shaped.offset_x;
        host_origin.is_finite()
            && shaped_origin.is_finite()
            && glyph_origin_matches_without_visible_drift(host_origin, shaped_origin)
            && retained_glyph_placements_share_bin_for_smoothing(
                host_origin,
                shaped_origin,
                smoothing,
            )
    })
}

fn shaped_position_matches_host_glyph(
    display_text: &str,
    host: &GlyphPosition,
    shaped: &ShapedGlyph,
) -> Option<()> {
    if shaped.glyph_id != host.key.glyph_index as u32
        || shaped.cluster_flags.rtl
        || shaped.cluster_flags.virtual_glyph
        || !shaped.x.is_finite()
        || !shaped.offset_x.is_finite()
        || !shaped.advance.is_finite()
        || shaped.advance < 0.0
    {
        return None;
    }
    let visual_start = shaped.visual_range.start.min(display_text.len());
    let visual_end = shaped
        .visual_range
        .end
        .min(display_text.len())
        .max(visual_start);
    (host.byte_offset >= visual_start && host.byte_offset < visual_end).then_some(())
}

fn shaped_glyph_run_width(shaped_glyphs: &[ShapedGlyph]) -> Option<f32> {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for glyph in shaped_glyphs {
        let start = glyph.x + glyph.offset_x;
        let end = start + non_negative_advance(glyph.advance);
        if !start.is_finite() || !end.is_finite() {
            return None;
        }
        min_x = min_x.min(start);
        max_x = max_x.max(end);
    }
    (min_x.is_finite() && max_x.is_finite()).then_some(non_negative_advance(max_x - min_x))
}

fn shaped_positions_match_host_advances(
    glyphs: &[GlyphPosition],
    shaped_glyphs: &[ShapedGlyph],
    font_face: HostTextFontFace,
) -> bool {
    if glyphs.len() != shaped_glyphs.len() {
        return false;
    }

    let shaped_origins = shaped_glyphs
        .iter()
        .map(|glyph| {
            let origin = glyph.x + glyph.offset_x;
            origin.is_finite().then_some(origin)
        })
        .collect::<Option<Vec<_>>>();
    let Some(shaped_origins) = shaped_origins else {
        return false;
    };
    let host_origins = glyphs
        .iter()
        .map(|glyph| {
            let origin = glyph_cursor_x(glyph, font_face);
            origin.is_finite().then_some(origin)
        })
        .collect::<Option<Vec<_>>>();
    let Some(host_origins) = host_origins else {
        return false;
    };

    let font = host_font_snapshot_for_face(font_face);
    shaped_glyphs
        .iter()
        .zip(glyphs)
        .enumerate()
        .all(|(index, (shaped, host))| {
            let shaped_next = shaped_origins
                .get(index + 1)
                .copied()
                .unwrap_or_else(|| shaped_origins[index] + non_negative_advance(shaped.advance));
            let host_next = host_origins.get(index + 1).copied().unwrap_or_else(|| {
                let advance = font
                    .font()
                    .map(|font| font.metrics_indexed(host.key.glyph_index, host.key.px))
                    .map(|metrics| non_negative_advance(metrics.advance_width))
                    .unwrap_or_else(missing_host_advance);
                host_origins[index] + advance
            });
            let shaped_advance = non_negative_advance(shaped_next - shaped_origins[index]);
            let host_advance = non_negative_advance(host_next - host_origins[index]);
            shaped_advance.is_finite()
                && host_advance.is_finite()
                && grapheme_advances_match(shaped_advance, host_advance)
        })
}

fn runtime_text_glyph_from_host_glyph(
    glyph: GlyphPosition,
    font_face: HostTextFontFace,
) -> RuntimeTextGlyph {
    let origin_x = glyph_cursor_x(&glyph, font_face);
    RuntimeTextGlyph {
        glyph_index: glyph.key.glyph_index,
        px: glyph.key.px,
        x: glyph.x,
        origin_x,
        y: glyph.y,
        raster_font_index: None,
    }
}

fn runtime_advances_match_host_layout(
    glyphs: &[GlyphPosition],
    graphemes: &[(usize, &str)],
    glyph_advances: &[f32],
    font_face: HostTextFontFace,
) -> bool {
    if glyph_advances.iter().any(|advance| !advance.is_finite()) {
        return false;
    }

    let runtime_width = glyph_advances
        .iter()
        .copied()
        .map(non_negative_advance)
        .sum::<f32>();
    let Some(host_width) = host_glyph_run_width(glyphs, font_face) else {
        return false;
    };
    if !total_advances_match(runtime_width, host_width) {
        return false;
    }

    let Some(host_advances) = host_grapheme_advances(glyphs, graphemes, font_face) else {
        return false;
    };
    advances_match_per_grapheme(glyph_advances, &host_advances)
}

fn host_glyph_run_width(glyphs: &[GlyphPosition], font_face: HostTextFontFace) -> Option<f32> {
    host_glyph_run_bounds(glyphs, font_face).map(|(start, end)| non_negative_advance(end - start))
}

fn host_glyph_run_bounds(
    glyphs: &[GlyphPosition],
    font_face: HostTextFontFace,
) -> Option<(f32, f32)> {
    let first = glyphs.first()?;
    let font = host_font_snapshot_for_face(font_face);
    let font = font.font()?;
    let start = glyph_cursor_x(first, font_face);
    let end = glyphs
        .iter()
        .map(|glyph| {
            let metrics = font.metrics_indexed(glyph.key.glyph_index, glyph.key.px);
            glyph_cursor_x(glyph, font_face) + non_negative_advance(metrics.advance_width)
        })
        .filter(|x| x.is_finite())
        .fold(start, f32::max);
    Some((start, end))
}

fn host_grapheme_advances(
    glyphs: &[GlyphPosition],
    graphemes: &[(usize, &str)],
    font_face: HostTextFontFace,
) -> Option<Vec<f32>> {
    let (_, run_end) = host_glyph_run_bounds(glyphs, font_face)?;
    let starts = graphemes
        .iter()
        .map(|(start, grapheme)| {
            let end = start + grapheme.len();
            glyphs
                .iter()
                .find(|glyph| glyph.byte_offset >= *start && glyph.byte_offset < end)
                .map(|glyph| glyph_cursor_x(glyph, font_face))
        })
        .collect::<Option<Vec<_>>>()?;

    let mut advances = Vec::with_capacity(starts.len());
    for index in 0..starts.len() {
        let next = starts.get(index + 1).copied().unwrap_or(run_end);
        advances.push(non_negative_advance(next - starts[index]));
    }
    Some(advances)
}

fn advances_match_per_grapheme(runtime_advances: &[f32], host_advances: &[f32]) -> bool {
    runtime_advances.len() == host_advances.len()
        && runtime_advances
            .iter()
            .zip(host_advances)
            .all(|(runtime, host)| grapheme_advances_match(*runtime, *host))
}

fn runtime_advances_preserve_retained_raster_bins(
    glyphs: &[GlyphPosition],
    graphemes: &[(usize, &str)],
    advances: &[f32],
    font_face: HostTextFontFace,
    start_x: f32,
    smoothing: HostTextSmoothing,
) -> bool {
    if graphemes.len() != advances.len() {
        return false;
    }

    let positions = grapheme_positions(glyphs, graphemes, advances, font_face, start_x);
    glyphs.iter().all(|glyph| {
        let host_origin = glyph_cursor_x(glyph, font_face);
        let projected_origin = runtime_glyph_origin_x(glyph, &positions, font_face);
        host_origin.is_finite()
            && projected_origin.is_finite()
            && glyph_origin_matches_without_visible_drift(host_origin, projected_origin)
            && retained_glyph_placements_share_bin_for_smoothing(
                host_origin,
                projected_origin,
                smoothing,
            )
    })
}

#[derive(Clone, Copy, Debug)]
struct GraphemePosition {
    start: usize,
    end: usize,
    original_x: f32,
    runtime_x: f32,
}

fn grapheme_positions(
    glyphs: &[GlyphPosition],
    graphemes: &[(usize, &str)],
    advances: &[f32],
    font_face: HostTextFontFace,
    start_x: f32,
) -> Vec<GraphemePosition> {
    let mut runtime_x = start_x;
    let mut positions = Vec::with_capacity(graphemes.len());
    for ((start, grapheme), advance) in graphemes.iter().copied().zip(advances.iter().copied()) {
        let end = start + grapheme.len();
        let original_x = glyphs
            .iter()
            .find(|glyph| glyph.byte_offset >= start && glyph.byte_offset < end)
            .map(|glyph| glyph_cursor_x(glyph, font_face))
            .unwrap_or(runtime_x);
        positions.push(GraphemePosition {
            start,
            end,
            original_x,
            runtime_x,
        });
        runtime_x += non_negative_advance(advance);
    }
    positions
}

fn runtime_glyph_origin_x(
    glyph: &GlyphPosition,
    positions: &[GraphemePosition],
    font_face: HostTextFontFace,
) -> f32 {
    let original_origin_x = glyph_cursor_x(glyph, font_face);
    positions
        .iter()
        .find(|position| glyph.byte_offset >= position.start && glyph.byte_offset < position.end)
        .map(|position| position.runtime_x + (original_origin_x - position.original_x))
        .unwrap_or(original_origin_x)
}

fn glyph_cursor_x(glyph: &GlyphPosition, font_face: HostTextFontFace) -> f32 {
    glyph.x - glyph_left_offset(glyph, font_face)
}

fn glyph_left_offset(glyph: &GlyphPosition, font_face: HostTextFontFace) -> f32 {
    host_font_snapshot_for_face(font_face)
        .font()
        .map(|font| font.metrics_indexed(glyph.key.glyph_index, glyph.key.px))
        .map(|metrics| retained_glyph_left_offset_px(metrics.bounds.xmin))
        .unwrap_or_else(missing_glyph_left_offset_px)
}

#[derive(Clone, Copy, Debug)]
struct GraphemeRange {
    start: usize,
    end: usize,
}

fn runtime_shaped_glyph_advances_from_run(
    display_text: &str,
    shaped: &zircon_runtime::text::ShapedGlyphRun,
    fallback: &[f32],
) -> Vec<f32> {
    if display_text.is_empty() {
        return Vec::new();
    }

    let Some(line) = shaped.lines.first() else {
        return fallback.to_vec();
    };
    let shaped_advances = shaped_grapheme_advances(display_text, &line.glyphs);
    let grapheme_count = display_text.graphemes(true).count();
    if shaped_advances.len() == grapheme_count
        && shaped_advances.iter().all(|advance| advance.is_finite())
        && advances_include_positive_width(&shaped_advances)
    {
        shaped_advances
    } else {
        fallback.to_vec()
    }
}

fn shaped_grapheme_advances(display_text: &str, glyphs: &[ShapedGlyph]) -> Vec<f32> {
    let graphemes = display_text
        .grapheme_indices(true)
        .map(|(start, grapheme)| GraphemeRange {
            start,
            end: start + grapheme.len(),
        })
        .collect::<Vec<_>>();
    let mut advances = vec![empty_grapheme_advance_px(); graphemes.len()];
    for glyph in glyphs {
        let advance = non_negative_advance(glyph.advance);
        if !advance.is_finite() {
            continue;
        }
        let source_start = glyph.source_range.start.min(display_text.len());
        let source_end = glyph
            .source_range
            .end
            .min(display_text.len())
            .max(source_start);
        let overlaps = graphemes
            .iter()
            .enumerate()
            .filter_map(|(index, grapheme)| {
                grapheme_ranges_overlap(grapheme.start, grapheme.end, source_start, source_end)
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        if overlaps.is_empty() {
            continue;
        }
        let grapheme_advance = advance / overlaps.len() as f32;
        for index in overlaps {
            advances[index] += grapheme_advance;
        }
    }
    advances
}

fn grapheme_ranges_overlap(
    grapheme_start: usize,
    grapheme_end: usize,
    source_start: usize,
    source_end: usize,
) -> bool {
    if source_start == source_end {
        return source_start >= grapheme_start && source_start < grapheme_end;
    }
    grapheme_start < source_end && source_start < grapheme_end
}

#[cfg(test)]
mod tests;
