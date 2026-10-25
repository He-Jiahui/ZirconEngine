use std::collections::HashSet;

use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::resource::ResourceManager;

use super::{bone, install, record, SkeletonReference};

#[test]
fn details_resolve_original_parent_slot_and_preserve_reference_local_values() {
    let resources = ResourceManager::new();
    let mut head = bone("Head", Some(2));
    head.local_translation = [1.5, -2.25, 3.75];
    head.local_rotation = [0.0, 0.0, 0.6, 0.8];
    head.local_scale = [-1.0, 0.5, 2.0];
    let version = install(
        &resources,
        record("reference-details"),
        vec![head, bone("Other", None), bone("Spine_4", None)],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let selected = reference.key(0).unwrap();
    let details = reference.details(&selected).unwrap();
    assert_eq!(details.key, selected);
    assert_eq!(details.name, "Head");
    let parent = details.parent.unwrap();
    assert_eq!(Some(parent.key), reference.key(2));
    assert_eq!(parent.name, "Spine_4");
    assert_eq!(*details.reference_local_translation, [1.5, -2.25, 3.75]);
    assert_eq!(*details.reference_local_rotation, [0.0, 0.0, 0.6, 0.8]);
    assert_eq!(*details.reference_local_scale, [-1.0, 0.5, 2.0]);
    let root = reference.details(&reference.key(2).unwrap()).unwrap();
    assert!(root.parent.is_none());
}

#[test]
fn details_reject_foreign_replaced_and_recreated_keys_despite_equal_names() {
    let resources = ResourceManager::new();
    let original_record = record("details-replacement");
    let version = install(
        &resources,
        original_record.clone(),
        vec![bone("Head", None)],
    );
    let original = SkeletonReference::read(&resources, &version).unwrap();
    let old_key = original.key(0).unwrap();
    let foreign_version = install(
        &resources,
        record("details-foreign"),
        vec![bone("Head", None)],
    );
    let foreign = SkeletonReference::read(&resources, &foreign_version).unwrap();
    assert!(foreign.details(&old_key).is_none());

    resources
        .store_payload(
            original_record.id,
            original.record().revision,
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone("Head", None)],
            },
        )
        .unwrap();
    let replacement_version = resources
        .projection_snapshot()
        .active()
        .key(original_record.id)
        .unwrap();
    let replacement = SkeletonReference::read(&resources, &replacement_version).unwrap();
    assert!(replacement.details(&old_key).is_none());
    let replacement_key = replacement.key(0).unwrap();

    resources
        .remove_by_locator(&original_record.primary_locator)
        .unwrap()
        .expect("remove installed skeleton before recreation");
    let recreated_version = install(&resources, original_record, vec![bone("Head", None)]);
    let recreated = SkeletonReference::read(&resources, &recreated_version).unwrap();
    assert_eq!(recreated.record().revision, original.record().revision);
    assert!(recreated.details(&replacement_key).is_none());
    assert!(recreated.details(&old_key).is_none());
    assert_eq!(original.details(&old_key).unwrap().name, "Head");
}

#[test]
fn details_remain_available_when_selection_is_hidden_by_filter_or_collapse() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("hidden-details"),
        vec![
            bone("根骨骼", None),
            bone("Spine_4", Some(0)),
            bone("Head", Some(1)),
            bone("Hand/手部", Some(1)),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let selected = reference.key(3).unwrap();
    let before = reference.details(&selected).unwrap();
    let mut expanded = HashSet::from([reference.key(0).unwrap()]);
    assert!(reference
        .rows_for_name_query("head", &expanded)
        .iter()
        .all(|row| row.key != selected));
    assert!(reference
        .rows_for_name_query("不存在的骨骼", &expanded)
        .is_empty());
    assert!(reference
        .rows_for_name_query("", &expanded)
        .iter()
        .all(|row| row.key != selected));
    assert_eq!(reference.details(&selected).unwrap(), before);

    expanded.insert(reference.key(1).unwrap());
    assert!(reference
        .rows_for_name_query("", &expanded)
        .iter()
        .any(|row| row.key == selected));
    let restored = reference.details(&selected).unwrap();
    assert_eq!(restored, before);
    assert_eq!(restored.name, "Hand/手部");
    assert_eq!(restored.parent.unwrap().name, "Spine_4");
}
