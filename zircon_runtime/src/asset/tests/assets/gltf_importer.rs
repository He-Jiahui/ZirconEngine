use std::fs;

use super::gltf_scene_fixtures::write_two_scene_gltf;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{AssetImportOutcome, AssetUri, ImportedAsset, ImportedAssetEntry};

#[test]
fn importer_emits_gltf_multi_scene_labels() {
    let root = unique_temp_project_root("gltf_multi_scene_labels");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_two_scene_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/two_scenes.gltf").unwrap();
    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    for label in [
        "Scene0",
        "Scene1",
        "Node0",
        "Node1",
        "Mesh0",
        "Mesh0/Primitive0",
        "Material0",
        "DefaultMaterial",
    ] {
        assert!(
            root_entry
                .dependencies
                .contains(&label_uri(&root_uri, label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(&root_uri, label)),
            "outcome should include {label}"
        );
    }

    let scene0 = entry_for_locator(&outcome, &label_uri(&root_uri, "Scene0"));
    assert!(scene0.dependencies.contains(&label_uri(&root_uri, "Node0")));
    assert!(!scene0.dependencies.contains(&label_uri(&root_uri, "Node1")));
    assert_scene_entity(scene0, "FirstSceneNode", &root_uri);

    let scene1 = entry_for_locator(&outcome, &label_uri(&root_uri, "Scene1"));
    assert!(scene1.dependencies.contains(&label_uri(&root_uri, "Node1")));
    assert!(!scene1.dependencies.contains(&label_uri(&root_uri, "Node0")));
    assert_scene_entity(scene1, "SecondSceneNode", &root_uri);

    assert_scene_entity(
        entry_for_locator(&outcome, &label_uri(&root_uri, "Node0")),
        "FirstSceneNode",
        &root_uri,
    );
    assert_scene_entity(
        entry_for_locator(&outcome, &label_uri(&root_uri, "Node1")),
        "SecondSceneNode",
        &root_uri,
    );

    let mesh_entry = entry_for_locator(&outcome, &label_uri(&root_uri, "Mesh0"));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Mesh0/Primitive0")));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Material0")));

    let _ = fs::remove_dir_all(root);
}

fn assert_scene_entity(entry: &ImportedAssetEntry, expected_name: &str, root_uri: &AssetUri) {
    match &entry.asset {
        ImportedAsset::Scene(scene) => {
            assert_eq!(scene.entities.len(), 1);
            let entity = &scene.entities[0];
            assert_eq!(entity.name, expected_name);
            assert_eq!(entity.parent, None);
            let mesh = entity.mesh.as_ref().expect("scene entity mesh");
            assert_eq!(mesh.model.locator, label_uri(root_uri, "Mesh0"));
            assert_eq!(mesh.material.locator, label_uri(root_uri, "Material0"));
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri(root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                mesh.primitives[0].material.locator,
                label_uri(root_uri, "Material0")
            );
        }
        other => panic!("unexpected scene asset: {other:?}"),
    }
}

fn entry_for_locator<'a>(
    outcome: &'a AssetImportOutcome,
    locator: &AssetUri,
) -> &'a ImportedAssetEntry {
    outcome
        .entries
        .iter()
        .find(|entry| entry.locator == *locator)
        .unwrap_or_else(|| panic!("missing gltf subasset {locator}"))
}

fn label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}")).unwrap()
}
