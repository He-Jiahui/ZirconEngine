mod edit_state;
mod font_registry;
mod grapheme;
mod hit_test;
mod layout_engine;
mod measure_cache;
mod raster;
mod resolved_layout;
mod rich_text;
pub(crate) mod shaper;

pub(crate) use edit_state::apply_text_edit_action;
#[cfg(test)]
pub(crate) use font_registry::UiFontRegistry;
pub(crate) use grapheme::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary, word_range_at,
};
pub(crate) use hit_test::{hit_test_text_layout, UiTextHitTest};
#[cfg(test)]
pub(crate) use measure_cache::{UiTextMeasureCache, UiWidthBucket};
#[cfg(test)]
pub(crate) use raster::{raster_path_for, UiGlyphRasterPath, UiGlyphRasterPolicy};
pub(crate) use resolved_layout::{resolve_text_layout, UiPreeditSpan, UiTextLayoutRequest};
pub use shaper::layout_text;
pub(crate) use shaper::measure_text_size;
