use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::ui::surface::UiTextAlign;

use crate::core::math::{Vec2, Vec4};

use super::{font::FontFamilyName, OpenTypeFeature};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextFormat {
    #[default]
    Plain,
    BbCode,
    Html,
    Markdown,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleOverride {
    pub weight: Option<u16>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strike: Option<bool>,
    pub color: Option<Vec4>,
    pub bg_color: Option<Vec4>,
    pub font_size: Option<f32>,
    pub family: Option<FontFamilyName>,
    pub letter_spacing: Option<f32>,
    pub features: Option<Vec<OpenTypeFeature>>,
    pub code: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParagraphOverride {
    pub align: Option<UiTextAlign>,
    pub indent: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InlineBaseline {
    #[default]
    Baseline,
    Center,
    Top,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InlineObjectRef {
    Image {
        texture: ResourceId,
        size: Vec2,
        baseline: InlineBaseline,
    },
    Icon {
        glyph: char,
        font: FontFamilyName,
    },
    Widget {
        id: u64,
        size: Vec2,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkRef {
    pub href: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyledRun {
    pub byte_range: (u32, u32),
    pub style: StyleOverride,
    pub inline: Option<InlineObjectRef>,
    pub link: Option<LinkRef>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichParseResult {
    pub text: String,
    pub runs: Vec<StyledRun>,
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
}
