#[cfg_attr(not(test), allow(dead_code))]
mod edit_state;
mod grapheme;
mod layout_engine;
mod rich_text;

pub(crate) use edit_state::apply_text_edit_action;
pub(crate) use grapheme::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary,
};
pub use layout_engine::layout_text;
pub(crate) use layout_engine::measure_text_size;
