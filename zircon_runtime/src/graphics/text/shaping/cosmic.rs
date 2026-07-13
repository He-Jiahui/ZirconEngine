use glyphon::{
    cosmic_text::{FeatureTag, FontFeatures},
    Attrs, Buffer, Family, LayoutGlyph, Metrics, Shaping, Weight, Wrap,
};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use crate::core::framework::render::{
    normalized_open_type_features, ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation,
    ShapedGlyphRun, ShapedTextLine, TextOrientation, TextShapeRequest,
};
use crate::graphics::text::font::FontDatabase;

use super::bidi::BidiParagraph;
use super::horizontal::apply_horizontal_backend_shaping;
use super::line_break::{ClusterLineBreakFlags, LineBreakOpportunityMap};
use super::normalize::ShapingTextView;
use super::script_segment::{
    script_for_range, script_segments, shaped_script_for_cluster, ScriptSegment,
};
use super::vertical::apply_vertical_layout;

mod font_system_cache;

use super::fallback_text_spans;
use font_system_cache::with_font_system;

const DEFAULT_FALLBACK_ADVANCE_EM: f32 = 0.56;

pub(crate) fn shape_text(request: TextShapeRequest<'_>) -> ShapedGlyphRun {
    let text_view = ShapingTextView::v1_disabled(request.text);
    let bidi = BidiParagraph::new(text_view.shaping_text(), request.base_direction);
    if let Some(shaped) = shape_with_cosmic(request, &text_view, &bidi) {
        return shaped;
    }
    let mut shaped = fallback_shape(request, &text_view, &bidi);
    apply_vertical_layout(&mut shaped, request, None);
    shaped
}

fn shape_with_cosmic(
    request: TextShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    bidi: &BidiParagraph<'_>,
) -> Option<ShapedGlyphRun> {
    if text_view.shaping_text().is_empty() {
        let mut shaped = empty_run(request, bidi);
        apply_vertical_layout(&mut shaped, request, None);
        return Some(shaped);
    }

    with_font_system(request.language, |font_system, font_database| {
        let line_height = resolved_line_height(request);
        let metrics = Metrics::new(request.style.font_size.max(1.0), line_height);
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_size(None, Some(line_height));
        buffer.set_wrap(Wrap::None);
        let default_attrs = attrs_for_style(request);
        let fallback_spans = fallback_text_spans(text_view.shaping_text(), request, font_database);
        if fallback_spans.is_empty() {
            buffer.set_text(
                text_view.shaping_text(),
                &default_attrs,
                Shaping::Advanced,
                None,
            );
        } else {
            buffer.set_rich_text(
                fallback_spans.iter().map(|span| {
                    let attrs = span
                        .family
                        .as_deref()
                        .map(|family| default_attrs.clone().family(Family::Name(family)))
                        .unwrap_or_else(|| default_attrs.clone());
                    (&text_view.shaping_text()[span.range.clone()], attrs)
                }),
                &default_attrs,
                Shaping::Advanced,
                None,
            );
        }
        buffer.shape_until_scroll(true);

        let line_breaks = LineBreakOpportunityMap::new(text_view.shaping_text());
        let scripts = script_segments(text_view.shaping_text());
        let mut lines = Vec::new();
        for run in buffer.layout_runs() {
            lines.push(line_from_layout_run(
                request,
                text_view,
                &run,
                &line_breaks,
                &scripts,
                bidi,
                font_database,
            ));
        }

        if lines.is_empty() {
            return None;
        }

        let measured_width = lines
            .iter()
            .map(|line| line.measured_width)
            .fold(0.0_f32, f32::max);
        let measured_height = lines.iter().map(|line| line.line_height).sum::<f32>();
        let mut shaped = ShapedGlyphRun {
            source_text: request.text.to_string(),
            source_range: request.source_range,
            direction: bidi.resolved_base_direction(),
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            include_kerning: request.include_kerning,
            measured_width,
            measured_height,
            lines,
        };
        apply_horizontal_backend_shaping(&mut shaped, request, font_database);
        apply_vertical_layout(&mut shaped, request, Some(font_database));
        Some(shaped)
    })
}

fn line_from_layout_run(
    request: TextShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    run: &glyphon::LayoutRun<'_>,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
    bidi: &BidiParagraph<'_>,
    font_database: &FontDatabase,
) -> ShapedTextLine {
    let line_visual_start = line_visual_start(text_view.shaping_text(), run.line_i);
    let line_shaping_range = line_visual_start..line_visual_start + run.text.len();
    let line_source_range = text_view.source_range_for_shaping_range(line_shaping_range);
    let line_source_start = request.source_range.start + line_source_range.start;
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
                text_view,
                glyph,
                run.rtl,
                line_visual_start,
                cluster_start,
                line_breaks,
                scripts,
                bidi,
                font_database,
            )
        })
        .collect::<Vec<_>>();

    ShapedTextLine {
        line_index: run.line_i,
        text: run.text.to_string(),
        source_range: UiTextRange {
            start: line_source_start,
            end: request.source_range.start + line_source_range.end,
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
    text_view: &ShapingTextView<'_>,
    glyph: &LayoutGlyph,
    run_rtl: bool,
    line_visual_start: usize,
    cluster_start: bool,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
    bidi: &BidiParagraph<'_>,
    font_database: &FontDatabase,
) -> ShapedGlyph {
    let shaping_range = line_visual_start + glyph.start..line_visual_start + glyph.end;
    let projected_source_range = text_view.source_range_for_shaping_range(shaping_range.clone());
    let source_range = absolute_range(
        request.source_range.start,
        projected_source_range.start,
        projected_source_range.end,
    );
    let cluster_text = text_view
        .shaping_text()
        .get(
            shaping_range.start.min(text_view.shaping_text().len())
                ..shaping_range.end.min(text_view.shaping_text().len()),
        )
        .unwrap_or_default();
    let local_range = UiTextRange {
        start: line_visual_start + glyph.start,
        end: line_visual_start + glyph.end,
    };
    let bidi_level = bidi.level_for_range(local_range);
    let direction = if bidi_level % 2 == 1 || glyph.level.is_rtl() || run_rtl {
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
    let script = shaped_script_for_cluster(cluster_text, script_for_range(scripts, local_range));

    let (offset_x, offset_y) =
        glyph_layout_offset_px(glyph.font_size, glyph.x_offset, glyph.y_offset);
    let font_id = font_database.font_face_id(glyph.font_id);
    ShapedGlyph {
        glyph_id: glyph.glyph_id as u32,
        font_id,
        font_instance_id: font_id.and_then(|face| {
            font_database
                .effective_instance_id(
                    face,
                    UiResolvedStyle::normalized_font_weight(request.style.font_weight),
                )
                .ok()
        }),
        source_range,
        visual_range: UiTextRange {
            start: glyph.start,
            end: glyph.end,
        },
        advance: glyph.w.max(0.0),
        x: glyph.x,
        y: glyph.y,
        offset_x,
        offset_y,
        direction,
        bidi_level,
        cluster_flags: cluster_flags(cluster_text, direction, cluster_start, cluster_line_breaks),
        rotation: ShapedGlyphRotation::None,
        script,
    }
}

fn glyph_layout_offset_px(font_size: f32, x_offset: f32, y_offset: f32) -> (f32, f32) {
    let font_size = font_size.max(1.0);
    (
        finite_offset_px(font_size, x_offset),
        finite_offset_px(font_size, y_offset),
    )
}

fn finite_offset_px(font_size: f32, offset: f32) -> f32 {
    if offset.is_finite() {
        font_size * offset
    } else {
        0.0
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

fn empty_run(request: TextShapeRequest<'_>, bidi: &BidiParagraph<'_>) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    ShapedGlyphRun {
        source_text: request.text.to_string(),
        source_range: request.source_range,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
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

fn fallback_shape(
    request: TextShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    bidi: &BidiParagraph<'_>,
) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    let baseline = request.style.font_size.max(1.0) * 0.8;
    let line_breaks = LineBreakOpportunityMap::new(text_view.shaping_text());
    let scripts = script_segments(text_view.shaping_text());
    let mut x = 0.0_f32;
    let mut glyphs = Vec::new();

    for (visual_start, grapheme) in text_view.shaping_text().grapheme_indices(true) {
        let visual_end = visual_start + grapheme.len();
        let advance = fallback_grapheme_advance(grapheme, request.style.font_size.max(1.0));
        let local_range = UiTextRange {
            start: visual_start,
            end: visual_end,
        };
        let bidi_level = bidi.level_for_range(local_range);
        let direction = if bidi_level % 2 == 1 {
            UiTextDirection::RightToLeft
        } else {
            UiTextDirection::LeftToRight
        };
        glyphs.push(ShapedGlyph {
            glyph_id: synthetic_glyph_id(grapheme),
            font_id: None,
            font_instance_id: None,
            source_range: {
                let projected = text_view.source_range_for_shaping_range(visual_start..visual_end);
                absolute_range(request.source_range.start, projected.start, projected.end)
            },
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
            bidi_level,
            cluster_flags: cluster_flags(
                grapheme,
                direction,
                true,
                line_breaks.flags_for_cluster(visual_start, visual_end),
            ),
            rotation: ShapedGlyphRotation::None,
            script: shaped_script_for_cluster(grapheme, script_for_range(&scripts, local_range)),
        });
        x += advance;
    }

    ShapedGlyphRun {
        source_text: request.text.to_string(),
        source_range: request.source_range,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
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
    let attrs = match request
        .style
        .font_family
        .as_deref()
        .or(request.style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        Some(family) => Attrs::new().family(Family::Name(family)),
        None => Attrs::new(),
    };
    let attrs = attrs.weight(Weight(UiResolvedStyle::normalized_font_weight(
        request.style.font_weight,
    )));
    let uses_vertical_features = matches!(request.orientation, TextOrientation::Vertical)
        && !matches!(
            request.vertical_mode,
            crate::core::framework::render::VerticalMode::Sideways
        );
    if request.include_kerning && request.features.is_empty() && !uses_vertical_features {
        return attrs;
    }

    let mut features = FontFeatures::new();
    if !request.include_kerning {
        features.disable(FeatureTag::KERNING);
    }
    let requested_features = normalized_open_type_features(request.features);
    if uses_vertical_features {
        if !requested_features
            .iter()
            .any(|feature| feature.tag == *b"vert")
        {
            features.set(FeatureTag::new(b"vert"), 1);
        }
        if !requested_features
            .iter()
            .any(|feature| feature.tag == *b"vrt2")
        {
            features.set(FeatureTag::new(b"vrt2"), 1);
        }
    }
    for feature in requested_features {
        features.set(FeatureTag::new(&feature.tag), feature.value);
    }
    attrs.font_features(features)
}

fn resolved_line_height(request: TextShapeRequest<'_>) -> f32 {
    request
        .style
        .line_height
        .max(request.style.font_size.max(1.0))
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

#[cfg(test)]
mod tests {
    use glyphon::cosmic_text::FeatureTag;
    use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

    use super::{attrs_for_style, glyph_layout_offset_px};
    use crate::core::framework::render::{OpenTypeFeature, TextShapeRequest};

    #[test]
    fn glyph_layout_offsets_are_projected_to_pixels() {
        let (x, y) = glyph_layout_offset_px(13.0, 0.25, -0.125);

        assert!((x - 3.25).abs() < 0.001);
        assert!((y + 1.625).abs() < 0.001);
    }

    #[test]
    fn glyph_layout_offsets_drop_non_finite_values() {
        let (x, y) = glyph_layout_offset_px(13.0, f32::NAN, f32::INFINITY);

        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn attrs_disable_kerning_when_requested() {
        let style = UiResolvedStyle::default();
        let attrs = attrs_for_style(TextShapeRequest::horizontal_with_kerning(
            "AV",
            &style,
            UiTextDirection::LeftToRight,
            UiTextRange { start: 0, end: 2 },
            false,
        ));

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::KERNING && feature.value == 0));
    }

    #[test]
    fn attrs_apply_normalized_open_type_features() {
        let style = UiResolvedStyle::default();
        let features = [
            OpenTypeFeature::new(*b"tnum", 1),
            OpenTypeFeature::new(*b"liga", 0),
        ];
        let attrs = attrs_for_style(
            TextShapeRequest::horizontal(
                "0123",
                &style,
                UiTextDirection::LeftToRight,
                UiTextRange { start: 0, end: 4 },
            )
            .with_features(&features),
        );

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"tnum") && feature.value == 1));
        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"liga") && feature.value == 0));
    }

    #[test]
    fn attrs_enable_vertical_substitution_features_for_upright_glyphs() {
        let style = UiResolvedStyle::default();
        let attrs = attrs_for_style(TextShapeRequest::vertical(
            "本文。",
            &style,
            UiTextDirection::LeftToRight,
            UiTextRange {
                start: 0,
                end: "本文。".len(),
            },
            crate::core::framework::render::VerticalMode::Mixed,
        ));

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vert") && feature.value == 1));
        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vrt2") && feature.value == 1));
    }
}
