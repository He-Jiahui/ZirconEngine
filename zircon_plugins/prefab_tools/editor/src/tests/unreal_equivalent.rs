//! R04: data-preservation counterparts from Unreal's existing unit tests.
//!
//! Reference root: E:/Git/ZirconEngine/dev/UnrealEngine
//! Engine/Source/Runtime/CoreUObject/Tests/ArchetypeHashTest.cpp:
//! DuplicatePreservesHash::{ContainerOverrides, MultipleInstancedComponents,
//! EmptyOverrideSet}, and HashDiscriminatesSetupsAndOverrides.
//! Only recorded property data is mirrored through ZE's existing read APIs;
//! no archetype hash, UObject inheritance or override application is asserted.

use super::*;
use crate::effective_prefab_overrides;

#[test]
fn unreal_equiv_prefab_container_overrides_survive_lossless_reads() {
    // UE's ContainerOverrides records an array and map before duplication.
    // ZE keeps these values opaque, both in serialized rows and grouped reads.
    let instance = instance(vec![
        PrefabPropertyOverrideAsset {
            entity_path: "Root".to_owned(),
            property_path: "custom.IntMap".to_owned(),
            value: serde_json::json!({"1": 10, "2": 20}),
        },
        PrefabPropertyOverrideAsset {
            entity_path: "Root".to_owned(),
            property_path: "custom.IntArray".to_owned(),
            value: serde_json::json!([1, 2, 3]),
        },
    ]);
    let before = instance.clone();
    let snapshot = PrefabInstanceAuthoringSnapshot::new(&instance, PrefabSourceState::Unchecked);
    let effective = effective_prefab_overrides(&instance);

    assert!(snapshot.diagnostics.is_empty());
    assert_eq!(snapshot.override_rows.len(), 2);
    assert_eq!(effective.len(), 2);
    for (index, expected) in before.overrides.iter().enumerate() {
        let row = snapshot.override_rows[index];
        assert_eq!(row.source_index, index);
        assert!(std::ptr::eq(row.override_value, &instance.overrides[index]));
        assert_eq!(row.override_value, expected);
        assert_eq!(
            effective.iter().find(|value| {
                value.entity_path == expected.entity_path
                    && value.property_path == expected.property_path
            }),
            Some(expected)
        );
    }
    assert_eq!(instance, before);
}

#[test]
fn unreal_equiv_prefab_reads_distinguish_override_values_and_empty_sets() {
    // UE's discrimination guard checks values 1/2 and then an added property;
    // EmptyOverrideSet separately covers a tracked object with no overrides.
    let mut first = instance(vec![override_value("Root", "custom.IntValue", 1)]);
    let second = instance(vec![override_value("Root", "custom.IntValue", 2)]);
    let empty = instance(Vec::new());
    let second_before = second.clone();
    let source = first.prefab.clone();

    let first_values = effective_prefab_overrides(&first);
    let second_values = effective_prefab_overrides(&second);
    assert_eq!(first_values.len(), 1);
    assert_eq!(second_values.len(), 1);
    assert_eq!(first_values[0].value, serde_json::json!(1));
    assert_eq!(second_values[0].value, serde_json::json!(2));
    assert_ne!(first_values, second_values);

    first.overrides.push(PrefabPropertyOverrideAsset {
        entity_path: "Root".to_owned(),
        property_path: "custom.NameValue".to_owned(),
        value: serde_json::json!("x"),
    });
    let first_snapshot = PrefabInstanceAuthoringSnapshot::new(&first, PrefabSourceState::Unchecked);
    let empty_snapshot = PrefabInstanceAuthoringSnapshot::new(&empty, PrefabSourceState::Unchecked);

    assert_eq!(first_snapshot.override_rows.len(), 2);
    assert_eq!(
        first_snapshot.override_rows[0].override_value.value,
        serde_json::json!(1)
    );
    assert_eq!(
        first_snapshot.override_rows[1].override_value.value,
        serde_json::json!("x")
    );
    assert!(first_snapshot.diagnostics.is_empty());
    assert_ne!(effective_prefab_overrides(&first), first_values);
    assert_eq!(effective_prefab_overrides(&second), second_values);
    assert_eq!(second, second_before);
    assert!(empty_snapshot.override_rows.is_empty());
    assert!(empty_snapshot.diagnostics.is_empty());
    assert!(effective_prefab_overrides(&empty).is_empty());
    assert_eq!(first.prefab, source);
    assert_eq!(second.prefab, source);
    assert_eq!(empty.prefab, source);
}

#[test]
fn unreal_equiv_prefab_source_read_preserves_distinct_component_values() {
    // UE's MultipleInstancedComponents keeps values 11/22 on separate components.
    // This counterpart checks existing source rows and definition metadata only.
    let instance = instance(Vec::new());
    let locator = &instance.prefab.locator;
    let registry = source_registry(&instance.prefab, locator, AssetKind::Prefab);
    let resources = ResourceManager::new();
    let mut source = source_asset(locator, "component property data");
    for (entity_id, value) in [(41, 11), (42, 22)] {
        let mut entity = source_entity(entity_id, "ComponentOwner", None);
        entity.components.push(
            SceneComponentAssetRecord::from_typed(
                "test.Component",
                "test.ComponentSchema",
                3,
                "test-provider",
                &serde_json::json!({"InnerInt": value}),
            )
            .unwrap(),
        );
        source.scene.entities.push(entity);
    }
    let before = source.clone();
    resources
        .register_ready(
            ResourceRecord::new(
                AssetId::from_asset_uuid(instance.prefab.uuid),
                AssetKind::Prefab,
                locator.clone(),
            ),
            source,
        )
        .unwrap();
    let read = PrefabSourceReadSnapshot::read(&instance.prefab, &registry, &resources).unwrap();

    for entity in &before.scene.entities {
        let expected = &entity.components[0];
        let PrefabSourceComponentLookup::Unique(actual) = read.component_by_descriptor(
            entity.entity,
            PrefabSourceComponentDescriptor::from(expected),
        ) else {
            panic!("expected the component from its own source entity");
        };
        assert_eq!(actual, expected);
    }
    assert_eq!(*read.source_snapshot().resource().as_ref(), before);
}
