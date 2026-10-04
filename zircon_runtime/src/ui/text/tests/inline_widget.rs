use crate::text::{RichTextFormat, RichTextParser};

use super::*;

#[test]
fn duplicate_widget_ids_publish_one_invalid_binding() {
    let compiled = RichTextParser::default()
        .compile("[widget=7|12x10][widget=7|12x10]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");

    let directory =
        inline_widget_layout_from_compiled(&compiled, None).expect("widget binding directory");

    assert_eq!(directory.bindings().len(), 1);
    assert_eq!(directory.bindings()[0].slot, RichInlineWidgetSlotId::new(7));
    assert!(!directory.bindings()[0].valid);
    assert_eq!(directory.bindings()[0].frame, None);
}

#[test]
fn omitted_widget_keeps_a_valid_binding_without_visible_geometry() {
    let compiled = RichTextParser::default()
        .compile("[widget=7|12x10]", RichTextFormat::BbCodeV1)
        .expect("test rich source fits parser budgets");

    let directory =
        inline_widget_layout_from_compiled(&compiled, Some(&UiResolvedTextLayout::default()))
            .expect("omitted widget binding directory");

    assert_eq!(directory.bindings().len(), 1);
    assert!(directory.bindings()[0].valid);
    assert_eq!(directory.bindings()[0].frame, None);
}

#[test]
fn visual_start_accumulation_recovers_from_finite_advance_overflow() {
    let mut graphemes = "ab".grapheme_indices(true).peekable();
    let mut advance_index = 0;
    let mut main_offset = FiniteGeometryAccumulator::default();

    let offset = advance_to_visual_start(
        &mut graphemes,
        &[f32::MAX, f32::MAX],
        &mut advance_index,
        &mut main_offset,
        2,
        2,
    )
    .expect("the text end is a visual boundary");

    assert_eq!(offset, f32::MAX);
    assert!(offset.is_finite());
}

#[test]
fn visual_start_rejects_non_finite_advance_geometry() {
    let mut graphemes = "ab".grapheme_indices(true).peekable();
    let mut advance_index = 0;
    let mut main_offset = FiniteGeometryAccumulator::default();

    assert_eq!(
        advance_to_visual_start(
            &mut graphemes,
            &[f32::INFINITY, 1.0],
            &mut advance_index,
            &mut main_offset,
            1,
            2,
        ),
        None
    );
}

#[test]
fn widget_frames_publish_finite_coordinates_for_extreme_finite_inputs() {
    for writing_mode in [
        UiTextWritingMode::HorizontalTb,
        UiTextWritingMode::VerticalRl,
    ] {
        let frame = widget_frame(
            UiFrame::new(f32::MAX, f32::MAX, f32::MAX, f32::MAX),
            f32::MAX,
            f32::MAX,
            crate::core::math::Vec2::new(1.0, f32::MAX),
            writing_mode,
        );

        assert!(frame.x.is_finite());
        assert!(frame.y.is_finite());
        assert!(frame.width.is_finite());
        assert!(frame.height.is_finite());
    }
}
