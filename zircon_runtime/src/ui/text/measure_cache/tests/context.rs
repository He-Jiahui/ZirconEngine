use crate::text::font::FontCollectionService;
use crate::text::{TextRuntimeContext, TextRuntimeContextLifecycleState};
use crate::ui::text::UiTextMeasureCache;

#[test]
fn runtime_text_context_constructs_measure_cache_with_exact_lineage() {
    let font_collection = FontCollectionService::new();
    let context = TextRuntimeContext::new_with_font_collection(font_collection.clone())
        .expect("runtime text context");
    let cache = UiTextMeasureCache::new_with_text_context(&context)
        .expect("active context should create a measure cache");

    assert_eq!(cache.text_runtime_context_id(), Some(context.id()));
    let session = cache
        .text_session_id()
        .expect("context cache keeps a runtime session identity");
    assert_eq!(session.context(), context.id());
    assert_eq!(
        cache.font_database_generation(),
        font_collection.generation()
    );
}

#[test]
fn draining_runtime_text_context_rejects_construction_until_cache_family_drops() {
    let context = TextRuntimeContext::new().expect("runtime text context");
    let cache = UiTextMeasureCache::new_with_text_context(&context)
        .expect("active context should create a measure cache");
    let cloned = cache.clone();
    assert_eq!(cache.text_session_id(), cloned.text_session_id());
    context.begin_draining();
    assert_eq!(context.close(), TextRuntimeContextLifecycleState::Draining);

    let error = UiTextMeasureCache::new_with_text_context(&context)
        .expect_err("draining context must reject a measure cache");

    assert_eq!(
        error,
        crate::text::TextRuntimeContextAccessError::Unavailable {
            context: context.id(),
            state: TextRuntimeContextLifecycleState::Draining,
        }
    );
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        1
    );

    drop(cache);
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        1
    );

    drop(cloned);
    assert_eq!(
        context.lifecycle_state(),
        TextRuntimeContextLifecycleState::Closed
    );
}
