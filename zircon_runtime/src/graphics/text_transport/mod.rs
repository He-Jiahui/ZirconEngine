//! Transport adapters between public UI DTOs and neutral runtime text contracts.

use crate::core::framework::text::{TextDirection, TextRenderMode, TextWritingMode};
use crate::text::{RichTextFormat, TextAlign, TextFrame, TextRange, TextSize, TextStyle, TextWrap};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiResolvedStyle, UiRichTextFormat, UiTextAlign, UiTextDirection, UiTextRange,
        UiTextRenderMode, UiTextWrap, UiTextWritingMode,
    },
};

impl From<UiTextRange> for TextRange {
    fn from(value: UiTextRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<TextRange> for UiTextRange {
    fn from(value: TextRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
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

impl From<UiTextWritingMode> for TextWritingMode {
    fn from(value: UiTextWritingMode) -> Self {
        match value {
            UiTextWritingMode::HorizontalTb => Self::HorizontalTopToBottom,
            UiTextWritingMode::VerticalRl => Self::VerticalRightToLeft,
        }
    }
}

impl From<TextWritingMode> for UiTextWritingMode {
    fn from(value: TextWritingMode) -> Self {
        match value {
            TextWritingMode::HorizontalTopToBottom => Self::HorizontalTb,
            TextWritingMode::VerticalRightToLeft => Self::VerticalRl,
        }
    }
}

impl From<UiTextAlign> for TextAlign {
    fn from(value: UiTextAlign) -> Self {
        match value {
            UiTextAlign::Left => Self::Left,
            UiTextAlign::Center => Self::Center,
            UiTextAlign::Right => Self::Right,
            UiTextAlign::Start => Self::Start,
            UiTextAlign::End => Self::End,
            UiTextAlign::Justify => Self::Justify,
        }
    }
}

impl From<TextAlign> for UiTextAlign {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Left => Self::Left,
            TextAlign::Center => Self::Center,
            TextAlign::Right => Self::Right,
            TextAlign::Start => Self::Start,
            TextAlign::End => Self::End,
            TextAlign::Justify => Self::Justify,
        }
    }
}

impl From<UiTextWrap> for TextWrap {
    fn from(value: UiTextWrap) -> Self {
        match value {
            UiTextWrap::None => Self::None,
            UiTextWrap::Word => Self::Word,
            UiTextWrap::WordSmart => Self::WordSmart,
            UiTextWrap::Glyph => Self::Glyph,
        }
    }
}

impl From<TextWrap> for UiTextWrap {
    fn from(value: TextWrap) -> Self {
        match value {
            TextWrap::None => Self::None,
            TextWrap::Word => Self::Word,
            TextWrap::WordSmart => Self::WordSmart,
            TextWrap::Glyph => Self::Glyph,
        }
    }
}

impl From<UiRichTextFormat> for RichTextFormat {
    fn from(value: UiRichTextFormat) -> Self {
        match value {
            UiRichTextFormat::Plain => Self::Plain,
            UiRichTextFormat::Markdown => Self::Markdown,
            UiRichTextFormat::BbCode => Self::BbCode,
            UiRichTextFormat::Html => Self::Html,
        }
    }
}

impl From<RichTextFormat> for UiRichTextFormat {
    fn from(value: RichTextFormat) -> Self {
        match value {
            RichTextFormat::Plain => Self::Plain,
            RichTextFormat::Markdown => Self::Markdown,
            RichTextFormat::BbCode => Self::BbCode,
            RichTextFormat::Html => Self::Html,
        }
    }
}

impl From<UiTextRenderMode> for TextRenderMode {
    fn from(value: UiTextRenderMode) -> Self {
        match value {
            UiTextRenderMode::Auto => Self::Auto,
            UiTextRenderMode::Native => Self::Native,
            UiTextRenderMode::Sdf => Self::Sdf,
            UiTextRenderMode::Msdf => Self::Msdf,
            UiTextRenderMode::Mtsdf => Self::Mtsdf,
        }
    }
}

impl From<UiFrame> for TextFrame {
    fn from(value: UiFrame) -> Self {
        Self::new(value.x, value.y, value.width, value.height)
    }
}

impl From<TextFrame> for UiFrame {
    fn from(value: TextFrame) -> Self {
        Self::new(value.x, value.y, value.width, value.height)
    }
}

impl From<TextSize> for UiSize {
    fn from(value: TextSize) -> Self {
        Self::new(value.width, value.height)
    }
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
            text_align: value.text_align.into(),
            wrap: value.wrap.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_text_format_transport_round_trips_every_variant() {
        let cases = [
            (UiRichTextFormat::Plain, RichTextFormat::Plain),
            (UiRichTextFormat::Markdown, RichTextFormat::Markdown),
            (UiRichTextFormat::BbCode, RichTextFormat::BbCode),
            (UiRichTextFormat::Html, RichTextFormat::Html),
        ];

        for (transport, neutral) in cases {
            assert_eq!(RichTextFormat::from(transport), neutral);
            assert_eq!(UiRichTextFormat::from(neutral), transport);
        }
    }

    #[test]
    fn text_direction_transport_round_trips_every_variant() {
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

    #[test]
    fn text_writing_mode_transport_round_trips_every_variant() {
        let cases = [
            (
                UiTextWritingMode::HorizontalTb,
                TextWritingMode::HorizontalTopToBottom,
            ),
            (
                UiTextWritingMode::VerticalRl,
                TextWritingMode::VerticalRightToLeft,
            ),
        ];

        for (transport, neutral) in cases {
            assert_eq!(TextWritingMode::from(transport), neutral);
            assert_eq!(UiTextWritingMode::from(neutral), transport);
        }
    }

    #[test]
    fn text_alignment_and_wrap_transport_round_trip_every_variant() {
        let align_cases = [
            (UiTextAlign::Left, TextAlign::Left),
            (UiTextAlign::Center, TextAlign::Center),
            (UiTextAlign::Right, TextAlign::Right),
            (UiTextAlign::Start, TextAlign::Start),
            (UiTextAlign::End, TextAlign::End),
            (UiTextAlign::Justify, TextAlign::Justify),
        ];
        for (transport, neutral) in align_cases {
            assert_eq!(TextAlign::from(transport), neutral);
            assert_eq!(UiTextAlign::from(neutral), transport);
        }

        let wrap_cases = [
            (UiTextWrap::None, TextWrap::None),
            (UiTextWrap::Word, TextWrap::Word),
            (UiTextWrap::WordSmart, TextWrap::WordSmart),
            (UiTextWrap::Glyph, TextWrap::Glyph),
        ];
        for (transport, neutral) in wrap_cases {
            assert_eq!(TextWrap::from(transport), neutral);
            assert_eq!(UiTextWrap::from(neutral), transport);
        }
    }

    #[test]
    fn text_geometry_transport_preserves_components() {
        let transport = UiFrame::new(1.25, -2.5, 320.0, 48.5);
        let neutral = TextFrame::from(transport);

        assert_eq!(neutral, TextFrame::new(1.25, -2.5, 320.0, 48.5));
        assert_eq!(UiFrame::from(neutral), transport);

        let transport_range = UiTextRange { start: 3, end: 17 };
        let neutral_range = TextRange::from(transport_range);
        assert_eq!(neutral_range, TextRange { start: 3, end: 17 });
        assert_eq!(UiTextRange::from(neutral_range), transport_range);

        assert_eq!(
            UiSize::from(TextSize::new(640.0, 360.0)),
            UiSize::new(640.0, 360.0)
        );
    }
}
