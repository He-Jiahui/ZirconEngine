use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{AssetImportOutcome, AssetUri, ImportedAsset, ImportedAssetEntry};

#[test]
fn importer_emits_obj_multi_mesh_subassets() {
    let root = unique_temp_project_root("obj_multi_mesh_subassets");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("two_objects.obj");
    fs::write(&obj_path, two_object_obj_source()).unwrap();
    let root_uri = AssetUri::parse("res://models/two_objects.obj").unwrap();

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(&obj_path, &root_uri, Default::default())
        .unwrap();
    let root_entry = outcome.root_entry().expect("root obj entry");

    match &root_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh1/Primitive0")
            );
        }
        other => panic!("unexpected root obj asset: {other:?}"),
    }

    for label in ["Mesh0/Primitive0", "Mesh1/Primitive0"] {
        let uri = label_uri(&root_uri, label);
        assert!(
            root_entry.dependencies.contains(&uri),
            "root dependencies should include {label}"
        );
        match &entry_for_locator(&outcome, &uri).asset {
            ImportedAsset::Mesh(mesh) => {
                assert_eq!(mesh.vertex_count().unwrap(), 3);
                assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
                assert!(
                    mesh.virtual_geometry.is_some(),
                    "{label} should retain cooked virtual geometry"
                );
            }
            other => panic!("unexpected {label} asset: {other:?}"),
        }
    }

    let _ = fs::remove_dir_all(root);
}

fn entry_for_locator<'a>(
    outcome: &'a AssetImportOutcome,
    locator: &AssetUri,
) -> &'a ImportedAssetEntry {
    outcome
        .entries
        .iter()
        .find(|entry| entry.locator == *locator)
        .unwrap_or_else(|| panic!("missing obj subasset {locator}"))
}

fn label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}")).unwrap()
}

fn two_object_obj_source() -> &'static str {
    "\
o FirstObject
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
o SecondObject
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.0 1.0 0.0
f 4 5 6
"
}
