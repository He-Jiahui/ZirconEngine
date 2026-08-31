use crate::core::framework::text::{TextRenderMode, TextWritingMode};
use crate::text::{RichTextFormat, TextAlign, TextFrame, TextRange, TextSize, TextWrap};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{
        UiRichTextFormat, UiTextAlign, UiTextRange, UiTextRenderMode, UiTextWrap, UiTextWritingMode,
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
            UiRichTextFormat::MarkdownInlineV1 => Self::MarkdownInlineV1,
            UiRichTextFormat::BbCodeV1 => Self::BbCodeV1,
            UiRichTextFormat::HtmlSubsetV1 => Self::HtmlSubsetV1,
        }
    }
}

impl From<RichTextFormat> for UiRichTextFormat {
    fn from(value: RichTextFormat) -> Self {
        match value {
            RichTextFormat::Plain => Self::Plain,
            RichTextFormat::MarkdownInlineV1 => Self::MarkdownInlineV1,
            RichTextFormat::BbCodeV1 => Self::BbCodeV1,
            RichTextFormat::HtmlSubsetV1 => Self::HtmlSubsetV1,
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
