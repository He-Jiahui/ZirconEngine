use zircon_plugin_physics_runtime::RagdollProfile;
use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::math::{Mat4, Quat, Real, Vec3};
use zircon_runtime::core::resource::{
    ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceId, ResourceKind,
    ResourceLocator, ResourceManager, ResourceRecord,
};

use crate::{
    generate_ragdoll_profile_from_skeleton, prepare_bound_ragdoll_body_translations,
    prepare_bound_ragdoll_profile_edits, RagdollBoneProperty, RagdollBoundBodyTranslation,
    RagdollBoundBoneEdit, RagdollBoundEditError as BoundError, RagdollProfileEditError,
    RagdollProfileFromSkeleton,
};

fn payload() -> AnimationSkeletonAsset {
    let bone = |name: &str, parent_index| AnimationSkeletonBoneAsset {
        name: name.to_owned(),
        parent_index,
        local_translation: [0.0, 0.5, 0.0],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    };
    AnimationSkeletonAsset {
        name: None,
        // Two equal labels, and child before parent: neither labels nor preorder are identities.
        bones: vec![
            bone("Head", Some(2)),
            bone("Head", None),
            bone("Root", None),
        ],
    }
}

fn record(name: &str) -> ResourceRecord {
    let locator = ResourceLocator::parse(&format!("res://ragdoll-bound/{name}.zranim")).unwrap();
    ResourceRecord::new(
        ResourceId::from_locator(&locator),
        ResourceKind::AnimationSkeleton,
        locator,
    )
}

fn install(resources: &ResourceManager, record: ResourceRecord) -> ResourceActiveVersionKey {
    let id = record.id;
    resources.register_ready(record, payload()).unwrap();
    resources.projection_snapshot().active().key(id).unwrap()
}

fn source(
    resources: &ResourceManager,
    version: &ResourceActiveVersionKey,
    paths: &[&str],
) -> RagdollProfileFromSkeleton {
    generate_ragdoll_profile_from_skeleton(
        resources,
        version,
        "character",
        &paths
            .iter()
            .map(|path| (*path).to_owned())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn setup(resources: &ResourceManager, name: &str) -> RagdollProfileFromSkeleton {
    let version = install(resources, record(name));
    source(resources, &version, &["角色/Head", "Other/Head", "角色"])
}

fn mass(source: &RagdollProfileFromSkeleton, path: &str, value: Real) -> RagdollBoundBoneEdit {
    RagdollBoundBoneEdit {
        bone: source.bone_key(path).unwrap(),
        property: RagdollBoneProperty::Mass(value),
    }
}

fn changed(
    resources: &ResourceManager,
    source: &RagdollProfileFromSkeleton,
    profile: &RagdollProfile,
) -> crate::RagdollProfileEditCandidate {
    prepare_bound_ragdoll_profile_edits(
        resources,
        source,
        profile,
        &[mass(source, "角色/Head", 3.0)],
    )
    .unwrap()
    .unwrap()
}

#[test]
fn keys_preserve_exact_authored_paths_original_slots_and_source_version() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "original-slots");
    let head = source.bone_key("角色/Head").unwrap();
    assert_eq!(head.original_joint_slot(), 0);
    assert_eq!(head.bone_path(), "角色/Head");
    assert_eq!(head.skeleton_version(), source.skeleton_version());
    assert_eq!(head.skeleton_revision(), source.skeleton_revision());
    assert_eq!(
        source.bone_key("Other/Head").unwrap().original_joint_slot(),
        1
    );
    assert_eq!(source.bone_key("角色").unwrap().original_joint_slot(), 2);
    for wrong in ["Head", "角色/head", " 角色/Head", "角色/Head "] {
        assert!(source.bone_key(wrong).is_none());
    }
}

#[test]
fn typed_batch_preserves_profile_order_source_and_unselected_properties() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "batch");
    let mut profile = source.profile().clone();
    profile.bones.reverse();
    let original = profile.clone();
    let candidate = prepare_bound_ragdoll_profile_edits(
        &resources,
        &source,
        &profile,
        &[
            mass(&source, "角色/Head", 3.0),
            mass(&source, "Other/Head", 4.0),
            mass(&source, "角色/Head", 5.0),
        ],
    )
    .unwrap()
    .unwrap();
    assert_eq!(profile, original);
    assert_eq!(candidate.before(), &original);
    assert_eq!(candidate.after().bones[0], original.bones[0]);
    assert_eq!(candidate.after().bones[1].mass, 4.0);
    assert_eq!(candidate.after().bones[2].mass, 5.0);
    let mut restored = candidate.after().clone();
    restored.bones[1].mass = original.bones[1].mass;
    restored.bones[2].mass = original.bones[2].mass;
    assert_eq!(restored, original);
    assert_eq!(candidate.skeleton_version(), source.skeleton_version());
    assert_eq!(candidate.skeleton_revision(), source.skeleton_revision());
    candidate.validate_current(&resources, &profile).unwrap();
}

#[test]
fn empty_net_zero_and_zero_delta_produce_no_history_candidate() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "no-op");
    for edits in [
        vec![],
        vec![
            mass(&source, "角色/Head", 2.0),
            mass(&source, "角色/Head", 1.0),
        ],
    ] {
        assert!(
            prepare_bound_ragdoll_profile_edits(&resources, &source, source.profile(), &edits)
                .unwrap()
                .is_none()
        );
    }
    assert!(prepare_bound_ragdoll_body_translations(
        &resources,
        &source,
        source.profile(),
        &[RagdollBoundBodyTranslation {
            bone: source.bone_key("角色/Head").unwrap(),
            bone_to_world: Mat4::IDENTITY,
            world_delta: Vec3::ZERO,
        }],
    )
    .unwrap()
    .is_none());
}

#[test]
fn later_foreign_resource_key_rejects_the_whole_batch() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "original");
    let foreign = setup(&resources, "foreign");
    let original = source.profile().clone();
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(
            &resources,
            &source,
            &original,
            &[
                mass(&source, "角色/Head", 3.0),
                mass(&foreign, "Other/Head", 4.0)
            ],
        ),
        Err(BoundError::WrongSkeletonBone { joint_index: 1, .. })
    ));
    assert_eq!(source.profile(), &original);
}

#[test]
fn same_version_with_different_path_to_slot_mapping_rejects_old_keys() {
    let resources = ResourceManager::new();
    let original = setup(&resources, "remapped");
    let remapped = source(
        &resources,
        original.skeleton_version(),
        &["Other/Head", "角色/Head", "角色"],
    );
    assert_eq!(original.skeleton_version(), remapped.skeleton_version());
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(
            &resources,
            &remapped,
            remapped.profile(),
            &[mass(&original, "角色/Head", 3.0)],
        ),
        Err(BoundError::WrongSkeletonBone { joint_index: 0, .. })
    ));
}

#[test]
fn current_source_key_cannot_edit_a_bone_absent_from_the_profile() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "profile-subset");
    let mut profile = source.profile().clone();
    profile.bones.remove(1);
    let before = profile.clone();
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(
            &resources, &source, &profile,
            &[mass(&source, "角色/Head", 3.0), mass(&source, "Other/Head", 4.0)],
        ),
        Err(BoundError::Edit(RagdollProfileEditError::MissingBone { bone_path }))
            if bone_path == "Other/Head"
    ));
    assert_eq!(profile, before);
}

#[test]
fn invalid_profile_is_rejected_even_with_no_selected_edits() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "invalid-profile");
    let mut profile = source.profile().clone();
    profile.bones[0].mass = 0.0;
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(&resources, &source, &profile, &[]),
        Err(BoundError::Edit(RagdollProfileEditError::InvalidProfile(_)))
    ));
}

#[test]
fn equal_payload_replacement_invalidates_retained_keys_and_candidates() {
    let resources = ResourceManager::new();
    let previous = setup(&resources, "replacement");
    let candidate = changed(&resources, &previous, previous.profile());
    resources
        .store_payload(
            previous.skeleton_version().id(),
            previous.skeleton_revision(),
            payload(),
        )
        .unwrap();
    let edits = [mass(&previous, "角色/Head", 3.0)];
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(&resources, &previous, previous.profile(), &edits),
        Err(BoundError::SkeletonSnapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
    assert!(matches!(
        candidate.validate_current(&resources, previous.profile()),
        Err(BoundError::SkeletonSnapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
    assert_eq!(
        previous
            .bone_key("角色/Head")
            .unwrap()
            .original_joint_slot(),
        0
    );
    assert_eq!(candidate.before(), previous.profile());
}

#[test]
fn equal_revision_remove_recreate_rejects_old_binding_and_old_target() {
    let resources = ResourceManager::new();
    let previous = setup(&resources, "aba");
    let candidate = changed(&resources, &previous, previous.profile());
    resources
        .remove_by_locator(&record("aba").primary_locator)
        .unwrap()
        .unwrap();
    let current = setup(&resources, "aba");
    assert_eq!(previous.skeleton_revision(), current.skeleton_revision());
    assert_ne!(previous.skeleton_version(), current.skeleton_version());
    assert!(matches!(
        candidate.validate_current(&resources, current.profile()),
        Err(BoundError::SkeletonSnapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
    assert!(matches!(
        prepare_bound_ragdoll_profile_edits(
            &resources,
            &current,
            current.profile(),
            &[mass(&previous, "角色/Head", 3.0)],
        ),
        Err(BoundError::WrongSkeletonBone { .. })
    ));
    changed(&resources, &current, current.profile())
        .validate_current(&resources, current.profile())
        .unwrap();
}

#[test]
fn revalidation_detects_changes_anywhere_in_the_profile_without_mutating_it() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "changed-profile");
    let candidate = changed(&resources, &source, source.profile());
    let mut current = source.profile().clone();
    current.bones[1].blend_weight = 0.25;
    let before = current.clone();
    assert!(matches!(
        candidate.validate_current(&resources, &current),
        Err(BoundError::ProfileChanged)
    ));
    assert!(matches!(
        candidate.validate_current(&resources, candidate.after()),
        Err(BoundError::ProfileChanged)
    ));
    assert_eq!(current, before);
    candidate
        .validate_current(&resources, source.profile())
        .unwrap();
}

#[test]
fn bound_multi_body_translation_uses_each_frame_and_rejects_duplicate_targets_atomically() {
    let resources = ResourceManager::new();
    let source = setup(&resources, "translation");
    let original = source.profile().clone();
    let local_delta = Vec3::new(0.5, -0.25, 0.125);
    let frames = [
        Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 3.0, 0.5),
            Quat::from_rotation_z(0.5),
            Vec3::splat(900.0),
        ),
        Mat4::from_scale_rotation_translation(
            Vec3::new(-1.0, 2.0, 4.0),
            Quat::IDENTITY,
            Vec3::splat(-700.0),
        ),
    ];
    let translations = ["角色/Head", "Other/Head"]
        .into_iter()
        .zip(frames)
        .map(|(path, frame)| RagdollBoundBodyTranslation {
            bone: source.bone_key(path).unwrap(),
            bone_to_world: frame,
            world_delta: frame.transform_vector3(local_delta),
        })
        .collect::<Vec<_>>();
    let candidate =
        prepare_bound_ragdoll_body_translations(&resources, &source, &original, &translations)
            .unwrap()
            .unwrap();
    for index in 0..2 {
        let delta = candidate.after().bones[index].body_offset.translation
            - original.bones[index].body_offset.translation;
        assert!((delta - local_delta).length() < 1.0e-5);
    }
    assert_eq!(source.profile(), &original);
    assert_eq!(candidate.before(), &original);
    let duplicates = [translations[0].clone(), translations[0].clone()];
    assert!(matches!(
        prepare_bound_ragdoll_body_translations(&resources, &source, &original, &duplicates,),
        Err(BoundError::Translation(
            crate::RagdollBodyTranslationError::DuplicateTarget { .. }
        ))
    ));
    assert_eq!(source.profile(), &original);
}
