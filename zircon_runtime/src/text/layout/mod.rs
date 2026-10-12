//! Text measurement and layout support backed by the shared runtime text stack.

#[path = "advance_index.rs"]
mod advance_index;
#[path = "align.rs"]
mod align;
#[path = "arabic_justification.rs"]
mod arabic_justification;
#[path = "horizontal_line_fragment.rs"]
mod horizontal_line_fragment;
#[path = "horizontal_line_policy.rs"]
mod horizontal_line_policy;
#[path = "kinsoku.rs"]
mod kinsoku;
#[path = "line_break/mod.rs"]
mod line_break;
#[path = "logical_virtual_line.rs"]
mod logical_virtual_line;
#[path = "measure.rs"]
mod measure;
#[path = "overflow.rs"]
mod overflow;
#[path = "physical_line_fragment.rs"]
mod physical_line_fragment;
#[path = "rich.rs"]
mod rich;
#[path = "rich_advance_index.rs"]
mod rich_advance_index;
#[path = "rich_source.rs"]
mod rich_source;
#[path = "rich_vertical.rs"]
mod rich_vertical;
#[path = "tab.rs"]
mod tab;
#[path = "vertical_layout.rs"]
mod vertical_layout;

pub(crate) use advance_index::GraphemeAdvanceIndex;
#[cfg(test)]
pub(crate) use advance_index::GraphemeAdvanceMetric;
pub(crate) use align::{
    arabic_kashida_insertion_offsets, arabic_kashida_insertion_offsets_bounded,
    justify_line_advances,
};
pub(crate) use arabic_justification::validate_arabic_tatweel_candidate;
pub(crate) use horizontal_line_fragment::HorizontalLineFragmentGeometry;
pub(crate) use horizontal_line_policy::resolve_horizontal_plain_line_policy;
pub(crate) use line_break::{
    corrected_glyph_ranges_with_provider, line_break_chunks_with_provider,
    line_text_fits_with_provider, should_wrap_before_accumulated, soft_hyphen_break_suffix_at,
    trailing_wrap_space_byte_len, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider, DiscretionaryHyphenDecision,
    BOUNDARY_SHAPING_CONTEXT_GRAPHEMES,
};
pub(crate) use logical_virtual_line::{
    CanonicalLogicalVirtualLineFragment, LogicalVirtualFragmentRole, LogicalVirtualLineSequence,
    LogicalVisualClusterReceipt,
};
pub(crate) use measure::{
    line_metrics_with_provider, measure_line_width, measure_line_width_with_provider,
    measure_line_with_provider, measure_text_size, measure_text_size_with_provider,
    measure_text_source_range_width, measure_text_source_range_width_with_provider,
    measured_grapheme_geometry_from_shaped, measured_grapheme_geometry_with_provider,
    measured_grapheme_widths, measured_grapheme_widths_with_provider,
    text_line_metrics_from_shaped, MeasuredClusterCaretPolicy, MeasuredGlyphCluster,
    MeasuredTextLine, TextLineMetrics,
};
pub(crate) use overflow::{
    retained_grapheme_counts, trim_end_ellipsis_trailing_graphemes, EllipsisPlacement, ELLIPSIS,
};
pub(crate) use physical_line_fragment::{
    shape_horizontal_physical_line_fragment_with_provider, CanonicalPhysicalLineFragment,
};
pub(crate) use rich::{
    layout_rich_line_with_provider, layout_rich_text_glyph_wrapped_with_provider,
    layout_rich_text_with_provider, layout_rich_text_word_wrapped_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, rich_glyph_line_ranges_with_provider,
    rich_word_line_ranges_with_provider, RichWordWrapMode,
};
pub(crate) use rich_advance_index::{resolved_text_spans, ResolvedRichTextSpan};
pub(crate) use rich_source::{
    checked_source_range, checked_source_range_to_u32, for_each_validated_rich_run,
    validate_rich_text_layout_source, RichTextLayoutRun, RichTextLayoutSource,
};
pub(crate) use rich_vertical::rich_vertical_columns_with_provider;
pub(crate) use tab::{tab_aligned_advances, tab_interval_width};
pub(crate) use vertical_layout::layout_vertical_rl_columns;
