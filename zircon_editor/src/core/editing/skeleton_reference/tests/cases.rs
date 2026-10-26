use std::collections::HashSet;

use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::resource::{
    ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceId, ResourceKind,
    ResourceLocator, ResourceManager, ResourceRecord, ResourceState,
};

use super::{SkeletonReference, SkeletonReferenceError, SkeletonReferenceHierarchyError};

#[path = "details.rs"]
mod details;
#[path = "reveal.rs"]
mod reveal;
#[path = "skin.rs"]
mod skin;
#[path = "space.rs"]
mod space;

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
    let locator = ResourceLocator::parse(&format!("res://skeleton-reference/{name}.zranim"))
        .expect("valid test skeleton locator");
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
        .expect("publish real typed skeleton payload");
    resources
        .projection_snapshot()
        .active()
        .key(id)
        .expect("published active skeleton version")
}

#[test]
fn source_slots_survive_hierarchy_order_duplicate_names_and_filtering() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("source-slots"),
        vec![
            bone("Head/头部", Some(2)),
            bone("Head/头部", Some(0)),
            bone("根骨骼", None),
            bone("其他根骨骼", None),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let first = reference.key(0).unwrap();
    let second = reference.key(1).unwrap();
    assert_ne!(first, second);
    assert_eq!(first.resource_version(), &version);
    assert_eq!(first.revision(), reference.record().revision);
    assert!(reference.key(4).is_none());

    let rows = reference.rows_for_name_query("hEaD/头部", &HashSet::new());
    assert_eq!(
        rows.iter()
            .map(|row| row.key.joint_index())
            .collect::<Vec<_>>(),
        [2, 0, 1]
    );
    assert_eq!(
        rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(!rows[0].matches_query);
    assert!(rows[0].has_children && rows[0].expanded);
    assert!(rows[1].matches_query);
    assert_eq!(rows[1].parent, reference.key(2));
    assert_eq!(rows[2].parent, Some(first));
    assert!(!rows[2].has_children && !rows[2].expanded);
    assert_eq!(rows[2].key, second);
    assert_eq!(rows[2].name, "Head/头部");
}

#[test]
fn filter_reveals_ancestors_without_mutating_expansion_or_hidden_selection() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("filter-state"),
        vec![
            bone("root", None),
            bone("Spine", Some(0)),
            bone("Head", Some(1)),
            bone("Hand", Some(1)),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let expanded = HashSet::from([reference.key(0).unwrap()]);
    let selection = reference.key(3).unwrap();
    let before = expanded.clone();
    let rows = reference.rows_for_name_query("head", &expanded);
    assert_eq!(rows.len(), 3);
    assert!(rows[1].expanded);
    assert_eq!(expanded, before);
    assert_eq!(reference.bone(&selection).unwrap().name, "Hand");
    assert!(reference
        .rows_for_name_query("不存在的骨骼", &expanded)
        .is_empty());
    let restored = reference.rows_for_name_query("", &expanded);
    assert_eq!(
        restored.iter().map(|row| row.name).collect::<Vec<_>>(),
        ["root", "Spine"]
    );
    assert!(!restored[1].expanded);
}

#[test]
fn replacement_payload_rejects_old_keys_without_retargeting_equal_names() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("replacement"),
        vec![bone("root", None), bone("Head", Some(0))],
    );
    let previous = SkeletonReference::read(&resources, &version).unwrap();
    let selected = previous.key(1).unwrap();
    resources
        .store_payload(
            previous.record().id,
            previous.record().revision,
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone("root", None), bone("Head", Some(0))],
            },
        )
        .unwrap();
    let current_version = resources
        .projection_snapshot()
        .active()
        .key(previous.record().id)
        .unwrap();
    let current = SkeletonReference::read(&resources, &current_version).unwrap();
    assert_ne!(current_version, version);
    assert!(current.bone(&selected).is_none());
    assert_ne!(current.key(1), Some(selected.clone()));
    assert_eq!(previous.bone(&selected).unwrap().name, "Head");
    assert!(matches!(
        SkeletonReference::read(&resources, &version),
        Err(SkeletonReferenceError::Snapshot(
            ResourceActiveSnapshotError::Stale
        ))
    ));
}

#[test]
fn remove_and_recreate_same_id_and_revision_does_not_restore_old_selection() {
    let resources = ResourceManager::new();
    let original_record = record("recreated");
    let version = install(
        &resources,
        original_record.clone(),
        vec![bone("root", None)],
    );
    let previous = SkeletonReference::read(&resources, &version).unwrap();
    let selected = previous.key(0).unwrap();
    resources
        .remove_by_locator(&original_record.primary_locator)
        .unwrap()
        .expect("remove actual resource");
    let current_version = install(&resources, original_record, vec![bone("root", None)]);
    let current = SkeletonReference::read(&resources, &current_version).unwrap();
    assert_eq!(current.record().id, previous.record().id);
    assert_eq!(current.record().revision, previous.record().revision);
    assert_ne!(current_version, version);
    assert!(current.bone(&selected).is_none());
}

#[test]
fn equal_bone_slots_in_different_resources_are_distinct() {
    let resources = ResourceManager::new();
    let left = install(&resources, record("left"), vec![bone("root", None)]);
    let right = install(&resources, record("right"), vec![bone("root", None)]);
    let left = SkeletonReference::read(&resources, &left).unwrap();
    let right = SkeletonReference::read(&resources, &right).unwrap();
    let left_key = left.key(0).unwrap();
    assert_ne!(Some(left_key.clone()), right.key(0));
    assert!(right.bone(&left_key).is_none());
}

#[test]
fn failed_reload_retains_installed_reference_without_proving_catalog_ready() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("failed-reload"),
        vec![bone("root", None)],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    resources
        .start_reload(reference.record().id, Vec::new())
        .unwrap();
    resources
        .fail_reload(reference.record().id, Vec::new())
        .unwrap();
    assert_eq!(
        resources
            .registry()
            .get(reference.record().id)
            .unwrap()
            .state,
        ResourceState::Error
    );
    let retained = SkeletonReference::read(&resources, &version).unwrap();
    assert_eq!(reference.key(0), retained.key(0));
    assert_eq!(retained.record().state, ResourceState::Ready);
}

#[test]
fn real_resource_lookup_rejects_wrong_kind_and_wrong_payload_type() {
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
            SkeletonReference::read(&resources, &version),
            Err(SkeletonReferenceError::Snapshot(
                ResourceActiveSnapshotError::KindConflict
            ))
        ));
    }
}

#[test]
fn invalid_imported_parent_chains_are_rejected_before_tree_projection() {
    let resources = ResourceManager::new();
    let cases = [
        (
            "missing-parent",
            vec![bone("root", Some(9))],
            SkeletonReferenceHierarchyError::ParentOutOfBounds {
                bone_index: 0,
                parent_index: 9,
                bone_count: 1,
            },
        ),
        (
            "cycle",
            vec![bone("a", Some(1)), bone("b", Some(0))],
            SkeletonReferenceHierarchyError::ParentCycle { bone_index: 0 },
        ),
        (
            "self-cycle",
            vec![bone("root", Some(0))],
            SkeletonReferenceHierarchyError::ParentCycle { bone_index: 0 },
        ),
    ];
    for (name, bones, expected) in cases {
        let version = install(&resources, record(name), bones);
        assert!(matches!(
            SkeletonReference::read(&resources, &version),
            Err(SkeletonReferenceError::Hierarchy(actual)) if actual == expected
        ));
    }
}

#[test]
fn empty_and_deep_skeletons_use_the_same_iterative_projection() {
    let resources = ResourceManager::new();
    let empty = install(&resources, record("empty"), Vec::new());
    let empty = SkeletonReference::read(&resources, &empty).unwrap();
    assert!(empty.key(0).is_none());
    assert!(empty.rows_for_name_query("", &HashSet::new()).is_empty());

    let bones = (0_u32..4096)
        .map(|index| bone(&format!("bone-{index}"), index.checked_sub(1)))
        .collect();
    let deep = install(&resources, record("deep"), bones);
    let deep = SkeletonReference::read(&resources, &deep).unwrap();
    let rows = deep.rows_for_name_query("bone-4095", &HashSet::new());
    assert_eq!(rows.len(), 4096);
    assert_eq!(rows.last().unwrap().depth, 4095);
    assert_eq!(rows.last().unwrap().key.joint_index(), 4095);
}
