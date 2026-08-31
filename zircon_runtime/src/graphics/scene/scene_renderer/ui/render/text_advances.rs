use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use crate::core::framework::text::{
    TextFontFaceHandle, TextFontRequest, TextGlyphRotation, TextLayoutError, TextRenderMode,
    TextShapeRequest, TextShapeResult,
};
use crate::text::font::FontCollectionService;
use crate::text::{ShapedGlyphRotation, shape_text_request_in_font_collection};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiShapedGlyph {
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_id: u32,
    pub(in crate::graphics::scene::scene_renderer::ui) font_id: Option<TextFontFaceHandle>,
    pub(in crate::graphics::scene::scene_renderer::ui) font_instance_id: Option<TextFontFaceHandle>,
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
    pub(super) layout_baseline: Option<f32>,
    pub(super) layout_error: Option<TextLayoutError>,
}

pub(super) fn resolve_screen_space_text_glyphs(
    request: ScreenSpaceTextShapingRequest<'_>,
    glyph_advances: Vec<f32>,
    font_collection: &Arc<FontCollectionService>,
) -> ResolvedScreenSpaceTextGlyphs {
    crate::profile_scope!("runtime", "text.render", "shape_renderer_fallback");
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
    let writing_mode = if matches!(request.writing_mode, UiTextWritingMode::VerticalRl) {
        crate::core::framework::text::TextWritingMode::VerticalRightToLeft
    } else {
        crate::core::framework::text::TextWritingMode::HorizontalTopToBottom
    };
    let shaped = shape_through_canonical_service(
        request.text,
        &style,
        request.direction,
        writing_mode,
        font_collection,
    );
    let (mut shaped_glyphs, layout_baseline, layout_error) = match shaped {
        Ok(shaped) => {
            let layout_baseline = shaped
                .metrics
                .baseline
                .is_finite()
                .then_some(shaped.metrics.baseline);
            (
                shaped_glyphs_for_screen_space(request.text, request.source_range, shaped),
                layout_baseline,
                None,
            )
        }
        Err(error) => (Vec::new(), None, Some(error)),
    };
    let glyph_advances = if matches!(request.writing_mode, UiTextWritingMode::VerticalRl) {
        let grapheme_ranges = source_grapheme_ranges(request.text, request.source_range);
        let resolved_advances = if glyph_advances.is_empty() {
            vertical_advances_for_source_ranges(&grapheme_ranges, &shaped_glyphs)
        } else {
            glyph_advances
        };
        apply_vertical_advances_for_source_ranges(
            &grapheme_ranges,
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
        layout_baseline,
        layout_error,
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn refresh_renderer_fallback_text_batch_glyphs(
    text: &mut super::ScreenSpaceUiTextBatch,
    font_collection: &Arc<FontCollectionService>,
) {
    if text.glyph_artifact_line.is_some() || text.preserve_shaped_glyphs {
        return;
    }
    let source_range = text.source_range.unwrap_or(UiTextRange {
        start: 0,
        end: text.text.len(),
    });
    // A resolved layout line owns its advances and its frame was computed from them. A raw
    // render-command batch has no source range, so any vertical advances it carries were derived
    // by this module before the project font became available and must be recomputed.
    let glyph_advances = if text.source_range.is_some() {
        std::mem::take(&mut text.glyph_advances)
    } else {
        text.glyph_advances.clear();
        Vec::new()
    };
    let resolved = resolve_screen_space_text_glyphs(
        ScreenSpaceTextShapingRequest {
            text: text.text.as_str(),
            font: text.font.as_deref(),
            font_family: text.font_family.as_deref(),
            language: text.language.as_deref(),
            font_weight: text.font_weight,
            font_size: text.font_size,
            line_height: text.line_height,
            direction: text.text_direction,
            writing_mode: text.writing_mode,
            source_range,
        },
        glyph_advances,
        font_collection,
    );
    text.glyph_advances = resolved.glyph_advances;
    text.shaped_glyphs = resolved.shaped_glyphs;
    if text.text_decoration_baseline.is_none() {
        text.text_decoration_baseline = resolved
            .layout_baseline
            .map(|baseline| text.frame.y + baseline);
    }
    text.layout_error = resolved.layout_error;
}

#[cfg(test)]
pub(super) fn resolved_vertical_text_glyphs(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> Result<Vec<ScreenSpaceUiShapedGlyph>, TextLayoutError> {
    let shaped = shape_through_canonical_service(
        text,
        style,
        direction,
        crate::core::framework::text::TextWritingMode::VerticalRightToLeft,
        &crate::text::font::shared_font_collection_service(),
    )?;
    Ok(shaped_glyphs_for_screen_space(text, source_range, shaped))
}

#[cfg(test)]
pub(super) fn resolved_horizontal_text_glyphs(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> Result<Vec<ScreenSpaceUiShapedGlyph>, TextLayoutError> {
    let shaped = shape_through_canonical_service(
        text,
        style,
        direction,
        crate::core::framework::text::TextWritingMode::HorizontalTopToBottom,
        &crate::text::font::shared_font_collection_service(),
    )?;
    Ok(shaped_glyphs_for_screen_space(text, source_range, shaped))
}

fn shaped_glyphs_for_screen_space(
    text: &str,
    source_range: UiTextRange,
    shaped: TextShapeResult,
) -> Vec<ScreenSpaceUiShapedGlyph> {
    shaped
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .filter_map(|glyph| {
            let glyph_source_range = UiTextRange {
                start: source_range.start + glyph.source_range.start,
                end: source_range.start + glyph.source_range.end,
            };
            let source_scalar = source_scalar_for_range(text, source_range, glyph_source_range)?;
            Some(ScreenSpaceUiShapedGlyph {
                glyph_id: glyph.glyph_id,
                font_id: glyph.font_face,
                font_instance_id: glyph.font_instance,
                source_scalar,
                source_range: glyph_source_range,
                advance: sanitized_advance(glyph.advance),
                offset_x: sanitized_position(glyph.offset[0]),
                offset_y: sanitized_position(glyph.offset[1]),
                rotation: match glyph.rotation {
                    TextGlyphRotation::None => ShapedGlyphRotation::None,
                    TextGlyphRotation::Clockwise90 => ShapedGlyphRotation::Cw90,
                },
                requires_atlas_slot: glyph.requires_rasterization && !source_scalar.is_whitespace(),
            })
        })
        .collect()
}

fn shape_through_canonical_service(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    writing_mode: crate::core::framework::text::TextWritingMode,
    font_collection: &Arc<FontCollectionService>,
) -> Result<TextShapeResult, TextLayoutError> {
    let family_storage = style.font_family.as_deref().map(|family| [family]);
    let families = family_storage
        .as_ref()
        .map_or(&[][..], |families| &families[..]);
    let font = TextFontRequest {
        families,
        asset: style.font.as_deref(),
        size: style.font_size,
        weight: style.font_weight,
        stretch: 100,
        italic: false,
        render_mode: TextRenderMode::Auto,
    };
    let mut request = TextShapeRequest::new(text, font);
    request.language = style.language.as_deref();
    request.direction = direction.into();
    request.writing_mode = writing_mode;
    request.line_height = style.line_height;
    request.tab_size = style.tab_size;
    shape_text_request_in_font_collection(request, font_collection)
}

pub(super) fn vertical_advances_by_source_grapheme(
    text: &str,
    source_range: UiTextRange,
    glyphs: &[ScreenSpaceUiShapedGlyph],
) -> Vec<f32> {
    let grapheme_ranges = source_grapheme_ranges(text, source_range);
    vertical_advances_for_source_ranges(&grapheme_ranges, glyphs)
}

fn source_grapheme_ranges(text: &str, source_range: UiTextRange) -> Vec<UiTextRange> {
    text.grapheme_indices(true)
        .map(|(start, grapheme)| UiTextRange {
            start: source_range.start + start,
            end: source_range.start + start + grapheme.len(),
        })
        .collect()
}

fn vertical_advances_for_source_ranges(
    grapheme_ranges: &[UiTextRange],
    glyphs: &[ScreenSpaceUiShapedGlyph],
) -> Vec<f32> {
    let mut advances = vec![0.0; grapheme_ranges.len()];
    for glyph in glyphs {
        let index = grapheme_ranges.partition_point(|range| range.end < glyph.source_range.end);
        let Some(range) = grapheme_ranges.get(index) else {
            continue;
        };
        if glyph.source_range.end > range.start {
            advances[index] += glyph.advance;
        }
    }
    advances
}

pub(super) fn apply_resolved_vertical_advances(
    text: &str,
    source_range: UiTextRange,
    resolved_advances: &[f32],
    glyphs: &mut [ScreenSpaceUiShapedGlyph],
) {
    let grapheme_ranges = source_grapheme_ranges(text, source_range);
    apply_vertical_advances_for_source_ranges(&grapheme_ranges, resolved_advances, glyphs);
}

fn apply_vertical_advances_for_source_ranges(
    grapheme_ranges: &[UiTextRange],
    resolved_advances: &[f32],
    glyphs: &mut [ScreenSpaceUiShapedGlyph],
) {
    if resolved_advances.len() != grapheme_ranges.len()
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
        let first = grapheme_ranges.partition_point(|range| range.end <= glyph.source_range.start);
        let mut advance = 0.0;
        for (range, resolved_advance) in grapheme_ranges[first..]
            .iter()
            .zip(&resolved_advances[first..])
        {
            if range.start >= glyph.source_range.end {
                break;
            }
            advance += sanitized_advance(*resolved_advance);
        }
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

fn sanitized_advance(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn sanitized_position(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use crate::text::font::shared_font_database_snapshot;

    #[test]
    fn vertical_advance_projection_uses_indexed_source_ranges() {
        let source = include_str!("text_advances.rs");
        let indexed_range_api = ["partition", "_point"].concat();
        let nested_glyph_scan = [".filter", "(|glyph|"].concat();

        assert!(source.contains(&indexed_range_api));
        assert!(!source.contains(&nested_glyph_scan));
    }

    #[test]
    fn renderer_fallback_module_cannot_rebuild_text_owned_artifacts() {
        let source = include_str!("text_advances.rs");
        let rebuild_api = ["rebuild_resolved_text_glyph_", "artifact_line"].concat();
        let session_constructor = ["SharedTextLayoutSession", "::new"].concat();
        let refresh_overlay = ["refreshed", "_line"].concat();

        assert!(!source.contains(&rebuild_api));
        assert!(!source.contains(&session_constructor));
        assert!(!source.contains(&refresh_overlay));
    }

    #[test]
    fn vertical_advance_projection_preserves_visual_order_and_spanning_clusters() {
        let text = "ab";
        let source_range = UiTextRange { start: 0, end: 2 };
        let mut glyphs = vec![
            test_glyph(UiTextRange { start: 1, end: 2 }, 3.0),
            test_glyph(UiTextRange { start: 0, end: 1 }, 2.0),
            test_glyph(UiTextRange { start: 0, end: 2 }, 5.0),
        ];

        assert_eq!(
            vertical_advances_by_source_grapheme(text, source_range, &glyphs),
            vec![2.0, 8.0]
        );

        apply_resolved_vertical_advances(text, source_range, &[10.0, 20.0], &mut glyphs);
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.advance).collect::<Vec<_>>(),
            vec![20.0, 10.0, 30.0]
        );
    }

    fn test_glyph(source_range: UiTextRange, advance: f32) -> ScreenSpaceUiShapedGlyph {
        ScreenSpaceUiShapedGlyph {
            glyph_id: 1,
            font_id: None,
            font_instance_id: None,
            source_scalar: 'a',
            source_range,
            advance,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation: ShapedGlyphRotation::None,
            requires_atlas_slot: false,
        }
    }

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
            resolved_vertical_text_glyphs(text, &style, UiTextDirection::LeftToRight, source_range)
                .expect("vertical canonical shaping");
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
        let face = punctuation
            .font_id
            .and_then(crate::text::font::resolve_font_face_handle)
            .expect("punctuation Text handle should resolve to a backend face");
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
        )
        .expect("horizontal canonical shaping");

        assert!(!glyphs.is_empty());
        assert!(glyphs.iter().all(|glyph| glyph.font_id.is_some()));
        assert!(glyphs.iter().all(|glyph| glyph.font_instance_id.is_some()));
    }

    #[test]
    fn renderer_records_canonical_layout_error_in_resolved_batch_contract() {
        let resolved = resolve_screen_space_text_glyphs(
            ScreenSpaceTextShapingRequest {
                text: "invalid",
                font: None,
                font_family: None,
                language: None,
                font_weight: 400,
                font_size: 0.0,
                line_height: 0.0,
                direction: UiTextDirection::LeftToRight,
                writing_mode:
                    zircon_runtime_interface::ui::surface::UiTextWritingMode::HorizontalTb,
                source_range: UiTextRange { start: 0, end: 7 },
            },
            Vec::new(),
            &crate::text::font::shared_font_collection_service(),
        );

        assert_eq!(
            resolved.layout_error,
            Some(TextLayoutError::InvalidFontSize)
        );
        assert!(resolved.shaped_glyphs.is_empty());
    }
}
