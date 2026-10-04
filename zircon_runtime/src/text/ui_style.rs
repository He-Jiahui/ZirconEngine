//! UI 已解析样式到中立文本输入的边界，供测量缓存与字形呈现共用同一布局样式。
//! UI 样式没有斜体和 OpenType 特性字段，此映射使用默认值；富文本局部样式另由 span 解析覆盖。

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
            italic: false,
            features: Default::default(),
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
#[path = "tests/ui_style.rs"]
mod tests;
