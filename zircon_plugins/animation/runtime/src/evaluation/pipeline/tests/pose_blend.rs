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

#[test]
fn anim147_r32_graph_clip_crosses_canonical_seam_without_pose_jump() {
    let [[root_before, hand_before], [root_after, hand_after]] = r32_graph_frames(
        [[-89.0; 2], [-89.0, -91.0], [90.0; 2]],
        Some("Root"),
        [(0.375, 0.5, 0.5), (0.625, 0.5, 0.5)],
    );

    // A and C reach Root with raw weights 1/4 and 1/2; B's Root track is masked out.
    // A and B reach Hand at 1/4 each; C's wide, heavier Hand track is masked out.
    for root in [root_before, root_after] {
        assert!((root.translation.x - 8.0).abs() < 1.0e-5, "{root:?}");
        assert!(r32_arc_degrees(root.rotation, Quat::from_rotation_y(0.4)) < 0.01);
    }
    for hand in [hand_before, hand_after] {
        assert!((hand.translation.x - 8.0).abs() < 1.0e-5, "{hand:?}");
    }
    let step = r32_arc_degrees(hand_before.rotation, hand_after.rotation);
    assert!(
        (0.20..0.30).contains(&step),
        "Hand should turn 0.25°, not {step}°"
    );
    // A constant or almost constant result cannot pass on apparent smoothness alone.
    for (actual, expected) in [(hand_before, -89.375), (hand_after, -89.625)] {
        let error = r32_arc_degrees(actual.rotation, anim147_z(expected));
        assert!(
            error < 0.02,
            "Hand midpoint expected {expected}°, error {error}°"
        );
    }
}

#[test]
fn anim147_r32_graph_wide_weight_crossing_does_not_switch_pose_branch() {
    let eps = 1.0 / 65_536.0;
    let center = 1.0 / 3.0;
    let bias = [0.5 - eps, 0.5, 0.5 + eps];
    let poses = r32_graph_frames(
        [[-170.0; 2], [170.0; 2], [0.0; 2]],
        None,
        [
            (0.25, bias[0], center),
            (0.5, bias[1], center),
            (0.75, bias[2], center),
        ],
    );
    for ([_, hand], bias) in poses.iter().zip(bias) {
        let expected_x = (1.0 - center) * ((1.0 - bias) * 2.0 + bias * 14.0) + center * 50.0;
        assert!(
            (hand.translation.x - expected_x).abs() < 1.0e-5,
            "Hand weights were not sampled at bias {bias}: {hand:?}"
        );
    }
    assert!(poses[2][1].translation.x - poses[0][1].translation.x > 0.0002);
    // Static clips; only the Blend parameter moves by 2^-16 per frame. No absolute
    // wide-angle output is prescribed. Include both sides and the intervening near-tie.
    for (from, to) in [(0, 1), (1, 2), (0, 2)] {
        let step = r32_arc_degrees(poses[from][1].rotation, poses[to][1].rotation);
        assert!(
            step < 0.05,
            "wide Hand weights on frames {from} -> {to} jumped {step}°"
        );
    }
}

#[test]
fn anim147_r32_vanishing_outlier_cannot_switch_the_close_pair_branch() {
    let outlier = anim147_z(90.0);
    let left = anim147_z(-89.0);
    let right = anim147_z(-91.0);
    let reference = r32_leg_blend(&[(outlier, 0.0), (left, 1.0), (right, 1.0)]);
    // This test is about the limit, not the absolute arc (the frozen seam test checks that).
    for weight in [f32::MIN_POSITIVE, 1.0 / 65_536.0, 1.0 / 1024.0] {
        for inputs in [
            [(outlier, weight), (left, 1.0), (right, 1.0)],
            [(-right, 1.0), (-outlier, weight), (left, 1.0)],
        ] {
            let step = r32_arc_degrees(r32_leg_blend(&inputs), reference);
            assert!(
                step < 0.1,
                "vanishing weight {weight}: {step}° instead of a nearby pose"
            );
        }
    }
}

#[test]
fn anim147_r32_ordinary_pair_matches_normalized_linear_interpolation() {
    let right = anim147_z(90.0);
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let expected = Quat::from_rotation_z((2.0 * (3.0 * s).atan2(1.0 + 3.0 * s)) as f32);
    // x3 influence gives ~68.40°, not the 67.5° of Slerp or ~71.57° of a projector mean.
    for a in [Quat::IDENTITY, -Quat::IDENTITY] {
        for b in [right, -right] {
            for input in [[(a, 1.0), (b, 3.0)], [(b, 3.0), (a, 1.0)]] {
                let error = r32_arc_degrees(r32_leg_blend(&input), expected);
                assert!(
                    error < 0.01,
                    "ordinary normalized linear rotation error {error}°"
                );
            }
        }
    }
}

#[test]
fn anim147_r32_exactly_orthogonal_pair_has_one_equidistant_tie() {
    let a = Quat::IDENTITY;
    let b = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0); // Exactly orthogonal, including under negation.
    let reference = r32_leg_blend(&[(a, 1.0), (b, 1.0)]);
    // Either half-turn arc is admissible; input representation/iteration cannot select it.
    for end in [a, b] {
        assert!((r32_arc_degrees(reference, end) - 90.0).abs() < 0.01);
    }
    for a in [a, -a] {
        for b in [b, -b] {
            for input in [[(a, 1.0), (b, 1.0)], [(b, 1.0), (a, 1.0)]] {
                let step = r32_arc_degrees(r32_leg_blend(&input), reference);
                assert!(step < 0.01, "orthogonal tie changed by {step}°");
            }
        }
    }
}

#[test]
fn anim147_r32_near_orthogonal_pair_with_a_bridge_cannot_snap_on_dot_sign() {
    let bridge = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    let eps = 1.0 / 32_768.0;
    let frame = |w| {
        r32_leg_blend(&[
            (Quat::IDENTITY, 1.0),
            (Quat::from_xyzw(1.0, 0.0, 0.0, w).normalize(), 1.0),
            (bridge, 0.5),
        ])
    };
    // The substantial +90° bridge does not move. In contrast to the unbridged
    // two-input 180° cut, this comparison does not require a single shortest arc.
    let step = r32_arc_degrees(frame(eps), frame(-eps));
    assert!(
        step < 0.05,
        "soft input motion with a +90° bridge jumped {step}°"
    );
}

fn r32_leg_blend(inputs: &[(Quat, f32)]) -> Quat {
    let pose = anim147_blend(
        inputs
            .iter()
            .map(|&(rotation, weight)| weighted(anim147_leg_pose(rotation, 0.0, 1.0), weight, None))
            .collect(),
    );
    pose.bones[0].local_transform.rotation
}

fn r32_arc_degrees(a: Quat, b: Quat) -> f64 {
    let unit = |q: Quat| {
        let values = q.to_array().map(f64::from);
        let length = values.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (length - 1.0).abs() < 1.0e-5,
            "expected a finite unit rotation: {q:?}"
        );
        values.map(|v| v / length)
    };
    let a = unit(a);
    let b = unit(b);
    let square = |sign: f64| (0..4).map(|i| (a[i] + sign * b[i]).powi(2)).sum::<f64>();
    // min chord on S^3, then the physical SO(3) angle. Avoid acos(f32-dot) near zero motion.
    (4.0 * (0.5 * square(-1.0).min(square(1.0)).sqrt()).asin()).to_degrees()
}

fn r32_graph_frames<const N: usize>(
    hand_degrees: [[f32; 2]; 3],
    center_mask: Option<&str>,
    frames: [(f32, f32, f32); N],
) -> [[Transform; 2]; N] {
    use zircon_runtime::asset::{AssetReference, AssetUri, ProjectAssetManager};
    use zircon_runtime::core::framework::animation::{
        AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
        AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationGraphAsset,
        AnimationGraphNodeAsset as Node, AnimationGraphParameterAsset, AnimationInterpolationAsset,
        AnimationParameterValue, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    };
    use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

    use super::super::{
        graph_evaluate::resolve_graph_pose_requests_with_demand, requests::PendingGraphPoseSample,
        AnimationEvaluationPipeline,
    };

    let assets = ProjectAssetManager::default();
    let resources = assets.resource_manager();
    let skeleton_uri = AssetUri::parse("res://animation/r32.skeleton.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let bone_asset = |name: &str, parent_index| AnimationSkeletonBoneAsset {
        name: name.into(),
        parent_index,
        local_translation: [0.0; 3],
        local_rotation: Quat::IDENTITY.to_array(),
        local_scale: [1.0; 3],
    };
    resources
        .register_ready(
            ResourceRecord::new(
                skeleton_id,
                ResourceKind::AnimationSkeleton,
                skeleton_uri.clone(),
            ),
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone_asset("Root", None), bone_asset("Hand", Some(0))],
            },
        )
        .expect("register r32 skeleton");

    let key = |time_seconds, value| AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    };
    let constant = |v| AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![key(0.0, AnimationChannelValueAsset::Vec3(v))],
    };
    let track =
        |name: &str, target: &str, x, [start, end]: [Quat; 2]| AnimationClipBoneTrackAsset {
            bone_name: name.into(),
            target_id: Some(target.into()),
            translation: constant([x, 0.0, 0.0]),
            scale: constant([1.0; 3]),
            rotation: AnimationChannelAsset {
                interpolation: AnimationInterpolationAsset::Linear,
                keys: vec![
                    key(
                        0.0,
                        AnimationChannelValueAsset::Quaternion(start.to_array()),
                    ),
                    key(1.0, AnimationChannelValueAsset::Quaternion(end.to_array())),
                ],
            },
        };
    let mut nodes = Vec::new();
    for (i, id) in ["a", "b", "c"].into_iter().enumerate() {
        let uri = AssetUri::parse(&format!("res://animation/r32-{id}.clip.zranim")).unwrap();
        let root = Quat::from_rotation_y([0.4, 1.6, 0.4][i]);
        resources
            .register_ready(
                ResourceRecord::new(
                    ResourceId::from_locator(&uri),
                    ResourceKind::AnimationClip,
                    uri.clone(),
                ),
                AnimationClipAsset {
                    name: None,
                    skeleton: AssetReference::from_locator(skeleton_uri.clone()),
                    duration_seconds: 1.0,
                    tracks: vec![
                        track("Root", "Root", [4.0, 400.0, 10.0][i], [root; 2]),
                        track(
                            "Hand",
                            "Root/Hand",
                            [2.0, 14.0, 50.0][i],
                            hand_degrees[i].map(anim147_z),
                        ),
                    ],
                    event_tracks: Vec::new(),
                },
            )
            .expect("register r32 clip");
        nodes.push(Node::Clip {
            id: id.into(),
            clip: AssetReference::from_locator(uri),
            playback_speed: 1.0,
            looping: false,
        });
    }
    nodes.push(Node::Mask {
        id: "b-hand".into(),
        input: "b".into(),
        target_ids: vec!["Root/Hand".into()],
    });
    let c = if let Some(target) = center_mask {
        nodes.push(Node::Mask {
            id: "c-root".into(),
            input: "c".into(),
            target_ids: vec![target.into()],
        });
        "c-root"
    } else {
        "c"
    };
    nodes.extend([
        Node::Blend {
            id: "outer-clips".into(),
            inputs: vec!["a".into(), "b-hand".into()],
            weight_parameter: Some("bias".into()),
        },
        Node::Blend {
            id: "all-clips".into(),
            inputs: vec!["outer-clips".into(), c.into()],
            weight_parameter: Some("center".into()),
        },
        Node::Output {
            source: "all-clips".into(),
        },
    ]);
    let graph_uri = AssetUri::parse("res://animation/r32.graph.zranim").unwrap();
    let graph_id = ResourceId::from_locator(&graph_uri);
    resources
        .register_ready(
            ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
            AnimationGraphAsset {
                name: None,
                parameters: ["bias", "center"]
                    .map(|name| AnimationGraphParameterAsset {
                        name: name.into(),
                        default_value: AnimationParameterValue::Scalar(0.0),
                    })
                    .into(),
                nodes,
            },
        )
        .expect("register r32 graph");

    let mut pipeline = AnimationEvaluationPipeline::default();
    let mut previous = 0.0;
    frames.map(|(time, bias, center)| {
        pipeline.begin_evaluation_frame(0);
        pipeline.clip_evaluator_mut().bind_resources(&resources);
        let (mut poses, _, wants_frame) = resolve_graph_pose_requests_with_demand(
            &mut pipeline,
            &assets,
            vec![PendingGraphPoseSample {
                entity: 147,
                skeleton_id,
                graph_id,
                playing: true,
                parameters: [
                    ("bias".into(), AnimationParameterValue::Scalar(bias)),
                    ("center".into(), AnimationParameterValue::Scalar(center)),
                ]
                .into(),
                from_time_seconds: previous,
                to_time_seconds: time,
            }],
        );
        previous = time;
        let pose = poses
            .remove(&147)
            .expect("compiled graph must sample its registered clips");
        assert!(wants_frame);
        assert_eq!(pose.bones.len(), 2);
        assert_eq!(pose.bones[0].name, "Root");
        assert_eq!(pose.bones[1].name, "Hand");
        [pose.bones[0].local_transform, pose.bones[1].local_transform]
    })
}
