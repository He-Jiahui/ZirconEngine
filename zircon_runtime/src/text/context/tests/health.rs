use crate::text::font::FontCollectionService;
use crate::text::{TextRuntimeContext, TextRuntimeContextLifecycleState, TextSystemFontPolicy};

#[test]
fn runtime_text_context_health_snapshot_correlates_authority_and_admission() {
    let font_collection = FontCollectionService::new();
    let context = TextRuntimeContext::new_with_font_collection(font_collection.clone())
        .expect("runtime text context");
    let initial = context.health_snapshot();

    assert_eq!(initial.context(), context.id());
    assert_eq!(
        initial.lifecycle(),
        TextRuntimeContextLifecycleState::Active
    );
    assert_eq!(initial.font_collection(), font_collection.collection_id());
    assert_eq!(initial.font_generation(), font_collection.generation());
    assert_eq!(
        initial.system_font_policy(),
        TextSystemFontPolicy::PackagedOnly
    );
    assert_eq!(initial.discovered_system_face_count(), 0);
    assert_eq!(initial.unicode_data(), context.unicode_data_snapshot().id());
    assert_eq!(initial.admitted_layout_session_total(), 0);
    assert_eq!(initial.active_layout_session_family_count(), 0);

    let _first = context
        .create_layout_session()
        .expect("first layout session");
    let _second = context
        .create_layout_session()
        .expect("second layout session");
    let admitted = context.health_snapshot();

    assert_eq!(admitted.admitted_layout_session_total(), 2);
    assert_eq!(admitted.active_layout_session_family_count(), 2);
}

#[test]
fn runtime_text_context_health_reports_construction_time_system_font_policy() {
    let context =
        TextRuntimeContext::new_with_system_font_policy(TextSystemFontPolicy::DiscoverPlatform)
            .expect("runtime text context");
    let health = context.health_snapshot();

    assert_eq!(
        health.system_font_policy(),
        TextSystemFontPolicy::DiscoverPlatform
    );
    assert_eq!(health.font_generation(), 1);
    assert_eq!(
        health.discovered_system_face_count(),
        context.discovered_system_face_count
    );
}

#[test]
fn rejected_layout_session_does_not_change_health_admission_total() {
    let context = TextRuntimeContext::new().expect("runtime text context");
    let _admitted = context
        .create_layout_session()
        .expect("active context layout session");
    assert_eq!(
        context.begin_draining(),
        TextRuntimeContextLifecycleState::Draining
    );
    assert_eq!(context.close(), TextRuntimeContextLifecycleState::Draining);

    assert!(context.create_layout_session().is_err());
    let draining = context.health_snapshot();

    assert_eq!(
        draining.lifecycle(),
        TextRuntimeContextLifecycleState::Draining
    );
    assert_eq!(draining.admitted_layout_session_total(), 1);
    assert_eq!(draining.active_layout_session_family_count(), 1);

    drop(_admitted);
    let closed = context.health_snapshot();

    assert_eq!(closed.lifecycle(), TextRuntimeContextLifecycleState::Closed);
    assert_eq!(closed.active_layout_session_family_count(), 0);
}

#[test]
fn cloned_layout_session_shares_one_active_family_lease() {
    let context = TextRuntimeContext::new().expect("runtime text context");
    let session = context
        .create_layout_session()
        .expect("active context layout session");
    let cloned = session.clone();

    assert_eq!(session.text_session_id(), cloned.text_session_id());

    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        1
    );

    drop(session);
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        1
    );

    drop(cloned);
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        0
    );
}
