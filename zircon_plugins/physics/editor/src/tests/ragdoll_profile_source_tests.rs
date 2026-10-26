use zircon_plugin_physics_runtime::RagdollProfileError;
use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::framework::physics::PhysicsColliderShape;
use zircon_runtime::core::math::{Real, Transform};
use zircon_runtime::core::resource::{
    AnimationSkeletonMarker, ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle,
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord,
};

use crate::{
    generate_ragdoll_profile_from_skeleton, RagdollProfileFromSkeletonError as SourceError,
};

fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.to_owned(),
        parent_index,
        local_translation: [0.0, 0.25, 0.0],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    }
}

fn record(name: &str) -> ResourceRecord {
    let locator = ResourceLocator::parse(&format!("res://ragdoll-source/{name}.zranim")).unwrap();
    ResourceRecord::new(
        ResourceId::from_locator(&locator),
        ResourceKind::AnimationSkeleton,
        locator,
    )
}

fn install(
    resources: &ResourceManager,
    record: ResourceRecord,
    bones: Vec<AnimationSkeletonBoneAsset>,
) -> ResourceActiveVersionKey {
    let id = record.id;
    resources
        .register_ready(record, AnimationSkeletonAsset { name: None, bones })
        .unwrap();
    resources.projection_snapshot().active().key(id).unwrap()
}

fn paths(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn installed_snapshot_preserves_original_slots_and_source_transforms() {
    let resources = ResourceManager::new();
    let mut child = bone("Head", Some(2));
    child.local_translation = [0.0, 2.0, 0.0];
    let bones = vec![child, bone("other-root", None), bone("root", None)];
    let version = install(&resources, record("original-slots"), bones.clone());
    let candidate = generate_ragdoll_profile_from_skeleton(
        &resources,
        &version,
        "character",
        &paths(&["root/Head", "other-root", "root"]),
    )
    .unwrap();
    assert_eq!(candidate.skeleton_version(), &version);
    assert_eq!(candidate.profile().id, "character");
    assert_eq!(candidate.original_joint_slot("root/Head"), Some(0));
    assert_eq!(candidate.original_joint_slot("root"), Some(2));
    assert_eq!(
        candidate.profile().bones[0].parent_bone_path.as_deref(),
        Some("root")
    );
    assert_eq!(
        candidate.profile().bones[0].shape,
        PhysicsColliderShape::Capsule {
            radius: 0.25,
            half_height: 1.0,
        }
    );
    assert!(candidate
        .profile()
        .bones
        .iter()
        .all(|bone| bone.body_offset == Transform::default()));
    let retained = resources
        .snapshot_expected::<AnimationSkeletonMarker, AnimationSkeletonAsset>(
            ResourceHandle::new(version.id()),
            &version,
        )
        .unwrap();
    assert_eq!(retained.bones, bones);
    assert_eq!(candidate.skeleton_revision(), retained.revision());
}

#[test]
fn duplicate_display_names_keep_exact_chinese_and_long_authored_paths() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("exact-paths"),
        vec![bone("Head", None), bone("Head", Some(0))],
    );
    let long_path = format!("角色/{}/Head ", "长骨骼名称".repeat(30));
    let source_paths = vec!["角色/Head".to_owned(), long_path.clone()];
    let candidate =
        generate_ragdoll_profile_from_skeleton(&resources, &version, "exact", &source_paths)
            .unwrap();
    assert_eq!(candidate.original_joint_slot(&long_path), Some(1));
    assert_eq!(candidate.original_joint_slot("角色/Head"), Some(0));
    assert_eq!(candidate.original_joint_slot("Head"), None);
    assert_eq!(candidate.original_joint_slot("角色/head"), None);
    assert_eq!(candidate.original_joint_slot(long_path.trim()), None);
    assert_eq!(candidate.profile().bones[1].bone_path, long_path);
}

#[test]
fn path_count_must_match_the_installed_payload_before_generation() {
    let resources = ResourceManager::new();
    let version = install(&resources, record("path-count"), vec![bone("root", None)]);
    for source_paths in [Vec::new(), paths(&["root", "extra"])] {
        assert!(matches!(generate_ragdoll_profile_from_skeleton(
            &resources, &version, "count", &source_paths,
        ), Err(SourceError::PathCountMismatch { bone_count: 1, path_count })
            if path_count == source_paths.len()));
    }
}

#[test]
fn invalid_parent_slots_and_cycles_cannot_produce_a_candidate() {
    let resources = ResourceManager::new();
    let missing = install(
        &resources,
        record("missing-parent"),
        vec![bone("root", Some(4))],
    );
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(&resources, &missing, "missing", &paths(&["root"]),),
        Err(SourceError::ParentOutOfBounds {
            joint_index: 0,
            parent_index: 4
        })
    ));
    for (name, bones, source_paths, expected) in [
        (
            "cycle",
            vec![bone("a", Some(1)), bone("b", Some(0))],
            paths(&["a", "b"]),
            RagdollProfileError::ParentCycle {
                bone_path: "a".to_owned(),
            },
        ),
        (
            "self-parent",
            vec![bone("a", Some(0))],
            paths(&["a"]),
            RagdollProfileError::MissingParent {
                bone_path: "a".to_owned(),
                parent_bone_path: "a".to_owned(),
            },
        ),
    ] {
        let version = install(&resources, record(name), bones);
        assert!(matches!(generate_ragdoll_profile_from_skeleton(
            &resources, &version, "cycle", &source_paths,
        ), Err(SourceError::Profile(actual)) if actual == expected));
    }
}

#[test]
fn authored_path_validation_reuses_the_profile_policy_without_normalizing() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("path-policy"),
        vec![bone("a", None), bone("b", Some(0))],
    );
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(
            &resources,
            &version,
            "duplicate",
            &paths(&["same", "same"]),
        ),
        Err(SourceError::Profile(
            RagdollProfileError::DuplicateBonePath { .. }
        ))
    ));
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(
            &resources,
            &version,
            "empty",
            &paths(&["root", "  "]),
        ),
        Err(SourceError::Profile(RagdollProfileError::EmptyBonePath {
            index: 1
        }))
    ));
}

#[test]
fn nonfinite_source_trs_is_rejected_even_for_an_unused_root_component() {
    let resources = ResourceManager::new();
    for index in 0..3 {
        let mut root = bone("root", None);
        match index {
            0 => root.local_translation[0] = Real::NAN,
            1 => root.local_rotation[0] = Real::INFINITY,
            _ => root.local_scale[0] = Real::NAN,
        }
        let version = install(
            &resources,
            record(&format!("nonfinite-{index}")),
            vec![root],
        );
        assert!(matches!(
            generate_ragdoll_profile_from_skeleton(
                &resources,
                &version,
                "invalid",
                &paths(&["root"]),
            ),
            Err(SourceError::NonFiniteTransform { joint_index: 0 })
        ));
    }
}

#[test]
fn resource_lookup_rejects_wrong_kind_and_wrong_payload_type() {
    let resources = ResourceManager::new();
    for (name, wrong_kind) in [("wrong-kind", true), ("wrong-payload", false)] {
        let mut record = record(name);
        if wrong_kind {
            record.kind = ResourceKind::Mesh;
        }
        let id = record.id;
        resources.register_ready(record, 17_u64).unwrap();
        let version = resources.projection_snapshot().active().key(id).unwrap();
        assert!(matches!(
            generate_ragdoll_profile_from_skeleton(
                &resources,
                &version,
                "wrong",
                &paths(&["root"]),
            ),
            Err(SourceError::Snapshot(
                ResourceActiveSnapshotError::KindConflict
            ))
        ));
    }
}

#[test]
fn payload_replacement_rejects_old_version_and_retains_old_candidate_only() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("replacement"),
        vec![bone("root", None), bone("child", Some(0))],
    );
    let source_paths = paths(&["root", "root/child"]);
    let previous =
        generate_ragdoll_profile_from_skeleton(&resources, &version, "before", &source_paths)
            .unwrap();
    let mut child = bone("child", Some(0));
    child.local_translation = [0.0, 2.0, 0.0];
    resources
        .store_payload(
            version.id(),
            previous.skeleton_revision(),
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone("root", None), child],
            },
        )
        .unwrap();
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(&resources, &version, "stale", &source_paths,),
        Err(SourceError::Snapshot(ResourceActiveSnapshotError::Stale))
    ));
    let current_version = resources
        .projection_snapshot()
        .active()
        .key(version.id())
        .unwrap();
    let current = generate_ragdoll_profile_from_skeleton(
        &resources,
        &current_version,
        "after",
        &source_paths,
    )
    .unwrap();
    assert_ne!(current.skeleton_version(), previous.skeleton_version());
    assert_ne!(
        current.profile().bones[1].shape,
        previous.profile().bones[1].shape
    );
    assert_eq!(previous.skeleton_version(), &version);
    assert_eq!(previous.profile().id, "before");
}

#[test]
fn removal_and_recreation_cannot_reuse_old_resource_authority() {
    let resources = ResourceManager::new();
    let original = record("recreated");
    let version = install(&resources, original.clone(), vec![bone("root", None)]);
    let previous =
        generate_ragdoll_profile_from_skeleton(&resources, &version, "before", &paths(&["root"]))
            .unwrap();
    resources
        .remove_by_locator(&original.primary_locator)
        .unwrap()
        .unwrap();
    let current_version = install(&resources, original, vec![bone("root", None)]);
    let current = generate_ragdoll_profile_from_skeleton(
        &resources,
        &current_version,
        "after",
        &paths(&["root"]),
    )
    .unwrap();
    assert_eq!(previous.skeleton_revision(), current.skeleton_revision());
    assert_ne!(previous.skeleton_version(), current.skeleton_version());
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(&resources, &version, "stale", &paths(&["root"]),),
        Err(SourceError::Snapshot(ResourceActiveSnapshotError::Stale))
    ));
}

#[test]
fn an_empty_installed_skeleton_is_not_a_generated_profile() {
    let resources = ResourceManager::new();
    let version = install(&resources, record("empty"), Vec::new());
    assert!(matches!(
        generate_ragdoll_profile_from_skeleton(&resources, &version, "empty", &[],),
        Err(SourceError::Profile(RagdollProfileError::EmptyProfile))
    ));
}
