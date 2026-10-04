use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::shaped_run::OpenTypeFeature;

/// 行内对齐策略；Start/End 随解析后的书写方向决定起止侧，Left/Right 保持物理侧语义。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Start,
    End,
    Justify,
}

/// 软折行策略；硬分隔符仍独立切分物理段落，WordSmart 再约束词尾标点的断行位置。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextWrap {
    None,
    #[default]
    Word,
    WordSmart,
    Glyph,
}

/// 富文本解析与缓存身份中的版本化语法选择；格式版本是编译产物语义的一部分。
/// 调用方须显式选择受支持的子集，不能把外部任意 HTML 或 Markdown 当成完整文档语法。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextFormat {
    #[default]
    Plain,
    #[serde(rename = "markdown_inline_v1")]
    MarkdownInlineV1,
    #[serde(rename = "bbcode_v1")]
    BbCodeV1,
    #[serde(rename = "html_subset_v1")]
    HtmlSubsetV1,
}

/// UI 样式投影后供测量、整形和字形产物共同消费的中性文本样式。
/// 可序列化字段不表示已经规范化；语言、字体选择、特性与几何约束在请求入口继续校验。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextStyle {
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub language: Option<String>,
    pub font_weight: u16,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub features: Arc<[OpenTypeFeature]>,
    pub font_size: f32,
    pub line_height: f32,
    pub tab_size: f32,
    pub text_align: TextAlign,
    pub wrap: TextWrap,
}

impl TextStyle {
    pub(crate) const DEFAULT_FONT_SIZE: f32 = 16.0;
    pub(crate) const DEFAULT_FONT_WEIGHT: u16 = 400;
    pub(crate) const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.2;
    pub(crate) const DEFAULT_TAB_SIZE: f32 = 4.0;
    pub(crate) const MIN_FONT_WEIGHT: u16 = 1;
    pub(crate) const MAX_FONT_WEIGHT: u16 = 1000;

    pub(crate) fn default_line_height(font_size: f32) -> f32 {
        font_size * Self::DEFAULT_LINE_HEIGHT_SCALE
    }

    pub(crate) const fn normalized_font_weight(font_weight: u16) -> u16 {
        if font_weight < Self::MIN_FONT_WEIGHT {
            Self::MIN_FONT_WEIGHT
        } else if font_weight > Self::MAX_FONT_WEIGHT {
            Self::MAX_FONT_WEIGHT
        } else {
            font_weight
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: None,
            font_family: None,
            language: None,
            font_weight: Self::DEFAULT_FONT_WEIGHT,
            italic: false,
            features: Arc::from([]),
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::default_line_height(Self::DEFAULT_FONT_SIZE),
            tab_size: Self::DEFAULT_TAB_SIZE,
            text_align: TextAlign::default(),
            wrap: TextWrap::default(),
        }
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
