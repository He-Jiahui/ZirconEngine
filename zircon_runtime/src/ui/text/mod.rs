mod adapter;
pub(crate) use adapter::text_style;
mod edit_state;
mod font_registry;
mod geometry;
mod grapheme;
mod hit_test;
mod layout_engine;
mod measure_cache;
mod resolved_layout;
mod rich_text;
pub(crate) mod shaper;

pub(crate) use edit_state::apply_text_edit_action;
#[cfg(test)]
pub(crate) use font_registry::UiFontRegistry;
pub(crate) use geometry::{
    caret_frame_for_text_layout, caret_frame_for_text_layout_with_source_metrics,
    text_range_frames_for_text_layout, text_range_frames_for_text_layout_with_source_metrics,
};
pub(crate) use grapheme::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary, word_range_at,
};
pub(crate) use hit_test::{hit_test_text_layout, UiTextHitTest};
pub(crate) use layout_engine::resolve_text_direction;
#[cfg(test)]
pub(crate) use measure_cache::UiWidthBucket;
pub(crate) use measure_cache::{UiTextMeasureCache, UiTextShapePrewarmRequest};
pub(crate) use resolved_layout::{
    resolve_text_layout, UiPreeditSpan, UiTextLayoutRequest, UiTextLayoutResolution,
};
pub(crate) use rich_text::link_at_layout_point;
pub use shaper::layout_text;
pub(crate) use shaper::{measure_text_size, measure_text_source_range_width};
