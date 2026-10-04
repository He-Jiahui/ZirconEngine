use super::*;

#[test]
fn text_style_from_ui_resolved_style_preserves_layout_fields() {
    let resolved = UiResolvedStyle {
        font: Some("res://fonts/ui.ttf".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: Some("sr-Latn".to_string()),
        font_weight: 650,
        font_size: 17.5,
        line_height: 24.0,
        tab_size: 6.0,
        text_align: UiTextAlign::End,
        wrap: UiTextWrap::WordSmart,
        ..UiResolvedStyle::default()
    };

    let style = TextStyle::from(&resolved);

    assert_eq!(style.font.as_deref(), Some("res://fonts/ui.ttf"));
    assert_eq!(style.font_family.as_deref(), Some("Zircon Sans"));
    assert_eq!(style.language.as_deref(), Some("sr-Latn"));
    assert_eq!(style.font_weight, 650);
    assert_eq!(style.font_size, 17.5);
    assert_eq!(style.line_height, 24.0);
    assert_eq!(style.tab_size, 6.0);
    assert_eq!(style.text_align, TextAlign::End);
    assert_eq!(style.wrap, TextWrap::WordSmart);
}

#[test]
fn text_direction_from_ui_transport_round_trips_every_variant() {
    let cases = [
        (UiTextDirection::Auto, TextDirection::Auto),
        (UiTextDirection::LeftToRight, TextDirection::LeftToRight),
        (UiTextDirection::RightToLeft, TextDirection::RightToLeft),
        (UiTextDirection::Mixed, TextDirection::Mixed),
    ];

    for (transport, neutral) in cases {
        assert_eq!(TextDirection::from(transport), neutral);
        assert_eq!(UiTextDirection::from(neutral), transport);
    }
}
