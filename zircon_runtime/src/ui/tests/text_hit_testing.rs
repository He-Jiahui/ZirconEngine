use crate::text::layout::measured_grapheme_widths;
use crate::ui::text::text_style;
use crate::ui::text::{hit_test_text_layout, layout_text, measure_text_size};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiPoint},
    surface::{UiResolvedStyle, UiTextAlign, UiTextCaretAffinity, UiTextWrap},
};

#[test]
fn text_hit_test_uses_grapheme_midpoints() {
    let style = fixed_text_style();
    let text = "a\u{0301}b";
    let layout = layout_text(text, &style, UiFrame::new(10.0, 0.0, 80.0, 20.0), None);
    let line = &layout.lines[0];
    let widths = measured_grapheme_widths(&line.text, &text_style(&style));

    let before = hit_test_text_layout(&layout, UiPoint::new(line.frame.x + widths[0] * 0.25, 4.0));
    let after_cluster =
        hit_test_text_layout(&layout, UiPoint::new(line.frame.x + widths[0] * 0.75, 4.0));
    let after_text = hit_test_text_layout(
        &layout,
        UiPoint::new(line.frame.x + line.measured_width + 1.0, 4.0),
    );

    assert_eq!(before.line_index, Some(0));
    assert_eq!(before.source_offset, 0);
    assert_eq!(after_cluster.source_offset, "a\u{0301}".len());
    assert_eq!(after_text.source_offset, text.len());
}

#[test]
fn text_hit_test_uses_resolved_tab_advances() {
    let mut style = fixed_text_style();
    style.tab_size = 4.0;
    let text = "a\tb";
    let layout = layout_text(text, &style, UiFrame::new(10.0, 0.0, 120.0, 20.0), None);
    let line = &layout.lines[0];

    assert_eq!(line.glyph_advances.len(), 3);
    let before_tab_midpoint = hit_test_text_layout(
        &layout,
        UiPoint::new(
            line.frame.x + line.glyph_advances[0] + line.glyph_advances[1] * 0.25,
            4.0,
        ),
    );
    let after_tab_midpoint = hit_test_text_layout(
        &layout,
        UiPoint::new(
            line.frame.x + line.glyph_advances[0] + line.glyph_advances[1] * 0.75,
            4.0,
        ),
    );

    assert_eq!(before_tab_midpoint.source_offset, "a".len());
    assert_eq!(before_tab_midpoint.visual_grapheme_index, 1);
    assert_eq!(after_tab_midpoint.source_offset, "a\t".len());
    assert_eq!(after_tab_midpoint.visual_grapheme_index, 2);
}

#[test]
fn text_hit_test_mixed_bidi_maps_visual_rtl_edges_to_logical_source_affinity() {
    let style = fixed_text_style();
    let text = "abc אב";
    let layout = layout_text(text, &style, UiFrame::new(0.0, 0.0, 120.0, 20.0), None);
    let line = &layout.lines[0];
    let rtl_visual_index = 4;
    let rtl_x = line.frame.x
        + line.glyph_advances[..rtl_visual_index].iter().sum::<f32>()
        + line.glyph_advances[rtl_visual_index] * 0.25;

    let hit = hit_test_text_layout(&layout, UiPoint::new(rtl_x, 4.0));

    assert_eq!(line.text, "abc בא");
    assert_eq!(hit.visual_grapheme_index, rtl_visual_index);
    assert_eq!(hit.source_offset, text.len());
    assert_eq!(hit.affinity, UiTextCaretAffinity::Downstream);
}

#[test]
fn text_hit_test_mixed_bidi_maps_rtl_trailing_edge_to_logical_start() {
    let style = fixed_text_style();
    let text = "abc אב";
    let layout = layout_text(text, &style, UiFrame::new(0.0, 0.0, 120.0, 20.0), None);
    let line = &layout.lines[0];
    let rtl_visual_index = 4;
    let rtl_x = line.frame.x
        + line.glyph_advances[..rtl_visual_index].iter().sum::<f32>()
        + line.glyph_advances[rtl_visual_index] * 0.75;

    let hit = hit_test_text_layout(&layout, UiPoint::new(rtl_x, 4.0));

    assert_eq!(line.text, "abc בא");
    assert_eq!(hit.visual_grapheme_index, rtl_visual_index + 1);
    assert_eq!(hit.source_offset, "abc א".len());
    assert_eq!(hit.affinity, UiTextCaretAffinity::Upstream);
}

#[test]
fn text_hit_test_vertical_rl_uses_column_x_and_vertical_advances() {
    let mut style = fixed_text_style();
    style.wrap = UiTextWrap::Word;
    style.text_writing_mode = zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl;
    let frame_height = measure_text_size("縦書", &style).width + 0.1;
    let layout = layout_text(
        "縦書文",
        &style,
        UiFrame::new(0.0, 0.0, style.line_height * 3.0, frame_height),
        None,
    );
    let first_column = &layout.lines[0];
    let second_column = &layout.lines[1];

    assert!(first_column.frame.x > second_column.frame.x);
    assert_eq!(first_column.glyph_advances.len(), 2);
    let before_first_midpoint = hit_test_text_layout(
        &layout,
        UiPoint::new(
            first_column.frame.center().x,
            first_column.frame.y + first_column.glyph_advances[0] * 0.25,
        ),
    );
    let after_first_midpoint = hit_test_text_layout(
        &layout,
        UiPoint::new(
            first_column.frame.center().x,
            first_column.frame.y + first_column.glyph_advances[0] * 0.75,
        ),
    );
    let second_column_start = hit_test_text_layout(
        &layout,
        UiPoint::new(second_column.frame.center().x, second_column.frame.y + 1.0),
    );

    assert_eq!(before_first_midpoint.line_index, Some(0));
    assert_eq!(before_first_midpoint.source_offset, 0);
    assert_eq!(before_first_midpoint.visual_grapheme_index, 0);
    assert_eq!(after_first_midpoint.source_offset, "縦".len());
    assert_eq!(after_first_midpoint.visual_grapheme_index, 1);
    assert_eq!(second_column_start.line_index, Some(1));
    assert_eq!(second_column_start.source_offset, "縦書".len());
}

#[test]
fn text_hit_test_selects_nearest_line_and_clamps_x() {
    let style = fixed_text_style();
    let layout = layout_text("one\ntwo", &style, UiFrame::new(0.0, 0.0, 80.0, 40.0), None);

    let before_first = hit_test_text_layout(&layout, UiPoint::new(-20.0, -10.0));
    let second_start = hit_test_text_layout(&layout, UiPoint::new(-20.0, 13.0));
    let after_last = hit_test_text_layout(&layout, UiPoint::new(200.0, 80.0));

    assert_eq!(before_first.line_index, Some(0));
    assert_eq!(before_first.source_offset, 0);
    assert_eq!(second_start.line_index, Some(1));
    assert_eq!(second_start.source_offset, 4);
    assert_eq!(after_last.line_index, Some(1));
    assert_eq!(after_last.source_offset, "one\ntwo".len());
}

#[test]
fn text_hit_test_respects_aligned_line_frame() {
    let mut style = fixed_text_style();
    style.text_align = UiTextAlign::Right;
    let layout = layout_text("abc", &style, UiFrame::new(0.0, 0.0, 100.0, 20.0), None);
    let line = &layout.lines[0];

    assert!((line.frame.right() - 100.0).abs() <= 0.01);
    assert_eq!(
        hit_test_text_layout(&layout, UiPoint::new(line.frame.x - 1.0, 4.0)).source_offset,
        0
    );
    assert_eq!(
        hit_test_text_layout(&layout, UiPoint::new(line.frame.right() + 1.0, 4.0)).source_offset,
        3
    );
}

#[test]
fn text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen() {
    let mut style = fixed_text_style();
    style.wrap = UiTextWrap::Word;
    let text = "pre\u{00ad}fix";
    let frame_width = measure_text_size("pre-", &style).width + 0.1;
    let layout = layout_text(
        text,
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );
    let first_line = &layout.lines[0];

    let after_break_suffix = hit_test_text_layout(
        &layout,
        UiPoint::new(first_line.frame.x + first_line.measured_width + 1.0, 4.0),
    );

    assert_eq!(first_line.text, "pre-");
    assert_eq!(after_break_suffix.line_index, Some(0));
    assert_eq!(after_break_suffix.source_offset, "pre\u{00ad}".len());
}

fn fixed_text_style() -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    }
}
