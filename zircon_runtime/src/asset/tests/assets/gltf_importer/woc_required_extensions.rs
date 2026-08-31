use super::*;
use std::path::{Path, PathBuf};

use crate::asset::project::ProjectPaths;
use crate::asset::{asset_kind_for_imported_asset, ArtifactStore, AssetId};
use crate::core::resource::ResourceRecord;

fn import_woc_beach_anchor() -> (AssetUri, AssetImportOutcome) {
    import_woc_model(
        "biome/beach_anchor.glb",
        "res://woc/models/beach_anchor.glb",
    )
}

fn import_woc_model(relative_path: &str, uri: &str) -> (AssetUri, AssetImportOutcome) {
    let source_path = woc_model_root().join(relative_path);
    assert!(source_path.is_file(), "missing checked-in WOC GLB fixture");
    let root_uri = AssetUri::parse(uri).unwrap();
    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &root_uri, Default::default())
        .unwrap();
    (root_uri, outcome)
}

fn woc_model_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dev/world-of-claudecraft/public/models")
}

fn checked_in_woc_glbs(root: &Path) -> Vec<PathBuf> {
    let mut directories = vec![root.to_path_buf()];
    let mut glbs = Vec::new();
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("glb"))
            {
                glbs.push(path);
            }
        }
    }
    glbs.sort();
    glbs
}

#[test]
fn woc_meshopt_required_glb_imports() {
    let (root_uri, outcome) = import_woc_beach_anchor();

    match &outcome.root_entry().expect("root WOC model").asset {
        ImportedAsset::Model(model) => {
            assert!(!model.primitives.is_empty());
            assert!(model
                .primitives
                .iter()
                .all(|primitive| primitive.vertices.is_empty()
                    && primitive.indices.is_empty()
                    && primitive.mesh.is_some()));
        }
        other => panic!("unexpected WOC root asset: {other:?}"),
    }

    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            assert!(mesh
                .attributes
                .get(MESH_ATTRIBUTE_POSITION)
                .is_some_and(|positions| !positions.is_empty()));
            assert!(mesh
                .indices
                .as_ref()
                .is_some_and(|indices| !indices.is_empty()));
        }
        other => panic!("unexpected WOC mesh asset: {other:?}"),
    }

    match &entry_for_label(&outcome, &root_uri, "Mesh0").asset {
        ImportedAsset::Model(model) => assert!(model.primitives.iter().all(|primitive| {
            primitive.vertices.is_empty()
                && primitive.indices.is_empty()
                && primitive.mesh.is_some()
        })),
        other => panic!("unexpected WOC mesh group asset: {other:?}"),
    }
}

#[test]
fn woc_required_gltf_extensions_preserve_render_assets() {
    let (root_uri, outcome) = import_woc_beach_anchor();
    let texture_locator = label_uri(&root_uri, "Texture0");

    match &entry_for_locator(&outcome, &texture_locator).asset {
        ImportedAsset::Texture(texture) => {
            assert!(texture.width > 0 && texture.height > 0);
            assert_eq!(
                texture.rgba.len(),
                texture.width as usize * texture.height as usize * 4
            );
        }
        other => panic!("unexpected WOC texture asset: {other:?}"),
    }

    let material_entry = entry_for_label(&outcome, &root_uri, "Material0");
    match &material_entry.asset {
        ImportedAsset::Material(material) => {
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                texture_locator
            );
        }
        other => panic!("unexpected WOC material asset: {other:?}"),
    }
    assert!(material_entry.dependencies.contains(&texture_locator));
}

#[test]
fn woc_required_material_extensions_project_owned_semantics() {
    let (wild_boar_uri, wild_boar) =
        import_woc_model("creatures/wild_boar.glb", "res://woc/models/wild_boar.glb");
    let unlit = match &entry_for_label(&wild_boar, &wild_boar_uri, "Material0").asset {
        ImportedAsset::Material(material) => material.standard_material_descriptor().unlit,
        other => panic!("unexpected WOC unlit material: {other:?}"),
    };
    assert!(unlit);

    let (staff_uri, staff) =
        import_woc_model("weapons/staff_c.glb", "res://woc/models/staff_c.glb");
    match &entry_for_label(&staff, &staff_uri, "Material1").asset {
        ImportedAsset::Material(material) => assert_eq!(material.emissive, [1.5; 3]),
        other => panic!("unexpected WOC emissive material: {other:?}"),
    }

    let (water_uri, water) = import_woc_model(
        "creatures/water_elemental.glb",
        "res://woc/models/water_elemental.glb",
    );
    match &entry_for_label(&water, &water_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            let features = material.advanced_pbr_features();
            assert_eq!(features.ior, 1.333);
            assert_eq!(features.specular_transmission, 0.9);
            assert_eq!(features.thickness, 0.24);
            assert_eq!(features.attenuation_distance, 4.5);
            assert_eq!(features.attenuation_color, [0.14, 0.7, 0.94]);
        }
        other => panic!("unexpected WOC volume material: {other:?}"),
    }
}

#[test]
fn woc_unsupported_specular_extension_has_explicit_diagnostic() {
    let (root_uri, outcome) =
        import_woc_model("creatures/yumi_cat.glb", "res://woc/models/yumi_cat.glb");
    match &entry_for_label(&outcome, &root_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.advanced_pbr_features().ior, 1.4500000476837158);
            assert!(material
                .validation_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("KHR_materials_specular")));
        }
        other => panic!("unexpected WOC specular material: {other:?}"),
    }
}

#[test]
fn optional_unsupported_material_extensions_have_explicit_diagnostics() {
    let root = unique_temp_project_root("gltf_optional_unsupported_material_extensions");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("optional_unsupported_material_extensions.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": [
                "KHR_materials_diffuse_transmission",
                "KHR_materials_dispersion",
                "KHR_materials_iridescence",
                "KHR_materials_pbrSpecularGlossiness",
                "KHR_materials_sheen",
                "KHR_materials_subsurface"
            ],
            "materials": [{ "extensions": {
                "KHR_materials_diffuse_transmission": {},
                "KHR_materials_dispersion": {},
                "KHR_materials_iridescence": {},
                "KHR_materials_pbrSpecularGlossiness": {},
                "KHR_materials_sheen": {},
                "KHR_materials_subsurface": {}
            } }]
        }"#,
    )
    .unwrap();
    let uri =
        AssetUri::parse("res://models/optional_unsupported_material_extensions.gltf").unwrap();

    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .expect("optional unsupported material extensions retain core PBR fallback");

    match &entry_for_label(&outcome, &uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            for extension in [
                "KHR_materials_diffuse_transmission",
                "KHR_materials_dispersion",
                "KHR_materials_iridescence",
                "KHR_materials_pbrSpecularGlossiness",
                "KHR_materials_sheen",
                "KHR_materials_subsurface",
            ] {
                assert!(
                    material
                        .validation_diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.contains(extension)),
                    "missing optional fallback diagnostic for {extension}"
                );
            }
        }
        other => panic!("unexpected optional fallback material: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn gltf_unknown_required_extension_is_not_silently_accepted() {
    let root = unique_temp_project_root("gltf_unknown_required_extension");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("unknown_required.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["VENDOR_future_rendering"],
            "extensionsRequired": ["VENDOR_future_rendering"]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/unknown_required.gltf").unwrap();

    let error = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("requires unsupported extension `VENDOR_future_rendering`"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn supported_material_extension_may_be_required() {
    let root = unique_temp_project_root("gltf_required_unlit_extension");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("required_unlit.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["KHR_materials_unlit"],
            "extensionsRequired": ["KHR_materials_unlit"],
            "materials": [{ "extensions": { "KHR_materials_unlit": {} } }]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/required_unlit.gltf").unwrap();

    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .unwrap();

    match &entry_for_label(&outcome, &uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert!(material.standard_material_descriptor().unlit);
        }
        other => panic!("unexpected required unlit material: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn required_anisotropy_factors_project_owned_semantics() {
    let root = unique_temp_project_root("gltf_required_anisotropy_factors");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("required_anisotropy_factors.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["KHR_materials_anisotropy"],
            "extensionsRequired": ["KHR_materials_anisotropy"],
            "materials": [{ "extensions": {
                "KHR_materials_anisotropy": {
                    "anisotropyStrength": 0.6,
                    "anisotropyRotation": 1.25
                }
            } }]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/required_anisotropy_factors.gltf").unwrap();

    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .expect("factor-only required anisotropy is owned by Standard PBR");

    match &entry_for_label(&outcome, &uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            let features = material.advanced_pbr_features();
            assert_eq!(features.anisotropy_strength, 0.6);
            assert_eq!(features.anisotropy_rotation, 1.25);
        }
        other => panic!("unexpected required anisotropy material: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn required_clearcoat_factors_project_owned_semantics() {
    let root = unique_temp_project_root("gltf_required_clearcoat_factors");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("required_clearcoat_factors.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["KHR_materials_clearcoat"],
            "extensionsRequired": ["KHR_materials_clearcoat"],
            "materials": [{ "extensions": {
                "KHR_materials_clearcoat": {
                    "clearcoatFactor": 0.8,
                    "clearcoatRoughnessFactor": 0.2
                }
            } }]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/required_clearcoat_factors.gltf").unwrap();

    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .expect("factor-only required clearcoat is owned by Standard PBR");

    match &entry_for_label(&outcome, &uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            let features = material.advanced_pbr_features();
            assert_eq!(features.clearcoat, 0.8);
            assert_eq!(features.clearcoat_perceptual_roughness, 0.2);
        }
        other => panic!("unexpected required clearcoat material: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn required_material_extensions_reject_semantics_without_shader_owners() {
    for (case_name, required_extensions, material_extensions, expected_field) in [
        (
            "clearcoat_texture",
            r#"["KHR_materials_clearcoat"]"#,
            r#"{
                "KHR_materials_clearcoat": {
                    "clearcoatFactor": 0.8,
                    "clearcoatTexture": { "index": 0 }
                }
            }"#,
            "KHR_materials_clearcoat.clearcoatTexture",
        ),
        (
            "clearcoat_roughness_texture",
            r#"["KHR_materials_clearcoat"]"#,
            r#"{
                "KHR_materials_clearcoat": {
                    "clearcoatRoughnessFactor": 0.2,
                    "clearcoatRoughnessTexture": { "index": 0 }
                }
            }"#,
            "KHR_materials_clearcoat.clearcoatRoughnessTexture",
        ),
        (
            "anisotropy_texture",
            r#"["KHR_materials_anisotropy"]"#,
            r#"{
                "KHR_materials_anisotropy": {
                    "anisotropyStrength": 0.6,
                    "anisotropyTexture": { "index": 0 }
                }
            }"#,
            "KHR_materials_anisotropy.anisotropyTexture",
        ),
        (
            "transmission_texture",
            r#"["KHR_materials_transmission"]"#,
            r#"{
                "KHR_materials_transmission": {
                    "transmissionFactor": 0.75,
                    "transmissionTexture": { "index": 0 }
                }
            }"#,
            "KHR_materials_transmission.transmissionTexture",
        ),
        (
            "thickness_texture",
            r#"["KHR_materials_transmission", "KHR_materials_volume"]"#,
            r#"{
                "KHR_materials_transmission": { "transmissionFactor": 0.75 },
                "KHR_materials_volume": {
                    "thicknessFactor": 0.25,
                    "thicknessTexture": { "index": 0 }
                }
            }"#,
            "KHR_materials_volume.thicknessTexture",
        ),
        (
            "zero_ior",
            r#"["KHR_materials_ior"]"#,
            r#"{ "KHR_materials_ior": { "ior": 0.0 } }"#,
            "KHR_materials_ior.ior=0",
        ),
    ] {
        let root = unique_temp_project_root(&format!("gltf_required_{case_name}"));
        fs::create_dir_all(&root).unwrap();
        let source_path = root.join(format!("{case_name}.gltf"));
        let source = format!(
            r#"{{
                "asset": {{ "version": "2.0" }},
                "extensionsUsed": {required_extensions},
                "extensionsRequired": {required_extensions},
                "images": [{{ "uri": "missing.png" }}],
                "textures": [{{ "source": 0 }}],
                "materials": [{{ "extensions": {material_extensions} }}]
            }}"#
        );
        fs::write(&source_path, source).unwrap();
        let uri = AssetUri::parse(&format!("res://models/{case_name}.gltf")).unwrap();

        let error = AssetImporter::default()
            .import_with_settings(&source_path, &uri, Default::default())
            .expect_err("required material semantics without shader owners must fail closed");
        assert!(
            error.to_string().contains(expected_field),
            "expected `{expected_field}` in {error}"
        );
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn external_webp_extension_source_uses_texture_asset_pipeline() {
    let root = unique_temp_project_root("gltf_external_webp_extension");
    fs::create_dir_all(&root).unwrap();
    image::save_buffer_with_format(
        root.join("pixel.webp"),
        &[32, 96, 160, 255],
        1,
        1,
        image::ColorType::Rgba8,
        image::ImageFormat::WebP,
    )
    .unwrap();
    let source_path = root.join("external_webp.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["EXT_texture_webp"],
            "extensionsRequired": ["EXT_texture_webp"],
            "images": [{ "uri": "pixel.webp", "mimeType": "image/webp" }],
            "textures": [{
                "extensions": { "EXT_texture_webp": { "source": 0 } }
            }]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/external_webp.gltf").unwrap();

    let outcome = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .unwrap();

    match &entry_for_label(&outcome, &uri, "Texture0").asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!((texture.width, texture.height), (1, 1));
            assert_eq!(texture.rgba.len(), 4);
            assert_eq!(texture.rgba[3], 255);
        }
        other => panic!("unexpected external WebP texture asset: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn optional_basisu_texture_reports_transcoder_requirement_before_image_decode() {
    let root = unique_temp_project_root("gltf_optional_basisu_texture");
    fs::create_dir_all(&root).unwrap();
    let source_path = root.join("basisu_texture.gltf");
    fs::write(
        &source_path,
        r#"{
            "asset": { "version": "2.0" },
            "extensionsUsed": ["KHR_texture_basisu"],
            "images": [{ "uri": "basis.ktx2", "mimeType": "image/ktx2" }],
            "textures": [{
                "extensions": { "KHR_texture_basisu": { "source": 0 } }
            }]
        }"#,
    )
    .unwrap();
    let uri = AssetUri::parse("res://models/basisu_texture.gltf").unwrap();

    let error = AssetImporter::default()
        .import_with_settings(&source_path, &uri, Default::default())
        .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("KHR_texture_basisu"));
    assert!(message.contains("KTX2/BasisU transcoder"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn all_checked_in_woc_glbs_import_from_original_bytes() {
    let root = woc_model_root();
    let glbs = checked_in_woc_glbs(&root);
    assert!(
        glbs.len() >= 938,
        "WOC model corpus is unexpectedly incomplete"
    );
    let importer = AssetImporter::default();

    for (index, source_path) in glbs.iter().enumerate() {
        let relative = source_path.strip_prefix(&root).unwrap();
        let uri = AssetUri::parse(&format!("res://woc/corpus/model_{index}.glb")).unwrap();
        let outcome = importer
            .import_with_settings(source_path, &uri, Default::default())
            .unwrap_or_else(|error| panic!("failed to import {}: {error}", relative.display()));
        assert!(matches!(
            &outcome.root_entry().expect("WOC root asset").asset,
            ImportedAsset::Model(model)
                if !model.primitives.is_empty()
                    && model.primitives.iter().all(|primitive| {
                        primitive.vertices.is_empty()
                            && primitive.indices.is_empty()
                            && primitive.mesh.is_some()
                    })
        ));
    }
}

#[test]
fn woc_imported_assets_and_dependency_graph_roundtrip_through_artifacts() {
    let (_, outcome) = import_woc_beach_anchor();
    let encoded_outcome = bincode::serialize(&outcome).unwrap();
    let decoded_outcome = bincode::deserialize::<AssetImportOutcome>(&encoded_outcome).unwrap();
    assert_eq!(decoded_outcome, outcome);

    let root = unique_temp_project_root("woc_gltf_artifact_roundtrip");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let store = ArtifactStore::default();

    for entry in &outcome.entries {
        let metadata = ResourceRecord::new(
            AssetId::new(),
            asset_kind_for_imported_asset(&entry.asset),
            entry.locator.clone(),
        );
        let artifact_uri = store.write(&paths, &metadata, &entry.asset).unwrap();
        let loaded = store.read(&paths, &artifact_uri).unwrap();
        assert_eq!(
            loaded, entry.asset,
            "artifact mismatch for {}",
            entry.locator
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn woc_gltf_importer_keeps_single_decode_and_cooked_payload_owners() {
    let importer = include_str!("../../../importer/ingest/import_gltf.rs");
    let labeled = include_str!("../../../importer/ingest/gltf_labeled_subassets.rs");
    let animation = include_str!("../../../importer/ingest/gltf_animation_subassets.rs");

    assert!(!importer.contains("primitive_asset.clone()"));
    assert!(importer.contains("primitive_asset.virtual_geometry.take()"));
    assert!(importer.contains("let primitive_reference = ModelPrimitiveAsset"));
    assert!(!labeled.contains("primitive.primitive.clone()"));
    assert!(labeled.contains("return Ok(image.pixels);"));
    assert!(animation.contains("let hierarchy = GltfHierarchyIndex::new(document)?;"));
    assert_eq!(
        animation
            .matches("GltfHierarchyIndex::new(document)")
            .count(),
        1
    );
    assert!(!animation.contains("fn parent_node_indices("));
    assert!(!animation.contains("document.skins().find("));
}
