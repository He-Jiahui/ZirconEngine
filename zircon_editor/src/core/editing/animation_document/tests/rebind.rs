use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationEditCommand,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

const SPEED: &str = "AnimationPlayer.playback_speed";
const WEIGHT: &str = "AnimationPlayer.weight";
const TIME: &str = "AnimationPlayer.time_seconds";

fn track(property: &str) -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse(property).unwrap(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Hermite,
            keys: [(0.0, 1.0), (2.0, 9.0)]
                .into_iter()
                .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                    time_seconds,
                    value: AnimationChannelValueAsset::Scalar(value),
                    in_tangent: Some(AnimationChannelValueAsset::Scalar(-0.25)),
                    out_tangent: Some(AnimationChannelValueAsset::Scalar(0.75)),
                })
                .collect(),
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

fn path(entity: &str, property: &str) -> AnimationTrackPath {
    AnimationTrackPath::new(
        EntityPath::parse(entity).unwrap(),
        ComponentPropertyPath::parse(property).unwrap(),
    )
}

fn source(
    keep_owner_track: bool,
    owner_target: Option<&str>,
    destination: Option<Option<&str>>,
) -> AnimationSequenceAsset {
    let mut owner_tracks = vec![track(SPEED)];
    if keep_owner_track {
        owner_tracks.push(track(TIME));
    }
    let mut bindings = vec![
        binding("Root/Hero", Some("9007199254740995"), Vec::new()),
        binding("Root/Hero", owner_target, owner_tracks),
        binding("Root/Other", Some("unrelated-empty-target"), Vec::new()),
        binding("Root/Hero", None, Vec::new()),
    ];
    if let Some(target) = destination {
        bindings.push(binding("Root/Destination", target, vec![track(TIME)]));
    }
    AnimationSequenceAsset {
        name: Some("Cross-entity rebind isolation".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings,
    }
}

fn expected_move(
    original: &AnimationSequenceAsset,
    keep_owner_track: bool,
) -> AnimationSequenceAsset {
    let mut moved = original.bindings[1].tracks[0].clone();
    moved.property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
    let mut bindings = vec![original.bindings[0].clone()];
    if keep_owner_track {
        let mut owner = original.bindings[1].clone();
        owner.tracks = vec![original.bindings[1].tracks[1].clone()];
        bindings.push(owner);
    }
    bindings.push(original.bindings[2].clone());
    bindings.push(original.bindings[3].clone());
    if let Some(destination) = original.bindings.get(4) {
        let mut destination = destination.clone();
        destination.tracks.push(moved);
        bindings.push(destination);
    } else {
        bindings.push(binding("Root/Destination", None, vec![moved]));
    }
    let mut expected = original.clone();
    expected.bindings = bindings;
    expected
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

#[test]
fn sequence_rebind_isolation_keeps_nonowning_empty_bindings_and_source_order() {
    for keep_owner_track in [false, true] {
        for owner_target in [None, Some("9007199254740993")] {
            for destination in [None, Some(None), Some(Some("9007199254740997"))] {
                let mut actual = source(keep_owner_track, owner_target, destination);
                let expected = expected_move(&actual, keep_owner_track);
                assert!(AnimationAuthoringAsset::Sequence(actual.clone())
                    .compile()
                    .is_successful());
                assert!(super::rebind_track(
                    &mut actual,
                    &path("Root/Hero", SPEED),
                    &path("Root/Destination", WEIGHT),
                )
                .unwrap());
                assert_source(&actual, &expected);
            }
        }
    }
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
fn sequence_rebind_isolation_prepares_once_and_undo_redo_restore_all_target_identities() {
    for keep_owner_track in [false, true] {
        for destination in [None, Some(None), Some(Some("9007199254740997"))] {
            let original = source(keep_owner_track, Some("9007199254740993"), destination);
            let expected = expected_move(&original, keep_owner_track);
            let id = DocumentId::new(9193);
            let history = HistoryContextId::Document(id);
            let mutation = AnimationDocumentMutation::RebindTrack {
                from_track_path: path("Root/Hero", SPEED),
                to_track_path: path("Root/Destination", WEIGHT),
            };
            let mut context = CoreEditContext::default();
            context
                .animation_documents_mut()
                .attach(AnimationAuthoringDocument::new(
                    id,
                    AssetUri::parse("res://animation/rebind-isolation.sequence.zranim").unwrap(),
                    AnimationAuthoringAsset::Sequence(original.clone()),
                ))
                .unwrap();
            let (revision, replacement) = context
                .animation_documents()
                .prepare_mutation(id, &mutation)
                .unwrap()
                .unwrap();
            assert_source(replacement.as_sequence().unwrap(), &expected);
            let engine = EditorTransactionEngine::new(context);
            assert_document(&engine, id, 1, &original);
            assert!(!engine.is_dirty(history).unwrap());
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
}
