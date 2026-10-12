#[path = "edit_state.rs"]
mod edit_state;
#[path = "geometry.rs"]
mod geometry;
#[path = "grapheme.rs"]
mod grapheme;
#[path = "hit_test.rs"]
mod hit_test;
#[path = "inline_widget.rs"]
mod inline_widget;
#[path = "layout_engine.rs"]
mod layout_engine;
#[path = "measure_cache.rs"]
mod measure_cache;
#[path = "presentation.rs"]
mod presentation;
#[path = "resolved_layout.rs"]
mod resolved_layout;
#[path = "rich_text.rs"]
mod rich_text;
#[path = "shaper.rs"]
pub(crate) mod shaper;

pub(crate) use edit_state::{
    apply_text_edit_action, apply_text_edit_action_with_intent,
    apply_text_edit_actions_with_intent, CommittedTextEditIntent, TextEditActionSequenceError,
    TextEditStateTransition,
};
pub(crate) use geometry::{
    caret_frame_for_text_layout, caret_frame_for_text_layout_with_font_collection,
    caret_frame_for_text_layout_with_source_metrics, text_range_frames_for_text_layout,
    text_range_frames_for_text_layout_with_font_collection,
    text_range_frames_for_text_layout_with_source_metrics,
};
pub(crate) use grapheme::{
    clamp_grapheme_boundary, line_end_boundary, line_start_boundary, next_grapheme_boundary,
    next_line_same_column_boundary, next_word_boundary, previous_grapheme_boundary,
    previous_line_same_column_boundary, previous_word_boundary, word_range_at,
};
pub(crate) use hit_test::{
    hit_test_text_layout, hit_test_text_layout_with_font_collection,
    hit_test_text_layout_with_source_metrics, UiTextHitTest,
};
pub(crate) use inline_widget::{inline_widget_layout_from_compiled, UiInlineWidgetLayout};
pub(crate) use layout_engine::{apply_secure_text_presentation, resolve_text_direction};
pub(crate) use measure_cache::{UiTextMeasureCache, UiTextShapePrewarmRequest};
pub(crate) use presentation::{
    UiSecureTextPresentation, UiSecureTextPresentationBidi, UiSecureTextPresentationCluster,
    UiSecureTextPresentationError, UiSecureTextPresentationLine,
};
pub(crate) use resolved_layout::{
    resolve_text_layout, UiPreeditSpan, UiTextLayoutRequest, UiTextLayoutResolution, UiTextViewport,
};
#[cfg(test)]
pub(crate) use rich_text::parse_source_text;
pub(crate) use rich_text::{link_at_layout_point, UiParsedText};
pub use shaper::layout_text;
pub(crate) use shaper::{
    measure_text_size, measure_text_source_range_width, measure_unwrapped_text_height,
};
