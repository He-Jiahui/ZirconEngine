use serde::{Deserialize, Serialize};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextWrap {
    None,
    #[default]
    Word,
    WordSmart,
    Glyph,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextFormat {
    #[default]
    Plain,
    Markdown,
    BbCode,
    Html,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TextStyle {
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub language: Option<String>,
    pub font_weight: u16,
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
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::default_line_height(Self::DEFAULT_FONT_SIZE),
            tab_size: Self::DEFAULT_TAB_SIZE,
            text_align: TextAlign::default(),
            wrap: TextWrap::default(),
        }
    }
}
