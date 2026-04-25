use crate::asset::assets::{
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use crate::core::framework::animation::{AnimationManager, AnimationPoseSource};
use crate::core::math::{Quat, Vec3};

use super::{asset_reference, quaternion_channel, vec3_channel};

#[test]
fn animation_manager_samples_clip_pose_against_skeleton() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".to_string(),
                parent_index: None,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: [1.0, 1.0, 1.0],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".to_string(),
                parent_index: Some(0),
                local_translation: [0.0, 1.0, 0.0],
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    };
    let clip = AnimationClipAsset {
        name: Some("Wave".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.0, 1.0, 0.0]), (1.0, [0.0, 2.0, 0.0])]),
            rotation: quaternion_channel([
                (0.0, Quat::IDENTITY.to_array()),
                (
                    1.0,
                    Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array(),
                ),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [2.0, 2.0, 2.0])]),
        }],
    };

    let pose = manager
        .sample_clip_pose(&skeleton, &clip, 0.5, true)
        .unwrap();

    assert_eq!(pose.source, AnimationPoseSource::Clip);
    assert_eq!(pose.bones.len(), 2);
    let hand = pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("missing hand pose");
    assert_eq!(hand.local_transform.translation, Vec3::new(0.0, 1.5, 0.0));
    assert_eq!(hand.local_transform.scale, Vec3::splat(1.5));
}

#[test]
fn animation_manager_samples_clip_pose_clamps_non_finite_timing_to_start() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "Hand".to_string(),
            parent_index: None,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: [1.0, 1.0, 1.0],
        }],
    };
    let mut clip = AnimationClipAsset {
        name: Some("Wave".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: f32::NAN,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.0, 1.0, 0.0]), (1.0, [0.0, 2.0, 0.0])]),
            rotation: quaternion_channel([
                (0.0, Quat::IDENTITY.to_array()),
                (1.0, Quat::IDENTITY.to_array()),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [2.0, 2.0, 2.0])]),
        }],
    };

    let pose_with_bad_duration = manager
        .sample_clip_pose(&skeleton, &clip, 0.75, true)
        .unwrap();
    let hand_with_bad_duration = pose_with_bad_duration
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("missing hand pose");
    assert_eq!(
        hand_with_bad_duration.local_transform.translation,
        Vec3::new(0.0, 1.0, 0.0)
    );
    assert_eq!(
        hand_with_bad_duration.local_transform.scale,
        Vec3::splat(1.0)
    );

    clip.duration_seconds = 1.0;
    let pose_with_bad_time = manager
        .sample_clip_pose(&skeleton, &clip, f32::INFINITY, true)
        .unwrap();
    let hand_with_bad_time = pose_with_bad_time
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("missing hand pose");
    assert_eq!(
        hand_with_bad_time.local_transform.translation,
        Vec3::new(0.0, 1.0, 0.0)
    );
    assert_eq!(hand_with_bad_time.local_transform.scale, Vec3::splat(1.0));
}

#[test]
fn animation_manager_rejects_clip_pose_with_non_finite_channel_values() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "Hand".to_string(),
            parent_index: None,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: [1.0, 1.0, 1.0],
        }],
    };
    let clip = AnimationClipAsset {
        name: Some("BadWave".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.0, f32::NAN, 0.0]), (1.0, [0.0, 2.0, 0.0])]),
            rotation: quaternion_channel([
                (0.0, Quat::IDENTITY.to_array()),
                (1.0, Quat::IDENTITY.to_array()),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [2.0, 2.0, 2.0])]),
        }],
    };

    match manager.sample_clip_pose(&skeleton, &clip, 0.0, true) {
        Ok(pose) => panic!("expected non-finite clip channel value to be rejected, got {pose:?}"),
        Err(error) => assert!(error.contains("non-finite"), "{error}"),
    }
}

#[test]
fn animation_manager_rejects_clip_pose_with_zero_length_quaternion_channel() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "Hand".to_string(),
            parent_index: None,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: [1.0, 1.0, 1.0],
        }],
    };
    let clip = AnimationClipAsset {
        name: Some("BadWave".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.0, 1.0, 0.0]), (1.0, [0.0, 1.0, 0.0])]),
            rotation: quaternion_channel([
                (0.0, [0.0, 0.0, 0.0, 0.0]),
                (1.0, [0.0, 0.0, 0.0, 0.0]),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [1.0, 1.0, 1.0])]),
        }],
    };

    match manager.sample_clip_pose(&skeleton, &clip, 0.0, true) {
        Ok(pose) => {
            panic!("expected zero-length quaternion sample to be rejected, got {pose:?}")
        }
        Err(error) => assert!(error.contains("zero-length"), "{error}"),
    }
}

#[test]
fn animation_manager_rejects_clip_pose_with_non_finite_skeleton_bind_pose() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("BadSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "Hand".to_string(),
            parent_index: None,
            local_translation: [0.0, f32::NAN, 0.0],
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: [1.0, 1.0, 1.0],
        }],
    };
    let clip = AnimationClipAsset {
        name: Some("EmptyWave".to_string()),
        skeleton: asset_reference("res://animation/bad.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: Vec::new(),
    };

    match manager.sample_clip_pose(&skeleton, &clip, 0.0, true) {
        Ok(pose) => {
            panic!("expected non-finite skeleton bind pose to be rejected, got {pose:?}")
        }
        Err(error) => assert!(error.contains("non-finite"), "{error}"),
    }
}

#[test]
fn animation_manager_rejects_clip_pose_with_zero_length_skeleton_bind_rotation() {
    let manager = super::super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("BadSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "Hand".to_string(),
            parent_index: None,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: [0.0, 0.0, 0.0, 0.0],
            local_scale: [1.0, 1.0, 1.0],
        }],
    };
    let clip = AnimationClipAsset {
        name: Some("EmptyWave".to_string()),
        skeleton: asset_reference("res://animation/bad.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: Vec::new(),
    };

    match manager.sample_clip_pose(&skeleton, &clip, 0.0, true) {
        Ok(pose) => {
            panic!("expected zero-length skeleton bind rotation to be rejected, got {pose:?}")
        }
        Err(error) => assert!(error.contains("zero-length"), "{error}"),
    }
}
