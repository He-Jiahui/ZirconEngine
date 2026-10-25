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

const HERO_TARGET: &str = "9007199254740993";
const OTHER_TARGET: &str = "9007199254740995";
const SPEED: &str = "AnimationPlayer.playback_speed";
const WEIGHT: &str = "AnimationPlayer.weight";
const TIME: &str = "AnimationPlayer.time_seconds";

fn track(property: &str) -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse(property).unwrap(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Hermite,
            keys: vec![
                AnimationChannelKeyAsset {
                    time_seconds: 0.0,
                    value: AnimationChannelValueAsset::Scalar(1.0),
                    in_tangent: Some(AnimationChannelValueAsset::Scalar(0.25)),
                    out_tangent: Some(AnimationChannelValueAsset::Scalar(0.5)),
                },
                AnimationChannelKeyAsset {
                    time_seconds: 2.0,
                    value: AnimationChannelValueAsset::Scalar(2.0),
                    in_tangent: Some(AnimationChannelValueAsset::Scalar(-0.25)),
                    out_tangent: Some(AnimationChannelValueAsset::Scalar(0.75)),
                },
            ],
        },
    }
}

fn source(target: Option<&str>) -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("Rebind target preservation".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: target.map(str::to_string),
            tracks: vec![track(SPEED)],
        }],
    }
}

fn path(entity: &str, property: &str) -> AnimationTrackPath {
    AnimationTrackPath::new(
        EntityPath::parse(entity).unwrap(),
        ComponentPropertyPath::parse(property).unwrap(),
    )
}

#[test]
fn sequence_rebind_property_preserves_target_and_binary_source() {
    for target in [Some(HERO_TARGET), None] {
        let mut asset = source(target);
        let mut expected = asset.clone();
        expected.bindings[0].tracks[0].property_path =
            ComponentPropertyPath::parse(WEIGHT).unwrap();

        assert!(super::rebind_track(
            &mut asset,
            &path("Root/Hero", SPEED),
            &path("Root/Hero", WEIGHT),
        )
        .unwrap());
        assert_eq!(asset, expected);
        let reopened = AnimationSequenceAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap();
        assert_eq!(reopened, expected);
        assert!(AnimationAuthoringAsset::Sequence(reopened)
            .compile()
            .is_successful());
    }
}

#[test]
fn sequence_rebind_property_preserves_track_order_and_unrelated_bindings() {
    let mut asset = source(Some(HERO_TARGET));
    asset.bindings[0].tracks.push(track(TIME));
    asset.bindings.push(AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse("Root/Other").unwrap(),
        target_id: Some(OTHER_TARGET.to_string()),
        tracks: vec![track(WEIGHT)],
    });
    let mut expected = asset.clone();
    expected.bindings[0].tracks[0].property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();

    assert!(super::rebind_track(
        &mut asset,
        &path("Root/Hero", SPEED),
        &path("Root/Hero", WEIGHT),
    )
    .unwrap());
    assert_eq!(asset, expected);
}

#[test]
fn sequence_rebind_noop_and_collision_leave_source_unchanged() {
    let mut original = source(Some(HERO_TARGET));
    original.bindings[0].tracks.push(track(WEIGHT));
    original.bindings.push(AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse("Root/Other").unwrap(),
        target_id: Some(OTHER_TARGET.to_string()),
        tracks: vec![track(WEIGHT)],
    });
    for (from, to) in [
        (path("Root/Hero", SPEED), path("Root/Hero", SPEED)),
        (path("Root/Hero", SPEED), path("Root/Hero", WEIGHT)),
        (path("Root/Hero", TIME), path("Root/Hero", SPEED)),
        (
            path("Root/Hero", TIME),
            path("Root/Hero", "AnimationPlayer.looping"),
        ),
        (path("Root/Hero", SPEED), path("Root/Other", WEIGHT)),
        (path("Root/Missing", SPEED), path("Root/Hero", TIME)),
    ] {
        let mut asset = original.clone();
        assert!(!super::rebind_track(&mut asset, &from, &to).unwrap());
        assert_eq!(asset, original);
    }
}

#[test]
fn sequence_rebind_cross_entity_keeps_destination_target_authority() {
    for destination in [None, Some(OTHER_TARGET)] {
        let mut asset = source(Some(HERO_TARGET));
        asset.bindings[0].tracks.push(track(TIME));
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Other").unwrap(),
            target_id: destination.map(str::to_string),
            tracks: vec![track(TIME)],
        });
        let mut expected = asset.clone();
        let mut moved = expected.bindings[0].tracks.remove(0);
        moved.property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
        expected.bindings[1].tracks.push(moved);

        assert!(super::rebind_track(
            &mut asset,
            &path("Root/Hero", SPEED),
            &path("Root/Other", WEIGHT),
        )
        .unwrap());
        assert_eq!(asset, expected);
    }

    let mut asset = source(Some(HERO_TARGET));
    let mut expected = asset.clone();
    expected.bindings[0].entity_path = EntityPath::parse("Root/New").unwrap();
    expected.bindings[0].target_id = None;
    expected.bindings[0].tracks[0].property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
    assert!(super::rebind_track(
        &mut asset,
        &path("Root/Hero", SPEED),
        &path("Root/New", WEIGHT),
    )
    .unwrap());
    assert_eq!(asset, expected);
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
            assert_eq!(document.asset().as_sequence(), Some(expected));
            let bytes = document.document_bytes().unwrap();
            assert_eq!(bytes, expected.to_bytes().unwrap());
            assert_eq!(
                AnimationSequenceAsset::from_bytes(&bytes).unwrap(),
                *expected
            );
            let compilation = document.compilation();
            assert_eq!(compilation.current_revision(), document.revision());
            assert!(compilation.current_product().is_successful());
            assert_eq!(compilation.last_good_revision(), Some(document.revision()));
        })
        .unwrap()
        .unwrap();
}

#[test]
fn sequence_rebind_history_preserves_target_through_undo_redo() {
    let id = DocumentId::new(9182);
    let original = source(Some(HERO_TARGET));
    let mut expected = original.clone();
    expected.bindings[0].tracks[0].property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/rebind.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let mutation = AnimationDocumentMutation::RebindTrack {
        from_track_path: path("Root/Hero", SPEED),
        to_track_path: path("Root/Hero", WEIGHT),
    };
    let (revision, replacement) = context
        .animation_documents()
        .prepare_mutation(id, &mutation)
        .unwrap()
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
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
}

#[test]
fn sequence_binding_remove_last_track_preserves_target_and_codec() {
    for target in [Some(HERO_TARGET), None] {
        let mut asset = source(target);
        let mut expected = asset.clone();
        expected.bindings[0].tracks.clear();

        assert!(super::remove_track(&mut asset, &path("Root/Hero", SPEED)).unwrap());
        assert_eq!(asset, expected);
        let reopened = AnimationSequenceAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap();
        assert_eq!(reopened, expected);
        assert!(AnimationAuthoringAsset::Sequence(reopened)
            .compile()
            .is_successful());
    }
}

#[test]
fn sequence_binding_recreate_track_uses_retained_target() {
    for property in [SPEED, WEIGHT] {
        let mut asset = source(Some(HERO_TARGET));
        asset.bindings.insert(
            0,
            AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("Root/Before").unwrap(),
                target_id: None,
                tracks: vec![track(TIME)],
            },
        );
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Other").unwrap(),
            target_id: Some(OTHER_TARGET.to_string()),
            tracks: vec![track(WEIGHT)],
        });
        let original = asset.clone();

        assert!(super::remove_track(&mut asset, &path("Root/Hero", SPEED)).unwrap());
        assert!(super::create_track(&mut asset, &path("Root/Hero", property)).unwrap());
        assert_eq!(asset.bindings.len(), original.bindings.len());
        assert_eq!(asset.bindings[0], original.bindings[0]);
        assert_eq!(asset.bindings[2], original.bindings[2]);
        assert_eq!(
            asset.bindings[1].entity_path,
            original.bindings[1].entity_path
        );
        assert_eq!(asset.bindings[1].target_id.as_deref(), Some(HERO_TARGET));
        assert_eq!(asset.bindings[1].tracks.len(), 1);
        assert_eq!(
            asset.bindings[1].tracks[0].property_path,
            ComponentPropertyPath::parse(property).unwrap()
        );
        let reopened = AnimationSequenceAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap();
        assert_eq!(reopened, asset);
        assert!(AnimationAuthoringAsset::Sequence(reopened)
            .compile()
            .is_successful());
    }
}

#[test]
fn sequence_binding_missing_remove_preserves_existing_empty_bindings() {
    let mut original = source(Some(HERO_TARGET));
    original.bindings[0].tracks.clear();
    original.bindings.push(AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse("Root/Other").unwrap(),
        target_id: Some(OTHER_TARGET.to_string()),
        tracks: Vec::new(),
    });
    let original_bytes = original.to_bytes().unwrap();
    for missing in [path("Root/Hero", SPEED), path("Root/Missing", WEIGHT)] {
        let mut asset = original.clone();
        assert!(!super::remove_track(&mut asset, &missing).unwrap());
        assert_eq!(asset, original);
        assert_eq!(asset.to_bytes().unwrap(), original_bytes);
    }
}

#[test]
fn sequence_binding_remove_recreate_history_keeps_target() {
    let id = DocumentId::new(9183);
    let original = source(Some(HERO_TARGET));
    let mut empty = original.clone();
    empty.bindings[0].tracks.clear();
    let mut recreated = empty.clone();
    assert!(super::create_track(&mut recreated, &path("Root/Hero", WEIGHT)).unwrap());
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/binding.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let apply = |mutation: AnimationDocumentMutation| {
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
    };

    apply(AnimationDocumentMutation::RemoveTrack {
        track_path: path("Root/Hero", SPEED),
    });
    assert_document(&engine, id, 2, &empty);
    assert!(engine.is_dirty(history).unwrap());
    apply(AnimationDocumentMutation::CreateTrack {
        track_path: path("Root/Hero", WEIGHT),
    });
    assert_document(&engine, id, 3, &recreated);
    assert!(engine.is_dirty(history).unwrap());

    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 4, &empty);
    assert!(engine.is_dirty(history).unwrap());
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 5, &original);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 6, &empty);
    assert!(engine.is_dirty(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, 7, &recreated);
    assert!(engine.is_dirty(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}

fn missing_rebind_sources() -> Vec<(AnimationSequenceAsset, AnimationTrackPath)> {
    let mut cases = Vec::new();
    for source_target in [Some(HERO_TARGET), None] {
        for source_empty in [true, false] {
            for (destination, destination_target, destination_exists) in [
                ("Root/New", None, false),
                ("Root/Other", Some(OTHER_TARGET), true),
                ("Root/Other", None, true),
            ] {
                let mut asset = source(source_target);
                asset.bindings[0].tracks = if source_empty {
                    Vec::new()
                } else {
                    vec![track(TIME)]
                };
                asset.bindings.insert(
                    0,
                    AnimationSequenceBindingAsset {
                        entity_path: EntityPath::parse("Root/Before").unwrap(),
                        target_id: None,
                        tracks: vec![track(WEIGHT)],
                    },
                );
                asset.bindings.push(AnimationSequenceBindingAsset {
                    entity_path: EntityPath::parse("Root/After").unwrap(),
                    target_id: None,
                    tracks: Vec::new(),
                });
                if destination_exists {
                    asset.bindings.push(AnimationSequenceBindingAsset {
                        entity_path: EntityPath::parse(destination).unwrap(),
                        target_id: destination_target.map(str::to_string),
                        tracks: vec![track(TIME)],
                    });
                }
                cases.push((asset, path(destination, WEIGHT)));
            }
        }
    }
    cases
}

#[test]
fn sequence_rebind_missing_track_preserves_bindings_and_binary_source() {
    for (mut asset, to_track_path) in missing_rebind_sources() {
        let original = asset.clone();
        let original_bytes = original.to_bytes().unwrap();
        assert!(AnimationAuthoringAsset::Sequence(original.clone())
            .compile()
            .is_successful());

        assert!(
            !super::rebind_track(&mut asset, &path("Root/Hero", SPEED), &to_track_path,).unwrap()
        );
        assert_eq!(asset, original);
        let unchanged_bytes = asset.to_bytes().unwrap();
        assert_eq!(unchanged_bytes, original_bytes);
        assert_eq!(
            AnimationSequenceAsset::from_bytes(&unchanged_bytes).unwrap(),
            original
        );
    }
}

#[test]
fn sequence_rebind_missing_track_prepares_no_replacement_or_history() {
    for (original, to_track_path) in missing_rebind_sources() {
        let id = DocumentId::new(9184);
        let mut context = CoreEditContext::default();
        context
            .animation_documents_mut()
            .attach(AnimationAuthoringDocument::new(
                id,
                AssetUri::parse("res://animation/rebind-noop.sequence.zranim").unwrap(),
                AnimationAuthoringAsset::Sequence(original.clone()),
            ))
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let history = HistoryContextId::Document(id);
        let before = engine.history_status(history).unwrap();
        assert_document(&engine, id, 1, &original);

        engine
            .with_context::<CoreEditContext, _>(|context| {
                assert!(context
                    .animation_documents()
                    .prepare_mutation(
                        id,
                        &AnimationDocumentMutation::RebindTrack {
                            from_track_path: path("Root/Hero", SPEED),
                            to_track_path,
                        },
                    )
                    .unwrap()
                    .is_none());
            })
            .unwrap()
            .unwrap();
        assert_document(&engine, id, 1, &original);
        assert_eq!(engine.history_status(history).unwrap(), before);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(!engine.redo(history).unwrap());
        assert_eq!(engine.history_status(history).unwrap(), before);
        assert_document(&engine, id, 1, &original);
    }
}

#[test]
fn sequence_rebind_source_preflight_scans_all_matching_bindings() {
    for destination_target in [Some(OTHER_TARGET), None] {
        let mut asset = source(Some(HERO_TARGET));
        asset.bindings.insert(
            0,
            AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("Root/Hero").unwrap(),
                target_id: None,
                tracks: Vec::new(),
            },
        );
        asset.bindings.push(AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Other").unwrap(),
            target_id: destination_target.map(str::to_string),
            tracks: vec![track(TIME)],
        });
        let mut expected_track = asset.bindings[1].tracks[0].clone();
        expected_track.property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
        assert!(AnimationAuthoringAsset::Sequence(asset.clone())
            .compile()
            .is_successful());

        assert!(super::rebind_track(
            &mut asset,
            &path("Root/Hero", SPEED),
            &path("Root/Other", WEIGHT),
        )
        .unwrap());
        let destination = asset
            .bindings
            .iter()
            .find(|binding| binding.entity_path == EntityPath::parse("Root/Other").unwrap())
            .unwrap();
        assert_eq!(destination.target_id.as_deref(), destination_target);
        assert_eq!(destination.tracks, vec![track(TIME), expected_track]);
    }
}

fn sequence_binding_lookup_sources() -> Vec<AnimationSequenceAsset> {
    let mut sources = Vec::new();
    for target in [Some(HERO_TARGET), None] {
        for leading_empty in [true, false] {
            let mut asset = source(target);
            asset.bindings.insert(
                0,
                AnimationSequenceBindingAsset {
                    entity_path: EntityPath::parse("Root/Hero").unwrap(),
                    target_id: Some(OTHER_TARGET.to_string()),
                    tracks: if leading_empty {
                        Vec::new()
                    } else {
                        vec![track(TIME)]
                    },
                },
            );
            asset.bindings.push(AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("Root/Other").unwrap(),
                target_id: None,
                tracks: vec![track(WEIGHT)],
            });
            assert!(AnimationAuthoringAsset::Sequence(asset.clone())
                .compile()
                .is_successful());
            sources.push(asset);
        }
    }
    sources
}

fn assert_sequence_binding_lookup_source(
    actual: &AnimationSequenceAsset,
    expected: &AnimationSequenceAsset,
) {
    assert_eq!(actual, expected);
    let bytes = actual.to_bytes().unwrap();
    assert_eq!(bytes, expected.to_bytes().unwrap());
    let reopened = AnimationSequenceAsset::from_bytes(&bytes).unwrap();
    assert_eq!(&reopened, expected);
    assert!(AnimationAuthoringAsset::Sequence(reopened)
        .compile()
        .is_successful());
}

#[test]
fn sequence_binding_lookup_rebinds_the_later_owning_binding() {
    for mut asset in sequence_binding_lookup_sources() {
        let mut expected = asset.clone();
        expected.bindings[1].tracks[0].property_path =
            ComponentPropertyPath::parse(WEIGHT).unwrap();
        assert!(super::rebind_track(
            &mut asset,
            &path("Root/Hero", SPEED),
            &path("Root/Hero", WEIGHT),
        )
        .unwrap());
        assert_sequence_binding_lookup_source(&asset, &expected);
    }
}

#[test]
fn sequence_binding_lookup_adds_and_removes_keys_in_the_later_binding() {
    for original in sequence_binding_lookup_sources() {
        let mut asset = original.clone();
        let mut added = original.clone();
        added.bindings[1].tracks[0].channel.keys.insert(
            1,
            AnimationChannelKeyAsset {
                time_seconds: 1.0,
                value: AnimationChannelValueAsset::Scalar(2.0),
                in_tangent: None,
                out_tangent: None,
            },
        );
        assert!(super::add_key(&mut asset, &path("Root/Hero", SPEED), 30).unwrap());
        assert_sequence_binding_lookup_source(&asset, &added);
        assert!(!super::add_key(&mut asset, &path("Root/Hero", SPEED), 30).unwrap());
        assert_sequence_binding_lookup_source(&asset, &added);
        assert!(super::remove_key(&mut asset, &path("Root/Hero", SPEED), 30).unwrap());
        assert_sequence_binding_lookup_source(&asset, &original);
        let mut removed = original.clone();
        removed.bindings[1].tracks[0].channel.keys.remove(1);
        assert!(super::remove_key(&mut asset, &path("Root/Hero", SPEED), 60).unwrap());
        assert_sequence_binding_lookup_source(&asset, &removed);
        assert!(!super::remove_key(&mut asset, &path("Root/Hero", SPEED), 120).unwrap());
        assert_sequence_binding_lookup_source(&asset, &removed);
    }
}

#[test]
fn sequence_binding_lookup_create_existing_is_noop_without_history() {
    for original in sequence_binding_lookup_sources() {
        let mut asset = original.clone();
        assert!(!super::create_track(&mut asset, &path("Root/Hero", SPEED)).unwrap());
        assert_sequence_binding_lookup_source(&asset, &original);
        let id = DocumentId::new(9185);
        let mut context = CoreEditContext::default();
        context
            .animation_documents_mut()
            .attach(AnimationAuthoringDocument::new(
                id,
                AssetUri::parse("res://animation/binding-lookup.sequence.zranim").unwrap(),
                AnimationAuthoringAsset::Sequence(original.clone()),
            ))
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let history = HistoryContextId::Document(id);
        let before = engine.history_status(history).unwrap();
        engine
            .with_context::<CoreEditContext, _>(|context| {
                assert!(context
                    .animation_documents()
                    .prepare_mutation(
                        id,
                        &AnimationDocumentMutation::CreateTrack {
                            track_path: path("Root/Hero", SPEED),
                        }
                    )
                    .unwrap()
                    .is_none());
            })
            .unwrap()
            .unwrap();
        assert_document(&engine, id, 1, &original);
        assert_eq!(engine.history_status(history).unwrap(), before);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(!engine.redo(history).unwrap());
        // The existing full path remains a no-op, but a missing property does not
        // identify which of these two source bindings should receive a new track.
        assert!(super::create_track(&mut asset, &path("Root/Hero", WEIGHT)).is_err());
        assert_sequence_binding_lookup_source(&asset, &original);
    }
}

#[test]
fn sequence_binding_lookup_remove_keeps_the_later_empty_target() {
    for original in sequence_binding_lookup_sources() {
        let mut asset = original.clone();
        let mut expected = original.clone();
        expected.bindings[1].tracks.clear();
        assert!(super::remove_track(&mut asset, &path("Root/Hero", SPEED)).unwrap());
        assert_sequence_binding_lookup_source(&asset, &expected);
        assert!(!super::remove_track(&mut asset, &path("Root/Hero", SPEED)).unwrap());
        assert!(!super::rebind_track(
            &mut asset,
            &path("Root/Hero", SPEED),
            &path("Root/New", WEIGHT)
        )
        .unwrap());
        assert!(super::add_key(&mut asset, &path("Root/Hero", SPEED), 30).is_err());
        assert!(super::remove_key(&mut asset, &path("Root/Hero", SPEED), 30).is_err());
        assert_sequence_binding_lookup_source(&asset, &expected);
    }
}

#[test]
fn sequence_binding_lookup_ambiguous_commands_preserve_source_and_history() {
    for first_target in [Some(HERO_TARGET), None] {
        for second_target in [Some(OTHER_TARGET), None] {
            let mut original = source(first_target);
            original.bindings.push(AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("Root/Hero").unwrap(),
                target_id: second_target.map(str::to_string),
                tracks: vec![track(SPEED)],
            });
            original.bindings.push(AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse("Root/Other").unwrap(),
                target_id: second_target.map(str::to_string),
                tracks: vec![track(TIME)],
            });
            assert!(AnimationAuthoringAsset::Sequence(original.clone())
                .compile()
                .is_successful());
            let original_bytes = original.to_bytes().unwrap();
            let id = DocumentId::new(9186);
            let mut context = CoreEditContext::default();
            context
                .animation_documents_mut()
                .attach(AnimationAuthoringDocument::new(
                    id,
                    AssetUri::parse("res://animation/binding-ambiguous.sequence.zranim").unwrap(),
                    AnimationAuthoringAsset::Sequence(original.clone()),
                ))
                .unwrap();
            let engine = EditorTransactionEngine::new(context);
            let history = HistoryContextId::Document(id);
            let before = engine.history_status(history).unwrap();
            for mutation in [
                AnimationDocumentMutation::AddKey {
                    track_path: path("Root/Hero", SPEED),
                    frame: 30,
                },
                AnimationDocumentMutation::RemoveKey {
                    track_path: path("Root/Hero", SPEED),
                    frame: 60,
                },
                AnimationDocumentMutation::CreateTrack {
                    track_path: path("Root/Hero", SPEED),
                },
                AnimationDocumentMutation::RemoveTrack {
                    track_path: path("Root/Hero", SPEED),
                },
                AnimationDocumentMutation::RebindTrack {
                    from_track_path: path("Root/Hero", SPEED),
                    to_track_path: path("Root/Hero", WEIGHT),
                },
                AnimationDocumentMutation::RebindTrack {
                    from_track_path: path("Root/Hero", SPEED),
                    to_track_path: path("Root/New", WEIGHT),
                },
                AnimationDocumentMutation::RebindTrack {
                    from_track_path: path("Root/Hero", SPEED),
                    to_track_path: path("Root/Other", WEIGHT),
                },
            ] {
                let mut asset = AnimationAuthoringAsset::Sequence(original.clone());
                let error = mutation.apply(&mut asset).unwrap_err();
                assert!(error.to_string().contains("ambiguous animation track"));
                assert_eq!(asset.as_sequence(), Some(&original));
                assert_eq!(asset.to_bytes().unwrap(), original_bytes);
                engine
                    .with_context::<CoreEditContext, _>(|context| {
                        let error = context
                            .animation_documents()
                            .prepare_mutation(id, &mutation)
                            .unwrap_err();
                        assert!(error.to_string().contains("ambiguous animation track"));
                    })
                    .unwrap()
                    .unwrap();
                assert_document(&engine, id, 1, &original);
                assert_eq!(engine.history_status(history).unwrap(), before);
                assert!(!engine.is_dirty(history).unwrap());
            }
            assert!(!engine.undo(history).unwrap());
            assert!(!engine.redo(history).unwrap());
            assert_eq!(engine.history_status(history).unwrap(), before);
            assert_document(&engine, id, 1, &original);
        }
    }
}

#[test]
fn sequence_binding_lookup_history_uses_one_original_transaction_per_edit() {
    for original in sequence_binding_lookup_sources() {
        let mut added = original.clone();
        added.bindings[1].tracks[0].channel.keys.insert(
            1,
            AnimationChannelKeyAsset {
                time_seconds: 1.0,
                value: AnimationChannelValueAsset::Scalar(2.0),
                in_tangent: None,
                out_tangent: None,
            },
        );
        let mut rebound = added.clone();
        rebound.bindings[1].tracks[0].property_path = ComponentPropertyPath::parse(WEIGHT).unwrap();
        let mut removed = rebound.clone();
        removed.bindings[1].tracks[0].channel.keys.remove(1);
        let states = [original.clone(), added, rebound, removed];
        let id = DocumentId::new(9187);
        let mut context = CoreEditContext::default();
        context
            .animation_documents_mut()
            .attach(AnimationAuthoringDocument::new(
                id,
                AssetUri::parse("res://animation/binding-history.sequence.zranim").unwrap(),
                AnimationAuthoringAsset::Sequence(original.clone()),
            ))
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let history = HistoryContextId::Document(id);
        for (index, mutation) in [
            AnimationDocumentMutation::AddKey {
                track_path: path("Root/Hero", SPEED),
                frame: 30,
            },
            AnimationDocumentMutation::RebindTrack {
                from_track_path: path("Root/Hero", SPEED),
                to_track_path: path("Root/Hero", WEIGHT),
            },
            AnimationDocumentMutation::RemoveKey {
                track_path: path("Root/Hero", WEIGHT),
                frame: 30,
            },
        ]
        .into_iter()
        .enumerate()
        {
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
            assert_document(&engine, id, 2 + index as u64, &states[index + 1]);
            assert!(engine.is_dirty(history).unwrap());
        }
        for (index, expected) in states[..3].iter().rev().enumerate() {
            assert!(engine.undo(history).unwrap());
            assert_document(&engine, id, 5 + index as u64, expected);
            assert_eq!(engine.is_dirty(history).unwrap(), index < 2);
        }
        assert!(!engine.undo(history).unwrap());
        for (index, expected) in states[1..].iter().enumerate() {
            assert!(engine.redo(history).unwrap());
            assert_document(&engine, id, 8 + index as u64, expected);
            assert!(engine.is_dirty(history).unwrap());
        }
        assert!(!engine.redo(history).unwrap());
    }
}

fn sequence_key_exact_source(include_zero: bool) -> AnimationSequenceAsset {
    let mut asset = source(Some(HERO_TARGET));
    let keys = &mut asset.bindings[0].tracks[0].channel.keys;
    let mut neighbor = keys[0].clone();
    neighbor.time_seconds = f32::EPSILON * 0.25;
    neighbor.value = AnimationChannelValueAsset::Scalar(8.0);
    neighbor.in_tangent = Some(AnimationChannelValueAsset::Scalar(-3.0));
    neighbor.out_tangent = Some(AnimationChannelValueAsset::Scalar(7.0));
    if include_zero {
        keys.insert(1, neighbor);
    } else {
        keys[0] = neighbor;
    }
    asset
}

#[test]
fn sequence_key_exact_add_does_not_alias_a_nearby_authored_key() {
    let original = sequence_key_exact_source(false);
    assert!(AnimationAuthoringAsset::Sequence(original.clone())
        .compile()
        .is_successful());
    let mut edited = AnimationAuthoringAsset::Sequence(original.clone());
    let add = AnimationDocumentMutation::AddKey {
        track_path: path("Root/Hero", SPEED),
        frame: 0,
    };
    assert!(add.apply(&mut edited).unwrap());
    let asset = edited.as_sequence().unwrap();
    let keys = &asset.bindings[0].tracks[0].channel.keys;
    assert_eq!(keys.len(), 3);
    assert_eq!(keys[0].time_seconds, 0.0);
    assert_eq!(
        &keys[1..],
        original.bindings[0].tracks[0].channel.keys.as_slice()
    );
    assert_eq!(asset.bindings[0].target_id.as_deref(), Some(HERO_TARGET));
    let bytes = asset.to_bytes().unwrap();
    assert_eq!(AnimationSequenceAsset::from_bytes(&bytes).unwrap(), *asset);
    assert!(edited.compile().is_successful());
    assert!(!add.apply(&mut edited).unwrap());
    assert_eq!(edited.to_bytes().unwrap(), bytes);
}

#[test]
fn sequence_key_exact_remove_preserves_nearby_key_metadata_and_binary_roundtrip() {
    let original = sequence_key_exact_source(true);
    let mut expected = original.clone();
    expected.bindings[0].tracks[0].channel.keys.remove(0);
    let mut edited = AnimationAuthoringAsset::Sequence(original);
    assert!(AnimationDocumentMutation::RemoveKey {
        track_path: path("Root/Hero", SPEED),
        frame: 0,
    }
    .apply(&mut edited)
    .unwrap());
    assert_eq!(edited.as_sequence(), Some(&expected));
    let bytes = edited.to_bytes().unwrap();
    assert_eq!(
        AnimationSequenceAsset::from_bytes(&bytes).unwrap(),
        expected
    );
    assert!(edited.compile().is_successful());
}

#[test]
fn sequence_key_exact_missing_time_is_noop_in_original_document_and_history() {
    let original = sequence_key_exact_source(false);
    let mutation = AnimationDocumentMutation::RemoveKey {
        track_path: path("Root/Hero", SPEED),
        frame: 0,
    };
    let mut edited = AnimationAuthoringAsset::Sequence(original.clone());
    let bytes = edited.to_bytes().unwrap();
    assert!(!mutation.apply(&mut edited).unwrap());
    assert_eq!(edited.to_bytes().unwrap(), bytes);
    assert_eq!(edited.as_sequence(), Some(&original));
    let id = DocumentId::new(9188);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/exact-key-noop.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let before = engine.history_status(history).unwrap();
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
    assert_eq!(engine.history_status(history).unwrap(), before);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    assert!(!engine.redo(history).unwrap());
    assert_eq!(engine.history_status(history).unwrap(), before);
}

#[test]
fn sequence_key_exact_changes_use_one_existing_transaction_and_restore_through_undo_redo() {
    let original = sequence_key_exact_source(true);
    let mut removed = original.clone();
    removed.bindings[0].tracks[0].channel.keys.remove(0);
    let mut added = removed.clone();
    added.bindings[0].tracks[0].channel.keys.insert(
        0,
        AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(2.0),
            in_tangent: None,
            out_tangent: None,
        },
    );
    let states = [original.clone(), removed, added];
    let id = DocumentId::new(9189);
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/exact-key-history.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    for (index, mutation) in [
        AnimationDocumentMutation::RemoveKey {
            track_path: path("Root/Hero", SPEED),
            frame: 0,
        },
        AnimationDocumentMutation::AddKey {
            track_path: path("Root/Hero", SPEED),
            frame: 0,
        },
    ]
    .into_iter()
    .enumerate()
    {
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
        assert_document(&engine, id, 2 + index as u64, &states[index + 1]);
    }
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 4, &states[1]);
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, 5, &states[0]);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    for (index, expected) in states[1..].iter().enumerate() {
        assert!(engine.redo(history).unwrap());
        assert_document(&engine, id, 6 + index as u64, expected);
    }
    assert!(!engine.redo(history).unwrap());
}

#[test]
fn sequence_key_exact_remove_does_not_normalize_other_invalid_source_times() {
    let mut original = sequence_key_exact_source(true);
    let nan_bits = 0x7fc0_1234;
    original.bindings[0].tracks[0].channel.keys[1].time_seconds = f32::from_bits(nan_bits);
    let neighbor = original.bindings[0].tracks[0].channel.keys[1].clone();
    let mut edited = AnimationAuthoringAsset::Sequence(original);
    assert!(!edited.compile().is_successful());
    assert!(AnimationDocumentMutation::RemoveKey {
        track_path: path("Root/Hero", SPEED),
        frame: 0,
    }
    .apply(&mut edited)
    .unwrap());
    let keys = &edited.as_sequence().unwrap().bindings[0].tracks[0]
        .channel
        .keys;
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].time_seconds.to_bits(), nan_bits);
    assert_eq!(keys[0].value, neighbor.value);
    assert_eq!(keys[0].in_tangent, neighbor.in_tangent);
    assert_eq!(keys[0].out_tangent, neighbor.out_tangent);
    assert_eq!(keys[1].time_seconds, 2.0);
    assert!(!edited.compile().is_successful());
}
