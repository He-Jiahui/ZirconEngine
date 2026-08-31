use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::ui::text::UiRichLinkTarget;

use crate::core::math::{Vec2, Vec4};

use super::{OpenTypeFeature, TextAlign, font::FontFamilyName};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichOrderedListMarker {
    Decimal,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichListItemKind {
    Unordered,
    Ordered {
        ordinal: u32,
        marker: RichOrderedListMarker,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichListItem {
    pub kind: RichListItemKind,
    /// One-based semantic nesting level, independent of visual indent policy.
    pub level: u32,
    /// Byte range of the real marker in [`RichParseResult::text`].
    pub marker_range: (u32, u32),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParagraphOverride {
    /// Paragraph-local alignment, resolved inside the indented content frame.
    pub align: Option<TextAlign>,
    /// First physical line inset in logical pixels.
    pub indent: Option<f32>,
    /// Nesting depth measured by the layout owner's tab interval.
    pub indent_level: Option<u16>,
    pub list_item: Option<RichListItem>,
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

/// Stable resource identity for an image-backed rich-text icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RichIconAssetId(ResourceId);

impl RichIconAssetId {
    pub const fn from_resource_id(resource_id: ResourceId) -> Self {
        Self(resource_id)
    }

    pub const fn resource_id(self) -> ResourceId {
        self.0
    }
}

/// Authoring-local slot resolved against one rich-text owner's current direct children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RichInlineWidgetSlotId(u64);

impl RichInlineWidgetSlotId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InlineObjectRef {
    Image {
        texture: ResourceId,
        size: Vec2,
        baseline: InlineBaseline,
        /// Explicit replacement text. `Some("")` marks a decorative image.
        #[serde(default)]
        alternative_text: Option<String>,
        /// Secondary authoring fallback used only when alternative text is absent.
        #[serde(default)]
        tooltip: Option<String>,
    },
    Icon {
        asset: RichIconAssetId,
        size: Vec2,
        baseline: InlineBaseline,
        /// Explicit replacement text. `Some("")` marks a decorative icon.
        #[serde(default)]
        alternative_text: Option<String>,
    },
    Widget {
        slot: RichInlineWidgetSlotId,
        size: Vec2,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkRef {
    #[serde(rename = "href")]
    pub target: UiRichLinkTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<Arc<str>>,
}

impl LinkRef {
    pub(crate) fn retained_heap_bytes(&self) -> usize {
        self.target
            .retained_heap_bytes()
            .saturating_add(self.tooltip.as_ref().map_or(0, |tooltip| tooltip.len()))
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextAuthoringDiagnosticSeverity {
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextAuthoringDiagnosticCode {
    UnsupportedTag,
    UnmatchedClosingTag,
    ImplicitlyClosedTag,
    UnclosedTag,
    UnsupportedAttribute,
    MalformedAttribute,
    InvalidAttributeValue,
    UnsupportedStyleProperty,
    MalformedTag,
    UnterminatedQuotedAttribute,
    MalformedEntity,
    UnrecognizedEntity,
    BidirectionalMark,
    BidirectionalEmbedding,
    BidirectionalOverride,
    BidirectionalIsolate,
}

impl RichTextAuthoringDiagnosticCode {
    pub const fn diagnostic_code(self) -> &'static str {
        match self {
            Self::UnsupportedTag => "ZR-TEXT-RICH-AUTHOR-001",
            Self::UnmatchedClosingTag => "ZR-TEXT-RICH-AUTHOR-002",
            Self::ImplicitlyClosedTag => "ZR-TEXT-RICH-AUTHOR-003",
            Self::UnclosedTag => "ZR-TEXT-RICH-AUTHOR-004",
            Self::UnsupportedAttribute => "ZR-TEXT-RICH-AUTHOR-005",
            Self::MalformedAttribute => "ZR-TEXT-RICH-AUTHOR-006",
            Self::InvalidAttributeValue => "ZR-TEXT-RICH-AUTHOR-007",
            Self::UnsupportedStyleProperty => "ZR-TEXT-RICH-AUTHOR-008",
            Self::MalformedTag => "ZR-TEXT-RICH-AUTHOR-009",
            Self::UnterminatedQuotedAttribute => "ZR-TEXT-RICH-AUTHOR-010",
            Self::MalformedEntity => "ZR-TEXT-RICH-AUTHOR-011",
            Self::UnrecognizedEntity => "ZR-TEXT-RICH-AUTHOR-012",
            Self::BidirectionalMark => "ZR-TEXT-RICH-AUTHOR-013",
            Self::BidirectionalEmbedding => "ZR-TEXT-RICH-AUTHOR-014",
            Self::BidirectionalOverride => "ZR-TEXT-RICH-AUTHOR-015",
            Self::BidirectionalIsolate => "ZR-TEXT-RICH-AUTHOR-016",
        }
    }

    pub const fn message_key(self) -> &'static str {
        match self {
            Self::UnsupportedTag => "text.rich.author.unsupported_tag",
            Self::UnmatchedClosingTag => "text.rich.author.unmatched_closing_tag",
            Self::ImplicitlyClosedTag => "text.rich.author.implicitly_closed_tag",
            Self::UnclosedTag => "text.rich.author.unclosed_tag",
            Self::UnsupportedAttribute => "text.rich.author.unsupported_attribute",
            Self::MalformedAttribute => "text.rich.author.malformed_attribute",
            Self::InvalidAttributeValue => "text.rich.author.invalid_attribute_value",
            Self::UnsupportedStyleProperty => "text.rich.author.unsupported_style_property",
            Self::MalformedTag => "text.rich.author.malformed_tag",
            Self::UnterminatedQuotedAttribute => "text.rich.author.unterminated_quoted_attribute",
            Self::MalformedEntity => "text.rich.author.malformed_entity",
            Self::UnrecognizedEntity => "text.rich.author.unrecognized_entity",
            Self::BidirectionalMark => "text.rich.author.bidirectional_mark",
            Self::BidirectionalEmbedding => "text.rich.author.bidirectional_embedding",
            Self::BidirectionalOverride => "text.rich.author.bidirectional_override",
            Self::BidirectionalIsolate => "text.rich.author.bidirectional_isolate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextAuthoringRecovery {
    DroppedMarkup,
    ImplicitlyClosed,
    ClosedAtEndOfInput,
    IgnoredAttribute,
    IgnoredStyleDeclaration,
    PreservedAsText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichTextAuthoringDiagnostic {
    pub severity: RichTextAuthoringDiagnosticSeverity,
    pub code: RichTextAuthoringDiagnosticCode,
    /// Byte range in the source markup, not in the stripped visible text.
    pub source_range: (u32, u32),
    pub recovery: RichTextAuthoringRecovery,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichParseResult {
    pub text: Arc<str>,
    pub runs: Vec<StyledRun>,
    pub paragraphs: Vec<((u32, u32), ParagraphOverride)>,
    pub tables: Vec<RichTable>,
    #[serde(default)]
    pub authoring_diagnostics: Vec<RichTextAuthoringDiagnostic>,
    #[serde(default)]
    pub authoring_diagnostics_truncated: bool,
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
