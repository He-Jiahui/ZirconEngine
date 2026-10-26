use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationDocumentMutationError, AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

const TARGET: &str = "9007199254740993";
const OTHER_TARGET: &str = "9007199254740995";
const SPEED: &str = "AnimationPlayer.playback_speed";
const TIME: &str = "AnimationPlayer.time_seconds";
const WEIGHT: &str = "AnimationPlayer.weight";

fn track(property: &str) -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse(property).unwrap(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Step,
            keys: vec![AnimationChannelKeyAsset {
                time_seconds: 0.0,
                value: AnimationChannelValueAsset::Scalar(0.0),
                in_tangent: None,
                out_tangent: None,
            }],
        },
    }
}

fn binding(
    entity: &str,
    target: Option<&str>,
    tracks: Vec<AnimationSequenceTrackAsset>,
) -> AnimationSequenceBindingAsset {
    AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse(entity).unwrap(),
        target_id: target.map(str::to_string),
        tracks,
    }
}

fn source(bindings: Vec<AnimationSequenceBindingAsset>) -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("Create track binding admission".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings,
    }
}

fn requested_path() -> AnimationTrackPath {
    AnimationTrackPath::new(
        EntityPath::parse("Root/Hero").unwrap(),
        ComponentPropertyPath::parse(WEIGHT).unwrap(),
    )
}

fn ambiguous_sources() -> Vec<AnimationSequenceAsset> {
    let mut sources = Vec::new();
    for (first, second) in [
        (Some(TARGET), None),
        (None, Some(OTHER_TARGET)),
        (Some(TARGET), Some(TARGET)),
        (None, None),
    ] {
        for leading_empty in [true, false] {
            for reverse in [true, false] {
                let mut heroes = vec![
                    binding(
                        "Root/Hero",
                        first,
                        if leading_empty {
                            vec![]
                        } else {
                            vec![track(SPEED)]
                        },
                    ),
                    binding("Root/Hero", second, vec![track(TIME)]),
                ];
                if reverse {
                    heroes.reverse();
                }
                let mut bindings = vec![binding("Root/Before", Some(OTHER_TARGET), vec![])];
                bindings.extend(heroes);
                bindings.push(binding("Root/Other", None, vec![track(WEIGHT)]));
                sources.push(source(bindings));
            }
        }
    }
    sources
}

fn assert_source(actual: &AnimationSequenceAsset, expected: &AnimationSequenceAsset) {
    assert_eq!(actual, expected);
    let bytes = actual.to_bytes().unwrap();
    assert_eq!(bytes, expected.to_bytes().unwrap());
    assert_eq!(
        AnimationSequenceAsset::from_bytes(&bytes).unwrap(),
        *expected
    );
    assert!(AnimationAuthoringAsset::Sequence(actual.clone())
        .compile()
        .is_successful());
}

fn engine_for(id: DocumentId, original: &AnimationSequenceAsset) -> EditorTransactionEngine {
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/create-admission.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    EditorTransactionEngine::new(context)
}

fn assert_document(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    revision: u64,
    expected: &AnimationSequenceAsset,
) {
    engine
        .with_context::<CoreEditContext, _>(|context| {
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            assert_eq!(document.revision().value(), revision);
            assert_source(document.asset().as_sequence().unwrap(), expected);
            assert_eq!(
                document.document_bytes().unwrap(),
                expected.to_bytes().unwrap()
            );
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
fn sequence_create_admission_rejects_multiple_bindings_before_any_source_write() {
    for original in ambiguous_sources() {
        let mut candidate = original.clone();
        let result = super::create_track(&mut candidate, &requested_path());
        assert!(matches!(
            result,
            Err(AnimationDocumentMutationError::InvalidTrackPath { message })
                if message.contains("ambiguous animation binding")
                    && message.contains("Root/Hero")
        ));
        assert_source(&candidate, &original);
    }
}

#[test]
fn sequence_create_admission_rejection_keeps_document_compilation_and_history() {
    for original in ambiguous_sources() {
        let id = DocumentId::new(9215);
        let engine = engine_for(id, &original);
        let history = HistoryContextId::Document(id);
        let before_history = engine.history_status(history).unwrap();
        let mutation = AnimationDocumentMutation::CreateTrack {
            track_path: requested_path(),
        };
        engine
            .with_context::<CoreEditContext, _>(|context| {
                assert!(matches!(
                    context
                        .animation_documents()
                        .prepare_mutation(id, &mutation),
                    Err(AnimationDocumentMutationError::InvalidTrackPath { .. })
                ));
            })
            .unwrap()
            .unwrap();
        assert_document(&engine, id, 1, &original);
        assert_eq!(engine.history_status(history).unwrap(), before_history);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(!engine.redo(history).unwrap());
    }
}

#[test]
fn sequence_create_admission_zero_or_unique_binding_commits_once_and_undoes_exact_source() {
    let mut cases = Vec::new();
    for target in [Some(TARGET), None] {
        for empty in [true, false] {
            let original = source(vec![
                binding("Root/Before", Some(OTHER_TARGET), vec![]),
                binding(
                    "Root/Hero",
                    target,
                    if empty { vec![] } else { vec![track(SPEED)] },
                ),
                binding("Root/Other", None, vec![track(WEIGHT)]),
            ]);
            let mut expected = original.clone();
            expected.bindings[1].tracks.push(track(WEIGHT));
            cases.push((original, expected));
        }
    }
    let original = source(vec![binding(
        "Root/Other",
        Some(OTHER_TARGET),
        vec![track(WEIGHT)],
    )]);
    let mut expected = original.clone();
    expected
        .bindings
        .push(binding("Root/Hero", None, vec![track(WEIGHT)]));
    cases.push((original, expected));
    for (original, expected) in cases {
        let id = DocumentId::new(9216);
        let engine = engine_for(id, &original);
        let history = HistoryContextId::Document(id);
        let mutation = AnimationDocumentMutation::CreateTrack {
            track_path: requested_path(),
        };
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
        let committed_history = engine.history_status(history).unwrap();
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
        assert_eq!(engine.history_status(history).unwrap(), committed_history);
        assert_document(&engine, id, 2, &expected);
        assert!(engine.undo(history).unwrap());
        assert_document(&engine, id, 3, &original);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(engine.redo(history).unwrap());
        assert_document(&engine, id, 4, &expected);
        assert!(engine.is_dirty(history).unwrap());
        assert!(!engine.redo(history).unwrap());
    }
}
