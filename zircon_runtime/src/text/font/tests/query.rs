use super::*;

#[test]
fn italic_text_style_requests_an_italic_font_face() {
    assert_eq!(
        font_query_for_text_style(&TextStyle {
            italic: true,
            ..TextStyle::default()
        })
        .style,
        FontStyle::Italic
    );
    assert_eq!(
        font_query_for_text_style(&TextStyle::default()).style,
        FontStyle::Normal
    );
}
