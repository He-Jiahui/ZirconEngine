use crate::ui::text::{hit_test_text_layout, layout_text};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiPoint},
    surface::{UiResolvedStyle, UiTextAlign, UiTextWrap},
};

#[test]
fn text_hit_test_uses_grapheme_midpoints() {
    let style = fixed_text_style();
    let text = "a\u{0301}b";
    let layout = layout_text(text, &style, UiFrame::new(10.0, 0.0, 80.0, 20.0), None);

    let before = hit_test_text_layout(&layout, UiPoint::new(10.2, 4.0));
    let after_cluster = hit_test_text_layout(&layout, UiPoint::new(12.6, 4.0));
    let after_text = hit_test_text_layout(&layout, UiPoint::new(18.0, 4.0));

    assert_eq!(before.line_index, Some(0));
    assert_eq!(before.source_offset, 0);
    assert_eq!(after_cluster.source_offset, "a\u{0301}".len());
    assert_eq!(after_text.source_offset, text.len());
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

    assert_eq!(layout.lines[0].frame.x, 85.0);
    assert_eq!(
        hit_test_text_layout(&layout, UiPoint::new(84.0, 4.0)).source_offset,
        0
    );
    assert_eq!(
        hit_test_text_layout(&layout, UiPoint::new(100.0, 4.0)).source_offset,
        3
    );
}

fn fixed_text_style() -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    }
}
