use std::sync::Arc;

use zircon_plugin_animation_runtime::{
    AnimationClipCompileError, CompiledAnimationClip, SkeletonTargetTable,
};
use zircon_runtime::core::framework::animation::AnimationTargetId;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationClipBoneTrackAsset, AnimationInterpolationAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::scene::EntityPath;

#[test]
fn target_id_is_stable_across_reimport_and_has_a_golden_encoding() {
    let imported_path = EntityPath::parse("Armature/Hips/Spine/Chest").unwrap();
    let reimported_path = EntityPath::new(
        ["Armature", "Hips", "Spine", "Chest"]
            .into_iter()
            .map(str::to_string)
            .collect(),
    )
    .unwrap();

    let imported_id = AnimationTargetId::from_path(&imported_path);
    assert_eq!(imported_id, AnimationTargetId::from_path(&reimported_path));
    assert_eq!(
        imported_id.as_bytes(),
        [135, 4, 92, 99, 71, 46, 103, 133, 47, 42, 67, 184, 28, 111, 114, 206]
    );
}

#[test]
fn target_id_hash_preserves_every_segment_boundary() {
    let cases = [
        (vec!["a", "bc"], vec!["ab", "c"]),
        (
            vec!["Armature/Hips", "Hand"],
            vec!["Armature", "Hips", "Hand"],
        ),
        (vec!["", "Hand"], vec!["Hand"]),
        (vec!["手", "Arm"], vec!["手A", "rm"]),
    ];

    for (left, right) in cases {
        assert_ne!(
            AnimationTargetId::from_segments(left),
            AnimationTargetId::from_segments(right)
        );
    }
}

#[test]
fn skeleton_table_is_compiled_once_and_clip_keeps_its_owner() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    let source_tracks = vec![track("legacy-hand-name", Some("Root/Hand"))];

    let compiled = CompiledAnimationClip::compile(Arc::clone(&table), &source_tracks).unwrap();

    assert!(std::ptr::eq(compiled.target_table(), table.as_ref()));
    assert_eq!(compiled.target_index_for_track(0), Some(1));
    assert_eq!(table.len(), 2);
    assert_eq!(
        table.bone_index_for_target(AnimationTargetId::from_segments(["Root", "Hand"])),
        Some(1)
    );
}

#[test]
fn compiled_channel_resolution_does_not_revisit_source_strings() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    let mut source_tracks = vec![track("legacy-hand-name", Some("Root/Hand"))];

    let compiled = CompiledAnimationClip::compile(table, &source_tracks).unwrap();
    source_tracks[0].bone_name = "renamed-after-compilation".to_string();
    source_tracks[0].target_id = Some("Missing/Target".to_string());

    assert_eq!(compiled.target_index_for_track(0), Some(1));
    assert_eq!(compiled.tracks().len(), 1);
}

#[test]
fn duplicate_leaf_name_requires_a_full_target_path() {
    let skeleton = skeleton(&[
        ("Root", None),
        ("Left", Some(0)),
        ("Hand", Some(1)),
        ("Right", Some(0)),
        ("Hand", Some(3)),
    ]);
    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());

    let error = CompiledAnimationClip::compile(table, &[track("Hand", None)]).unwrap_err();

    assert!(matches!(
        error,
        AnimationClipCompileError::AmbiguousTrack {
            track_index: 0,
            ref target,
        } if target == "Hand"
    ));
}

#[test]
fn missing_leaf_name_is_reported_as_unresolved() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());

    let error = CompiledAnimationClip::compile(table, &[track("Missing", None)]).unwrap_err();

    assert_eq!(
        error,
        AnimationClipCompileError::UnresolvedTrack {
            track_index: 0,
            target: "Missing".to_string(),
        }
    );
}

#[test]
fn duplicate_clip_tracks_for_one_target_are_rejected() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);
    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    let expected_id = table.target_id_for_bone(1).unwrap();

    let error = CompiledAnimationClip::compile(
        table,
        &[track("ignored", Some("Root/Hand")), track("Hand", None)],
    )
    .unwrap_err();

    assert_eq!(
        error,
        AnimationClipCompileError::DuplicateTrackTarget {
            first_track_index: 0,
            duplicate_track_index: 1,
            target_id: expected_id,
        }
    );
}

#[test]
fn skeleton_rejects_non_canonical_bone_names() {
    for bone_name in [" Hand", "Hand ", "Arm/Hand"] {
        let error =
            SkeletonTargetTable::compile(&skeleton(&[("Root", None), (bone_name, Some(0))]))
                .unwrap_err();
        assert!(matches!(
            error,
            AnimationClipCompileError::NonCanonicalBoneName {
                bone_index: 1,
                ref name,
            } if name == bone_name
        ));
    }

    assert_eq!(
        SkeletonTargetTable::compile(&skeleton(&[("", None)])).unwrap_err(),
        AnimationClipCompileError::EmptyBoneName { bone_index: 0 }
    );
}

#[test]
fn clip_rejects_non_canonical_explicit_paths_and_leaf_names() {
    let skeleton = skeleton(&[("Root", None), ("Hand", Some(0))]);

    for target in [" Root/Hand", "Root//Hand", "Root/Hand/", "Root /Hand"] {
        let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
        let error =
            CompiledAnimationClip::compile(table, &[track("ignored", Some(target))]).unwrap_err();
        assert_eq!(
            error,
            AnimationClipCompileError::NonCanonicalTrackTarget {
                track_index: 0,
                target: target.to_string(),
            }
        );
    }

    let table = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    assert_eq!(
        CompiledAnimationClip::compile(table, &[track(" Hand ", None)]).unwrap_err(),
        AnimationClipCompileError::NonCanonicalTrackTarget {
            track_index: 0,
            target: " Hand ".to_string(),
        }
    );
}

#[test]
fn skeleton_parent_errors_report_the_immediate_bad_bone() {
    let invalid_parent = skeleton(&[("Root", None), ("Child", Some(99))]);
    assert_eq!(
        SkeletonTargetTable::compile(&invalid_parent).unwrap_err(),
        AnimationClipCompileError::InvalidParentIndex {
            bone_index: 1,
            parent_index: 99,
        }
    );

    let self_cycle = skeleton(&[("Root", Some(0))]);
    assert_eq!(
        SkeletonTargetTable::compile(&self_cycle).unwrap_err(),
        AnimationClipCompileError::ParentCycle { bone_index: 0 }
    );

    let two_bone_cycle = skeleton(&[("A", Some(1)), ("B", Some(0))]);
    assert!(matches!(
        SkeletonTargetTable::compile(&two_bone_cycle).unwrap_err(),
        AnimationClipCompileError::ParentCycle { .. }
    ));
}

#[test]
fn duplicate_skeleton_paths_are_rejected_with_their_stable_id() {
    let skeleton = skeleton(&[("Root", None), ("Root", None)]);
    let expected_id = AnimationTargetId::from_segments(["Root"]);

    assert_eq!(
        SkeletonTargetTable::compile(&skeleton).unwrap_err(),
        AnimationClipCompileError::DuplicateTarget {
            target_id: expected_id,
        }
    );
}

fn skeleton(bones: &[(&str, Option<u32>)]) -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("ContractSkeleton".to_string()),
        bones: bones
            .iter()
            .map(|(name, parent_index)| bone(name, *parent_index))
            .collect(),
    }
}

fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.to_string(),
        parent_index,
        local_translation: [0.0; 3],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    }
}

fn track(bone_name: &str, target_id: Option<&str>) -> AnimationClipBoneTrackAsset {
    AnimationClipBoneTrackAsset {
        bone_name: bone_name.to_string(),
        target_id: target_id.map(str::to_string),
        translation: empty_channel(),
        rotation: empty_channel(),
        scale: empty_channel(),
    }
}

fn empty_channel() -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: Vec::new(),
    }
}
