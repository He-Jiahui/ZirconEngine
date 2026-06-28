use std::cell::RefCell;

use glyphon::{Attrs, Buffer, Family, FontSystem, LayoutGlyph, Metrics, Shaping, Wrap};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

use crate::core::framework::render::{
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedTextLine,
    TextOrientation, TextShapeRequest,
};

use super::line_break::{ClusterLineBreakFlags, LineBreakOpportunityMap};
use super::script_segment::{
    script_for_range, script_segments, shaped_script_for_cluster, ScriptSegment,
};

thread_local! {
    static FONT_SYSTEM: RefCell<FontSystem> = RefCell::new(FontSystem::new());
}

const DEFAULT_FALLBACK_ADVANCE_EM: f32 = 0.56;

pub(crate) fn shape_text(request: TextShapeRequest<'_>) -> ShapedGlyphRun {
    shape_with_cosmic(request).unwrap_or_else(|| fallback_shape(request))
}

fn shape_with_cosmic(request: TextShapeRequest<'_>) -> Option<ShapedGlyphRun> {
    if request.text.is_empty() {
        return Some(empty_run(request));
    }

    FONT_SYSTEM.with(|font_system| {
        let mut font_system = font_system.borrow_mut();
        let line_height = resolved_line_height(request);
        let metrics = Metrics::new(request.style.font_size.max(1.0), line_height);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        let mut buffer = buffer.borrow_with(&mut font_system);
        buffer.set_size(None, Some(line_height));
        buffer.set_wrap(Wrap::None);
        buffer.set_text(
            request.text,
            &attrs_for_style(request),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(true);

        let line_breaks = LineBreakOpportunityMap::new(request.text);
        let scripts = script_segments(request.text);
        let mut lines = Vec::new();
        for run in buffer.layout_runs() {
            lines.push(line_from_layout_run(request, &run, &line_breaks, &scripts));
        }

        if lines.is_empty() {
            return None;
        }

        let measured_width = lines
            .iter()
            .map(|line| line.measured_width)
            .fold(0.0_f32, f32::max);
        let measured_height = lines.iter().map(|line| line.line_height).sum::<f32>();
        Some(ShapedGlyphRun {
            source_text: request.text.to_string(),
            source_range: request.source_range,
            direction: request.base_direction,
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            measured_width,
            measured_height,
            lines,
        })
    })
}

fn line_from_layout_run(
    request: TextShapeRequest<'_>,
    run: &glyphon::LayoutRun<'_>,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
) -> ShapedTextLine {
    let line_visual_start = line_visual_start(request.text, run.line_i);
    let line_source_start = request.source_range.start + line_visual_start;
    let visual_range = UiTextRange {
        start: 0,
        end: run.text.len(),
    };
    let mut previous_range = None;
    let glyphs = run
        .glyphs
        .iter()
        .map(|glyph| {
            let current_range = (glyph.start, glyph.end);
            let cluster_start = previous_range != Some(current_range);
            previous_range = Some(current_range);
            glyph_from_layout_glyph(
                request,
                glyph,
                run.rtl,
                line_visual_start,
                cluster_start,
                line_breaks,
                scripts,
            )
        })
        .collect::<Vec<_>>();

    ShapedTextLine {
        line_index: run.line_i,
        text: run.text.to_string(),
        source_range: UiTextRange {
            start: line_source_start,
            end: line_source_start + run.text.len(),
        },
        visual_range,
        measured_width: run.line_w.max(0.0),
        baseline: run.line_y.max(0.0),
        line_height: run.line_height.max(resolved_line_height(request)),
        glyphs,
    }
}

fn glyph_from_layout_glyph(
    request: TextShapeRequest<'_>,
    glyph: &LayoutGlyph,
    run_rtl: bool,
    line_visual_start: usize,
    cluster_start: bool,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
) -> ShapedGlyph {
    let source_range = absolute_range(
        request.source_range.start + line_visual_start,
        glyph.start,
        glyph.end,
    );
    let cluster_text = request
        .text
        .get(
            (line_visual_start + glyph.start).min(request.text.len())
                ..(line_visual_start + glyph.end).min(request.text.len()),
        )
        .unwrap_or_default();
    let direction = if glyph.level.is_rtl() || run_rtl {
        UiTextDirection::RightToLeft
    } else {
        UiTextDirection::LeftToRight
    };
    let cluster_line_breaks = if cluster_start {
        line_breaks.flags_for_cluster(
            line_visual_start + glyph.start,
            line_visual_start + glyph.end,
        )
    } else {
        ClusterLineBreakFlags::default()
    };
    let local_range = UiTextRange {
        start: line_visual_start + glyph.start,
        end: line_visual_start + glyph.end,
    };
    let script = shaped_script_for_cluster(cluster_text, script_for_range(scripts, local_range));

    ShapedGlyph {
        glyph_id: glyph.glyph_id as u32,
        font_id: None,
        source_range,
        visual_range: UiTextRange {
            start: glyph.start,
            end: glyph.end,
        },
        advance: glyph.w.max(0.0),
        x: glyph.x,
        y: glyph.y,
        offset_x: glyph.x_offset,
        offset_y: glyph.y_offset,
        direction,
        cluster_flags: cluster_flags(cluster_text, direction, cluster_start, cluster_line_breaks),
        rotation: rotation_for_request(request),
        script,
    }
}

fn line_visual_start(text: &str, line_i: usize) -> usize {
    let mut offset = 0;
    for (index, segment) in text.split_inclusive('\n').enumerate() {
        if index == line_i {
            return offset;
        }
        offset += segment.len();
    }
    offset
}

fn empty_run(request: TextShapeRequest<'_>) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    ShapedGlyphRun {
        source_text: request.text.to_string(),
        source_range: request.source_range,
        direction: request.base_direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        measured_width: 0.0,
        measured_height: line_height,
        lines: vec![ShapedTextLine {
            line_index: 0,
            text: String::new(),
            source_range: request.source_range,
            visual_range: UiTextRange::default(),
            measured_width: 0.0,
            baseline: request.style.font_size.max(1.0) * 0.8,
            line_height,
            glyphs: Vec::new(),
        }],
    }
}

fn fallback_shape(request: TextShapeRequest<'_>) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    let baseline = request.style.font_size.max(1.0) * 0.8;
    let line_breaks = LineBreakOpportunityMap::new(request.text);
    let scripts = script_segments(request.text);
    let mut x = 0.0_f32;
    let mut glyphs = Vec::new();

    for (visual_start, grapheme) in request.text.grapheme_indices(true) {
        let visual_end = visual_start + grapheme.len();
        let advance = fallback_grapheme_advance(grapheme, request.style.font_size.max(1.0));
        let direction = match request.base_direction {
            UiTextDirection::RightToLeft => UiTextDirection::RightToLeft,
            _ => UiTextDirection::LeftToRight,
        };
        glyphs.push(ShapedGlyph {
            glyph_id: synthetic_glyph_id(grapheme),
            font_id: None,
            source_range: absolute_range(request.source_range.start, visual_start, visual_end),
            visual_range: UiTextRange {
                start: visual_start,
                end: visual_end,
            },
            advance,
            x,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            direction,
            cluster_flags: cluster_flags(
                grapheme,
                direction,
                true,
                line_breaks.flags_for_cluster(visual_start, visual_end),
            ),
            rotation: rotation_for_request(request),
            script: shaped_script_for_cluster(
                grapheme,
                script_for_range(
                    &scripts,
                    UiTextRange {
                        start: visual_start,
                        end: visual_end,
                    },
                ),
            ),
        });
        x += advance;
    }

    ShapedGlyphRun {
        source_text: request.text.to_string(),
        source_range: request.source_range,
        direction: request.base_direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        measured_width: x,
        measured_height: line_height,
        lines: vec![ShapedTextLine {
            line_index: 0,
            text: request.text.to_string(),
            source_range: request.source_range,
            visual_range: UiTextRange {
                start: 0,
                end: request.text.len(),
            },
            measured_width: x,
            baseline,
            line_height,
            glyphs,
        }],
    }
}

fn cluster_flags(
    cluster_text: &str,
    direction: UiTextDirection,
    cluster_start: bool,
    line_breaks: ClusterLineBreakFlags,
) -> ShapedGlyphClusterFlags {
    ShapedGlyphClusterFlags {
        cluster_start,
        rtl: matches!(direction, UiTextDirection::RightToLeft),
        whitespace: cluster_text.chars().any(char::is_whitespace),
        space: cluster_text
            .chars()
            .any(|ch| matches!(ch, ' ' | '\u{00a0}')),
        tab: cluster_text.contains('\t'),
        mandatory_break: line_breaks.mandatory_break
            || cluster_text.chars().any(|ch| matches!(ch, '\n' | '\r')),
        soft_break: line_breaks.soft_break,
        virtual_glyph: cluster_text.chars().any(char::is_control),
    }
}

fn attrs_for_style<'a>(request: TextShapeRequest<'a>) -> Attrs<'a> {
    match request
        .style
        .font_family
        .as_deref()
        .or(request.style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        Some(family) => Attrs::new().family(Family::Name(family)),
        None => Attrs::new(),
    }
}

fn resolved_line_height(request: TextShapeRequest<'_>) -> f32 {
    request
        .style
        .line_height
        .max(request.style.font_size.max(1.0))
}

fn rotation_for_request(request: TextShapeRequest<'_>) -> ShapedGlyphRotation {
    match request.orientation {
        TextOrientation::Horizontal => ShapedGlyphRotation::None,
        TextOrientation::Vertical => ShapedGlyphRotation::Cw90,
    }
}

fn absolute_range(source_start: usize, visual_start: usize, visual_end: usize) -> UiTextRange {
    UiTextRange {
        start: source_start + visual_start,
        end: source_start + visual_end.max(visual_start),
    }
}

fn fallback_grapheme_advance(grapheme: &str, font_size: f32) -> f32 {
    if grapheme.chars().all(char::is_whitespace) {
        return font_size * 0.33;
    }
    if grapheme.chars().any(is_wide_fallback_grapheme) {
        return font_size;
    }
    if grapheme
        .chars()
        .all(|ch| matches!(ch, 'i' | 'l' | 'I' | '!' | '|' | '.' | ','))
    {
        return font_size * 0.3;
    }
    if grapheme
        .chars()
        .any(|ch| matches!(ch, 'W' | 'M' | 'w' | 'm'))
    {
        return font_size * 0.85;
    }
    font_size * DEFAULT_FALLBACK_ADVANCE_EM
}

fn is_wide_fallback_grapheme(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF00..=0xFFEF
            | 0x1F300..=0x1FAFF
    )
}

fn synthetic_glyph_id(grapheme: &str) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in grapheme.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash.max(1)
}
