use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::ResourceId;

use crate::core::math::{Vec2, Vec4};

use super::{font::FontFamilyName, OpenTypeFeature, TextAlign};

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
    /// Paragraph-local alignment, resolved inside the indented content frame.
    pub align: Option<TextAlign>,
    /// First physical line inset in logical pixels.
    pub indent: Option<f32>,
    /// Nesting depth measured by the layout owner's tab interval.
    pub indent_level: Option<u16>,
    /// Byte range of a real list marker in [`RichParseResult::text`].
    pub list_prefix: Option<(u32, u32)>,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RichTableColumn {
    pub expand: bool,
    pub shrink: bool,
    pub expand_ratio: u16,
}

/// Shared hostile-input bound for a single rich-table cell's vertical span.
pub const MAX_RICH_TABLE_ROW_SPAN: u16 = 64;

impl Default for RichTableColumn {
    fn default() -> Self {
        Self {
            expand: false,
            shrink: true,
            expand_ratio: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichTableCellPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichTableCellBoxStyle {
    /// Authored logical-pixel padding. `None` preserves the font-relative table fallback.
    pub padding: Option<RichTableCellPadding>,
    pub odd_row_background: Option<Vec4>,
    pub even_row_background: Option<Vec4>,
    pub border_color: Option<Vec4>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RichTableCell {
    /// Byte range of the cell content in [`RichParseResult::text`].
    pub byte_range: (u32, u32),
    /// Resolved zero-based row after deterministic table auto-placement.
    #[serde(default)]
    pub row_index: u32,
    /// Resolved zero-based column after deterministic table auto-placement.
    #[serde(default)]
    pub column_index: u16,
    /// Number of consecutive columns covered by this cell.
    #[serde(default = "default_rich_table_cell_span")]
    pub column_span: u16,
    /// Number of consecutive rows covered by this cell.
    #[serde(default = "default_rich_table_cell_span")]
    pub row_span: u16,
    /// Parser-resolved visual box options; geometry remains owned by UI layout.
    #[serde(default)]
    pub box_style: RichTableCellBoxStyle,
}

impl Default for RichTableCell {
    fn default() -> Self {
        Self {
            byte_range: (0, 0),
            row_index: 0,
            column_index: 0,
            column_span: default_rich_table_cell_span(),
            row_span: default_rich_table_cell_span(),
            box_style: RichTableCellBoxStyle::default(),
        }
    }
}

const fn default_rich_table_cell_span() -> u16 {
    1
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichTable {
    /// Byte range covering every cell in this table.
    pub byte_range: (u32, u32),
    /// Zero-based nesting depth. Nested tables remain range-contained by their parent cell.
    pub depth: u16,
    pub columns: Vec<RichTableColumn>,
    /// Cells are stored in parser order and carry resolved row-major grid coordinates.
    pub cells: Vec<RichTableCell>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichParseResult {
    pub text: String,
    pub runs: Vec<StyledRun>,
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    pub tables: Vec<RichTable>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutItem {
    Text {
        run_index: u32,
        source_range: (u32, u32),
        origin: Vec2,
        advance: f32,
    },
    Inline {
        run_index: u32,
        source_range: (u32, u32),
        object: InlineObjectRef,
        size: Vec2,
        baseline: InlineBaseline,
        origin: Vec2,
        advance: f32,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LaidOutLine {
    pub item_range: (u32, u32),
    pub origin: Vec2,
    pub baseline: f32,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LaidOutText {
    pub items: Vec<LayoutItem>,
    pub lines: Vec<LaidOutLine>,
    pub size: Vec2,
}
