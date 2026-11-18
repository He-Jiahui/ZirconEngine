use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Transform, Vec3};

use super::{blend_graph_base_poses, GraphWeightedPose};

#[test]
fn masked_base_blend_normalizes_weights_per_bone() {
    let first = pose_with_translations(10.0, 20.0);
    let masked = pose_with_translations(100.0, 40.0);

    let blended = blend_graph_base_poses(
        vec![
            weighted(first, 0.5, None),
            weighted(masked, 0.5, Some(Arc::from([false, true]))),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("base poses should blend");

    assert_eq!(blended.bones[0].local_transform.translation.x, 10.0);
    assert_eq!(blended.bones[1].local_transform.translation.x, 30.0);
}

#[test]
fn base_blend_ignores_non_positive_and_non_finite_weights() {
    let blended = blend_graph_base_poses(
        vec![
            weighted(pose_with_translations(10.0, 20.0), 0.5, None),
            weighted(pose_with_translations(100.0, 200.0), f32::NAN, None),
            weighted(pose_with_translations(300.0, 400.0), -1.0, None),
            weighted(pose_with_translations(500.0, 600.0), f32::INFINITY, None),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("one valid base pose should blend");

    assert_eq!(blended.bones[0].local_transform.translation.x, 10.0);
    assert_eq!(blended.bones[1].local_transform.translation.x, 20.0);
}

#[test]
fn overlapping_base_blend_is_deterministic_across_input_order() {
    let left = weighted(pose_with_translations(10.0, 20.0), 0.25, None);
    let right = weighted(pose_with_translations(30.0, 40.0), 0.75, None);

    let forward = blend_graph_base_poses(
        vec![left.clone(), right.clone()],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("forward base poses should blend");
    let reverse = blend_graph_base_poses(vec![right, left], AnimationPoseSource::Graph, None)
        .expect("reverse base poses should blend");

    assert_eq!(forward, reverse);
    assert_eq!(forward.bones[0].local_transform.translation.x, 25.0);
    assert_eq!(forward.bones[1].local_transform.translation.x, 35.0);
}

#[test]
fn base_blend_equivalent_quaternion_signs_use_one_canonical_result() {
    let rotation = zircon_runtime::core::math::Quat::from_rotation_y(0.7);
    let mut positive = pose_with_translations(10.0, 20.0);
    positive.bones[0].local_transform.rotation = rotation;
    let mut negative = pose_with_translations(10.0, 20.0);
    negative.bones[0].local_transform.rotation = -rotation;

    let forward = blend_graph_base_poses(
        vec![
            weighted(positive.clone(), 0.5, None),
            weighted(negative.clone(), 0.5, None),
        ],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("signed quaternion pair should blend");
    let reverse = blend_graph_base_poses(
        vec![weighted(negative, 0.5, None), weighted(positive, 0.5, None)],
        AnimationPoseSource::Graph,
        None,
    )
    .expect("reversed signed quaternion pair should blend");

    assert_eq!(
        forward.bones[0].local_transform.rotation,
        reverse.bones[0].local_transform.rotation
    );
}

fn weighted(
    pose: AnimationPoseOutput,
    weight: f32,
    target_mask: Option<Arc<[bool]>>,
) -> GraphWeightedPose {
    GraphWeightedPose {
        pose,
        weight,
        target_mask,
        legacy_target_ids: Vec::new(),
    }
}

fn pose_with_translations(leg_x: f32, arm_x: f32) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Graph,
        active_state: None,
        bones: vec![bone("leg", leg_x), bone("arm", arm_x)],
    }
}

fn bone(name: &str, translation_x: f32) -> AnimationPoseBone {
    AnimationPoseBone {
        name: name.to_string(),
        local_transform: Transform {
            translation: Vec3::new(translation_x, 0.0, 0.0),
            ..Transform::default()
        },
    }
}

use zircon_runtime::core::math::Quat;

#[test]
fn anim147_equal_weight_seam_uses_shortest_midpoint() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    for (case, [a, b], expected) in [
        (
            "Z seam",
            [anim147_z(-89.0), anim147_z(-91.0)],
            anim147_z(-90.0),
        ),
        (
            "#147 xyzw witness",
            [
                Quat::from_xyzw(0.8, -0.6, 0.0, 0.0),
                Quat::from_xyzw(0.6, -0.8, 0.0, 0.0),
            ],
            Quat::from_xyzw(s, -s, 0.0, 0.0),
        ),
    ] {
        for (order, pair) in [[a, b], [b, a]].into_iter().enumerate() {
            let pose = anim147_blend(pair.map(anim147_half_weight).into());
            anim147_assert_rotation(
                pose.bones[0].local_transform.rotation,
                expected,
                -Vec3::Y,
                (case, order),
            );
        }
    }
}

#[test]
fn anim147_leg_mask_excludes_first_anchor_and_rescales_applied_weights() {
    let excluded = anim147_leg_pose(anim147_z(90.0), 100.0, 64.0);
    let mut left = anim147_leg_pose(anim147_z(-89.0), 12.0, 6.0);
    left.bones[0].local_transform.scale = Vec3::splat(2.0);
    let mut right = anim147_leg_pose(anim147_z(-91.0), 28.0, 10.0);
    right.bones[0].local_transform.scale = Vec3::splat(4.0);
    let leg_only: Arc<[bool]> = Arc::from([true, false]);

    for leg_weight in [0.25, 2.0] {
        let pose = anim147_blend(vec![
            weighted(excluded.clone(), 4.0, Some(Arc::from([false, true]))),
            weighted(left.clone(), leg_weight, Some(leg_only.clone())),
            weighted(right.clone(), leg_weight, Some(leg_only.clone())),
        ]);
        let [leg, arm] = pose.bones.as_slice() else {
            panic!("expected leg and arm");
        };
        assert_eq!(leg.name, "leg");
        assert_eq!(arm.name, "arm");
        assert_eq!(leg.local_transform.translation, Vec3::new(20.0, 0.0, 0.0));
        assert_eq!(leg.local_transform.scale, Vec3::splat(3.0));
        assert_eq!(arm.local_transform.translation, Vec3::new(64.0, 0.0, 0.0));
        assert_eq!(arm.local_transform.scale, Vec3::ONE);
        anim147_assert_z_midpoint(leg.local_transform.rotation, leg_weight);
    }
}

#[test]
fn anim147_invalid_first_weight_cannot_anchor_rotation() {
    let excluded = anim147_leg_pose(anim147_z(90.0), 100.0, 64.0);
    let [left, right] = [-89.0, -91.0].map(|degrees| anim147_half_weight(anim147_z(degrees)));
    for weight in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let pose = anim147_blend(vec![
            weighted(excluded.clone(), weight, None),
            left.clone(),
            right.clone(),
        ]);
        anim147_assert_z_midpoint(pose.bones[0].local_transform.rotation, weight);
    }
}

#[test]
fn anim147_equivalent_quaternion_signs_preserve_physical_rotation() {
    let angle = 0.7_f32;
    let q = Quat::from_rotation_y(angle);
    let expected_x = Vec3::new(angle.cos(), 0.0, -angle.sin());
    for (order, pair) in [[q, -q], [-q, q]].into_iter().enumerate() {
        let pose = anim147_blend(pair.map(anim147_half_weight).into());
        anim147_assert_rotation(pose.bones[0].local_transform.rotation, q, expected_x, order);
    }
}

#[test]
fn anim147_adjacent_rotations_preserve_orientation_across_orders_and_signs() {
    let rotations = [-88.0, -90.0, -92.0].map(anim147_z);
    let weights = [0.25, 0.5, 0.25];
    let leg_x = [8.0, 28.0, 4.0];
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        for signs in 0_u8..8 {
            let inputs = order.map(|i| {
                let q = if signs & (1 << i) == 0 {
                    rotations[i]
                } else {
                    -rotations[i]
                };
                weighted(anim147_leg_pose(q, leg_x[i], 0.0), weights[i], None)
            });
            let pose = anim147_blend(inputs.into());
            let leg = pose.bones[0].local_transform;
            assert_eq!(leg.translation.x, 17.0);
            anim147_assert_z_midpoint(leg.rotation, (order, signs));
        }
    }
}

fn anim147_z(degrees: f32) -> Quat {
    Quat::from_rotation_z(degrees.to_radians())
}

fn anim147_leg_pose(rotation: Quat, leg_x: f32, arm_x: f32) -> AnimationPoseOutput {
    let mut pose = pose_with_translations(leg_x, arm_x);
    pose.bones[0].local_transform.rotation = rotation;
    pose
}

fn anim147_half_weight(rotation: Quat) -> GraphWeightedPose {
    weighted(anim147_leg_pose(rotation, 0.0, 0.0), 0.5, None)
}

fn anim147_blend(inputs: Vec<GraphWeightedPose>) -> AnimationPoseOutput {
    blend_graph_base_poses(inputs, AnimationPoseSource::Graph, None)
        .expect("base poses should blend")
}

#[track_caller]
fn anim147_assert_z_midpoint(actual: Quat, case: impl std::fmt::Debug) {
    anim147_assert_rotation(actual, anim147_z(-90.0), -Vec3::Y, case);
}

#[track_caller]
fn anim147_assert_rotation(
    actual: Quat,
    expected: Quat,
    expected_x: Vec3,
    case: impl std::fmt::Debug,
) {
    const EPS: f32 = 1.0e-5;
    assert!(
        actual.is_finite(),
        "{case:?}: non-finite rotation {actual:?}"
    );
    let length_squared = actual.length_squared();
    assert!(
        (length_squared - 1.0).abs() <= EPS,
        "{case:?}: rotation length squared {length_squared}"
    );
    let abs_dot = actual.dot(expected).abs();
    let x = actual * Vec3::X;
    assert!(
        abs_dot >= 1.0 - EPS && (x - expected_x).length() <= EPS,
        "{case:?}: abs(dot)={abs_dot}; rotated X={x:?}, expected {expected_x:?}"
    );
}

#[test]
fn anim147_wide_rotations_preserve_orientation_across_orders_and_signs() {
    let rotations = [-170.0, 0.0, 170.0].map(anim147_z);
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for weights in [[0.30, 0.33, 0.37], [1.0; 3]] {
        let blend = |order: [usize; 3], signs: u8| {
            let inputs = order.map(|i| {
                let q = if signs & (1 << i) == 0 {
                    rotations[i]
                } else {
                    -rotations[i]
                };
                weighted(anim147_leg_pose(q, 0.0, 0.0), weights[i], None)
            });
            anim147_blend(inputs.into()).bones[0]
                .local_transform
                .rotation
        };
        let reference = blend(orders[0], 0);
        assert!(
            reference.is_finite() && (reference.length_squared() - 1.0).abs() <= 1.0e-5,
            "{weights:?}: reference must be finite and unit: {reference:?}"
        );
        let reference_x = reference * Vec3::X;
        for order in orders {
            for signs in 0_u8..8 {
                anim147_assert_rotation(
                    blend(order, signs),
                    reference,
                    reference_x,
                    (weights, order, signs),
                );
            }
        }
    }
}
