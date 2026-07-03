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
