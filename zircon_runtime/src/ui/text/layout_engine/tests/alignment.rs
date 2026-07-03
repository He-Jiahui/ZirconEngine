use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextAlign, UiTextDirection, UiTextOverflow, UiTextWrap},
};

use super::{layout_text, test_style};

#[test]
fn text_align_start_end_follow_rtl_base_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::RightToLeft;
    let frame = UiFrame::new(10.0, 0.0, 80.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("שלום", &style, frame, None);
    let start_line = &start_layout.lines[0];
    assert_eq!(start_layout.direction, UiTextDirection::RightToLeft);
    assert!(
        (start_line.frame.right() - frame.right()).abs() < 0.01,
        "RTL start alignment must anchor text to the right edge"
    );

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("שלום", &style, frame, None);
    assert!(
        (end_layout.lines[0].frame.x - frame.x).abs() < 0.01,
        "RTL end alignment must anchor text to the left edge"
    );
}

#[test]
fn text_align_start_end_auto_uses_first_strong_rtl_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("שלום abc", &style, frame, None);
    let start_line = &start_layout.lines[0];
    assert_eq!(start_layout.direction, UiTextDirection::RightToLeft);
    assert!((start_line.frame.right() - frame.right()).abs() < 0.01);

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("שלום abc", &style, frame, None);
    assert_eq!(end_layout.direction, UiTextDirection::RightToLeft);
    assert!((end_layout.lines[0].frame.x - frame.x).abs() < 0.01);
}

#[test]
fn text_align_start_end_auto_uses_first_strong_ltr_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Auto;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    style.text_align = UiTextAlign::Start;
    let start_layout = layout_text("abc שלום", &style, frame, None);
    assert_eq!(start_layout.direction, UiTextDirection::LeftToRight);
    assert!((start_layout.lines[0].frame.x - frame.x).abs() < 0.01);

    style.text_align = UiTextAlign::End;
    let end_layout = layout_text("abc שלום", &style, frame, None);
    assert_eq!(end_layout.direction, UiTextDirection::LeftToRight);
    assert!((end_layout.lines[0].frame.right() - frame.right()).abs() < 0.01);
}

#[test]
fn text_align_start_end_mixed_request_uses_first_strong_base_direction() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_direction = UiTextDirection::Mixed;
    style.text_align = UiTextAlign::Start;
    let frame = UiFrame::new(10.0, 0.0, 120.0, 12.0);

    let layout = layout_text("שלום abc", &style, frame, None);

    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert!((layout.lines[0].frame.right() - frame.right()).abs() < 0.01);
}
