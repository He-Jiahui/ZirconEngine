use zircon_plugin_animation_runtime::{
    AnimationAssetRevision, AnimationClipEvaluator, AnimationEvaluationError,
    AnimationTransformChannel, DefaultAnimationManager,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::AnimationManager;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSkeletonMarker, ResourceHandle, ResourceKind, ResourceManager,
    ResourceRecord,
};

#[test]
fn production_clip_evaluation_reuses_compiled_targets_without_source_strings() {
    let mut evaluator = AnimationClipEvaluator::with_pool_size(2);
    let skeleton_revision = AnimationAssetRevision::new(ResourceId::new(), 7);
    let clip_revision = AnimationAssetRevision::new(ResourceId::new(), 11);
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let mut clip = clip(
        "Root/Hand",
        linear_vec3_channel([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
    );

    let first = evaluator
        .sample_clip(
            skeleton_revision,
            clip_revision,
            &skeleton,
            &clip,
            0.5,
            false,
        )
        .unwrap();
    clip.tracks[0].bone_name = "renamed-after-cache-fill".to_string();
    clip.tracks[0].target_id = Some("Missing/Target".to_string());
    let second = evaluator
        .sample_clip(
            skeleton_revision,
            clip_revision,
            &skeleton,
            &clip,
            0.5,
            false,
        )
        .unwrap();

    assert!(
        first.bones[1]
            .local_transform
            .translation
            .abs_diff_eq(Vec3::new(5.0, 0.0, 0.0), 0.0001)
    );
    assert_eq!(
        second.bones[1].local_transform,
        first.bones[1].local_transform
    );
    let stats = evaluator.stats();
    assert_eq!(stats.skeleton_compile_count, 1);
    assert_eq!(stats.clip_compile_count, 1);
    assert_eq!(stats.clip_cache_hit_count, 1);
    assert_eq!(stats.pose_pool_miss_count, 0);
}

#[test]
fn evaluator_caches_are_bounded_and_retain_the_most_recent_entries() {
    let mut evaluator = AnimationClipEvaluator::with_limits(2, 2, 2, 4);
    let skeleton_id = ResourceId::new();
    let skeleton_revision = AnimationAssetRevision::new(skeleton_id, 1);
    let skeleton = skeleton(&[("Root", None)]);
    let mut clips = (0..3)
        .map(|index| {
            (
                ResourceId::new(),
                clip("Root", constant_vec3_channel([index as f32, 0.0, 0.0])),
            )
        })
        .collect::<Vec<_>>();

    for (clip_id, clip) in &clips[..2] {
        evaluator
            .sample_clip(
                skeleton_revision,
                AnimationAssetRevision::new(*clip_id, 1),
                &skeleton,
                clip,
                0.0,
                false,
            )
            .unwrap();
    }
    evaluator
        .sample_clip(
            skeleton_revision,
            AnimationAssetRevision::new(clips[0].0, 1),
            &skeleton,
            &clips[0].1,
            0.0,
            false,
        )
        .unwrap();
    evaluator
        .sample_clip(
            skeleton_revision,
            AnimationAssetRevision::new(clips[2].0, 1),
            &skeleton,
            &clips[2].1,
            0.0,
            false,
        )
        .unwrap();

    clips[0].1.tracks[0].target_id = Some("Missing/ButStillCached".to_string());
    evaluator
        .sample_clip(
            skeleton_revision,
            AnimationAssetRevision::new(clips[0].0, 1),
            &skeleton,
            &clips[0].1,
            0.0,
            false,
        )
        .expect("the recently touched clip remains cached");
    clips[1].1.tracks[0].target_id = Some("Missing/Evicted".to_string());
    assert!(
        evaluator
            .sample_clip(
                skeleton_revision,
                AnimationAssetRevision::new(clips[1].0, 1),
                &skeleton,
                &clips[1].1,
                0.0,
                false,
            )
            .is_err()
    );

    let stats = evaluator.stats();
    assert_eq!(stats.cached_skeleton_count, 1);
    assert_eq!(stats.cached_clip_count, 2);
    assert!(stats.clip_eviction_count >= 1);
    assert!(stats.clip_cache_hit_count >= 2);
}

#[test]
fn skeleton_cache_limit_evicts_associated_clips() {
    let mut evaluator = AnimationClipEvaluator::with_limits(1, 2, 8, 4);
    let clip = clip("Root", constant_vec3_channel([1.0, 0.0, 0.0]));

    for _ in 0..3 {
        evaluator
            .sample_clip(
                AnimationAssetRevision::new(ResourceId::new(), 1),
                AnimationAssetRevision::new(ResourceId::new(), 1),
                &skeleton(&[("Root", None)]),
                &clip,
                0.0,
                false,
            )
            .unwrap();
    }

    let stats = evaluator.stats();
    assert_eq!(stats.cached_skeleton_count, 2);
    assert!(stats.cached_clip_count <= 2);
    assert_eq!(stats.skeleton_eviction_count, 1);
}

#[test]
fn clip_revision_change_recompiles_and_rejects_the_new_invalid_target() {
    let mut evaluator = AnimationClipEvaluator::default();
    let skeleton_revision = AnimationAssetRevision::new(ResourceId::new(), 1);
    let clip_id = ResourceId::new();
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let mut clip = clip("Root/Hand", linear_vec3_channel([0.0; 3], [1.0, 0.0, 0.0]));
    evaluator
        .sample_clip(
            skeleton_revision,
            AnimationAssetRevision::new(clip_id, 1),
            &skeleton,
            &clip,
            0.0,
            false,
        )
        .unwrap();
    clip.tracks[0].target_id = Some("Missing/Target".to_string());

    let error = evaluator
        .sample_clip(
            skeleton_revision,
            AnimationAssetRevision::new(clip_id, 2),
            &skeleton,
            &clip,
            0.0,
            false,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        AnimationEvaluationError::Compile(
            zircon_plugin_animation_runtime::AnimationClipCompileError::UnresolvedTrack {
                track_index: 0,
                ref target,
            }
        ) if target == "Missing/Target"
    ));
    assert_eq!(evaluator.stats().clip_compile_count, 1);
}

#[test]
fn remove_and_readd_with_the_same_revision_cannot_reuse_the_old_payload_cache() {
    let resources = ResourceManager::new();
    let skeleton_uri = AssetUri::parse("res://animation/readd.skeleton").unwrap();
    let clip_uri = AssetUri::parse("res://animation/readd.clip").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    resources.register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        skeleton(&[("Root", None)]),
    );
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        clip("Root", constant_vec3_channel([1.0, 0.0, 0.0])),
    );
    let skeleton_snapshot = resources
        .snapshot::<AnimationSkeletonMarker, AnimationSkeletonAsset>(ResourceHandle::new(
            skeleton_id,
        ))
        .unwrap();
    let first_clip_snapshot = resources
        .snapshot::<AnimationClipMarker, AnimationClipAsset>(ResourceHandle::new(clip_id))
        .unwrap();
    let mut evaluator = AnimationClipEvaluator::for_resources(&resources);
    let first = evaluator
        .sample_clip(
            AnimationAssetRevision::new(skeleton_id, skeleton_snapshot.revision()),
            AnimationAssetRevision::new(clip_id, first_clip_snapshot.revision()),
            &skeleton_snapshot,
            &first_clip_snapshot,
            0.0,
            false,
        )
        .unwrap();

    resources.remove_by_locator(&clip_uri).unwrap();
    resources.register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip("Root", constant_vec3_channel([9.0, 0.0, 0.0])),
    );
    let second_clip_snapshot = resources
        .snapshot::<AnimationClipMarker, AnimationClipAsset>(ResourceHandle::new(clip_id))
        .unwrap();
    let second = evaluator
        .sample_clip(
            AnimationAssetRevision::new(skeleton_id, skeleton_snapshot.revision()),
            AnimationAssetRevision::new(clip_id, second_clip_snapshot.revision()),
            &skeleton_snapshot,
            &second_clip_snapshot,
            0.0,
            false,
        )
        .unwrap();

    assert_eq!(first_clip_snapshot.revision(), 1);
    assert_eq!(second_clip_snapshot.revision(), 1);
    assert_eq!(first.bones[0].local_transform.translation.x, 1.0);
    assert_eq!(second.bones[0].local_transform.translation.x, 9.0);
    assert_eq!(evaluator.stats().clip_compile_count, 2);
}

#[test]
fn duplicate_leaf_skeleton_samples_two_explicit_dense_rows() {
    let mut evaluator = AnimationClipEvaluator::default();
    let skeleton = skeleton(&[
        ("Root", None),
        ("Left", Some(0)),
        ("Hand", Some(1)),
        ("Right", Some(0)),
        ("Hand", Some(3)),
    ]);
    let clip = AnimationClipAsset {
        name: Some("TwoHands".to_string()),
        skeleton: AssetReference::from_locator(AssetUri::parse("res://hero.skeleton").unwrap()),
        duration_seconds: 1.0,
        tracks: vec![
            track("Root/Left/Hand", constant_vec3_channel([1.0, 0.0, 0.0])),
            track("Root/Right/Hand", constant_vec3_channel([2.0, 0.0, 0.0])),
        ],
        event_tracks: Vec::new(),
    };

    let pose = evaluator
        .sample_clip(
            AnimationAssetRevision::new(ResourceId::new(), 1),
            AnimationAssetRevision::new(ResourceId::new(), 1),
            &skeleton,
            &clip,
            0.0,
            false,
        )
        .unwrap();

    assert_eq!(pose.bones[2].local_transform.translation.x, 1.0);
    assert_eq!(pose.bones[4].local_transform.translation.x, 2.0);
}

#[test]
fn transform_channel_type_mismatch_is_a_structured_error() {
    let mut evaluator = AnimationClipEvaluator::default();
    let skeleton = skeleton(&[("Root", None)]);
    let clip = clip(
        "Root",
        AnimationChannelAsset {
            interpolation: AnimationInterpolationAsset::Linear,
            keys: vec![key(0.0, AnimationChannelValueAsset::Scalar(1.0))],
        },
    );

    assert_eq!(
        evaluator
            .sample_clip(
                AnimationAssetRevision::new(ResourceId::new(), 1),
                AnimationAssetRevision::new(ResourceId::new(), 1),
                &skeleton,
                &clip,
                0.0,
                false,
            )
            .unwrap_err(),
        AnimationEvaluationError::InvalidChannelValueType {
            track_index: 0,
            channel: AnimationTransformChannel::Translation,
            key_index: 0,
            role: zircon_plugin_animation_runtime::AnimationChannelDataRole::Value,
        }
    );
}

#[test]
fn clip_compile_rejects_non_finite_key_time_with_track_channel_and_key_location() {
    let mut clip = clip("Root", constant_vec3_channel([0.0; 3]));
    clip.tracks[0].translation.keys[0].time_seconds = f32::NAN;

    assert_eq!(
        sample_error(&clip),
        AnimationEvaluationError::NonFiniteChannelTime {
            track_index: 0,
            channel: AnimationTransformChannel::Translation,
            key_index: 0,
        }
    );
}

#[test]
fn clip_compile_rejects_duplicate_or_unsorted_key_times() {
    for invalid_time in [0.0, -1.0] {
        let mut clip = clip("Root", linear_vec3_channel([0.0; 3], [1.0, 0.0, 0.0]));
        clip.tracks[0].translation.keys[1].time_seconds = invalid_time;

        assert_eq!(
            sample_error(&clip),
            AnimationEvaluationError::NonIncreasingChannelTime {
                track_index: 0,
                channel: AnimationTransformChannel::Translation,
                previous_key_index: 0,
                key_index: 1,
            }
        );
    }
}

#[test]
fn clip_compile_rejects_bad_hermite_tangent_before_sampling() {
    let mut clip = clip("Root", hermite_vec3_channel());
    clip.tracks[0].translation.keys[0].out_tangent = Some(AnimationChannelValueAsset::Scalar(1.0));

    assert_eq!(
        sample_error(&clip),
        AnimationEvaluationError::InvalidChannelValueType {
            track_index: 0,
            channel: AnimationTransformChannel::Translation,
            key_index: 0,
            role: zircon_plugin_animation_runtime::AnimationChannelDataRole::OutTangent,
        }
    );
}

#[test]
fn clip_compile_rejects_non_finite_and_zero_length_rotations() {
    for (rotation, expected) in [
        (
            [f32::INFINITY, 0.0, 0.0, 1.0],
            AnimationEvaluationError::NonFiniteChannelValue {
                track_index: 0,
                channel: AnimationTransformChannel::Rotation,
                key_index: 0,
                role: zircon_plugin_animation_runtime::AnimationChannelDataRole::Value,
            },
        ),
        (
            [0.0; 4],
            AnimationEvaluationError::ZeroLengthChannelRotation {
                track_index: 0,
                key_index: 0,
            },
        ),
    ] {
        let mut clip = clip("Root", constant_vec3_channel([0.0; 3]));
        clip.tracks[0].rotation = constant_quaternion_channel(rotation);
        assert_eq!(sample_error(&clip), expected);
    }
}

#[test]
fn compiled_evaluator_matches_the_plugin_canonical_sampling_golden() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let clip = clip("Root/Hand", hermite_vec3_channel());
    assert_ne!(
        std::any::TypeId::of::<DefaultAnimationManager>(),
        std::any::TypeId::of::<zircon_runtime::animation::DefaultAnimationManager>(),
    );
    let manager = DefaultAnimationManager::default();
    let expected = manager
        .sample_clip_pose(&skeleton, &clip, 0.5, false)
        .unwrap();
    let actual = AnimationClipEvaluator::default()
        .sample_clip(
            AnimationAssetRevision::new(ResourceId::new(), 1),
            AnimationAssetRevision::new(ResourceId::new(), 1),
            &skeleton,
            &clip,
            0.5,
            false,
        )
        .unwrap();

    assert_eq!(actual.bones.len(), expected.bones.len());
    for (actual, expected) in actual.bones.iter().zip(&expected.bones) {
        assert_eq!(actual.name, expected.name);
        assert!(
            actual
                .local_transform
                .translation
                .abs_diff_eq(expected.local_transform.translation, 0.0001)
        );
        assert!(
            actual
                .local_transform
                .rotation
                .abs_diff_eq(expected.local_transform.rotation, 0.0001)
        );
        assert!(
            actual
                .local_transform
                .scale
                .abs_diff_eq(expected.local_transform.scale, 0.0001)
        );
    }
}

#[test]
fn production_scene_sampling_routes_through_the_compiled_evaluator() {
    let source = include_str!("../src/evaluation/pipeline/clip_sample.rs");
    let runtime_system = include_str!("../src/runtime_system.rs");
    let tick = include_str!("../src/evaluation/pipeline/tick.rs");

    assert!(source.contains("AnimationClipEvaluator"));
    assert!(source.contains("sample_clip("));
    assert!(!source.contains(".sample_clip_pose("));
    assert!(
        runtime_system.contains("module.resource(crate::AnimationEvaluationPipeline::default)")
    );
    assert!(!tick.contains("insert_resource(AnimationClipEvaluator"));
}

fn sample_error(clip: &AnimationClipAsset) -> AnimationEvaluationError {
    AnimationClipEvaluator::default()
        .sample_clip(
            AnimationAssetRevision::new(ResourceId::new(), 1),
            AnimationAssetRevision::new(ResourceId::new(), 1),
            &skeleton(&[("Root", None)]),
            clip,
            0.0,
            false,
        )
        .unwrap_err()
}

fn skeleton(bones: &[(&str, Option<u32>)]) -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("EvaluatorSkeleton".to_string()),
        bones: bones
            .iter()
            .map(|(name, parent_index)| AnimationSkeletonBoneAsset {
                name: (*name).to_string(),
                parent_index: *parent_index,
                local_translation: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            })
            .collect(),
    }
}

fn clip(target: &str, translation: AnimationChannelAsset) -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some("EvaluatorClip".to_string()),
        skeleton: AssetReference::from_locator(AssetUri::parse("res://hero.skeleton").unwrap()),
        duration_seconds: 1.0,
        tracks: vec![track(target, translation)],
        event_tracks: Vec::new(),
    }
}

fn track(target: &str, translation: AnimationChannelAsset) -> AnimationClipBoneTrackAsset {
    AnimationClipBoneTrackAsset {
        bone_name: target.rsplit('/').next().unwrap().to_string(),
        target_id: Some(target.to_string()),
        translation,
        rotation: constant_quaternion_channel([0.0, 0.0, 0.0, 1.0]),
        scale: constant_vec3_channel([1.0; 3]),
    }
}

fn linear_vec3_channel(left: [f32; 3], right: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![
            key(0.0, AnimationChannelValueAsset::Vec3(left)),
            key(1.0, AnimationChannelValueAsset::Vec3(right)),
        ],
    }
}

fn constant_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![key(0.0, AnimationChannelValueAsset::Vec3(value))],
    }
}

fn constant_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![key(0.0, AnimationChannelValueAsset::Quaternion(value))],
    }
}

fn hermite_vec3_channel() -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: vec![
            AnimationChannelKeyAsset {
                time_seconds: 0.0,
                value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
                in_tangent: None,
                out_tangent: Some(AnimationChannelValueAsset::Vec3([2.0, 0.0, 0.0])),
            },
            AnimationChannelKeyAsset {
                time_seconds: 1.0,
                value: AnimationChannelValueAsset::Vec3([4.0, 2.0, 0.0]),
                in_tangent: Some(AnimationChannelValueAsset::Vec3([2.0, 0.0, 0.0])),
                out_tangent: None,
            },
        ],
    }
}

fn key(time_seconds: f32, value: AnimationChannelValueAsset) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    }
}
