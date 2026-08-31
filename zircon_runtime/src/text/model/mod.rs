pub mod font;
mod geometry;
pub mod rich;
pub mod shaped_run;
mod shaping_receipt;
mod style;

pub use geometry::TextRange;
pub(crate) use geometry::{TextFrame, TextSize};
pub(crate) use style::TextStyle;
pub use style::{RichTextFormat, TextAlign, TextWrap};

pub use rich::{
    InlineBaseline, InlineObjectRef, LaidOutLine, LaidOutText, LayoutItem, LinkRef,
    MAX_RICH_TABLE_ROW_SPAN, ParagraphOverride, RichIconAssetId, RichInlineWidgetSlotId,
    RichListItem, RichListItemKind, RichOrderedListMarker, RichParseResult, RichTable,
    RichTableCell, RichTableCellBoxStyle, RichTableCellPadding, RichTableColumn,
    RichTextAuthoringDiagnostic, RichTextAuthoringDiagnosticCode,
    RichTextAuthoringDiagnosticSeverity, RichTextAuthoringRecovery, StyleOverride, StyledRun,
};
pub(crate) use shaped_run::BackendShapeRequest;
pub(crate) use shaped_run::HorizontalGlyphMetricSpan;
pub(crate) use shaped_run::HorizontalLineRawMetrics;
pub use shaped_run::{
    Iso15924Tag, LineBreakTailoringProfile, OpenTypeFeature, ShapedGlyph, ShapedGlyphBreakSafety,
    ShapedGlyphClusterFlags, ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt,
    ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript, ShapedHardLine, TextOrientation,
    VerticalGlyphDecision, VerticalMode, normalized_open_type_features,
};
pub(crate) use shaping_receipt::{TextFontResolutionReport, TextShapingRequestDiagnostics};
pub use shaping_receipt::{
    TextHorizontalCompositionReceipt, TextShapingBudgetKind, TextShapingFailureCode,
    TextShapingFailureDependency, TextShapingFailureDisposition, TextShapingFailurePhase,
    TextShapingFailureReceipt,
};
