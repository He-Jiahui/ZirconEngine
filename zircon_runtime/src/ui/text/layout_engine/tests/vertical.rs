use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap, UiTextWritingMode},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn vertical_rl_wraps_columns_on_frame_height() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame_height = measure_text_size("縦書", &style).width + 0.1;

    let layout = layout_text(
        "縦書文",
        &style,
        UiFrame::new(0.0, 0.0, style.line_height * 3.0, frame_height),
        None,
    );

    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "縦書");
    assert_eq!(layout.lines[1].text, "文");
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
    assert_eq!(layout.lines[0].frame.y, layout.lines[1].frame.y);
    assert!(layout.lines[0].frame.height > layout.lines[0].frame.width);
    assert!(
        layout
            .lines
            .iter()
            .all(|line| { (line.baseline - line.frame.width * 0.5).abs() <= f32::EPSILON })
    );
}
