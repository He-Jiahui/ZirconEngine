use std::fs;

use super::gltf_external_fixtures::{write_external_texture_gltf, write_missing_buffer_gltf};
use super::gltf_primitive_fixtures::{
    write_line_gltf, write_skinned_triangle_gltf, write_triangle_gltf, write_two_primitive_gltf,
};
use super::gltf_scene_fixtures::write_two_scene_gltf;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::importer_with_first_wave_plugin_fixtures;
use crate::asset::{
    AssetImportOutcome, AssetUri, DataAssetFormat, ImportedAsset, ImportedAssetEntry,
    MeshAttributeValues, ModelPrimitiveAsset, MESH_ATTRIBUTE_POSITION,
};

#[test]
fn importer_decodes_triangle_gltf_into_model_asset() {
    let root = unique_temp_project_root("triangle_gltf_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/triangle.gltf").unwrap();

    let gltf = importer.import_from_source(&gltf_path, &root_uri).unwrap();

    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_cooked_virtual_geometry(&model.primitives[0], "res://models/triangle.gltf");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_bevy_style_gltf_labeled_subassets() {
    let root = unique_temp_project_root("gltf_labeled_subassets");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/triangle.gltf").unwrap();
    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    match &root_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }
    for label in [
        "Scene0",
        "Node0",
        "Mesh0",
        "Mesh0/Primitive0",
        "Material0",
        "Texture0",
        "DefaultMaterial",
        "Animation0",
        "Skin0",
        "Skin0/InverseBindMatrices",
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

    match &entry_for_label(&outcome, &root_uri, "Texture0").asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("TriangleMaterial"));
            assert_eq!(material.base_color, [0.2, 0.3, 0.4, 1.0]);
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                label_uri(&root_uri, "Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(
                mesh.skin
                    .as_ref()
                    .expect("skinned gltf mesh primitive should keep inverse bind matrices")
                    .inverse_bind_matrices,
                vec![identity_bind_matrix()]
            );
            assert_eq!(mesh.morph_targets.len(), 1);
            assert_eq!(
                mesh.morph_targets[0]
                    .attributes
                    .get(MESH_ATTRIBUTE_POSITION),
                Some(&MeshAttributeValues::Float32x3(vec![
                    [0.1, 0.0, 0.0],
                    [0.0, 0.1, 0.0],
                    [0.0, 0.0, 0.1],
                ]))
            );
        }
        other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Mesh0").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
        }
        other => panic!("unexpected Mesh0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Node0").asset {
        ImportedAsset::Scene(scene) => {
            let entity = scene.entities.first().expect("node entity");
            assert_eq!(entity.name, "TriangleNode");
            let mesh = entity.mesh.as_ref().expect("node mesh");
            assert_eq!(mesh.model.locator, label_uri(&root_uri, "Mesh0"));
            assert_eq!(mesh.material.locator, label_uri(&root_uri, "Material0"));
            assert_eq!(mesh.morph_weights, vec![0.5]);
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                mesh.primitives[0].material.locator,
                label_uri(&root_uri, "Material0")
            );
        }
        other => panic!("unexpected Node0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Animation0").asset {
        ImportedAsset::Data(data) => assert!(
            data.text.contains("not implemented yet"),
            "Animation0 should remain a diagnostic placeholder"
        ),
        other => panic!("unexpected Animation0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Skin0").asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.canonical_json["kind"], "gltf_skin");
            assert_eq!(data.canonical_json["skin_index"], 0);
            assert_eq!(data.canonical_json["joint_count"], 1);
            assert_eq!(
                data.canonical_json["joints"][0]["node"],
                label_uri(&root_uri, "Node0").to_string()
            );
            assert_eq!(
                data.canonical_json["inverse_bind_matrices"],
                label_uri(&root_uri, "Skin0/InverseBindMatrices").to_string()
            );
            assert_eq!(data.canonical_json["inverse_bind_matrix_count"], 1);
        }
        other => panic!("unexpected Skin0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, &root_uri, "Skin0/InverseBindMatrices").asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.canonical_json["kind"], "gltf_inverse_bind_matrices");
            assert_eq!(data.canonical_json["matrix_count"], 1);
            assert_eq!(
                data.canonical_json["matrices"][0],
                serde_json::json!(identity_bind_matrix())
            );
        }
        other => panic!("unexpected Skin0/InverseBindMatrices asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

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

#[test]
fn importer_decodes_gltf_external_texture_image() {
    let root = unique_temp_project_root("gltf_external_texture");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_external_texture_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/external_texture.gltf").unwrap();

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
    assert!(root_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Texture0")));
    assert!(root_entry
        .dependencies
        .contains(&label_uri(&root_uri, "Material0")));

    match &entry_for_locator(&outcome, &label_uri(&root_uri, "Texture0")).asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    match &entry_for_locator(&outcome, &label_uri(&root_uri, "Material0")).asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("ExternalTextureMaterial"));
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                label_uri(&root_uri, "Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_reports_missing_gltf_external_buffer() {
    let root = unique_temp_project_root("gltf_missing_external_buffer");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_missing_buffer_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();

    let error = importer
        .import_with_settings(
            &gltf_path,
            &AssetUri::parse("res://models/missing_buffer.gltf").unwrap(),
            Default::default(),
        )
        .expect_err("missing external gltf buffer should fail import");

    let message = error.to_string();
    assert!(
        message.contains("parse gltf"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("missing.bin"),
        "missing buffer path should be named in the diagnostic: {message}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_rejects_unsupported_gltf_primitive_mode() {
    let root = unique_temp_project_root("gltf_unsupported_primitive_mode");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_line_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();

    let error = importer
        .import_with_settings(
            &gltf_path,
            &AssetUri::parse("res://models/line.gltf").unwrap(),
            Default::default(),
        )
        .expect_err("non-triangle gltf primitive should be rejected");

    let message = error.to_string();
    assert!(
        message.contains("unsupported gltf primitive mode"),
        "unexpected error: {message}"
    );
    assert!(
        message.contains("Lines"),
        "unsupported mode should be named in the diagnostic: {message}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_skinning_channels_on_model_vertices() {
    let root = unique_temp_project_root("skinned_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_skinned_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/skinned_triangle.gltf").unwrap();

    let gltf = importer.import_from_source(&gltf_path, &root_uri).unwrap();

    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(model.primitives[0].vertices[0].joint_indices, [0, 1, 0, 0]);
            assert_eq!(model.primitives[0].vertices[1].joint_indices, [1, 0, 0, 0]);
            assert_eq!(
                model.primitives[0].vertices[0].joint_weights,
                [0.75, 0.25, 0.0, 0.0]
            );
            assert_eq!(
                model.primitives[0].vertices[1].joint_weights,
                [1.0, 0.0, 0.0, 0.0]
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

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

fn entry_for_label<'a>(
    outcome: &'a AssetImportOutcome,
    root_uri: &AssetUri,
    label: &str,
) -> &'a ImportedAssetEntry {
    let locator = label_uri(root_uri, label);
    entry_for_locator(outcome, &locator)
}

fn assert_cooked_virtual_geometry(primitive: &ModelPrimitiveAsset, source_hint: &str) {
    let virtual_geometry = primitive
        .virtual_geometry
        .as_ref()
        .expect("imported model primitive should carry cooked virtual geometry");
    assert!(!virtual_geometry.hierarchy_buffer.is_empty());
    assert!(!virtual_geometry.cluster_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_headers.is_empty());
    assert!(!virtual_geometry.cluster_page_data.is_empty());
    assert!(!virtual_geometry.root_page_table.is_empty());
    assert_eq!(
        virtual_geometry.debug.source_hint.as_deref(),
        Some(source_hint)
    );
}

fn identity_bind_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
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
