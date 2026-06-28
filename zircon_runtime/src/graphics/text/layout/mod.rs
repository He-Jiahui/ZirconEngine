//! Text measurement and layout support backed by the shared runtime text stack.

mod kinsoku;
mod line_break;
mod measure;

pub(crate) use line_break::line_break_chunks;
pub(crate) use measure::{
    line_metrics, measure_line_width, measure_text_size, measured_grapheme_widths, TextLineMetrics,
};
