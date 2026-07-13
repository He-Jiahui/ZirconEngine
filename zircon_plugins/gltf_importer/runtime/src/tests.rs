use super::*;
use crate::test_fixtures::{
    write_external_texture_gltf, write_missing_buffer_gltf, write_two_scene_gltf,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::{AssetUri, DataAssetFormat, ImportedAssetEntry, MaterialAsset};

#[test]
fn package_declares_gltf_importer() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert_eq!(
        manifest.maturity,
        zircon_runtime::plugin::PluginMaturity::Stable
    );
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"glb".to_string())));
}

#[test]
fn package_manifest_declares_gltf_importer_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    let distribution = manifest.distribution.as_ref().expect("dist metadata");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, GLTF_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, GLTF_IMPORTER_DIST_RUNTIME_ENTRY);
    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "gltf_importer.dist")
        .expect("dist native module");
    assert_eq!(dist_module.crate_name, GLTF_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.id == "gltf_importer.gltf"));
}

#[test]
fn registration_contributes_module_and_importer() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert!(report
        .extensions
        .asset_importers()
        .descriptors()
        .iter()
        .any(|importer| importer.id == "gltf_importer.gltf"));
}

#[test]
fn importer_decodes_triangle_gltf_into_model_asset() {
    let root = unique_temp_root("gltf_importer_decode");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        AssetUri::parse("res://models/triangle.gltf").unwrap(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    let entry = outcome.root_entry().expect("root gltf asset entry");
    match &entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive0")
            );
            let virtual_geometry = model.primitives[0]
                .virtual_geometry
                .as_ref()
                .expect("plugin importer should cook virtual geometry");
            assert_eq!(
                virtual_geometry.debug.source_hint.as_deref(),
                Some("res://models/triangle.gltf")
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert!(outcome
        .entries
        .iter()
        .any(|entry| matches!(&entry.asset, ImportedAsset::Mesh(_))));
    assert!(entry.dependencies.contains(&label_uri("Scene0")));
    assert!(entry.dependencies.contains(&label_uri("Node0")));
    assert!(entry.dependencies.contains(&label_uri("Mesh0")));
    assert!(entry.dependencies.contains(&label_uri("Mesh0/Primitive0")));
    assert!(entry.dependencies.contains(&label_uri("Material0")));
    assert!(entry.dependencies.contains(&label_uri("Texture0")));
    assert!(entry.dependencies.contains(&label_uri("DefaultMaterial")));
    assert!(entry.dependencies.contains(&label_uri("Animation0")));
    assert!(entry.dependencies.contains(&label_uri("Skin0")));
    assert!(entry
        .dependencies
        .contains(&label_uri("Skin0/InverseBindMatrices")));

    match &entry_for_label(&outcome, "Texture0").asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    let material_entry = entry_for_label(&outcome, "Material0");
    assert!(material_entry.dependencies.contains(&label_uri("Texture0")));
    assert!(material_entry
        .dependencies
        .contains(&default_pbr_shader_uri()));
    match &material_entry.asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("TriangleMaterial"));
            assert_eq!(material.base_color, [0.2, 0.3, 0.4, 1.0]);
            assert_eq!(material.metallic, 0.5);
            assert_eq!(material.roughness, 0.6);
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                label_uri("Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, "Mesh0").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive0")
            );
        }
        other => panic!("unexpected Mesh0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, "Mesh0/Primitive0").asset {
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
    match &entry_for_label(&outcome, "Node0").asset {
        ImportedAsset::Scene(scene) => {
            assert_eq!(scene.entities.len(), 1);
            let entity = &scene.entities[0];
            assert_eq!(entity.name, "TriangleNode");
            let mesh = entity.mesh.as_ref().expect("node mesh instance");
            assert_eq!(mesh.model.locator, label_uri("Mesh0"));
            assert_eq!(mesh.material.locator, label_uri("Material0"));
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri("Mesh0/Primitive0")
            );
            assert_eq!(mesh.primitives[0].material.locator, label_uri("Material0"));
        }
        other => panic!("unexpected Node0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, "Scene0").asset {
        ImportedAsset::Scene(scene) => assert_eq!(scene.entities.len(), 1),
        other => panic!("unexpected Scene0 asset: {other:?}"),
    }
    let default_material_entry = entry_for_label(&outcome, "DefaultMaterial");
    assert!(default_material_entry
        .dependencies
        .contains(&default_pbr_shader_uri()));
    assert!(matches!(
        &default_material_entry.asset,
        ImportedAsset::Material(_)
    ));
    match &entry_for_label(&outcome, "Animation0").asset {
        ImportedAsset::Data(data) => assert!(
            data.text.contains("not implemented yet"),
            "Animation0 should remain a diagnostic placeholder"
        ),
        other => panic!("unexpected Animation0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, "Skin0").asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.canonical_json["kind"], "gltf_skin");
            assert_eq!(data.canonical_json["skin_index"], 0);
            assert_eq!(data.canonical_json["joint_count"], 1);
            assert_eq!(
                data.canonical_json["joints"][0]["node"],
                label_uri("Node0").to_string()
            );
            assert_eq!(
                data.canonical_json["inverse_bind_matrices"],
                label_uri("Skin0/InverseBindMatrices").to_string()
            );
            assert_eq!(data.canonical_json["inverse_bind_matrix_count"], 1);
        }
        other => panic!("unexpected Skin0 asset: {other:?}"),
    }
    match &entry_for_label(&outcome, "Skin0/InverseBindMatrices").asset {
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
fn importer_preserves_gltf_material_texcoord_as_texture_slot_uv_channel() {
    let root = unique_temp_root("gltf_importer_material_uv_channel");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_material_uv_channel_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let root_uri = AssetUri::parse("res://models/material_uv_channel.gltf").unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        root_uri.clone(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    match &entry_for_locator(&outcome, &label_uri_for(&root_uri, "Material0")).asset {
        ImportedAsset::Material(material) => {
            let base_color = material
                .texture_slots
                .get("base_color")
                .expect("base_color texture slot should be imported");
            assert_eq!(
                base_color.reference.as_ref().unwrap().locator,
                label_uri_for(&root_uri, "Texture0")
            );
            assert_eq!(base_color.texture_uv_channel(), 1);
            assert_eq!(
                material
                    .standard_material_descriptor()
                    .base_color_texture_uv_channel,
                1
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_texture_transform_on_standard_material_slot() {
    let root = unique_temp_root("gltf_importer_texture_transform");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_material_texture_transform_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let root_uri = AssetUri::parse("res://models/material_texture_transform.gltf").unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        root_uri.clone(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    match &entry_for_locator(&outcome, &label_uri_for(&root_uri, "Material0")).asset {
        ImportedAsset::Material(material) => {
            assert_texture_slot_transform(
                material,
                &root_uri,
                "base_color",
                [0.3, 0.4],
                [0.1, 0.2],
                1,
            );
            assert_texture_slot_transform(material, &root_uri, "normal", [0.5, 0.6], [0.3, 0.4], 0);
            assert_texture_slot_transform(
                material,
                &root_uri,
                "metallic_roughness",
                [0.4, 0.5],
                [0.2, 0.3],
                0,
            );
            assert_texture_slot_transform(
                material,
                &root_uri,
                "occlusion",
                [0.6, 0.7],
                [0.4, 0.5],
                1,
            );
            assert_texture_slot_transform(
                material,
                &root_uri,
                "emissive",
                [0.7, 0.8],
                [0.5, 0.6],
                1,
            );

            let descriptor = material.standard_material_descriptor();
            assert_vec2_near(descriptor.base_color_texture_transform.scale, [0.3, 0.4]);
            assert_vec2_near(descriptor.base_color_texture_transform.offset, [0.1, 0.2]);
            assert_eq!(descriptor.base_color_texture_uv_channel, 1);
            assert_vec2_near(descriptor.normal_texture_transform.scale, [0.5, 0.6]);
            assert_vec2_near(descriptor.normal_texture_transform.offset, [0.3, 0.4]);
            assert_eq!(descriptor.normal_texture_uv_channel, 0);
            assert_vec2_near(
                descriptor.metallic_roughness_texture_transform.scale,
                [0.4, 0.5],
            );
            assert_vec2_near(
                descriptor.metallic_roughness_texture_transform.offset,
                [0.2, 0.3],
            );
            assert_eq!(descriptor.metallic_roughness_texture_uv_channel, 0);
            assert_vec2_near(descriptor.occlusion_texture_transform.scale, [0.6, 0.7]);
            assert_vec2_near(descriptor.occlusion_texture_transform.offset, [0.4, 0.5]);
            assert_eq!(descriptor.occlusion_texture_uv_channel, 1);
            assert_vec2_near(descriptor.emissive_texture_transform.scale, [0.7, 0.8]);
            assert_vec2_near(descriptor.emissive_texture_transform.offset, [0.5, 0.6]);
            assert_eq!(descriptor.emissive_texture_uv_channel, 1);
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_multi_primitive_material_labels() {
    let root = unique_temp_root("gltf_importer_multi_primitive");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_two_primitive_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let root_uri = AssetUri::parse("res://models/two_primitives.gltf").unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        root_uri.clone(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    let label_uri = |label: &str| label_uri_for(&root_uri, label);
    let root_entry = outcome.root_entry().expect("root gltf asset entry");
    match &root_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive1")
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
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
            root_entry.dependencies.contains(&label_uri(label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(label)),
            "outcome should include {label}"
        );
    }

    let mesh_entry = entry_for_locator(&outcome, &label_uri("Mesh0"));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri("Mesh0/Primitive0")));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri("Mesh0/Primitive1")));
    assert!(mesh_entry.dependencies.contains(&label_uri("Material0")));
    assert!(mesh_entry.dependencies.contains(&label_uri("Material1")));
    match &mesh_entry.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 2);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive0")
            );
            assert_eq!(
                model.primitives[1].mesh.as_ref().unwrap().locator,
                label_uri("Mesh0/Primitive1")
            );
        }
        other => panic!("unexpected Mesh0 asset: {other:?}"),
    }

    for (primitive_label, material_label) in [
        ("Mesh0/Primitive0", "Material0"),
        ("Mesh0/Primitive1", "Material1"),
    ] {
        let primitive_entry = entry_for_locator(&outcome, &label_uri(primitive_label));
        assert!(
            primitive_entry
                .dependencies
                .contains(&label_uri(material_label)),
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
        match &entry_for_locator(&outcome, &label_uri(material_label)).asset {
            ImportedAsset::Material(material) => {
                assert_eq!(material.name.as_deref(), Some(material_name));
            }
            other => panic!("unexpected {material_label} asset: {other:?}"),
        }
    }

    match &entry_for_locator(&outcome, &label_uri("Scene0")).asset {
        ImportedAsset::Scene(scene) => {
            let entity = scene.entities.first().expect("scene entity");
            let mesh = entity.mesh.as_ref().expect("scene entity mesh");
            assert_eq!(mesh.primitives.len(), 2);
            for (primitive, primitive_label, material_label) in [
                (&mesh.primitives[0], "Mesh0/Primitive0", "Material0"),
                (&mesh.primitives[1], "Mesh0/Primitive1", "Material1"),
            ] {
                assert_eq!(primitive.mesh.locator, label_uri(primitive_label));
                assert_eq!(primitive.material.locator, label_uri(material_label));
            }
        }
        other => panic!("unexpected Scene0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_external_texture_image() {
    let root = unique_temp_root("gltf_importer_external_texture");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_external_texture_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let root_uri = AssetUri::parse("res://models/external_texture.gltf").unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        root_uri.clone(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    let label_uri = |label: &str| label_uri_for(&root_uri, label);
    let root_entry = outcome.root_entry().expect("root gltf asset entry");
    assert!(root_entry.dependencies.contains(&label_uri("Texture0")));
    assert!(root_entry.dependencies.contains(&label_uri("Material0")));

    match &entry_for_locator(&outcome, &label_uri("Texture0")).asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    match &entry_for_locator(&outcome, &label_uri("Material0")).asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("ExternalTextureMaterial"));
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                label_uri("Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_reports_missing_external_buffer() {
    let root = unique_temp_root("gltf_importer_missing_buffer");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_missing_buffer_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();

    let error = import_gltf(&AssetImportContext::new(
        gltf_path,
        AssetUri::parse("res://models/missing_buffer.gltf").unwrap(),
        source_bytes,
        toml::Table::new(),
    ))
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
fn importer_emits_multi_scene_labels() {
    let root = unique_temp_root("gltf_importer_multi_scene");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_two_scene_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();
    let root_uri = AssetUri::parse("res://models/two_scenes.gltf").unwrap();
    let outcome = import_gltf(&AssetImportContext::new(
        gltf_path,
        root_uri.clone(),
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap();

    let label_uri = |label: &str| label_uri_for(&root_uri, label);
    let root_entry = outcome.root_entry().expect("root gltf asset entry");
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
            root_entry.dependencies.contains(&label_uri(label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == label_uri(label)),
            "outcome should include {label}"
        );
    }

    let scene0 = entry_for_locator(&outcome, &label_uri("Scene0"));
    assert!(scene0.dependencies.contains(&label_uri("Node0")));
    assert!(!scene0.dependencies.contains(&label_uri("Node1")));
    assert_scene_entity(scene0, "FirstSceneNode", &root_uri);

    let scene1 = entry_for_locator(&outcome, &label_uri("Scene1"));
    assert!(scene1.dependencies.contains(&label_uri("Node1")));
    assert!(!scene1.dependencies.contains(&label_uri("Node0")));
    assert_scene_entity(scene1, "SecondSceneNode", &root_uri);

    assert_scene_entity(
        entry_for_locator(&outcome, &label_uri("Node0")),
        "FirstSceneNode",
        &root_uri,
    );
    assert_scene_entity(
        entry_for_locator(&outcome, &label_uri("Node1")),
        "SecondSceneNode",
        &root_uri,
    );

    let mesh_entry = entry_for_locator(&outcome, &label_uri("Mesh0"));
    assert!(mesh_entry
        .dependencies
        .contains(&label_uri("Mesh0/Primitive0")));
    assert!(mesh_entry.dependencies.contains(&label_uri("Material0")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_rejects_unsupported_gltf_primitive_mode() {
    let root = unique_temp_root("gltf_importer_unsupported_mode");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_line_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();

    let error = import_gltf(&AssetImportContext::new(
        gltf_path,
        AssetUri::parse("res://models/line.gltf").unwrap(),
        source_bytes,
        toml::Table::new(),
    ))
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

fn entry_for_label<'a>(outcome: &'a AssetImportOutcome, label: &str) -> &'a ImportedAssetEntry {
    let locator = label_uri(label);
    entry_for_locator(outcome, &locator)
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

fn assert_scene_entity(entry: &ImportedAssetEntry, expected_name: &str, root_uri: &AssetUri) {
    match &entry.asset {
        ImportedAsset::Scene(scene) => {
            assert_eq!(scene.entities.len(), 1);
            let entity = &scene.entities[0];
            assert_eq!(entity.name, expected_name);
            assert_eq!(entity.parent, None);
            let mesh = entity.mesh.as_ref().expect("scene entity mesh");
            assert_eq!(mesh.model.locator, label_uri_for(root_uri, "Mesh0"));
            assert_eq!(mesh.material.locator, label_uri_for(root_uri, "Material0"));
            assert_eq!(mesh.primitives.len(), 1);
            assert_eq!(
                mesh.primitives[0].mesh.locator,
                label_uri_for(root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(
                mesh.primitives[0].material.locator,
                label_uri_for(root_uri, "Material0")
            );
        }
        other => panic!("unexpected scene asset: {other:?}"),
    }
}

fn label_uri(label: &str) -> AssetUri {
    AssetUri::parse(&format!("res://models/triangle.gltf#{label}")).unwrap()
}

fn label_uri_for(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}")).unwrap()
}

fn default_pbr_shader_uri() -> AssetUri {
    AssetUri::parse("res://shaders/default_pbr.zshader").unwrap()
}

fn identity_bind_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn assert_texture_slot_transform(
    material: &MaterialAsset,
    root_uri: &AssetUri,
    slot: &str,
    expected_scale: [f32; 2],
    expected_offset: [f32; 2],
    expected_uv_channel: u32,
) {
    let value = material
        .texture_slots
        .get(slot)
        .unwrap_or_else(|| panic!("{slot} texture slot should be imported"));
    assert_eq!(
        value.reference.as_ref().unwrap().locator,
        label_uri_for(root_uri, "Texture0")
    );
    let transform = value.texture_transform();
    assert_vec2_near(transform.scale, expected_scale);
    assert_vec2_near(transform.offset, expected_offset);
    assert_eq!(value.texture_uv_channel(), expected_uv_channel);
}

fn assert_vec2_near(actual: [f32; 2], expected: [f32; 2]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 0.000_001,
            "expected {expected:?}, got {actual:?}"
        );
    }
}

fn unique_temp_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_plugin_{label}_{unique}"))
}

fn write_triangle_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("triangle.bin");
    let gltf_path = root.join("triangle.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    for value in [
        1.0_f32, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0.0_f32.to_le_bytes());
    for value in [0.0_f32, 0.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [0.1_f32, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
            &gltf_path,
            r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "triangle.bin", "byteLength": 160 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 64 },
    { "buffer": 0, "byteOffset": 108, "byteLength": 4 },
    { "buffer": 0, "byteOffset": 112, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 124, "byteLength": 36, "target": 34962 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 1,
      "type": "MAT4"
    },
    {
      "bufferView": 3,
      "componentType": 5126,
      "count": 1,
      "type": "SCALAR",
      "min": [0.0],
      "max": [0.0]
    },
    {
      "bufferView": 4,
      "componentType": 5126,
      "count": 1,
      "type": "VEC3"
    },
    {
      "bufferView": 5,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "TriangleMaterial",
      "pbrMetallicRoughness": {
        "baseColorFactor": [0.2, 0.3, 0.4, 1.0],
        "baseColorTexture": { "index": 0 },
        "metallicFactor": 0.5,
        "roughnessFactor": 0.6
      }
    }
  ],
  "meshes": [
    {
      "name": "TriangleMesh",
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0,
          "targets": [{ "POSITION": 5 }]
        }
      ]
    }
  ],
  "nodes": [{ "name": "TriangleNode", "mesh": 0, "skin": 0 }],
  "skins": [{ "inverseBindMatrices": 2, "joints": [0] }],
  "animations": [
    {
      "samplers": [{ "input": 3, "output": 4, "interpolation": "LINEAR" }],
      "channels": [{ "sampler": 0, "target": { "node": 0, "path": "translation" } }]
    }
  ],
  "scenes": [{ "name": "SceneRoot", "nodes": [0] }],
  "scene": 0
}"#,
        )
        .unwrap();

    gltf_path
}

fn write_line_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("line.bin");
    let gltf_path = root.join("line.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "line.bin", "byteLength": 24 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 24, "target": 34962 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 0.0, 0.0]
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "mode": 1,
          "attributes": { "POSITION": 0 }
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}
"#,
    )
    .unwrap();

    gltf_path
}

fn write_material_uv_channel_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("material_uv_channel.bin");
    let gltf_path = root.join("material_uv_channel.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        0.0_f32, 0.0, //
        0.5, 0.0, //
        0.0, 0.5,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        1.0_f32, 0.25, //
        0.25, 1.0, //
        0.75, 0.75,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "material_uv_channel.bin", "byteLength": 90 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "MaterialUvChannel",
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0, "texCoord": 1 }
      }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "TEXCOORD_0": 1,
            "TEXCOORD_1": 2
          },
          "indices": 3,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#,
    )
    .unwrap();

    gltf_path
}

fn write_material_texture_transform_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("material_texture_transform.bin");
    let gltf_path = root.join("material_texture_transform.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        0.0_f32, 0.0, //
        0.5, 0.0, //
        0.0, 0.5,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        1.0_f32, 0.25, //
        0.25, 1.0, //
        0.75, 0.75,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_texture_transform"],
  "buffers": [
    { "uri": "material_texture_transform.bin", "byteLength": 90 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "TextureTransformMaterial",
      "pbrMetallicRoughness": {
        "baseColorTexture": {
          "index": 0,
          "texCoord": 0,
          "extensions": {
            "KHR_texture_transform": {
              "offset": [0.1, 0.2],
              "scale": [0.3, 0.4],
              "texCoord": 1
            }
          }
        },
        "metallicRoughnessTexture": {
          "index": 0,
          "extensions": {
            "KHR_texture_transform": {
              "offset": [0.2, 0.3],
              "scale": [0.4, 0.5]
            }
          }
        }
      },
      "normalTexture": {
        "index": 0,
        "texCoord": 1,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.3, 0.4],
            "scale": [0.5, 0.6],
            "texCoord": 0
          }
        }
      },
      "occlusionTexture": {
        "index": 0,
        "texCoord": 0,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.4, 0.5],
            "scale": [0.6, 0.7],
            "texCoord": 1
          }
        }
      },
      "emissiveTexture": {
        "index": 0,
        "texCoord": 1,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.5, 0.6],
            "scale": [0.7, 0.8]
          }
        }
      }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "TEXCOORD_0": 1,
            "TEXCOORD_1": 2
          },
          "indices": 3,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#,
    )
    .unwrap();

    gltf_path
}

fn write_two_primitive_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("two_primitives.bin");
    let gltf_path = root.join("two_primitives.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    for value in [
        1.0_f32, 1.0, 0.0, //
        2.0, 1.0, 0.0, //
        1.0, 2.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "two_primitives.bin", "byteLength": 86 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 80, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [1.0, 1.0, 0.0],
      "max": [2.0, 2.0, 0.0]
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "materials": [
    {
      "name": "FirstMaterial",
      "pbrMetallicRoughness": { "baseColorFactor": [1.0, 0.0, 0.0, 1.0] }
    },
    {
      "name": "SecondMaterial",
      "pbrMetallicRoughness": { "baseColorFactor": [0.0, 0.0, 1.0, 1.0] }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
        },
        {
          "attributes": { "POSITION": 2 },
          "indices": 3,
          "material": 1
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}
"#,
    )
    .unwrap();

    gltf_path
}
