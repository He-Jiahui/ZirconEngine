use std::sync::Arc;

use super::{
    RETAINED_PLAIN_DOCUMENT_MAX_BYTES, UiTextMeasureCache, UiTextMeasureKey, UiTextMeasureSizeKey,
};
use crate::text::TextDocumentKey;
use crate::ui::text::{UiTextLayoutRequest, UiTextViewport};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
};

#[test]
fn ui_text_cache_keys_change_with_font_database_generation() {
    let style = UiResolvedStyle::default();
    let request = UiTextLayoutRequest::new(
        "generation",
        &style,
        UiFrame::new(0.0, 0.0, 100.0, 20.0),
        None,
    );

    assert_ne!(
        UiTextMeasureSizeKey::from_text_style_at_generation("generation", &style, 1),
        UiTextMeasureSizeKey::from_text_style_at_generation("generation", &style, 2)
    );
    assert_ne!(
        UiTextMeasureKey::from_request_at_generation(&request, 1),
        UiTextMeasureKey::from_request_at_generation(&request, 2)
    );
}

#[test]
fn retained_plain_document_cache_reuses_a_document_revision() {
    let style = UiResolvedStyle::default();
    let frame = UiFrame::new(0.0, 0.0, 120.0, 40.0);
    let viewport = UiTextViewport::new(0.0, 40.0, 0).expect("finite viewport");
    let mut cache = UiTextMeasureCache::default();
    let first_request = UiTextLayoutRequest::new("first\nsecond", &style, frame, None)
        .with_viewport(viewport)
        .with_document_key(TextDocumentKey::new(7, 1));
    let repeat_request = UiTextLayoutRequest::new("first\nsecond", &style, frame, None)
        .with_viewport(viewport)
        .with_document_key(TextDocumentKey::new(7, 1));
    let revised_request = UiTextLayoutRequest::new("first\nsecond\nthird", &style, frame, None)
        .with_viewport(viewport)
        .with_document_key(TextDocumentKey::new(7, 2));

    let first = cache.retained_plain_document(&first_request);
    let repeated = cache.retained_plain_document(&repeat_request);
    let revised = cache.retained_plain_document(&revised_request);

    assert!(Arc::ptr_eq(&first.rich, &repeated.rich));
    assert!(!Arc::ptr_eq(&first.rich, &revised.rich));
    let report = cache.retained_plain_documents.report();
    assert_eq!(report.entry_count, 2);
    assert!(report.estimated_bytes <= RETAINED_PLAIN_DOCUMENT_MAX_BYTES);
}

#[test]
fn retained_plain_document_cache_rejects_a_same_revision_source_alias() {
    let style = UiResolvedStyle::default();
    let frame = UiFrame::new(0.0, 0.0, 120.0, 40.0);
    let viewport = UiTextViewport::new(0.0, 40.0, 0).expect("finite viewport");
    let key = TextDocumentKey::new(7, 1);
    let mut cache = UiTextMeasureCache::default();
    let before_request = UiTextLayoutRequest::new("aa\nbbbb", &style, frame, None)
        .with_viewport(viewport)
        .with_document_key(key);
    let aliased_request = UiTextLayoutRequest::new("aaaa\nbb", &style, frame, None)
        .with_viewport(viewport)
        .with_document_key(key);

    let before = cache.retained_plain_document(&before_request);
    let aliased = cache.retained_plain_document(&aliased_request);

    assert_eq!(before.text(), "aa\nbbbb");
    assert_eq!(aliased.text(), "aaaa\nbb");
    assert!(!Arc::ptr_eq(&before.rich, &aliased.rich));
    let report = cache.retained_plain_documents.report();
    assert_eq!(report.entry_count, 1);
    assert_eq!(report.miss_count, 2);
    assert_eq!(report.hit_count, 0);
    assert_eq!(report.stale_source_alias_count, 1);
}

#[test]
fn complete_viewport_layout_cache_hit_skips_the_hard_line_index_probe() {
    let style = UiResolvedStyle {
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let request = UiTextLayoutRequest::new(
        "first\nsecond",
        &style,
        UiFrame::new(0.0, 0.0, 120.0, 48.0),
        Some(UiFrame::new(0.0, 0.0, 120.0, 48.0)),
    )
    .with_document_key(TextDocumentKey::new(9, 1))
    .with_viewport(UiTextViewport::new(0.0, 48.0, 2).expect("finite viewport"));
    let mut cache = UiTextMeasureCache::default();

    cache.begin_frame();
    cache.resolve_or_shape(&request);
    cache.finish_frame();
    let first = cache.text_layout_session.hard_line_index_report();

    cache.begin_frame();
    cache.resolve_or_shape(&request);
    let second = cache.text_layout_session.hard_line_index_report();

    assert_eq!(first.build_count, 1);
    assert_eq!(second.hit_count, first.hit_count);
    assert_eq!(cache.frame_layout_report().hit_count, 1);
}
