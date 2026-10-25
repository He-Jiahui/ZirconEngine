use super::*;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle,
    AnimationDocumentRevision, AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

fn source() -> (
    AnimationSequenceAsset,
    AnimationTrackPath,
    AnimationTrackPath,
) {
    let entity = EntityPath::parse("Root/Hero").unwrap();
    let other_entity = EntityPath::parse("Root/Other").unwrap();
    let property = ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap();
    let track = AnimationSequenceTrackAsset {
        property_path: property.clone(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Linear,
            keys: vec![
                AnimationChannelKeyAsset {
                    time_seconds: 0.0,
                    value: AnimationChannelValueAsset::Scalar(0.25),
                    in_tangent: None,
                    out_tangent: None,
                },
                AnimationChannelKeyAsset {
                    time_seconds: 2.0,
                    value: AnimationChannelValueAsset::Scalar(0.75),
                    in_tangent: None,
                    out_tangent: None,
                },
            ],
        },
    };
    (
        AnimationSequenceAsset {
            name: Some("Current source selection admission".to_string()),
            duration_seconds: 2.0,
            frames_per_second: 30.0,
            bindings: vec![
                AnimationSequenceBindingAsset {
                    entity_path: entity.clone(),
                    target_id: Some("9007199254740995".to_string()),
                    tracks: Vec::new(),
                },
                AnimationSequenceBindingAsset {
                    entity_path: entity.clone(),
                    target_id: Some("9007199254740993".to_string()),
                    tracks: vec![track.clone()],
                },
                AnimationSequenceBindingAsset {
                    entity_path: other_entity.clone(),
                    target_id: None,
                    tracks: vec![track],
                },
            ],
        },
        AnimationTrackPath::new(entity, property.clone()),
        AnimationTrackPath::new(other_entity, property),
    )
}

fn duplicate_selected_track(asset: &mut AnimationSequenceAsset) {
    asset.bindings.push(AnimationSequenceBindingAsset {
        entity_path: asset.bindings[1].entity_path.clone(),
        target_id: Some("9007199254740997".to_string()),
        tracks: asset.bindings[1].tracks.clone(),
    });
}

fn session(asset: AnimationSequenceAsset) -> AnimationEditorSession {
    let document = AnimationAuthoringDocument::new(
        DocumentId::new(9192),
        AssetUri::parse("res://tests/selection-admission.sequence.zranim").unwrap(),
        AnimationAuthoringAsset::Sequence(asset),
    );
    AnimationEditorSession::new(
        "res://tests/selection-admission.sequence.zranim".to_string(),
        AnimationAuthoringDocumentReadHandle::detached_for_test(document),
    )
}

fn source_state(session: &AnimationEditorSession) -> (Vec<u8>, AnimationDocumentRevision, String) {
    let document = session.document().read();
    (
        document.document_bytes().unwrap(),
        document.revision(),
        format!("{:?}", document.compilation()),
    )
}

#[test]
fn sequence_selection_admission_resolves_complete_unique_path_and_preserves_source() {
    for target_id in [Some("9007199254740993".to_string()), None] {
        let (mut asset, path, other_path) = source();
        asset.bindings[1].target_id = target_id;
        let mut session = session(asset);
        let source = source_state(&session);
        assert!(session.select_timeline_span(&path, 90, 15).unwrap());
        assert_eq!(
            session.sequence.as_ref().unwrap().selected_span,
            Some((path.clone(), 15, 60))
        );
        assert!(!session.select_timeline_span(&path, 15, 60).unwrap());
        assert_eq!(
            session.curve_foundation().unwrap().selected_track_id,
            Some(path.to_string())
        );
        assert_eq!(
            session.pane_presentation().selection_summary,
            format!("{path} [15..60]")
        );
        assert!(session.select_timeline_span(&other_path, 4, 28).unwrap());
        session.reconcile_source_change();
        assert_eq!(
            session.sequence.as_ref().unwrap().selected_span,
            Some((other_path, 4, 28))
        );
        assert_eq!(source_state(&session), source);
    }
}

#[test]
fn sequence_selection_admission_ambiguous_or_missing_keeps_previous_selection_and_transport() {
    let (mut asset, path, other_path) = source();
    duplicate_selected_track(&mut asset);
    assert!(AnimationAuthoringAsset::Sequence(asset.clone())
        .compile()
        .is_successful());
    let mut session = session(asset);
    assert!(session.select_timeline_span(&other_path, 4, 28).unwrap());
    assert!(session.scrub_timeline(15).unwrap());
    assert!(session.set_playback(true, true, 1.25).unwrap());
    let source = source_state(&session);
    let selection = session.sequence.as_ref().unwrap().selected_span.clone();
    let timeline = session.timeline_foundation().unwrap();
    let visible_selection = session.pane_presentation().selection_summary;
    for (start, end) in [(0, 30), (60, 0)] {
        let error = session.select_timeline_span(&path, start, end).unwrap_err();
        assert!(error.contains("ambiguous animation track"));
        assert!(error.contains(&path.to_string()));
        assert_eq!(session.sequence.as_ref().unwrap().selected_span, selection);
        assert_eq!(session.timeline_foundation().unwrap(), timeline);
        assert_eq!(
            session.pane_presentation().selection_summary,
            visible_selection
        );
        assert_eq!(source_state(&session), source);
    }
    let missing = AnimationTrackPath::new(
        EntityPath::parse("Root/Missing").unwrap(),
        path.split().unwrap().1,
    );
    assert!(!session.select_timeline_span(&missing, 0, 30).unwrap());
    assert_eq!(session.sequence.as_ref().unwrap().selected_span, selection);
    assert_eq!(session.timeline_foundation().unwrap(), timeline);
    assert_eq!(source_state(&session), source);
}

#[test]
fn sequence_selection_admission_replay_uses_current_source_without_another_history_entry() {
    for replacement_kind in ["unique", "ambiguous", "missing"] {
        let (original, path, _) = source();
        let original_bytes = original.to_bytes().unwrap();
        let mut replacement = original.clone();
        match replacement_kind {
            "unique" => replacement.name = Some("Unrelated source edit".to_string()),
            "ambiguous" => duplicate_selected_track(&mut replacement),
            "missing" => replacement.bindings[1].tracks.clear(),
            _ => unreachable!(),
        }
        let replacement_bytes = replacement.to_bytes().unwrap();
        assert_eq!(
            AnimationSequenceAsset::from_bytes(&replacement_bytes).unwrap(),
            replacement
        );
        assert!(AnimationAuthoringAsset::Sequence(replacement.clone())
            .compile()
            .is_successful());
        let id = DocumentId::new(9193);
        let history = HistoryContextId::Document(id);
        let mut context = CoreEditContext::default();
        let handle = context
            .animation_documents_mut()
            .attach(AnimationAuthoringDocument::new(
                id,
                AssetUri::parse("res://tests/selection-replay.sequence.zranim").unwrap(),
                AnimationAuthoringAsset::Sequence(original),
            ))
            .unwrap();
        let mut session = AnimationEditorSession::new(
            "res://tests/selection-replay.sequence.zranim".to_string(),
            handle,
        );
        assert!(session.select_timeline_span(&path, 4, 28).unwrap());
        session.scrub_timeline(15).unwrap();
        session.set_playback(true, true, 1.25).unwrap();
        let expected_selection = session.sequence.as_ref().unwrap().selected_span.clone();
        let expected_playback = session.timeline_foundation().unwrap().playback;
        let revision = session.document().read().revision();
        let engine = EditorTransactionEngine::new(context);
        assert!(!engine.is_dirty(history).unwrap());
        let mut transaction = engine
            .begin("Replace source for selection replay", history)
            .unwrap();
        transaction
            .push(AnimationEditCommand::new(
                "Replace source for selection replay",
                id,
                revision,
                AnimationAuthoringAsset::Sequence(replacement),
            ))
            .unwrap();
        transaction.commit().unwrap();
        let source = source_state(&session);
        let history_before = format!("{:?}", engine.history_status(history).unwrap());
        session.reconcile_source_change();
        assert_eq!(source_state(&session), source);
        assert_eq!(
            format!("{:?}", engine.history_status(history).unwrap()),
            history_before
        );
        assert_eq!(session.document_bytes().unwrap(), replacement_bytes);
        assert_eq!(
            session.pane_presentation().selection_summary.is_empty(),
            replacement_kind != "unique"
        );
        assert_eq!(
            session.timeline_foundation().unwrap().playback,
            expected_playback
        );
        assert_eq!(
            session.sequence.as_ref().unwrap().selected_span,
            if replacement_kind == "unique" {
                expected_selection.clone()
            } else {
                None
            }
        );
        assert!(engine.is_dirty(history).unwrap());

        assert!(engine.undo(history).unwrap());
        session.reconcile_source_change();
        assert_eq!(session.document_bytes().unwrap(), original_bytes);
        assert!(!engine.is_dirty(history).unwrap());
        // Undo restores source, not a retired transient selection.
        assert_eq!(
            session.sequence.as_ref().unwrap().selected_span,
            if replacement_kind == "unique" {
                expected_selection
            } else {
                None
            }
        );
        session.select_timeline_span(&path, 4, 28).unwrap();
        assert!(engine.redo(history).unwrap());
        session.reconcile_source_change();
        assert_eq!(session.document_bytes().unwrap(), replacement_bytes);
        assert_eq!(
            session.sequence.as_ref().unwrap().selected_span.is_some(),
            replacement_kind == "unique"
        );
        assert_eq!(
            session.timeline_foundation().unwrap().playback,
            expected_playback
        );
        assert!(engine.is_dirty(history).unwrap());
    }
}
