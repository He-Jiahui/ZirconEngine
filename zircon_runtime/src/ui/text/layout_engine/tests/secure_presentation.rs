use super::{apply_secure_text_presentation, frame_with_projected_direction};
use crate::{core::framework::text::TextDirection, ui::text::UiSecureTextPresentation};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRange, UiTextWrap,
    },
};

#[test]
fn secure_direction_projection_keeps_extreme_logical_edges_finite() {
    let cases = [
        (
            UiFrame::new(f32::MAX, 4.0, f32::MAX, 12.0),
            UiTextAlign::Start,
            UiTextDirection::RightToLeft,
            UiTextDirection::LeftToRight,
            f32::MAX,
        ),
        (
            UiFrame::new(-f32::MAX, 4.0, f32::MAX, 12.0),
            UiTextAlign::Start,
            UiTextDirection::LeftToRight,
            UiTextDirection::RightToLeft,
            -f32::MAX,
        ),
        (
            UiFrame::new(f32::MAX, 4.0, f32::MAX, 12.0),
            UiTextAlign::End,
            UiTextDirection::LeftToRight,
            UiTextDirection::RightToLeft,
            f32::MAX,
        ),
        (
            UiFrame::new(-f32::MAX, 4.0, f32::MAX, 12.0),
            UiTextAlign::End,
            UiTextDirection::RightToLeft,
            UiTextDirection::LeftToRight,
            -f32::MAX,
        ),
    ];

    for (frame, align, generic_direction, projected_direction, expected_x) in cases {
        let projected =
            frame_with_projected_direction(frame, align, generic_direction, projected_direction);

        assert_eq!(projected.x, expected_x);
        assert!(projected.x.is_finite());
    }
}

#[test]
fn wrapped_rtl_secure_rows_replay_each_rows_source_owned_bidi_order() {
    let source = "\u{05d0}\u{05d1}\u{05d2}\u{05d3}\u{05d4}\u{05d5}\u{05d6}\u{05d7}";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
        .expect("a valid RTL source must produce a secure presentation");
    let style = UiResolvedStyle {
        wrap: UiTextWrap::Glyph,
        text_overflow: UiTextOverflow::Clip,
        font_size: 18.0,
        line_height: 22.0,
        ..UiResolvedStyle::default()
    };
    let unwrapped_style = UiResolvedStyle {
        wrap: UiTextWrap::None,
        ..style.clone()
    };
    let unwrapped = super::super::layout_text(
        presentation.display_text(),
        &unwrapped_style,
        UiFrame::new(0.0, 0.0, f32::INFINITY, 64.0),
        None,
    );
    let mut layout = super::super::layout_text(
        presentation.display_text(),
        &style,
        UiFrame::new(0.0, 0.0, (unwrapped.measured_width * 0.6).max(1.0), 256.0),
        None,
    );
    let physical_display_ranges = layout
        .lines
        .iter()
        .map(|line| line.source_range)
        .collect::<Vec<UiTextRange>>();

    assert!(
        physical_display_ranges.len() > 1,
        "the measured mask must soft-wrap before projection"
    );
    apply_secure_text_presentation(&mut layout, &presentation)
        .expect("each wrapped row must map through its own source signature");

    for (line, display_range) in layout.lines.iter().zip(physical_display_ranges) {
        let clusters = presentation
            .clusters_for_display_range(display_range)
            .expect("a physical row must contain complete mask graphemes");
        let bidi = presentation
            .bidi_for_display_range(display_range)
            .expect("source-owned bidi replay must remain valid")
            .expect("a non-empty physical row must have bidi metadata");
        let expected_ranges = bidi
            .visual_indices
            .iter()
            .map(|&index| clusters[index].source_range)
            .collect::<Vec<_>>();

        assert_eq!(line.direction, bidi.resolved_base_direction.into());
        assert_eq!(
            line.runs
                .iter()
                .map(|run| run.source_range)
                .collect::<Vec<_>>(),
            expected_ranges,
            "a wrapped row must not reuse the full hard-line visual order"
        );
        assert_eq!(
            line.source_range,
            UiTextRange {
                start: expected_ranges
                    .iter()
                    .map(|range| range.start)
                    .min()
                    .expect("a physical row has source ranges"),
                end: expected_ranges
                    .iter()
                    .map(|range| range.end)
                    .max()
                    .expect("a physical row has source ranges"),
            }
        );
    }
}
