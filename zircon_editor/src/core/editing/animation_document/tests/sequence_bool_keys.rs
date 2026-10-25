use zircon_runtime::animation::sample_animation_bool_channel;
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

fn key(time_seconds: f32, value: bool) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value: AnimationChannelValueAsset::Bool(value),
        in_tangent: None,
        out_tangent: None,
    }
}

fn source() -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("Bool insertion source".to_string()),
        duration_seconds: 4.0,
        frames_per_second: 2.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: Some("9007199254740993".to_string()),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.looping").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![key(0.0, false), key(1.0, true)],
                },
            }],
        }],
    }
}

fn path() -> AnimationTrackPath {
    AnimationTrackPath::new(
        EntityPath::parse("Root/Hero").unwrap(),
        ComponentPropertyPath::parse("AnimationPlayer.looping").unwrap(),
    )
}

fn channel(asset: &AnimationSequenceAsset) -> &AnimationChannelAsset {
    &asset.bindings[0].tracks[0].channel
}

#[test]
fn sequence_bool_insert_uses_existing_step_sample_and_preserves_nearby_keys() {
    for (fps, last_time) in [(2.0, 1.0), (8.0 / f32::EPSILON, f32::EPSILON / 4.0)] {
        let mut asset = source();
        asset.frames_per_second = fps;
        asset.bindings[0].tracks[0].channel.keys[1].time_seconds = last_time;
        let original = asset.clone();
        assert_eq!(
            sample_animation_bool_channel(channel(&asset), 1.0 / fps),
            Some(false)
        );
        assert!(super::add_key(&mut asset, &path(), 1).unwrap());
        let mut expected = original.clone();
        expected.bindings[0].tracks[0]
            .channel
            .keys
            .insert(1, key(1.0 / fps, false));
        assert_eq!(asset, expected);
        assert_eq!(
            asset.bindings[0].target_id.as_deref(),
            Some("9007199254740993")
        );
        assert_eq!(
            AnimationSequenceAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap(),
            expected
        );
        assert!(AnimationAuthoringAsset::Sequence(asset.clone())
            .compile()
            .is_successful());
        // Insertion inside an existing interval keeps the existing Step samples.
        for time in [0.0, 0.25 / fps, 1.0 / fps, 1.5 / fps, last_time] {
            assert_eq!(
                sample_animation_bool_channel(channel(&asset), time),
                sample_animation_bool_channel(channel(&original), time)
            );
        }
    }
}

#[test]
fn sequence_bool_insert_uses_preimage_sampler_endpoint_clamp_without_owning_a_clock() {
    for (frame, expected_value) in [(0, true), (2, true), (6, false), (12, true)] {
        let mut asset = source();
        asset.frames_per_second = 4.0;
        asset.bindings[0].tracks[0].channel.keys =
            vec![key(0.25, true), key(1.0, false), key(2.5, true)];
        let original = asset.clone();
        let requested = frame as f32 / 4.0;
        assert_eq!(
            sample_animation_bool_channel(channel(&original), requested),
            Some(expected_value)
        );
        assert!(super::add_key(&mut asset, &path(), frame).unwrap());
        let inserted = channel(&asset)
            .keys
            .iter()
            .find(|key| key.time_seconds == requested)
            .unwrap();
        assert_eq!(inserted, &key(requested, expected_value));
        assert_eq!(asset.frames_per_second, original.frames_per_second);
        assert_eq!(asset.duration_seconds, original.duration_seconds);
        assert_eq!(asset.bindings[0].target_id, original.bindings[0].target_id);
        for original_key in &channel(&original).keys {
            assert!(channel(&asset).keys.contains(original_key));
        }
        assert_eq!(
            AnimationSequenceAsset::from_bytes(&asset.to_bytes().unwrap()).unwrap(),
            asset
        );
    }
}

#[test]
fn sequence_bool_insert_rejects_unsupported_or_invalid_channel_without_source_writes() {
    let base = source();
    let mut invalid = Vec::new();
    for interpolation in [
        AnimationInterpolationAsset::Linear,
        AnimationInterpolationAsset::Hermite,
    ] {
        let mut asset = base.clone();
        asset.bindings[0].tracks[0].channel.interpolation = interpolation;
        invalid.push(asset);
    }
    for tangent_in in [true, false] {
        let mut asset = base.clone();
        let first = &mut asset.bindings[0].tracks[0].channel.keys[0];
        if tangent_in {
            first.in_tangent = Some(AnimationChannelValueAsset::Bool(true));
        } else {
            first.out_tangent = Some(AnimationChannelValueAsset::Bool(false));
        }
        invalid.push(asset);
    }
    for bool_first in [true, false] {
        let mut asset = base.clone();
        let index = if bool_first { 1 } else { 0 };
        asset.bindings[0].tracks[0].channel.keys[index].value =
            AnimationChannelValueAsset::Scalar(7.0);
        invalid.push(asset);
    }
    for time in [-0.25, 1.0, 2.0, f32::NAN, f32::INFINITY] {
        let mut asset = base.clone();
        asset.bindings[0].tracks[0].channel.keys[0].time_seconds = time;
        invalid.push(asset);
    }
    for mut asset in invalid {
        let original_bytes = asset.to_bytes().unwrap();
        let original_time_bits: Vec<_> = channel(&asset)
            .keys
            .iter()
            .map(|key| key.time_seconds.to_bits())
            .collect();
        assert!(matches!(
            super::add_key(&mut asset, &path(), 1),
            Err(AnimationDocumentMutationError::InvalidTrackPath { message })
                if message.contains("Boolean channel")
        ));
        assert_eq!(asset.to_bytes().unwrap(), original_bytes);
        assert_eq!(
            channel(&asset)
                .keys
                .iter()
                .map(|key| key.time_seconds.to_bits())
                .collect::<Vec<_>>(),
            original_time_bits
        );
    }
}

#[test]
fn sequence_bool_insert_preserves_exact_duplicate_noop_and_existing_non_bool_policy() {
    let mut bool_asset = source();
    let before = bool_asset.to_bytes().unwrap();
    // The exact authored key is retained even though the evaluator's interior-key rule differs.
    assert!(!super::add_key(&mut bool_asset, &path(), 2).unwrap());
    assert_eq!(bool_asset.to_bytes().unwrap(), before);

    let mut scalar = source();
    scalar.bindings[0].tracks[0].channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![
            AnimationChannelKeyAsset {
                time_seconds: 0.0,
                value: AnimationChannelValueAsset::Scalar(1.0),
                in_tangent: None,
                out_tangent: None,
            },
            AnimationChannelKeyAsset {
                time_seconds: 1.0,
                value: AnimationChannelValueAsset::Scalar(9.0),
                in_tangent: None,
                out_tangent: None,
            },
        ],
    };
    assert!(super::add_key(&mut scalar, &path(), 1).unwrap());
    assert_eq!(
        channel(&scalar).keys[1].value,
        AnimationChannelValueAsset::Scalar(9.0)
    );
    scalar.bindings[0].tracks[0].channel.keys.clear();
    assert!(super::add_key(&mut scalar, &path(), 0).unwrap());
    assert_eq!(
        channel(&scalar).keys[0].value,
        AnimationChannelValueAsset::Scalar(0.0)
    );
}

fn assert_document(
    engine: &EditorTransactionEngine,
    id: DocumentId,
    expected: &AnimationSequenceAsset,
    revision: u64,
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
fn sequence_bool_insert_uses_original_document_history_and_codec_undo_redo() {
    let id = DocumentId::new(9186);
    let original = source();
    let mut expected = original.clone();
    expected.bindings[0].tracks[0]
        .channel
        .keys
        .insert(1, key(0.5, false));
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/bool-insert.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let mutation = AnimationDocumentMutation::AddKey {
        track_path: path(),
        frame: 1,
    };
    let (revision, replacement) = engine
        .with_context::<CoreEditContext, _>(|context| {
            context
                .animation_documents()
                .prepare_mutation(id, &mutation)
        })
        .unwrap()
        .unwrap()
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
    assert_document(&engine, id, &expected, 2);
    assert!(engine.is_dirty(history).unwrap());
    assert!(engine.undo(history).unwrap());
    assert_document(&engine, id, &original, 3);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert_document(&engine, id, &expected, 4);
    assert!(engine.is_dirty(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}

#[test]
fn sequence_bool_insert_rejection_preserves_store_compile_revision_and_empty_history() {
    let id = DocumentId::new(9187);
    let mut original = source();
    original.bindings[0].tracks[0].channel.interpolation = AnimationInterpolationAsset::Linear;
    let bytes = original.to_bytes().unwrap();
    let mut context = CoreEditContext::default();
    context
        .animation_documents_mut()
        .attach(AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/bool-rejected.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(original.clone()),
        ))
        .unwrap();
    let engine = EditorTransactionEngine::new(context);
    let history = HistoryContextId::Document(id);
    let before = engine.history_status(history).unwrap();
    engine
        .with_context::<CoreEditContext, _>(|context| {
            assert!(matches!(
                context.animation_documents().prepare_mutation(
                    id,
                    &AnimationDocumentMutation::AddKey {
                        track_path: path(),
                        frame: 1
                    }
                ),
                Err(AnimationDocumentMutationError::InvalidTrackPath { .. })
            ));
            let handle = context.animation_documents().handle(id).unwrap();
            let document = handle.read();
            assert_eq!(document.revision().value(), 1);
            assert_eq!(document.asset().as_sequence(), Some(&original));
            assert_eq!(document.document_bytes().unwrap(), bytes);
            assert_eq!(
                document.compilation().current_revision(),
                document.revision()
            );
            assert!(!document.compilation().current_product().is_successful());
            assert!(document.compilation().last_good_revision().is_none());
        })
        .unwrap()
        .unwrap();
    assert_eq!(engine.history_status(history).unwrap(), before);
    assert!(!engine.is_dirty(history).unwrap());
    assert!(!engine.undo(history).unwrap());
    assert!(!engine.redo(history).unwrap());
}
