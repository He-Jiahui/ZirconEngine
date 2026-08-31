use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiRichTextFormat, UiTextOverflow, UiTextWrap, UiTextWritingMode,
    },
};

use super::{layout_text, test_style};

fn assert_split_i_graphemes(layout: &UiResolvedTextLayout) {
    let line_text = layout
        .lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(line_text, ["i", "i"]);
}

#[test]
fn text_layout_wraps_at_a_narrow_horizontal_frame_width() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, 0.25, 48.0), None);

    assert_split_i_graphemes(&layout);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.placement_frame.width <= 0.25)
    );
    assert!(
        layout
            .lines
            .iter()
            .all(|line| (line.frame.width - line.measured_width).abs() <= 0.01)
    );
}

#[test]
fn text_layout_keeps_an_unbounded_horizontal_frame_on_one_line() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    let layout = layout_text(
        "ii",
        &style,
        UiFrame::new(0.0, 0.0, f32::INFINITY, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "ii");
}

#[test]
fn render_clip_filters_paint_lines_without_changing_intrinsic_layout_size() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let frame = UiFrame::new(0.0, 0.0, 160.0, 48.0);
    let complete = layout_text("i\nMMMM", &style, frame, None);
    let clipped = layout_text(
        "i\nMMMM",
        &style,
        frame,
        Some(UiFrame::new(0.0, 0.0, frame.width, complete.line_height)),
    );

    assert_eq!(complete.lines.len(), 2);
    assert_eq!(clipped.lines.len(), 1);
    assert_eq!(clipped.lines[0].text, "i");
    assert!(clipped.overflow_clipped);
    assert!(
        (clipped.measured_width - complete.measured_width).abs() < 0.01,
        "render clipping must not change intrinsic width"
    );
    assert!(
        (clipped.measured_height - complete.measured_height).abs() < 0.01,
        "render clipping must not change intrinsic height"
    );
}

#[test]
fn rich_render_clip_does_not_change_intrinsic_layout_size() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let frame = UiFrame::new(0.0, 0.0, 160.0, 48.0);
    let complete = layout_text("[b]i[/b]\n[b]MMMM[/b]", &style, frame, None);
    let first_line = complete
        .lines
        .first()
        .expect("rich fixture must resolve a first line");
    let clipped = layout_text(
        "[b]i[/b]\n[b]MMMM[/b]",
        &style,
        frame,
        Some(UiFrame::new(
            frame.x,
            first_line.frame.y,
            frame.width,
            first_line.frame.height,
        )),
    );

    assert!(complete.lines.len() >= 2);
    assert_eq!(clipped.lines.len(), 1);
    assert!(
        (clipped.measured_width - complete.measured_width).abs() < 0.01,
        "render clipping must not change rich intrinsic width"
    );
    assert!(
        (clipped.measured_height - complete.measured_height).abs() < 0.01,
        "render clipping must not change rich intrinsic height"
    );
}

#[test]
fn vertical_render_clip_does_not_change_intrinsic_layout_size() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 0.0, 80.0, 80.0);
    let complete = layout_text("i\nMMMM", &style, frame, None);
    let first_column = complete
        .lines
        .first()
        .expect("vertical fixture must resolve a first column");
    let clipped = layout_text("i\nMMMM", &style, frame, Some(first_column.frame));

    assert_eq!(complete.lines.len(), 2);
    assert_eq!(clipped.lines.len(), 1);
    assert!(
        (clipped.measured_width - complete.measured_width).abs() < 0.01,
        "render clipping must not change vertical intrinsic width"
    );
    assert!(
        (clipped.measured_height - complete.measured_height).abs() < 0.01,
        "render clipping must not change vertical intrinsic height"
    );
}

#[test]
fn text_layout_fails_closed_for_invalid_horizontal_wrap_extents() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    for width in [-1.0, f32::NAN] {
        let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, width, 48.0), None);

        assert!(layout.lines.is_empty());
        assert!(layout.overflow_clipped);
    }
}

#[test]
fn text_layout_wraps_vertical_columns_at_a_narrow_frame_height() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let layout = layout_text("ii", &style, UiFrame::new(0.0, 0.0, 48.0, 0.25), None);

    assert_split_i_graphemes(&layout);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.placement_frame.height <= 0.25)
    );
    assert!(
        layout
            .lines
            .iter()
            .all(|line| (line.frame.height - line.measured_width).abs() <= 0.01)
    );
}

#[test]
fn text_block_layout_preserves_a_narrow_horizontal_extent_after_indent() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[p indent=12]ii[/p]",
        &style,
        UiFrame::new(0.0, 0.0, 0.25, 48.0),
        None,
    );

    assert_split_i_graphemes(&layout);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.placement_frame.width <= 0.25)
    );
}

#[test]
fn text_block_layout_preserves_a_narrow_vertical_extent_after_indent() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let layout = layout_text(
        "[p indent=12]ii[/p]",
        &style,
        UiFrame::new(0.0, 0.0, 48.0, 0.25),
        None,
    );

    assert_split_i_graphemes(&layout);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.placement_frame.height <= 0.25)
    );
}
