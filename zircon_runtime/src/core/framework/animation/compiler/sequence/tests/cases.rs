use crate::core::framework::animation::compiler::sequence::AnimationCompiledSequenceValueKind;
use crate::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::compile_animation_sequence;

fn key(time_seconds: f32, value: AnimationChannelValueAsset) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    }
}

fn track(property_path: &str, channel: AnimationChannelAsset) -> AnimationSequenceTrackAsset {
    AnimationSequenceTrackAsset {
        property_path: ComponentPropertyPath::parse(property_path).expect("fixture path is valid"),
        channel,
    }
}

fn binding(tracks: Vec<AnimationSequenceTrackAsset>) -> AnimationSequenceBindingAsset {
    AnimationSequenceBindingAsset {
        entity_path: EntityPath::parse("Root/Hero").expect("fixture path is valid"),
        target_id: None,
        tracks,
    }
}

#[test]
fn sequence_compiler_preserves_canonical_track_data() {
    let sequence = AnimationSequenceAsset {
        name: Some("Move".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![binding(vec![track(
            "Transform.translation.x",
            AnimationChannelAsset {
                interpolation: AnimationInterpolationAsset::Linear,
                keys: vec![
                    key(0.0, AnimationChannelValueAsset::Scalar(0.0)),
                    key(1.0, AnimationChannelValueAsset::Scalar(5.0)),
                ],
            },
        )])],
    };

    let compilation = compile_animation_sequence(&sequence);

    assert!(compilation.diagnostics().is_empty());
    let artifact = compilation
        .artifact()
        .expect("valid sequence must produce an artifact");
    assert_eq!(artifact.duration_seconds(), 1.0);
    assert_eq!(artifact.frames_per_second(), 30.0);
    let track = &artifact.bindings()[0].tracks()[0];
    assert_eq!(
        track.value_kind(),
        AnimationCompiledSequenceValueKind::Scalar
    );
    assert_eq!(track.keys()[0].time_seconds(), 0.0);
    assert_eq!(track.keys()[1].time_seconds(), 1.0);
}

#[test]
fn sequence_compiler_rejects_time_type_and_write_conflicts() {
    let duplicate = track(
        "Transform.translation.x",
        AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Linear,
            keys: vec![
                key(0.5, AnimationChannelValueAsset::Scalar(0.0)),
                key(0.25, AnimationChannelValueAsset::Bool(true)),
            ],
        },
    );
    let sequence = AnimationSequenceAsset {
        name: None,
        duration_seconds: 0.0,
        frames_per_second: 0.0,
        bindings: vec![AnimationSequenceBindingAsset {
            target_id: Some("  ".to_string()),
            ..binding(vec![duplicate.clone(), duplicate])
        }],
    };

    let compilation = compile_animation_sequence(&sequence);

    assert!(compilation.artifact().is_none());
    let codes: Vec<_> = compilation
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-002"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-003"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-004"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-007"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-008"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-009"));
    assert!(codes.contains(&"ZR-ANIM-COMP-SEQUENCE-012"));
}

#[test]
fn sequence_compiler_rejects_invalid_quaternions_and_hermite_tangents() {
    let mut invalid_tangent = key(0.0, AnimationChannelValueAsset::Quaternion([0.0; 4]));
    invalid_tangent.out_tangent = Some(AnimationChannelValueAsset::Vec2([1.0, 1.0]));
    let sequence = AnimationSequenceAsset {
        name: None,
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![binding(vec![track(
            "Transform.rotation",
            AnimationChannelAsset {
                interpolation: AnimationInterpolationAsset::Hermite,
                keys: vec![
                    invalid_tangent,
                    key(
                        1.0,
                        AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 1.0]),
                    ),
                ],
            },
        )])],
    };

    let compilation = compile_animation_sequence(&sequence);

    assert!(compilation.artifact().is_none());
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-SEQUENCE-011"));
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-SEQUENCE-013"));
}
