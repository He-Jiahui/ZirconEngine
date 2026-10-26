use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateTransitionAsset,
};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

fn source(duration_seconds: f32) -> AnimationAuthoringAsset {
    let graph = AssetReference::from_locator(
        AssetUri::parse("res://animation/duration.graph.zranim").unwrap(),
    );
    AnimationAuthoringAsset::StateMachine(AnimationStateMachineAsset {
        name: Some("Transition duration editing".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset::graph_ref("Idle", graph.clone()),
            AnimationStateAsset::graph_ref("Run", graph),
        ],
        transitions: vec![
            AnimationStateTransitionAsset {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds,
                exit_time: Some(0.25),
                interruption: Default::default(),
                conditions: Vec::new(),
            },
            AnimationStateTransitionAsset {
                from_state: "Run".to_string(),
                to_state: "Idle".to_string(),
                duration_seconds: 0.5,
                exit_time: None,
                interruption: Default::default(),
                conditions: Vec::new(),
            },
        ],
        layers: Vec::new(),
    })
}

fn change(duration_frames: u32) -> AnimationDocumentMutation {
    AnimationDocumentMutation::CreateTransition {
        from_state: "Idle".to_string(),
        to_state: "Run".to_string(),
        duration_frames,
    }
}

fn assert_document(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: u64,
    expected: &AnimationAuthoringAsset,
) {
    engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            assert_eq!(document.revision().value(), revision);
            assert_eq!(
                document.document_bytes().unwrap(),
                expected.to_bytes().unwrap()
            );
            let reopened = AnimationAuthoringAsset::from_bytes(
                document.kind(),
                &document.document_bytes().unwrap(),
            )
            .unwrap();
            assert_eq!(reopened.to_bytes().unwrap(), expected.to_bytes().unwrap());
            assert_eq!(
                document.compilation().current_revision(),
                document.revision()
            );
            assert!(document.compilation().current_product().is_successful());
            assert_eq!(
                document.compilation().last_good_revision(),
                Some(document.revision())
            );
        })
        .unwrap()
        .unwrap();
}

#[test]
fn transition_duration_receipt_keeps_exact_changes_and_other_source_fields() {
    let one_frame_seconds = 1.0_f32 / 30.0;
    for (before_duration, frames, after_duration) in [
        (1.0e-20_f32, 0, 0.0_f32),
        (f32::EPSILON * 0.5, 0, 0.0),
        (-0.0, 0, 0.0),
        (
            f32::from_bits(one_frame_seconds.to_bits() + 1),
            1,
            one_frame_seconds,
        ),
    ] {
        let mut asset = source(before_duration);
        let original_bytes = asset.to_bytes().unwrap();
        assert!(asset.compile().is_successful());
        let expected = source(after_duration);
        assert!(change(frames).apply(&mut asset).unwrap());
        let changed_bytes = asset.to_bytes().unwrap();
        assert_ne!(changed_bytes, original_bytes);
        assert_eq!(changed_bytes, expected.to_bytes().unwrap());
        assert!(asset.compile().is_successful());
        // The same request must leave the complete source, not only the duration, unchanged.
        assert!(!change(frames).apply(&mut asset).unwrap());
        assert_eq!(asset.to_bytes().unwrap(), changed_bytes);
    }
}

#[test]
fn transition_duration_receipt_uses_document_history_and_exact_codec_undo_redo() {
    let id = DocumentId::new(9192);
    let original = source(1.0e-20);
    let expected = source(0.0);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/duration.state_machine.zranim").unwrap(),
            original.clone(),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let mutation = change(0);
    let before_history = engine.history_status(history).unwrap();
    let (revision, replacement) = engine
        .with_context::<CoreEditContext, _>(|context| {
            context
                .animation_documents()
                .prepare_mutation(id, &mutation)
                .unwrap()
                .unwrap()
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        replacement.to_bytes().unwrap(),
        expected.to_bytes().unwrap()
    );
    assert_document(&engine, id, 1, &original);
    assert!(!engine.is_dirty(history).unwrap());
    assert_eq!(engine.history_status(history).unwrap(), before_history);

    let mut transaction = engine.begin(mutation.label(), history).unwrap();
    transaction
        .push(AnimationEditCommand::new(
            mutation.label(),
            id,
            revision,
            replacement,
        ))
        .unwrap();
    transaction.commit().unwrap();
    assert_document(&engine, id, 2, &expected);
    assert!(engine.is_dirty(history).unwrap());
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 3, &original);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 4, &expected);
    assert!(engine.is_dirty(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}

#[test]
fn transition_duration_receipt_noop_or_missing_state_keeps_store_and_history_clean() {
    let id = DocumentId::new(9193);
    let original = source(0.0);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/duration-noop.state_machine.zranim").unwrap(),
            original.clone(),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let before_history = engine.history_status(history).unwrap();
    for mutation in [
        change(0),
        AnimationDocumentMutation::CreateTransition {
            from_state: "Missing".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 0,
        },
        AnimationDocumentMutation::CreateTransition {
            from_state: "Idle".to_string(),
            to_state: "Missing".to_string(),
            duration_frames: 0,
        },
    ] {
        engine
            .with_context::<CoreEditContext, _>(|context| {
                assert!(context
                    .animation_documents()
                    .prepare_mutation(id, &mutation)
                    .unwrap()
                    .is_none());
            })
            .unwrap()
            .unwrap();
        assert_document(&engine, id, 1, &original);
        assert!(!engine.is_dirty(history).unwrap());
        assert_eq!(engine.history_status(history).unwrap(), before_history);
    }
    assert!(!engine.undo(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}
