//! R04：预制体实例的覆盖记录经过 `.zasset` 二进制缓存写出再读回后必须保持不变。
//!
//! Unreal reference (E:/Git/ZirconEngine/dev/UnrealEngine):
//! Engine/Source/Runtime/CoreUObject/Tests/ArchetypeHashTest.cpp,
//! DuplicatePreservesHash::{ContainerOverrides, EmptyOverrideSet} and
//! HashDiscriminatesSetupsAndOverrides. Their DuplicateObject step is a binary
//! write/read (StaticDuplicateObjectEx, FDuplicateDataWriter/FDuplicateDataReader).
//! Only recorded data is checked; no override is resolved or applied.

use super::*;
use crate::asset::{PrefabAsset, PrefabInstanceAsset, PrefabPropertyOverrideAsset};

fn instance_entity(entity: u64, overrides: Vec<PrefabPropertyOverrideAsset>) -> SceneEntityAsset {
    let instance = PrefabInstanceAsset {
        prefab: asset_reference("res://prefabs/hero.prefab.toml"),
        local_transform: TransformAsset {
            translation: [entity as f32, 2.0, 1.0],
            ..Default::default()
        },
        overrides,
    };
    serde_json::from_value(serde_json::json!({
        "entity": entity,
        "name": format!("Instance{entity}"),
        "parent": null,
        "transform": TransformAsset::default(),
        "prefab_instance": instance,
    }))
    .unwrap()
}

fn container_overrides(value: i32) -> Vec<PrefabPropertyOverrideAsset> {
    vec![
        PrefabPropertyOverrideAsset {
            entity_path: "Root/Component".to_owned(),
            property_path: "custom.IntArray".to_owned(),
            value: serde_json::json!([value, 2, 3]),
        },
        PrefabPropertyOverrideAsset {
            entity_path: "Root/Component".to_owned(),
            property_path: "custom.IntMap".to_owned(),
            value: serde_json::json!({"1": value, "2": 20, "nested": [true, null, 0.5, "x"]}),
        },
    ]
}

fn roundtrip(label: &str, kind: AssetKind, uri: &str, asset: &ImportedAsset) -> ImportedAsset {
    let root = unique_temp_project_root(label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let metadata = ResourceRecord::new(AssetId::new(), kind, AssetUri::parse(uri).unwrap());
    let store = ArtifactStore::default();
    let artifact_uri = store.write(&paths, &metadata, asset).unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();
    assert_binary_artifact_payload(&paths, &artifact_uri);
    let _ = fs::remove_dir_all(root);
    loaded
}

#[test]
fn unreal_equiv_artifact_store_roundtrips_scene_prefab_instance_overrides() {
    let scene = SceneAsset {
        entities: vec![
            instance_entity(42, container_overrides(1)),
            instance_entity(43, container_overrides(2)),
            instance_entity(44, Vec::new()),
        ],
    };

    let loaded = roundtrip(
        "artifact_store_scene_prefab_overrides",
        AssetKind::Scene,
        "res://scenes/prefab_overrides.scene.toml",
        &ImportedAsset::Scene(scene.clone()),
    );

    let ImportedAsset::Scene(loaded) = loaded else {
        panic!("expected a scene payload");
    };
    assert_eq!(loaded, scene);
    let overrides = |index: usize| {
        &loaded.entities[index]
            .prefab_instance
            .as_ref()
            .unwrap()
            .overrides
    };
    assert_ne!(overrides(0), overrides(1));
    assert_eq!(overrides(0)[0].value, serde_json::json!([1, 2, 3]));
    assert_eq!(overrides(1)[0].value, serde_json::json!([2, 2, 3]));
    assert!(overrides(2).is_empty());
}

#[test]
fn unreal_equiv_artifact_store_roundtrips_prefab_source_with_nested_instance_overrides() {
    let uri = "res://prefabs/squad.prefab.toml";
    let prefab = PrefabAsset {
        uri: AssetUri::parse(uri).unwrap(),
        name: "Squad".to_owned(),
        scene: SceneAsset {
            entities: vec![
                instance_entity(1, container_overrides(11)),
                instance_entity(2, container_overrides(22)),
            ],
        },
        exposed_properties: vec!["custom.IntArray".to_owned()],
    };

    let loaded = roundtrip(
        "artifact_store_prefab_nested_overrides",
        AssetKind::Prefab,
        uri,
        &ImportedAsset::Prefab(prefab.clone()),
    );

    assert_eq!(loaded, ImportedAsset::Prefab(prefab));
}
