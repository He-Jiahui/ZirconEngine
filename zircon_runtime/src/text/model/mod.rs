pub mod font;
mod geometry;
pub mod rich;
pub mod shaped_run;
mod style;

pub use geometry::TextRange;
pub(crate) use geometry::{TextFrame, TextSize};
pub(crate) use style::TextStyle;
pub use style::{RichTextFormat, TextAlign, TextWrap};

pub use rich::{
    InlineBaseline, InlineObjectRef, LaidOutLine, LaidOutText, LayoutItem, LinkRef,
    ParagraphOverride, RichParseResult, RichTable, RichTableCell, RichTableCellBoxStyle,
    RichTableCellPadding, RichTableColumn, StyleOverride, StyledRun, MAX_RICH_TABLE_ROW_SPAN,
};
pub(crate) use shaped_run::BackendShapeRequest;
pub use shaped_run::{
    normalized_open_type_features, Iso15924Tag, OpenTypeFeature, ShapedGlyph,
    ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedTextLine, TextOrientation, VerticalMode,
};
