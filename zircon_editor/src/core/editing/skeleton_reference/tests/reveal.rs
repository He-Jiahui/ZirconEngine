use std::collections::HashSet;

use zircon_runtime::core::framework::animation::AnimationSkeletonAsset;
use zircon_runtime::core::resource::{ResourceManager, ResourceState};

use super::super::{SkeletonBoneKey, SkeletonBoneReveal};
use super::{bone, install, record, SkeletonReference};

fn included(
    reference: &SkeletonReference,
    index: u32,
    query: &str,
) -> (SkeletonBoneKey, Vec<SkeletonBoneKey>) {
    let key = reference.key(index).unwrap();
    match reference.reveal_for_name_query(&key, query).unwrap() {
        SkeletonBoneReveal::Included { key, ancestors } => (key, ancestors),
        SkeletonBoneReveal::FilteredOut { .. } => panic!("expected included source bone"),
    }
}

#[test]
fn reveal_uses_source_ancestors_with_later_parents_and_duplicate_names() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("reveal-source-slots"),
        vec![
            bone("Head/头部", Some(2)),
            bone("Head/头部", Some(0)),
            bone("Root", None),
            bone("Other root", None),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let (target, ancestors) = included(&reference, 1, "");
    assert_eq!(target, reference.key(1).unwrap());
    assert_eq!(
        ancestors
            .iter()
            .map(|key| key.joint_index())
            .collect::<Vec<_>>(),
        [2, 0]
    );
    assert!(ancestors.iter().all(|key| key != &target));
    let expanded: HashSet<_> = ancestors.into_iter().collect();
    let rows = reference.rows_for_name_query("", &expanded);
    assert_eq!(
        rows.iter()
            .map(|row| row.key.joint_index())
            .collect::<Vec<_>>(),
        [2, 0, 1, 3]
    );
    assert_eq!(rows[2].key, target);
    assert!(included(&reference, 2, "").1.is_empty());
}

#[test]
fn reveal_preserves_filtered_out_selection_and_existing_expansion() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("reveal-filter"),
        vec![
            bone("root", None),
            bone("Spine", Some(0)),
            bone("Head", Some(1)),
            bone("Hand", Some(1)),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let selection = reference.key(3).unwrap();
    let expanded = HashSet::from([reference.key(0).unwrap()]);
    let before = expanded.clone();
    assert_eq!(
        reference.reveal_for_name_query(&selection, "head"),
        Some(SkeletonBoneReveal::FilteredOut {
            key: selection.clone()
        })
    );
    assert_eq!(expanded, before);
    assert_eq!(reference.details(&selection).unwrap().name, "Hand");
    assert_eq!(
        included(&reference, 1, "HEAD").1,
        [reference.key(0).unwrap()]
    );
    assert_eq!(included(&reference, 2, "head").1.len(), 2);
    assert_eq!(reference.rows_for_name_query("head", &expanded).len(), 3);
    let (_, parents) = included(&reference, 3, "");
    let mut applied = expanded.clone();
    applied.extend(parents);
    assert!(reference
        .rows_for_name_query("", &applied)
        .iter()
        .any(|row| row.key == selection));
    assert_eq!(expanded, before);
}

#[test]
fn reveal_and_rows_share_unicode_literal_substring_and_forest_visibility() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("reveal-unicode"),
        vec![
            bone("根骨骼", None),
            bone("İHEAD/头部 [长名称]", Some(0)),
            bone("另一棵树", None),
            bone("手部", Some(2)),
        ],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let empty_expansion = HashSet::new();
    for query in ["head/头部", "İhead", "[长名称]", "手部", "不存在", " "] {
        let rows = reference.rows_for_name_query(query, &empty_expansion);
        for index in 0..4 {
            let key = reference.key(index).unwrap();
            let plan = reference.reveal_for_name_query(&key, query).unwrap();
            assert_eq!(
                matches!(plan, SkeletonBoneReveal::Included { .. }),
                rows.iter().any(|row| row.key == key),
                "visibility must agree for query {query:?} and source slot {index}"
            );
        }
    }
    assert!(reference.key(4).is_none());
}

#[test]
fn reveal_rejects_foreign_replaced_and_recreated_payload_keys() {
    let resources = ResourceManager::new();
    let source = record("reveal-version");
    let version = install(&resources, source.clone(), vec![bone("root", None)]);
    let original = SkeletonReference::read(&resources, &version).unwrap();
    let key = original.key(0).unwrap();
    let foreign_version = install(&resources, record("reveal-other"), vec![bone("root", None)]);
    let foreign = SkeletonReference::read(&resources, &foreign_version).unwrap();
    assert!(foreign.reveal_for_name_query(&key, "").is_none());
    resources
        .store_payload(
            source.id,
            original.record().revision,
            AnimationSkeletonAsset {
                name: None,
                bones: vec![bone("root", None)],
            },
        )
        .unwrap();
    let replaced_version = resources
        .projection_snapshot()
        .active()
        .key(source.id)
        .unwrap();
    let replacement = SkeletonReference::read(&resources, &replaced_version).unwrap();
    assert!(replacement.reveal_for_name_query(&key, "").is_none());
    let replacement_key = replacement.key(0).unwrap();
    resources
        .remove_by_locator(&source.primary_locator)
        .unwrap()
        .unwrap();
    let recreated_version = install(&resources, source, vec![bone("root", None)]);
    let recreated = SkeletonReference::read(&resources, &recreated_version).unwrap();
    assert_eq!(recreated.record().revision, original.record().revision);
    assert!(recreated.reveal_for_name_query(&key, "").is_none());
    assert!(recreated
        .reveal_for_name_query(&replacement_key, "")
        .is_none());
    assert_eq!(included(&original, 0, "").0, key);
}

#[test]
fn retained_reveal_after_failed_reload_is_not_current_preview_readiness() {
    let resources = ResourceManager::new();
    let version = install(
        &resources,
        record("reveal-retained"),
        vec![bone("root", None)],
    );
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let key = reference.key(0).unwrap();
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
    assert_eq!(included(&reference, 0, "").0, key);
    assert_eq!(reference.record().state, ResourceState::Ready);
}

#[test]
fn reveal_walks_twenty_thousand_ancestors_iteratively_without_expanding_target() {
    let resources = ResourceManager::new();
    let bones = (0..20_000)
        .map(|index| bone("joint", (index > 0).then(|| index - 1)))
        .collect();
    let version = install(&resources, record("reveal-deep"), bones);
    let reference = SkeletonReference::read(&resources, &version).unwrap();
    let (target, ancestors) = included(&reference, 19_999, "joint");
    assert_eq!(ancestors.len(), 19_999);
    assert_eq!(ancestors.first().unwrap(), &reference.key(0).unwrap());
    assert_eq!(ancestors.last().unwrap(), &reference.key(19_998).unwrap());
    assert!(!ancestors.contains(&target));
    assert!(ancestors
        .windows(2)
        .all(|keys| keys[0].joint_index() + 1 == keys[1].joint_index()));
}
