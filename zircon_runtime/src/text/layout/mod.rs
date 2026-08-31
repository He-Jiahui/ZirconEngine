//! Text measurement and layout support backed by the shared runtime text stack.

mod advance_index;
mod align;
mod arabic_justification;
mod horizontal_line_fragment;
mod horizontal_line_policy;
mod kinsoku;
mod line_break;
mod logical_virtual_line;
mod measure;
mod overflow;
mod physical_line_fragment;
mod rich;
mod rich_advance_index;
mod rich_source;
mod rich_vertical;
mod tab;
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
    BOUNDARY_SHAPING_CONTEXT_GRAPHEMES, DiscretionaryHyphenDecision,
    corrected_glyph_ranges_with_provider, line_break_chunks_with_provider,
    line_text_fits_with_provider, should_wrap_before_accumulated, soft_hyphen_break_suffix_at,
    trailing_wrap_space_byte_len, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider,
};
pub(crate) use logical_virtual_line::{
    CanonicalLogicalVirtualLineFragment, LogicalVirtualFragmentRole, LogicalVirtualLineSequence,
    LogicalVisualClusterReceipt,
};
pub(crate) use measure::{
    MeasuredClusterCaretPolicy, MeasuredGlyphCluster, MeasuredTextLine, TextLineMetrics,
    line_metrics_with_provider, measure_line_width, measure_line_width_with_provider,
    measure_line_with_provider, measure_text_size, measure_text_size_with_provider,
    measure_text_source_range_width, measure_text_source_range_width_with_provider,
    measured_grapheme_geometry_from_shaped, measured_grapheme_geometry_with_provider,
    measured_grapheme_widths, measured_grapheme_widths_with_provider,
    text_line_metrics_from_shaped,
};
pub(crate) use overflow::{
    ELLIPSIS, EllipsisPlacement, retained_grapheme_counts, trim_end_ellipsis_trailing_graphemes,
};
pub(crate) use physical_line_fragment::{
    CanonicalPhysicalLineFragment, shape_horizontal_physical_line_fragment_with_provider,
};
pub(crate) use rich::{
    RichWordWrapMode, layout_rich_line_with_provider, layout_rich_text_glyph_wrapped_with_provider,
    layout_rich_text_with_provider, layout_rich_text_word_wrapped_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, rich_glyph_line_ranges_with_provider,
    rich_word_line_ranges_with_provider,
};
pub(crate) use rich_advance_index::{ResolvedRichTextSpan, resolved_text_spans};
pub(crate) use rich_source::{
    RichTextLayoutRun, RichTextLayoutSource, checked_source_range, checked_source_range_to_u32,
    for_each_validated_rich_run, validate_rich_text_layout_source,
};
pub(crate) use rich_vertical::rich_vertical_columns_with_provider;
pub(crate) use tab::{tab_aligned_advances, tab_interval_width};
pub(crate) use vertical_layout::layout_vertical_rl_columns;
