use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use crate::core::framework::render::{
    FontFaceId, InstancedFaceId, ShapedGlyphRotation, TextShapeRequest, VerticalMode,
};
use crate::graphics::text::shaping::shape_text;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiShapedGlyph {
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_id: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) font_id: Option<FontFaceId>,
    pub(in crate::graphics::scene::scene_renderer::ui) font_instance_id: Option<InstancedFaceId>,
    pub(in crate::graphics::scene::scene_renderer::ui) source_scalar: char,
    pub(in crate::graphics::scene::scene_renderer::ui) source_range: UiTextRange,
    pub(in crate::graphics::scene::scene_renderer::ui) advance: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) offset_x: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) offset_y: f32,
    pub(in crate::graphics::scene::scene_renderer::ui) rotation: ShapedGlyphRotation,
    pub(in crate::graphics::scene::scene_renderer::ui) requires_atlas_slot: bool,
}

pub(super) struct ScreenSpaceTextShapingRequest<'a> {
    pub(super) text: &'a str,
    pub(super) font: Option<&'a str>,
    pub(super) font_family: Option<&'a str>,
    pub(super) language: Option<&'a str>,
    pub(super) font_weight: u16,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) direction: UiTextDirection,
    pub(super) writing_mode: zircon_runtime_interface::ui::surface::UiTextWritingMode,
    pub(super) source_range: UiTextRange,
}

pub(super) struct ResolvedScreenSpaceTextGlyphs {
    pub(super) glyph_advances: Vec<f32>,
    pub(super) shaped_glyphs: Vec<ScreenSpaceUiShapedGlyph>,
}

pub(super) fn resolve_screen_space_text_glyphs(
    request: ScreenSpaceTextShapingRequest<'_>,
    glyph_advances: Vec<f32>,
) -> ResolvedScreenSpaceTextGlyphs {
    use zircon_runtime_interface::ui::surface::UiTextWritingMode;

    let style = UiResolvedStyle {
        font: request.font.map(str::to_string),
        font_family: request.font_family.map(str::to_string),
        language: request.language.map(str::to_string),
        font_weight: request.font_weight,
        font_size: request.font_size,
        line_height: request.line_height,
        text_direction: request.direction,
        text_writing_mode: request.writing_mode,
        ..UiResolvedStyle::default()
    };
    let mut shaped_glyphs = if matches!(request.writing_mode, UiTextWritingMode::VerticalRl) {
        resolved_vertical_text_glyphs(
            request.text,
            &style,
            request.direction,
            request.source_range,
        )
    } else {
        resolved_horizontal_text_glyphs(
            request.text,
            &style,
            request.direction,
            request.source_range,
        )
    };
    let glyph_advances = if matches!(request.writing_mode, UiTextWritingMode::VerticalRl) {
        let resolved_advances = if glyph_advances.is_empty() {
            vertical_advances_by_source_grapheme(request.text, request.source_range, &shaped_glyphs)
        } else {
            glyph_advances
        };
        apply_resolved_vertical_advances(
            request.text,
            request.source_range,
            resolved_advances.as_slice(),
            &mut shaped_glyphs,
        );
        resolved_advances
    } else {
        glyph_advances
    };
    ResolvedScreenSpaceTextGlyphs {
        glyph_advances,
        shaped_glyphs,
    }
}

pub(super) fn resolved_vertical_text_glyphs(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> Vec<ScreenSpaceUiShapedGlyph> {
    let shaped = shape_text(TextShapeRequest::vertical(
        text,
        style,
        direction,
        source_range,
        VerticalMode::Mixed,
    ));
    shaped_glyphs_for_screen_space(text, source_range, shaped)
}

pub(super) fn resolved_horizontal_text_glyphs(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> Vec<ScreenSpaceUiShapedGlyph> {
    let shaped = shape_text(TextShapeRequest::horizontal(
        text,
        style,
        direction,
        source_range,
    ));
    shaped_glyphs_for_screen_space(text, source_range, shaped)
}

fn shaped_glyphs_for_screen_space(
    text: &str,
    source_range: UiTextRange,
    shaped: crate::core::framework::render::ShapedGlyphRun,
) -> Vec<ScreenSpaceUiShapedGlyph> {
    shaped
        .lines
        .iter()
        .flat_map(|line| &line.glyphs)
        .filter_map(|glyph| {
            let source_scalar = source_scalar_for_range(text, source_range, glyph.source_range)?;
            Some(ScreenSpaceUiShapedGlyph {
                glyph_id: glyph.glyph_id,
                font_id: glyph.font_id,
                font_instance_id: glyph.font_instance_id,
                source_scalar,
                source_range: glyph.source_range,
                advance: sanitized_advance(glyph.advance),
                offset_x: sanitized_position(glyph.offset_x),
                offset_y: sanitized_position(glyph.offset_y),
                rotation: glyph.rotation,
                requires_atlas_slot: !glyph.cluster_flags.virtual_glyph
                    && !glyph.cluster_flags.whitespace
                    && !glyph.cluster_flags.space
                    && !glyph.cluster_flags.tab
                    && !source_scalar.is_whitespace(),
            })
        })
        .collect()
}

pub(super) fn vertical_advances_by_source_grapheme(
    text: &str,
    source_range: UiTextRange,
    glyphs: &[ScreenSpaceUiShapedGlyph],
) -> Vec<f32> {
    text.grapheme_indices(true)
        .map(|(start, grapheme)| {
            let range = UiTextRange {
                start: source_range.start + start,
                end: source_range.start + start + grapheme.len(),
            };
            glyphs
                .iter()
                .filter(|glyph| {
                    glyph.source_range.end > range.start && glyph.source_range.end <= range.end
                })
                .map(|glyph| glyph.advance)
                .sum::<f32>()
        })
        .collect()
}

pub(super) fn apply_resolved_vertical_advances(
    text: &str,
    source_range: UiTextRange,
    resolved_advances: &[f32],
    glyphs: &mut [ScreenSpaceUiShapedGlyph],
) {
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| {
            (
                UiTextRange {
                    start: source_range.start + start,
                    end: source_range.start + start + grapheme.len(),
                },
                grapheme,
            )
        })
        .collect::<Vec<_>>();
    if resolved_advances.len() != graphemes.len()
        || !resolved_advances.iter().any(|advance| *advance > 0.0)
    {
        return;
    }

    let mut previous_range = None;
    for glyph in glyphs {
        if previous_range == Some(glyph.source_range) {
            glyph.advance = 0.0;
            continue;
        }
        previous_range = Some(glyph.source_range);
        let advance = graphemes
            .iter()
            .zip(resolved_advances)
            .filter(|((range, _), _)| ranges_overlap(*range, glyph.source_range))
            .map(|(_, advance)| sanitized_advance(*advance))
            .sum::<f32>();
        if advance > 0.0 {
            glyph.advance = advance;
        }
    }
}

fn source_scalar_for_range(
    text: &str,
    source_range: UiTextRange,
    glyph_range: UiTextRange,
) -> Option<char> {
    let start = glyph_range.start.checked_sub(source_range.start)?;
    let end = glyph_range.end.checked_sub(source_range.start)?;
    text.get(start..end)?.chars().next()
}

fn ranges_overlap(lhs: UiTextRange, rhs: UiTextRange) -> bool {
    lhs.start < rhs.end && rhs.start < lhs.end
}

fn sanitized_advance(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn sanitized_position(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use crate::graphics::text::font::shared_font_database_snapshot;

    #[test]
    fn text_vertical_renderer_projects_backend_advances_by_source_grapheme() {
        let text = "布局。";
        let style = UiResolvedStyle {
            font_family: Some("Microsoft YaHei UI".to_string()),
            language: Some("zh-Hans".to_string()),
            font_size: 30.0,
            line_height: 38.0,
            ..UiResolvedStyle::default()
        };

        let source_range = UiTextRange {
            start: 0,
            end: text.len(),
        };
        let glyphs =
            resolved_vertical_text_glyphs(text, &style, UiTextDirection::LeftToRight, source_range);
        let advances = vertical_advances_by_source_grapheme(text, source_range, &glyphs);

        assert_eq!(advances.len(), 3);
        assert!(advances.iter().all(|advance| *advance > 0.0));
        assert!(glyphs.iter().all(|glyph| glyph.font_id.is_some()));
        assert!(glyphs.iter().all(|glyph| glyph.glyph_id > 0));
        let punctuation = glyphs
            .iter()
            .find(|glyph| glyph.source_scalar == '。')
            .expect("vertical punctuation glyph");
        let (_, font_database) = shared_font_database_snapshot();
        let face = punctuation.font_id.expect("punctuation backend face");
        let bytes = font_database
            .face_bytes(face)
            .expect("punctuation face bytes");
        let face_index = font_database
            .face_index(face)
            .expect("punctuation face index");
        let parsed =
            ttf_parser::Face::parse(bytes.as_ref(), face_index).expect("punctuation OpenType face");
        let scalar_glyph_id = parsed.glyph_index('。').expect("punctuation cmap glyph").0 as u32;
        assert_ne!(
            punctuation.glyph_id, scalar_glyph_id,
            "TTB shaping must select the face's vertical punctuation glyph"
        );
    }

    #[test]
    fn text_horizontal_renderer_preserves_face_and_instance_identity() {
        let text = "Variable text";
        let style = UiResolvedStyle {
            font_family: Some("Segoe UI".to_string()),
            language: Some("en".to_string()),
            font_size: 24.0,
            line_height: 30.0,
            ..UiResolvedStyle::default()
        };
        let source_range = UiTextRange {
            start: 0,
            end: text.len(),
        };

        let glyphs = resolved_horizontal_text_glyphs(
            text,
            &style,
            UiTextDirection::LeftToRight,
            source_range,
        );

        assert!(!glyphs.is_empty());
        assert!(glyphs.iter().all(|glyph| glyph.font_id.is_some()));
        assert!(glyphs.iter().all(|glyph| glyph.font_instance_id.is_some()));
    }
}
