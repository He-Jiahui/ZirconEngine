//! Text measurement and layout support backed by the shared runtime text stack.

mod align;
mod kinsoku;
mod line_break;
mod measure;
mod overflow;
mod tab;
mod vertical_layout;

pub(crate) use align::justify_line_advances;
pub(crate) use line_break::{
    line_break_chunks_with_provider, line_text_fits_with_provider,
    should_wrap_before_chunk_with_provider, trailing_wrap_space_byte_len, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider,
};
pub(crate) use measure::{
    line_metrics_with_provider, measure_line_width, measure_line_width_with_provider,
    measure_text_size, measure_text_size_with_provider, measure_text_source_range_width,
    measured_grapheme_widths, measured_grapheme_widths_with_provider, TextLineMetrics,
};
pub(crate) use overflow::{ellipsize_text, EllipsisPlacement, EllipsisSegment, ELLIPSIS};
pub(crate) use tab::tab_aligned_advances;
pub(crate) use vertical_layout::layout_vertical_rl_columns;
