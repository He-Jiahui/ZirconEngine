use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::shaped_run::OpenTypeFeature;

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
mod tests {
    use super::{RichTextFormat, TextStyle};

    #[test]
    fn rich_text_formats_use_versioned_artifact_identity() {
        for (format, wire_value) in [
            (RichTextFormat::Plain, "plain"),
            (RichTextFormat::MarkdownInlineV1, "markdown_inline_v1"),
            (RichTextFormat::BbCodeV1, "bbcode_v1"),
            (RichTextFormat::HtmlSubsetV1, "html_subset_v1"),
        ] {
            let encoded = serde_json::to_string(&format).expect("format serializes");
            assert_eq!(encoded, format!("\"{wire_value}\""));
            assert_eq!(
                serde_json::from_str::<RichTextFormat>(&encoded).expect("format round trips"),
                format
            );
        }

        for legacy_value in ["markdown", "bbcode", "html"] {
            assert!(
                serde_json::from_str::<RichTextFormat>(&format!("\"{legacy_value}\"")).is_err()
            );
        }
    }

    #[test]
    fn legacy_text_style_defaults_new_shaping_identity_fields() {
        let legacy = r#"{
            "font": null,
            "font_family": null,
            "language": null,
            "font_weight": 400,
            "font_size": 16.0,
            "line_height": 19.2,
            "tab_size": 4.0,
            "text_align": "left",
            "wrap": "word"
        }"#;

        let style: TextStyle = serde_json::from_str(legacy).expect("legacy style remains readable");

        assert!(!style.italic);
        assert!(style.features.is_empty());
    }
}
