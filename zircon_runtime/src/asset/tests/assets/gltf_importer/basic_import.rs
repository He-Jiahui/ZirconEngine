use super::*;

#[test]
fn importer_decodes_triangle_gltf_into_model_asset() {
    let root = unique_temp_project_root("triangle_gltf_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/triangle.gltf").unwrap();

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    match &outcome.root_entry().expect("root gltf entry").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert!(model.primitives[0].vertices.is_empty());
            assert!(model.primitives[0].indices.is_empty());
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert!(model.primitives[0].virtual_geometry.is_none());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            let primitive = mesh.to_model_primitive().unwrap();
            assert_cooked_virtual_geometry(&primitive, "res://models/triangle.gltf");
        }
        other => panic!("unexpected gltf mesh asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn default_importer_decodes_gltf_without_first_wave_plugin_fixture() {
    let root = unique_temp_project_root("default_triangle_gltf_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = AssetImporter::default();
    let root_uri = AssetUri::parse("res://models/default_triangle.gltf").unwrap();

    let report = importer
        .capability_report_for_source(&gltf_path)
        .expect("default importer should report gltf capability");
    assert_eq!(report.descriptor.id, "zircon.builtin.model.gltf");
    assert_eq!(
        report.descriptor.plugin_id,
        "zircon.builtin.asset_importers"
    );
    assert_eq!(report.status, AssetImporterCapabilityStatus::Available);

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();
    match &outcome.root_entry().expect("root gltf entry").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert!(model.primitives[0].vertices.is_empty());
            assert!(model.primitives[0].indices.is_empty());
            assert!(model.primitives[0].mesh.is_some());
            assert!(model.primitives[0].virtual_geometry.is_none());
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            let primitive = mesh.to_model_primitive().unwrap();
            assert_cooked_virtual_geometry(&primitive, "res://models/default_triangle.gltf");
        }
        other => panic!("unexpected default gltf mesh asset: {other:?}"),
    }
    for label in [
        "Scene0",
        "Mesh0",
        "Mesh0/Primitive0",
        "Material0",
        "Texture0",
    ] {
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(&root_uri, label)),
            "outcome should include {label}"
        );
    }

    let _ = fs::remove_dir_all(root);
}
