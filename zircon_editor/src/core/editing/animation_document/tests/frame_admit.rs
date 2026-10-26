use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationTrackPath,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationDocumentMutation,
    AnimationDocumentMutationError,
};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditorTransactionEngine, HistoryContextId};
use crate::core::editor_message::DocumentId;

fn path() -> AnimationTrackPath {
    AnimationTrackPath::new(
        EntityPath::parse("Root/Hero").unwrap(),
        ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
    )
}

fn key(time_seconds: f32, value: f32) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value: AnimationChannelValueAsset::Scalar(value),
        in_tangent: None,
        out_tangent: None,
    }
}

fn source() -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("Frame conversion admission".into()),
        duration_seconds: 4.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: Some("9007199254740993".into()),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![key(0.0, -0.0), key(0.5, 1.0)],
                },
            }],
        }],
    }
}

fn invalid_rates() -> [f32; 8] {
    [
        0.0,
        -0.0,
        -1.0,
        f32::EPSILON,
        f32::EPSILON / 2.0,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ]
}

#[test]
fn sequence_frame_admission_rejects_invalid_fps_without_touching_any_source_key() {
    for fps in invalid_rates() {
        for remove in [false, true] {
            let mut asset = source();
            asset.frames_per_second = fps;
            let bytes = asset.to_bytes().unwrap();
            // With the former 30fps fallback, AddKey created 7/30 and RemoveKey deleted 0.5.
            let result = if remove {
                super::remove_key(&mut asset, &path(), 15)
            } else {
                super::add_key(&mut asset, &path(), 7)
            };
            assert!(matches!(result,
                Err(AnimationDocumentMutationError::InvalidTrackPath { message })
                    if message.contains("frames_per_second")));
            assert_eq!(asset.frames_per_second.to_bits(), fps.to_bits());
            assert_eq!(asset.to_bytes().unwrap(), bytes);
            // An apparent exact duplicate must not bypass the invalid-rate admission either.
            assert!(super::add_key(&mut asset, &path(), 0).is_err());
            assert_eq!(asset.to_bytes().unwrap(), bytes);
        }
    }
}

#[test]
fn sequence_frame_admission_preserves_the_actual_valid_rate_and_existing_noops() {
    for fps in [f32::from_bits(f32::EPSILON.to_bits() + 1), 0.5, 30.0, 120.0] {
        let mut asset = source();
        asset.frames_per_second = fps;
        asset.duration_seconds = (2.0 / fps).max(asset.duration_seconds);
        let before = asset.to_bytes().unwrap();
        assert!(AnimationAuthoringAsset::Sequence(asset.clone())
            .compile()
            .is_successful());
        let expected = 1.0 / fps;
        assert!(super::add_key(&mut asset, &path(), 1).unwrap());
        let inserted = asset.bindings[0].tracks[0]
            .channel
            .keys
            .iter()
            .find(|key| key.time_seconds == expected)
            .unwrap();
        assert_eq!(inserted.time_seconds.to_bits(), expected.to_bits());
        assert_eq!(inserted.value, AnimationChannelValueAsset::Scalar(1.0));
        let with_key = asset.to_bytes().unwrap();
        assert!(!super::add_key(&mut asset, &path(), 1).unwrap());
        assert_eq!(asset.to_bytes().unwrap(), with_key);
        assert!(super::remove_key(&mut asset, &path(), 1).unwrap());
        assert_eq!(asset.to_bytes().unwrap(), before);
        assert!(!super::remove_key(&mut asset, &path(), 1).unwrap());
        assert_eq!(asset.to_bytes().unwrap(), before);
    }
}

#[test]
fn sequence_frame_admission_rejected_prepare_keeps_revision_last_good_and_history() {
    for fps in invalid_rates() {
        let id = DocumentId::new(9188);
        let mut document = AnimationAuthoringDocument::new(
            id,
            AssetUri::parse("res://animation/frame-admission.sequence.zranim").unwrap(),
            AnimationAuthoringAsset::Sequence(source()),
        );
        assert!(document.compilation().current_product().is_successful());
        let good_revision = document.revision();
        let mut invalid = source();
        invalid.frames_per_second = fps;
        let bytes = invalid.to_bytes().unwrap();
        let mut replacement = AnimationAuthoringAsset::Sequence(invalid);
        let bad_revision = document
            .swap_asset_if_revision(good_revision, &mut replacement)
            .unwrap();
        assert!(!document.compilation().current_product().is_successful());
        assert_eq!(
            document.compilation().last_good_revision(),
            Some(good_revision)
        );
        let mut context = CoreEditContext::default();
        context.animation_documents_mut().attach(document).unwrap();
        let engine = EditorTransactionEngine::new(context);
        let history = HistoryContextId::Document(id);
        let before = engine.history_status(history).unwrap();
        for mutation in [
            AnimationDocumentMutation::AddKey {
                track_path: path(),
                frame: 7,
            },
            AnimationDocumentMutation::RemoveKey {
                track_path: path(),
                frame: 15,
            },
        ] {
            engine
                .with_context::<CoreEditContext, _>(|context| {
                    assert!(matches!(
                        context
                            .animation_documents()
                            .prepare_mutation(id, &mutation),
                        Err(AnimationDocumentMutationError::InvalidTrackPath { .. })
                    ));
                    let handle = context.animation_documents().handle(id).unwrap();
                    let document = handle.read();
                    assert_eq!(document.revision(), bad_revision);
                    assert_eq!(document.document_bytes().unwrap(), bytes);
                    assert_eq!(
                        document
                            .asset()
                            .as_sequence()
                            .unwrap()
                            .frames_per_second
                            .to_bits(),
                        fps.to_bits()
                    );
                    assert_eq!(document.compilation().current_revision(), bad_revision);
                    assert!(!document.compilation().current_product().is_successful());
                    assert_eq!(
                        document.compilation().last_good_revision(),
                        Some(good_revision)
                    );
                })
                .unwrap()
                .unwrap();
        }
        assert_eq!(engine.history_status(history).unwrap(), before);
        assert!(!engine.is_dirty(history).unwrap());
        assert!(!engine.undo(history).unwrap());
        assert!(!engine.redo(history).unwrap());
        assert_eq!(engine.history_status(history).unwrap(), before);
    }
}
