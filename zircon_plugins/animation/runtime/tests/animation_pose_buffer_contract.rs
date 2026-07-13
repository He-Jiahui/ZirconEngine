use zircon_plugin_animation_runtime::{
    MaskWeights, PoseBlendError, PoseBuffer, PoseBufferError, PoseLayer, PoseLayerBlendMode,
};
use zircon_runtime::core::math::{Quat, Transform, Vec3};

#[test]
fn blend_override_additive_golden_values() {
    let mut base = PoseBuffer::new(1);
    let mut override_pose = PoseBuffer::new(1);
    let override_rotation = Quat::from_rotation_y(std::f32::consts::PI);
    override_pose
        .set_transform(
            0,
            Transform {
                translation: Vec3::new(10.0, 4.0, -2.0),
                rotation: override_rotation,
                scale: Vec3::new(2.0, 0.5, 3.0),
            },
        )
        .unwrap();

    base.blend_override(&override_pose, 0.25).unwrap();
    let overridden = base.transform(0).unwrap();
    assert_eq!(base.weight(0), Some(0.25));
    assert!(overridden
        .translation
        .abs_diff_eq(Vec3::new(2.5, 1.0, -0.5), 0.0001));
    assert!(overridden
        .scale
        .abs_diff_eq(Vec3::new(1.25, 0.875, 1.5), 0.0001));
    assert!(overridden
        .rotation
        .abs_diff_eq(Quat::IDENTITY.slerp(override_rotation, 0.25), 0.0001));

    let mut additive = PoseBuffer::new(1);
    let additive_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    additive
        .set_transform(
            0,
            Transform {
                translation: Vec3::new(4.0, -2.0, 8.0),
                rotation: additive_rotation,
                scale: Vec3::new(1.5, 0.5, 2.0),
            },
        )
        .unwrap();

    base.accumulate_additive(&additive, 0.5).unwrap();
    let accumulated = base.transform(0).unwrap();
    assert_eq!(base.weight(0), Some(0.5));
    assert!(accumulated
        .translation
        .abs_diff_eq(Vec3::new(4.5, 0.0, 3.5), 0.0001));
    assert!(accumulated
        .scale
        .abs_diff_eq(Vec3::new(1.5, 0.625, 2.0), 0.0001));
    let expected_rotation = (Quat::IDENTITY.slerp(additive_rotation, 0.5)
        * Quat::IDENTITY.slerp(override_rotation, 0.25))
    .normalize();
    assert!(accumulated.rotation.abs_diff_eq(expected_rotation, 0.0001));
}

#[test]
fn partial_sample_weights_scale_override_channels() {
    let mut base = PoseBuffer::new(1);
    let mut source = PoseBuffer::new(1);
    source
        .set_transform(
            0,
            Transform {
                translation: Vec3::splat(8.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(2.0),
            },
        )
        .unwrap();
    source.set_weight(0, 0.5).unwrap();

    base.blend_override(&source, 0.5).unwrap();

    assert!(base
        .transform(0)
        .unwrap()
        .translation
        .abs_diff_eq(Vec3::splat(2.0), 0.0001));
    assert_eq!(base.weight(0), Some(0.25));
}

#[test]
fn pose_blend_rejects_shape_and_weight_mismatches() {
    let mut one_joint = PoseBuffer::new(1);
    let two_joints = PoseBuffer::new(2);

    assert_eq!(
        one_joint.blend_override(&two_joints, 0.5).unwrap_err(),
        PoseBlendError::ShapeMismatch {
            destination_len: 1,
            source_len: 2,
        }
    );
    assert!(matches!(
        one_joint
            .accumulate_additive(&PoseBuffer::new(1), f32::NAN)
            .unwrap_err(),
        PoseBlendError::InvalidWeight { weight } if weight.is_nan()
    ));
    assert_eq!(
        one_joint
            .blend_override(&PoseBuffer::new(1), 1.01)
            .unwrap_err(),
        PoseBlendError::InvalidWeight { weight: 1.01 }
    );
    assert_eq!(
        one_joint.set_weight(1, 0.5).unwrap_err(),
        PoseBufferError::IndexOutOfBounds { index: 1, len: 1 }
    );
    assert_eq!(
        one_joint.set_weight(0, -0.1).unwrap_err(),
        PoseBufferError::InvalidWeight {
            index: 0,
            weight: -0.1,
        }
    );
}

#[test]
fn upper_lower_body_split_blend_scenario() {
    let mut output = PoseBuffer::new(2);
    let mut lower = PoseBuffer::new(2);
    let mut upper = PoseBuffer::new(2);
    lower
        .set_transform(
            0,
            Transform {
                translation: Vec3::new(2.0, 0.0, 0.0),
                ..Transform::default()
            },
        )
        .unwrap();
    upper
        .set_transform(
            1,
            Transform {
                translation: Vec3::new(0.0, 4.0, 0.0),
                ..Transform::default()
            },
        )
        .unwrap();
    let lower_mask = MaskWeights::try_from_weights(vec![1.0, 0.0]).unwrap();
    let upper_mask = MaskWeights::try_from_weights(vec![0.0, 1.0]).unwrap();

    output
        .blend_layers(&[
            PoseLayer::new(&lower, 1.0, PoseLayerBlendMode::Override).with_mask(&lower_mask),
            PoseLayer::new(&upper, 1.0, PoseLayerBlendMode::Override).with_mask(&upper_mask),
        ])
        .unwrap();

    assert!(output
        .transform(0)
        .unwrap()
        .translation
        .abs_diff_eq(Vec3::new(2.0, 0.0, 0.0), 0.0001));
    assert!(output
        .transform(1)
        .unwrap()
        .translation
        .abs_diff_eq(Vec3::new(0.0, 4.0, 0.0), 0.0001));
}
