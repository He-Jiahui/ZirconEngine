use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextWrap,
};

use super::{TextAlign, TextStyle, TextWrap};
use crate::core::framework::text::TextDirection;

pub(crate) fn text_style(value: &UiResolvedStyle) -> TextStyle {
    value.into()
}

impl From<&UiResolvedStyle> for TextStyle {
    fn from(value: &UiResolvedStyle) -> Self {
        Self {
            font: value.font.clone(),
            font_family: value.font_family.clone(),
            language: value.language.clone(),
            font_weight: value.font_weight,
            font_size: value.font_size,
            line_height: value.line_height,
            tab_size: value.tab_size,
            text_align: text_align(value.text_align),
            wrap: text_wrap(value.wrap),
        }
    }
}

impl From<UiTextDirection> for TextDirection {
    fn from(value: UiTextDirection) -> Self {
        match value {
            UiTextDirection::Auto => Self::Auto,
            UiTextDirection::LeftToRight => Self::LeftToRight,
            UiTextDirection::RightToLeft => Self::RightToLeft,
            UiTextDirection::Mixed => Self::Mixed,
        }
    }
}

impl From<TextDirection> for UiTextDirection {
    fn from(value: TextDirection) -> Self {
        match value {
            TextDirection::Auto => Self::Auto,
            TextDirection::LeftToRight => Self::LeftToRight,
            TextDirection::RightToLeft => Self::RightToLeft,
            TextDirection::Mixed => Self::Mixed,
        }
    }
}

fn text_align(value: UiTextAlign) -> TextAlign {
    match value {
        UiTextAlign::Left => TextAlign::Left,
        UiTextAlign::Center => TextAlign::Center,
        UiTextAlign::Right => TextAlign::Right,
        UiTextAlign::Start => TextAlign::Start,
        UiTextAlign::End => TextAlign::End,
        UiTextAlign::Justify => TextAlign::Justify,
    }
}

fn text_wrap(value: UiTextWrap) -> TextWrap {
    match value {
        UiTextWrap::None => TextWrap::None,
        UiTextWrap::Word => TextWrap::Word,
        UiTextWrap::WordSmart => TextWrap::WordSmart,
        UiTextWrap::Glyph => TextWrap::Glyph,
    }
}

#[cfg(test)]
mod tests {
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
}
