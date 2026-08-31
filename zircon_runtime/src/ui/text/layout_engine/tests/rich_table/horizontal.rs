use super::*;

#[test]
fn text_rich_bbcode_table_places_cells_in_shared_columns_and_rows() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=2][cell]Name[/cell][cell]Value[/cell][cell]alpha[/cell][cell]beta[/cell][/table]",
        &style,
        UiFrame::new(10.0, 20.0, 220.0, 160.0),
        None,
    );

    assert_eq!(layout.lines.len(), 4);
    assert_eq!(layout.lines[0].frame.x, layout.lines[2].frame.x);
    assert_eq!(layout.lines[1].frame.x, layout.lines[3].frame.x);
    assert_eq!(layout.lines[0].frame.y, layout.lines[1].frame.y);
    assert_eq!(layout.lines[2].frame.y, layout.lines[3].frame.y);
    assert!(layout.lines[1].frame.x > layout.lines[0].frame.x);
    assert!(layout.lines[2].frame.y > layout.lines[0].frame.y);
}

#[test]
fn text_rich_bbcode_table_wraps_each_cell_inside_its_column() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let frame = UiFrame::new(0.0, 0.0, 120.0, 220.0);
    let layout = layout_text(
        "[table=2][cell]alpha beta gamma delta[/cell][cell]short[/cell][/table]",
        &style,
        frame,
        None,
    );

    assert!(layout.lines.len() >= 3);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| line.frame.x >= frame.x && line.frame.right() <= frame.right() + 0.01)
    );
}

#[test]
fn text_rich_bbcode_table_row_height_tracks_tallest_cell() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=2][cell]alpha beta gamma delta epsilon[/cell][cell]one[/cell][cell]next[/cell][cell]row[/cell][/table]",
        &style,
        UiFrame::new(0.0, 0.0, 130.0, 260.0),
        None,
    );

    let next_row = layout
        .lines
        .iter()
        .find(|line| line.text == "next")
        .expect("second table row");
    let first_row_start = layout
        .lines
        .iter()
        .map(|line| line.frame.y)
        .fold(f32::INFINITY, f32::min);
    assert!(next_row.frame.y >= first_row_start + layout.line_height * 2.0);
}

#[test]
fn text_rich_bbcode_table_preserves_surrounding_block_order() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "before[table=2][cell]A[/cell][cell]B[/cell][/table]after",
        &style,
        UiFrame::new(0.0, 0.0, 200.0, 160.0),
        None,
    );

    let before_y = layout
        .lines
        .iter()
        .find(|line| line.text == "before")
        .unwrap()
        .frame
        .y;
    let table_y = layout
        .lines
        .iter()
        .find(|line| line.text == "A")
        .unwrap()
        .frame
        .y;
    let after_y = layout
        .lines
        .iter()
        .find(|line| line.text == "after")
        .unwrap()
        .frame
        .y;
    assert!(before_y < table_y);
    assert!(table_y < after_y);
}

#[test]
fn text_rich_bbcode_table_colspan_uses_the_combined_track_width() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=3][cell colspan=3]merged heading across tracks[/cell][cell]A[/cell][cell]B[/cell][cell]C[/cell][/table]",
        &style,
        UiFrame::new(0.0, 0.0, 300.0, 180.0),
        None,
    );

    assert!(
        layout
            .lines
            .iter()
            .any(|line| line.text == "merged heading across tracks")
    );
    let heading = find_line(&layout, "merged heading");
    let first = find_line(&layout, "A");
    let second = find_line(&layout, "B");
    assert_eq!(heading.frame.x, first.frame.x);
    assert!(second.frame.x > first.frame.x);
    assert!(first.frame.y > heading.frame.y);
}

#[test]
fn text_rich_bbcode_table_rowspan_reserves_its_column_on_following_rows() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=2][cell rowspan=2]span[/cell][cell]top[/cell][cell]beside[/cell][cell]after[/cell][/table]",
        &style,
        UiFrame::new(10.0, 20.0, 240.0, 240.0),
        None,
    );

    let span = find_line(&layout, "span");
    let top = find_line(&layout, "top");
    let beside = find_line(&layout, "beside");
    let after = find_line(&layout, "after");
    assert_eq!(span.frame.x, after.frame.x);
    assert_eq!(top.frame.x, beside.frame.x);
    assert_eq!(span.frame.y, top.frame.y);
    assert!(beside.frame.y > top.frame.y);
    assert!(after.frame.y > beside.frame.y);
}

#[test]
fn text_rich_bbcode_table_colspan_reduces_before_a_rowspan_collision() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=3][cell]A0[/cell][cell rowspan=2]B0[/cell][cell]C0[/cell][cell colspan=2]D1[/cell][cell]E1[/cell][/table]",
        &style,
        UiFrame::new(0.0, 0.0, 300.0, 220.0),
        None,
    );

    let a = find_line(&layout, "A0");
    let b = find_line(&layout, "B0");
    let c = find_line(&layout, "C0");
    let d = find_line(&layout, "D1");
    let e = find_line(&layout, "E1");
    assert_eq!(a.frame.x, d.frame.x);
    assert_eq!(c.frame.x, e.frame.x);
    assert_eq!(a.frame.y, b.frame.y);
    assert_eq!(b.frame.y, c.frame.y);
    assert_eq!(d.frame.y, e.frame.y);
    assert!(d.frame.y > a.frame.y);
}

#[test]
fn text_rich_bbcode_table_rowspan_height_keeps_the_next_row_below_its_content() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=2][cell rowspan=2]alpha beta gamma delta epsilon zeta eta theta[/cell][cell]top[/cell][cell]beside[/cell][cell]after[/cell][/table]",
        &style,
        UiFrame::new(0.0, 0.0, 150.0, 360.0),
        None,
    );

    let first_span_line = find_line(&layout, "alpha");
    let after = find_line(&layout, "after");
    let span_bottom = layout
        .lines
        .iter()
        .filter(|line| {
            (line.frame.x - first_span_line.frame.x).abs() <= 0.01 && line.frame.y < after.frame.y
        })
        .map(|line| line.frame.bottom())
        .fold(first_span_line.frame.bottom(), f32::max);
    assert!(after.frame.y >= span_bottom);
}

#[test]
fn text_rich_bbcode_table_authored_padding_controls_content_origin_and_measure() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=1][cell bg=#12202C padding=18,9,24,11]padded[/cell][/table]",
        &style,
        UiFrame::new(10.0, 20.0, 220.0, 160.0),
        None,
    );

    let cell_box = layout.boxes.first().expect("resolved cell box");
    let line = find_line(&layout, "padded");
    assert!((line.frame.x - cell_box.frame.x - 18.0).abs() < 0.01);
    assert!((line.frame.y - cell_box.frame.y - 9.0).abs() < 0.01);
    assert!(cell_box.frame.width >= line.measured_width + 18.0 + 24.0);
    assert!(cell_box.frame.height >= line.frame.height + 9.0 + 11.0);
}

#[test]
fn text_rich_bbcode_table_span_box_uses_final_combined_tracks_and_rows() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    let layout = layout_text(
        "[table=3][cell colspan=2 rowspan=2 border=#73D7FF bg=#12202C,#182F3D padding=8,4,12,6]span[/cell][cell]right[/cell][cell]below[/cell][cell]tail[/cell][/table]",
        &style,
        UiFrame::new(10.0, 20.0, 300.0, 220.0),
        None,
    );

    assert_eq!(layout.boxes.len(), 1);
    let cell_box = &layout.boxes[0];
    let span = find_line(&layout, "span");
    let right = find_line(&layout, "right");
    let below = find_line(&layout, "below");
    let tail = find_line(&layout, "tail");
    assert_eq!(cell_box.range, span.source_range);
    assert!(cell_box.frame.x <= span.frame.x);
    assert!(cell_box.frame.right() < right.frame.x);
    assert_eq!(right.frame.x, below.frame.x);
    assert!(cell_box.frame.right() < below.frame.x);
    assert!(cell_box.frame.bottom() <= tail.frame.y + 0.01);
    assert!(cell_box.background_color.is_some());
    assert!(cell_box.border_color.is_some());
    assert_eq!(cell_box.border_width, 1.0);
}
