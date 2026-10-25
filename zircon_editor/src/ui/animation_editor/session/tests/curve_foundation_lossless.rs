use super::*;
use crate::core::editing::animation_document::{
    AnimationAuthoringAsset, AnimationAuthoringDocument, AnimationAuthoringDocumentReadHandle,
};
use crate::core::editor_message::DocumentId;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

fn track() -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
        channel: AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Hermite,
            keys: vec![
                AnimationChannelKeyAsset {
                    time_seconds: 0.0,
                    value: AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
                    in_tangent: Some(AnimationChannelValueAsset::Vec3([-0.0, 0.5, -2.0])),
                    out_tangent: Some(AnimationChannelValueAsset::Vec3([4.0, -0.0, 8.0])),
                },
                AnimationChannelKeyAsset {
                    time_seconds: 2.0,
                    value: AnimationChannelValueAsset::Vec3([4.0, 5.0, 6.0]),
                    in_tangent: None,
                    out_tangent: None,
                },
            ],
        },
    }
}

fn session(track: AnimationSequenceTrackAsset) -> AnimationEditorSession {
    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let track_path = AnimationTrackPath::new(entity_path.clone(), track.property_path.clone());
    let asset = AnimationSequenceAsset {
        name: Some("Curve projection admission".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path,
            target_id: Some("9007199254740993".to_string()),
            tracks: vec![track],
        }],
    };
    let document = AnimationAuthoringDocument::new(
        DocumentId::new(9184),
        AssetUri::parse("res://tests/curve-projection.sequence.zranim").unwrap(),
        AnimationAuthoringAsset::Sequence(asset),
    );
    let mut session = AnimationEditorSession::new(
        "res://tests/curve-projection.sequence.zranim".to_string(),
        AnimationAuthoringDocumentReadHandle::detached_for_test(document),
    );
    assert!(session.select_timeline_span(&track_path, 0, 60).unwrap());
    session
}

#[test]
fn curve_lossless_independent_slopes_absent_tangents_and_signed_zero_are_preserved() {
    let session = session(track());
    let before = session.document_bytes().unwrap();
    let projection = session.curve_foundation().unwrap();
    assert_eq!(projection.curves.len(), 3);
    assert_eq!(
        projection.curves[0].interpolation,
        CurveInterpolation::Hermite
    );
    for (index, (incoming, outgoing)) in [(-0.0_f32, 4.0_f32), (0.5, -0.0), (-2.0, 8.0)]
        .into_iter()
        .enumerate()
    {
        let keys = &projection.curves[index].keys;
        assert_eq!(keys[0].in_tangent.unwrap().to_bits(), incoming.to_bits());
        assert_eq!(keys[0].out_tangent.unwrap().to_bits(), outgoing.to_bits());
        assert_eq!(keys[1].in_tangent, None);
        assert_eq!(keys[1].out_tangent, None);
    }
    assert_eq!(session.document_bytes().unwrap(), before);
}

#[test]
fn curve_lossless_scalar_and_vector_modes_keep_finite_hdr_values_and_authored_slopes() {
    for (value, incoming, outgoing, expected) in [
        (
            AnimationChannelValueAsset::Scalar(8.0),
            AnimationChannelValueAsset::Scalar(-2.0),
            AnimationChannelValueAsset::Scalar(4.0),
            vec![(8.0, -2.0, 4.0)],
        ),
        (
            AnimationChannelValueAsset::Vec2([8.0, -3.0]),
            AnimationChannelValueAsset::Vec2([-2.0, 1.0]),
            AnimationChannelValueAsset::Vec2([4.0, -5.0]),
            vec![(8.0, -2.0, 4.0), (-3.0, 1.0, -5.0)],
        ),
        (
            AnimationChannelValueAsset::Vec4([8.0, -3.0, 2.0, 16.0]),
            AnimationChannelValueAsset::Vec4([-2.0, 1.0, -4.0, 0.5]),
            AnimationChannelValueAsset::Vec4([4.0, -5.0, 6.0, -0.5]),
            vec![
                (8.0, -2.0, 4.0),
                (-3.0, 1.0, -5.0),
                (2.0, -4.0, 6.0),
                (16.0, 0.5, -0.5),
            ],
        ),
    ] {
        for interpolation in [
            AnimationInterpolationAsset::Step,
            AnimationInterpolationAsset::Linear,
            AnimationInterpolationAsset::Hermite,
        ] {
            let mut track = track();
            track.channel.interpolation = interpolation;
            for key in &mut track.channel.keys {
                key.value = value.clone();
                key.in_tangent = Some(incoming.clone());
                key.out_tangent = Some(outgoing.clone());
            }
            let projection = session(track).curve_foundation().unwrap();
            assert_eq!(projection.curves.len(), expected.len());
            assert_eq!(
                projection.curves[0].interpolation,
                curve_interpolation(interpolation)
            );
            for (curve, &(value, incoming, outgoing)) in projection.curves.iter().zip(&expected) {
                assert_eq!(curve.keys[0].point.value, value);
                assert_eq!(curve.keys[0].in_tangent, Some(incoming));
                assert_eq!(curve.keys[0].out_tangent, Some(outgoing));
            }
        }
    }
}

#[test]
fn curve_lossless_mismatched_tangents_reject_without_changing_document_or_compilation() {
    for interpolation in [
        AnimationInterpolationAsset::Step,
        AnimationInterpolationAsset::Linear,
        AnimationInterpolationAsset::Hermite,
    ] {
        for incoming in [true, false] {
            let mut track = track();
            track.channel.interpolation = interpolation;
            let tangent = Some(AnimationChannelValueAsset::Vec2([1.0, 2.0]));
            if incoming {
                track.channel.keys[1].in_tangent = tangent;
            } else {
                track.channel.keys[1].out_tangent = tangent;
            }
            let session = session(track);
            let before = session.document_bytes().unwrap();
            let (revision, compilation) = {
                let document = session.document().read();
                (document.revision(), format!("{:?}", document.compilation()))
            };
            let error = session.curve_foundation().unwrap_err().to_string();
            assert!(error.contains("Root/Hero"));
            assert!(error.contains("key 1"));
            assert!(error.contains(if incoming {
                "in_tangent"
            } else {
                "out_tangent"
            }));
            assert_eq!(session.document_bytes().unwrap(), before);
            let document = session.document().read();
            assert_eq!(document.revision(), revision);
            assert_eq!(format!("{:?}", document.compilation()), compilation);
        }
    }
}

#[test]
fn curve_lossless_nonfinite_component_or_tangent_rejects_the_entire_projection() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for field in ["value", "in_tangent", "out_tangent"] {
            let mut track = track();
            let value = AnimationChannelValueAsset::Vec3([1.0, 2.0, invalid]);
            match field {
                "value" => track.channel.keys[1].value = value,
                "in_tangent" => track.channel.keys[1].in_tangent = Some(value),
                _ => track.channel.keys[1].out_tangent = Some(value),
            }
            let session = session(track);
            let error = session.curve_foundation().unwrap_err().to_string();
            assert!(error.contains("key 1 z"));
            assert!(error.contains(field));
        }
    }
    let mut track = track();
    track.channel.keys[1].value = AnimationChannelValueAsset::Scalar(4.0);
    assert!(session(track)
        .curve_foundation()
        .unwrap_err()
        .to_string()
        .contains("value must match"));
}

#[test]
fn curve_lossless_invalid_time_and_duplicate_identity_are_rejected() {
    for time in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, -0.0, 0.0] {
        let mut track = track();
        track.channel.keys[1].time_seconds = time;
        assert!(session(track)
            .curve_foundation()
            .unwrap_err()
            .to_string()
            .contains("key times must be finite and strictly increasing"));
    }
}

#[test]
fn curve_lossless_empty_discrete_and_unselected_tracks_keep_their_existing_scope() {
    let mut empty = track();
    empty.channel.keys.clear();
    assert!(session(empty).curve_foundation().unwrap().curves.is_empty());
    for value in [
        AnimationChannelValueAsset::Bool(true),
        AnimationChannelValueAsset::Integer(7),
        AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 1.0]),
    ] {
        let mut discrete = track();
        discrete.channel.interpolation = AnimationInterpolationAsset::Step;
        discrete.channel.keys.truncate(1);
        discrete.channel.keys[0].value = value;
        discrete.channel.keys[0].in_tangent = None;
        discrete.channel.keys[0].out_tangent = None;
        assert!(session(discrete)
            .curve_foundation()
            .unwrap()
            .curves
            .is_empty());
    }
    let mut session = session(track());
    session.sequence.as_mut().unwrap().selected_span = None;
    assert_eq!(
        session.curve_foundation().unwrap(),
        AnimationCurveFoundationView::default()
    );
}
