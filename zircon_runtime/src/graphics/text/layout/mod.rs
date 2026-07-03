//! Text measurement and layout support backed by the shared runtime text stack.

mod align;
mod kinsoku;
mod line_break;
mod measure;
mod overflow;
mod tab;

pub(crate) use align::justify_line_advances;
pub(crate) use line_break::{
    line_break_chunks, line_text_fits, should_wrap_before_chunk, trailing_wrap_space_byte_len,
    trim_leading_wrap_spaces, word_smart_line_break_chunks,
};
pub(crate) use measure::{
    line_metrics, measure_line_width, measure_text_size, measure_text_source_range_width,
    measured_grapheme_widths, TextLineMetrics,
};
pub(crate) use overflow::{ellipsize_text, EllipsisPlacement, EllipsisSegment, ELLIPSIS};
pub(crate) use tab::tab_aligned_advances;
