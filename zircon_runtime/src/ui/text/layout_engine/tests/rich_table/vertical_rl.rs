use super::*;

const FRAME_EPSILON: f32 = 0.01;

#[test]
fn text_rich_bbcode_vertical_table_maps_columns_down_and_rows_left() {
    let layout = vertical_table(
        "[table=2][cell]甲[/cell][cell]乙[/cell][cell]丙[/cell][cell]丁[/cell][/table]",
        UiFrame::new(10.0, 20.0, 300.0, 240.0),
    );

    let a = find_line(&layout, "甲");
    let b = find_line(&layout, "乙");
    let c = find_line(&layout, "丙");
    let d = find_line(&layout, "丁");
    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_close(a.frame.x, b.frame.x, "first physical row x");
    assert_close(c.frame.x, d.frame.x, "second physical row x");
    assert!(a.frame.x > c.frame.x);
    assert!(b.frame.y > a.frame.y);
    assert!(d.frame.y > c.frame.y);
}

#[test]
fn text_rich_bbcode_vertical_table_spans_expand_physical_height_and_width() {
    let layout = vertical_table(
        "[table=2][cell colspan=2 rowspan=2 border=#73D7FF bg=#12202C padding=6,8,10,12]跨行[/cell][cell]后甲[/cell][cell]后乙[/cell][/table]",
        UiFrame::new(10.0, 20.0, 320.0, 260.0),
    );

    assert_eq!(layout.boxes.len(), 1);
    let span_box = &layout.boxes[0];
    let span = find_line(&layout, "跨行");
    let after_a = find_line(&layout, "后甲");
    let after_b = find_line(&layout, "后乙");
    assert_eq!(span_box.range, span.source_range);
    assert!(span_box.frame.x > after_a.frame.x);
    assert_close(after_a.frame.x, after_b.frame.x, "post-span physical row x");
    assert!(after_b.frame.y > after_a.frame.y);
    assert!(span_box.frame.height > after_b.frame.y - after_a.frame.y);
    assert!(span_box.frame.width >= layout.line_height * 2.0 - FRAME_EPSILON);
}

#[test]
fn text_rich_bbcode_vertical_table_padding_keeps_physical_sides() {
    let layout = vertical_table(
        "[table=1][cell border=#F6B65A bg=#182F3D padding=13,17,19,23]縦余白[/cell][/table]",
        UiFrame::new(10.0, 20.0, 260.0, 260.0),
    );

    let cell_box = layout.boxes.first().expect("vertical resolved cell box");
    let line = find_line(&layout, "縦余白");
    assert_close(
        line.frame.y - cell_box.frame.y,
        17.0,
        "physical top padding",
    );
    assert_close(
        cell_box.frame.right() - line.frame.right(),
        19.0,
        "physical right padding",
    );
    assert!(cell_box.frame.width >= line.frame.width + 13.0 + 19.0 - FRAME_EPSILON);
    assert!(cell_box.frame.height >= line.frame.height + 17.0 + 23.0 - FRAME_EPSILON);
    assert!(cell_box.background_color.is_some());
    assert!(cell_box.border_color.is_some());
}

#[test]
fn text_rich_bbcode_vertical_table_preserves_surrounding_block_order() {
    let layout = vertical_table(
        "前[table=1][cell]中[/cell][/table]後",
        UiFrame::new(10.0, 20.0, 300.0, 120.0),
    );

    let before = find_line(&layout, "前");
    let table = find_line(&layout, "中");
    let after = find_line(&layout, "後");
    assert!(before.frame.x > table.frame.x);
    assert!(table.frame.x > after.frame.x);
}

fn assert_close(lhs: f32, rhs: f32, label: &str) {
    assert!(
        (lhs - rhs).abs() <= FRAME_EPSILON,
        "{label} must match: lhs={lhs}, rhs={rhs}"
    );
}
