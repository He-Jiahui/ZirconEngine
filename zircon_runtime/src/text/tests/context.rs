use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::{
    compiled_unicode_data_snapshot_id, TextRange, TextRuntimeContext,
    TextRuntimeContextAccessError, TextRuntimeContextLifecycleState, TextSessionId, TextStyle,
};

use super::{
    FontCollectionService, TextRuntimeContextLifecycleState as LifecycleState,
    TextRuntimeSessionLease, TEXT_CONTEXT_LIFECYCLE_STATE_BITS,
};

#[test]
fn text_runtime_context_pins_one_font_and_unicode_authority() {
    let font_collection = FontCollectionService::new();
    let context = TextRuntimeContext::new_with_font_collection(font_collection.clone())
        .expect("text runtime context");
    let mut session = context
        .create_layout_session()
        .expect("active context should create a layout session");
    let run = session
        .shape_horizontal_range(
            "context authority",
            &TextStyle::default(),
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 17 },
        )
        .into_result()
        .expect("context layout session should shape");

    assert_ne!(context.id().get(), 0);
    assert_eq!(
        context.lifecycle_state(),
        TextRuntimeContextLifecycleState::Active
    );
    assert_eq!(
        context.unicode_data_snapshot().id(),
        compiled_unicode_data_snapshot_id()
    );
    assert_eq!(
        session.font_collection_revision(),
        font_collection.revision()
    );
    assert_eq!(
        run.unicode_data_snapshot,
        context.unicode_data_snapshot().id()
    );
}

#[test]
fn text_runtime_context_identity_is_not_font_collection_identity() {
    let font_collection = FontCollectionService::new();
    let first = TextRuntimeContext::new_with_font_collection(font_collection.clone())
        .expect("first text runtime context");
    let second = TextRuntimeContext::new_with_font_collection(font_collection)
        .expect("second text runtime context");

    assert_ne!(first.id(), second.id());
}

#[test]
fn text_runtime_context_allocates_context_qualified_session_family_identities() {
    let context = TextRuntimeContext::new().expect("text runtime context");
    let first = context
        .create_layout_session()
        .expect("first layout session");
    let second = context
        .create_layout_session()
        .expect("second layout session");
    let first_id = first.text_session_id().expect("runtime session identity");
    let second_id = second.text_session_id().expect("runtime session identity");

    assert_eq!(first_id.context(), context.id());
    assert_eq!(second_id.context(), context.id());
    assert_ne!(first_id, second_id);
    assert_ne!(first_id.sequence(), 0);
    assert_ne!(second_id.sequence(), 0);
}

#[test]
fn closed_lifecycle_rejects_direct_family_admission_without_incrementing_health() {
    let context = TextRuntimeContext::new().expect("text runtime context");
    assert_eq!(context.close(), TextRuntimeContextLifecycleState::Closed);
    let session_id = TextSessionId {
        context: context.id(),
        sequence: 1,
    };

    let error = TextRuntimeSessionLease::admit(session_id, Arc::clone(&context.lifecycle))
        .expect_err("closed lifecycle must reject family admission atomically");

    assert_eq!(
        error,
        TextRuntimeContextAccessError::Unavailable {
            context: context.id(),
            state: TextRuntimeContextLifecycleState::Closed,
        }
    );
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        0
    );
}

#[test]
fn layout_session_identity_exhaustion_does_not_reuse_or_admit_a_family() {
    let context = TextRuntimeContext::new().expect("text runtime context");
    context
        .next_layout_session_sequence
        .store(u64::MAX, Ordering::Release);

    let error = context
        .create_layout_session()
        .expect_err("exhausted session identities must fail closed");

    assert_eq!(
        error,
        TextRuntimeContextAccessError::LayoutSessionIdentityExhausted {
            context: context.id(),
        }
    );
    assert_eq!(context.health_snapshot().admitted_layout_session_total(), 0);
    assert_eq!(
        context
            .health_snapshot()
            .active_layout_session_family_count(),
        0
    );
}

#[test]
fn active_family_count_exhaustion_preserves_the_packed_lifecycle_word() {
    let context = TextRuntimeContext::new().expect("text runtime context");
    let max_active_families = u64::MAX >> TEXT_CONTEXT_LIFECYCLE_STATE_BITS;
    let exhausted_word =
        LifecycleState::Active as u64 | (max_active_families << TEXT_CONTEXT_LIFECYCLE_STATE_BITS);
    context
        .lifecycle
        .state_and_active_families
        .store(exhausted_word, Ordering::Release);
    let session_id = TextSessionId {
        context: context.id(),
        sequence: 1,
    };

    let error = TextRuntimeSessionLease::admit(session_id, Arc::clone(&context.lifecycle))
        .expect_err("exhausted active family count must fail closed");

    assert_eq!(
        error,
        TextRuntimeContextAccessError::ActiveLayoutSessionFamilyExhausted {
            context: context.id(),
        }
    );
    assert_eq!(
        context
            .lifecycle
            .state_and_active_families
            .load(Ordering::Acquire),
        exhausted_word
    );
}

#[test]
fn family_release_underflow_faults_the_lifecycle_without_wrapping() {
    let context = TextRuntimeContext::new().expect("text runtime context");

    context.lifecycle.release_layout_session_family();

    let health = context.health_snapshot();
    assert_eq!(
        health.lifecycle(),
        TextRuntimeContextLifecycleState::Faulted
    );
    assert_eq!(health.active_layout_session_family_count(), 0);
}

#[test]
fn closed_text_runtime_context_rejects_new_layout_sessions() {
    let context = TextRuntimeContext::new().expect("text runtime context");

    assert_eq!(
        context.begin_draining(),
        TextRuntimeContextLifecycleState::Draining
    );
    assert_eq!(context.close(), TextRuntimeContextLifecycleState::Closed);
    assert_eq!(
        context.lifecycle_state(),
        TextRuntimeContextLifecycleState::Closed
    );
    assert!(context.create_layout_session().is_err());
}
