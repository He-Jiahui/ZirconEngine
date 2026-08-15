//! Text measurement and layout support backed by the shared runtime text stack.

mod advance_index;
mod align;
mod kinsoku;
mod line_break;
mod measure;
mod overflow;
mod rich;
mod rich_advance_index;
mod rich_source;
mod rich_vertical;
mod tab;
mod vertical_layout;

pub(crate) use advance_index::GraphemeAdvanceIndex;
pub(crate) use align::{
    arabic_kashida_insertion_offsets, arabic_kashida_insertion_offsets_bounded,
    justify_line_advances,
};
pub(crate) use line_break::{
    corrected_glyph_ranges_with_provider, line_break_chunks_with_provider,
    line_text_fits_with_provider, should_wrap_before_accumulated, soft_hyphen_break_suffix_at,
    trailing_wrap_space_byte_len, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider, BOUNDARY_SHAPING_CONTEXT_GRAPHEMES,
};
pub(crate) use measure::{
    line_metrics_with_provider, measure_line_width, measure_line_width_with_provider,
    measure_text_size, measure_text_size_with_provider, measure_text_source_range_width,
    measure_text_source_range_width_with_provider, measured_grapheme_widths,
    measured_grapheme_widths_with_provider, TextLineMetrics,
};
pub(crate) use overflow::{
    retained_grapheme_counts, trim_end_ellipsis_trailing_graphemes, EllipsisPlacement, ELLIPSIS,
};
pub(crate) use rich::{
    layout_rich_line_with_provider, layout_rich_text_glyph_wrapped_with_provider,
    layout_rich_text_with_provider, layout_rich_text_word_wrapped_with_provider,
    resolve_rich_run_style, rich_forced_line_ranges, rich_glyph_line_ranges_with_provider,
    rich_word_line_ranges_with_provider, RichWordWrapMode,
};
pub(crate) use rich_advance_index::{resolved_text_spans, ResolvedRichTextSpan};
pub(crate) use rich_source::{RichTextLayoutRun, RichTextLayoutSource};
pub(crate) use rich_vertical::rich_vertical_columns_with_provider;
pub(crate) use tab::{tab_aligned_advances, tab_interval_width};
pub(crate) use vertical_layout::layout_vertical_rl_columns;
