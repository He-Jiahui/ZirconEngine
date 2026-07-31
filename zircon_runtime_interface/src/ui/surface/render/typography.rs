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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRichTextFormat {
    #[default]
    Plain,
    Markdown,
    BbCode,
    Html,
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
mod tests {
    use super::{UiTextRenderMode, resolve_ui_text_render_mode};

    #[test]
    fn text_render_mode_resolution_uses_explicit_request_then_font_default() {
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Native, Some(UiTextRenderMode::Sdf)),
            UiTextRenderMode::Native
        );
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Sdf, Some(UiTextRenderMode::Native)),
            UiTextRenderMode::Sdf
        );
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Sdf)),
            UiTextRenderMode::Sdf
        );
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Auto, None),
            UiTextRenderMode::Native
        );
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Msdf, Some(UiTextRenderMode::Native)),
            UiTextRenderMode::Msdf
        );
        assert_eq!(
            resolve_ui_text_render_mode(UiTextRenderMode::Auto, Some(UiTextRenderMode::Mtsdf)),
            UiTextRenderMode::Mtsdf
        );
    }
}
