use super::*;

#[test]
fn importer_emits_gltf_multi_primitive_material_labels() {
    let root = unique_temp_project_root("gltf_multi_primitive_materials");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_two_primitive_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/two_primitives.gltf").unwrap();
    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    match &root_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive1")
            );
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }

    for label in [
        "Mesh0",
        "Mesh0/Primitive0",
        "Mesh0/Primitive1",
        "Material0",
        "Material1",
        "Node0",
        "Scene0",
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

    let mesh_entry = entry_for_locator(&outcome, &label_uri(&root_uri, "Mesh0"));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Mesh0/Primitive0")));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Mesh0/Primitive1")));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Material0")));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Material1")));
    match &mesh_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive1")
            );
        }
        other => panic!("unexpected Mesh0 asset: {other:?}"),
    }

    for (primitive_label, material_label) in [
        ("Mesh0/Primitive0", "Material0"),
        ("Mesh0/Primitive1", "Material1"),
    ] {
        let primitive_entry = entry_for_locator(&outcome, &label_uri(&root_uri, primitive_label));
        assert!(
            primitive_entry
                .dependencies
                .contains(&label_uri(&root_uri, material_label)),
            "{primitive_label} should depend on {material_label}"
        );
        match &primitive_entry.asset {
            ImportedAsset::Mesh(mesh) => assert_eq!(mesh.vertex_count().unwrap(), 3),
            other => panic!("unexpected {primitive_label} asset: {other:?}"),
        }
    }

    for (material_label, material_name) in [
        ("Material0", "FirstMaterial"),
        ("Material1", "SecondMaterial"),
    ] {
        match &entry_for_locator(&outcome, &label_uri(&root_uri, material_label)).asset {
            ImportedAsset::Material(material) => {
                assert_eq!(material.name.as_deref(), Some(material_name));
            }
            other => panic!("unexpected {material_label} asset: {other:?}"),
        }
    }

    match &entry_for_locator(&outcome, &label_uri(&root_uri, "Scene0")).asset {
        ImportedAsset::Scene(scene) => {
            let entity = scene.entities.first().expect("scene entity");
            let mesh = entity.mesh.as_ref().expect("scene entity mesh");
            assert_eq!(mesh.primitives.len(), 2);
            for (primitive, primitive_label, material_label) in [
                (&mesh.primitives[0], "Mesh0/Primitive0", "Material0"),
                (&mesh.primitives[1], "Mesh0/Primitive1", "Material1"),
            ] {
                assert_eq!(
                    primitive.mesh.locator,
                    label_uri(&root_uri, primitive_label)
                );
                assert_eq!(
                    primitive.material.locator,
                    label_uri(&root_uri, material_label)
                );
            }
        }
        other => panic!("unexpected Scene0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
