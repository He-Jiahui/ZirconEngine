use super::{TextDirection, TextFontRequest, TextOpenTypeFeature, TextWritingMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextShapeRequest<'a> {
    pub text: &'a str,
    pub language: Option<&'a str>,
    pub direction: TextDirection,
    pub writing_mode: TextWritingMode,
    pub line_height: f32,
    pub tab_size: f32,
    pub include_kerning: bool,
    pub features: &'a [TextOpenTypeFeature],
    pub font: TextFontRequest<'a>,
}

impl<'a> TextShapeRequest<'a> {
    pub const fn new(text: &'a str, font: TextFontRequest<'a>) -> Self {
        Self {
            text,
            language: None,
            direction: TextDirection::Auto,
            writing_mode: TextWritingMode::HorizontalTopToBottom,
            line_height: font.size * 1.2,
            tab_size: 4.0,
            include_kerning: true,
            features: &[],
            font,
        }
    }

    pub const fn with_features(mut self, features: &'a [TextOpenTypeFeature]) -> Self {
        self.features = features;
        self
    }
}
