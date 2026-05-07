#[cfg_attr(not(test), allow(dead_code))]
mod edit_state;
mod grapheme;
mod layout_engine;
mod rich_text;

pub(crate) use edit_state::apply_text_edit_action;
pub use layout_engine::layout_text;
pub(crate) use layout_engine::measure_text_size;
