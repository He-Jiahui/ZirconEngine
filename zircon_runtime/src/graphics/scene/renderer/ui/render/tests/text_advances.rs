use super::*;
use crate::text::font::shared_font_database_snapshot;

#[test]
fn vertical_advance_projection_uses_indexed_source_ranges() {
    let source = include_str!("../text_advances.rs");
    let indexed_range_api = ["partition", "_point"].concat();
    let nested_glyph_scan = [".filter", "(|glyph|"].concat();

    assert!(source.contains(&indexed_range_api));
    assert!(!source.contains(&nested_glyph_scan));
}

#[test]
fn renderer_fallback_module_cannot_rebuild_text_owned_artifacts() {
    let source = include_str!("../text_advances.rs");
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

    let glyphs =
        resolved_horizontal_text_glyphs(text, &style, UiTextDirection::LeftToRight, source_range)
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
            writing_mode: zircon_runtime_interface::ui::surface::UiTextWritingMode::HorizontalTb,
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
