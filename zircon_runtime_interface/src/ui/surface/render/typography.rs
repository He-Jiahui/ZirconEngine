use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextAlign {
    #[default]
    Left,
    Center,
    Right,
    Start,
    End,
    Justify,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextWrap {
    None,
    #[default]
    Word,
    WordSmart,
    Glyph,
}

/// 文本渲染后端偏好；`Auto` 允许字体资产提供默认模式。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextRenderMode {
    #[default]
    Auto,
    Native,
    Sdf,
    Msdf,
    Mtsdf,
}

/// 显式请求优先于字体默认值；两者都未指定具体模式时回退到原生渲染。
pub const fn resolve_ui_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_render_mode: Option<UiTextRenderMode>,
) -> UiTextRenderMode {
    match requested_mode {
        UiTextRenderMode::Native => UiTextRenderMode::Native,
        UiTextRenderMode::Sdf => UiTextRenderMode::Sdf,
        UiTextRenderMode::Msdf => UiTextRenderMode::Msdf,
        UiTextRenderMode::Mtsdf => UiTextRenderMode::Mtsdf,
        UiTextRenderMode::Auto => match font_render_mode {
            Some(UiTextRenderMode::Native) => UiTextRenderMode::Native,
            Some(UiTextRenderMode::Sdf) => UiTextRenderMode::Sdf,
            Some(UiTextRenderMode::Msdf) => UiTextRenderMode::Msdf,
            Some(UiTextRenderMode::Mtsdf) => UiTextRenderMode::Mtsdf,
            Some(UiTextRenderMode::Auto) | None => UiTextRenderMode::Native,
        },
    }
}

/// 富文本语法的版本化序列化标识，解析端只接受列出的具体版本。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRichTextFormat {
    #[default]
    Plain,
    #[serde(rename = "markdown_inline_v1")]
    MarkdownInlineV1,
    #[serde(rename = "bbcode_v1")]
    BbCodeV1,
    #[serde(rename = "html_subset_v1")]
    HtmlSubsetV1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextDirection {
    #[default]
    Auto,
    LeftToRight,
    RightToLeft,
    Mixed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextWritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextOverflow {
    #[default]
    Clip,
    Ellipsis,
    EllipsisWord,
    EllipsisStart,
    EllipsisMiddle,
    ShrinkToFit,
    ClampFontSize {
        min_px: f32,
        max_px: f32,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextRunKind {
    #[default]
    Plain,
    Strong,
    Emphasis,
    Code,
    Link,
}

#[cfg(test)]
#[path = "tests/typography.rs"]
mod tests;
