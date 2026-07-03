use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_wrap_soft_hyphen_inserts_hyphen() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("pre-", &style).width + 0.1;
    assert!(frame_width < measure_text_size("prefix", &style).width);
    assert!(measure_text_size("fix", &style).width <= frame_width);

    let layout = layout_text(
        "pre\u{00ad}fix",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "pre-");
    assert_eq!(layout.lines[1].text, "fix");
    assert!(
        layout
            .lines
            .iter()
            .all(|line| !line.text.contains('\u{00ad}')),
        "soft hyphen is a source break hint and must not be retained in visual text"
    );
}
