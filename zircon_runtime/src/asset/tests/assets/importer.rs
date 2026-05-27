use std::fs;
use std::path::Path;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    importer_with_first_wave_plugin_fixtures, sample_animation_sequence_asset,
    sample_physics_material_asset, write_default_animation_sequence,
    write_default_physics_material,
};
use crate::asset::{
    AssetImportContext, AssetImportOutcome, AssetImporter, AssetImporterCapabilityStatus,
    AssetImporterDescriptor, AssetImporterRegistry, AssetImporterRegistryError, AssetUri,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, ImportedAssetEntry,
    MeshAttributeValues, MeshVertex, ModelAsset, ModelPrimitiveAsset, MESH_ATTRIBUTE_POSITION,
};
use crate::core::math::{Vec2, Vec3};
use crate::ui::template::UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;

#[test]
fn importer_subtree_uses_ingest_namespace_without_service_shell() {
    let importer_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/asset/importer");
    let importer_mod = fs::read_to_string(importer_root.join("mod.rs")).unwrap_or_default();

    assert!(
        importer_mod.contains("mod ingest;"),
        "asset importer root should declare the ingest subtree directly"
    );
    assert!(
        !importer_mod.contains("mod service;"),
        "asset importer root should not keep a migration-smell service subtree"
    );
    assert!(
        importer_root.join("ingest").exists(),
        "asset importer ingest subtree should exist after the hard cutover"
    );
    assert!(
        !importer_root.join("service").exists(),
        "asset importer service subtree should be deleted after the hard cutover"
    );
}

#[test]
fn importer_registry_uses_full_suffix_before_plain_extension_fallback() {
    let root = unique_temp_project_root("typed_toml_registry");
    fs::create_dir_all(&root).unwrap();
    let typed_path = root.join("layout.ui.toml");
    let plain_path = root.join("settings.toml");
    fs::write(
        &typed_path,
        r#"
[asset]
kind = "layout"
id = "main"
"#,
    )
    .unwrap();
    fs::write(&plain_path, "answer = 42").unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let typed = importer
        .import_from_source(
            &typed_path,
            &AssetUri::parse("res://ui/layout.ui.toml").unwrap(),
        )
        .unwrap();
    let plain = importer
        .import_from_source(
            &plain_path,
            &AssetUri::parse("res://data/settings.toml").unwrap(),
        )
        .unwrap();

    assert!(matches!(typed, ImportedAsset::UiLayout(_)));
    match plain {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format.as_str(), "toml");
            assert_eq!(data.uri.to_string(), "res://data/settings.toml");
            assert!(data.text.contains("answer = 42"));
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_routes_zui_to_component_backend() {
    let root = unique_temp_project_root("zui_registry");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("hud_overlay.zui");
    fs::write(&path, minimal_zui_component_toml()).unwrap();

    let default_descriptor = AssetImporter::default()
        .registry()
        .descriptor_for_source(&path)
        .unwrap();
    assert_eq!(default_descriptor.id, "zircon.builtin.ui_component.zui");
    assert_eq!(default_descriptor.importer_version, 2);
    assert_eq!(default_descriptor.full_suffixes, vec![".zui"]);
    let default_imported = AssetImporter::default()
        .import_from_source(&path, &AssetUri::parse("res://ui/hud_overlay.zui").unwrap())
        .unwrap();
    assert!(matches!(default_imported, ImportedAsset::UiV2Component(_)));

    let fixture_importer = importer_with_first_wave_plugin_fixtures();
    let fixture_descriptor = fixture_importer
        .registry()
        .descriptor_for_source(&path)
        .unwrap();
    assert_eq!(fixture_descriptor.id, "ui_document_importer.zui_component");
    assert_eq!(fixture_descriptor.full_suffixes, vec![".zui"]);

    let imported = fixture_importer
        .import_from_source(&path, &AssetUri::parse("res://ui/hud_overlay.zui").unwrap())
        .unwrap();
    assert!(matches!(imported, ImportedAsset::UiV2Component(_)));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_default_routes_zmaterial_and_rejects_legacy_material_toml() {
    let default_importer = AssetImporter::default();
    let zmaterial_descriptor = default_importer
        .registry()
        .descriptor_for_source(Path::new("hero.zmaterial"))
        .unwrap();

    assert_eq!(zmaterial_descriptor.id, "zircon.builtin.zmaterial");
    assert_eq!(zmaterial_descriptor.full_suffixes, vec![".zmaterial"]);

    let error = default_importer
        .registry()
        .descriptor_for_source(Path::new("hero.material.toml"))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("typed toml asset suffix `.material.toml` has no registered importer"),
        "unexpected error: {error}"
    );
}

#[test]
fn importer_default_rejects_legacy_v2_ui_toml_without_migration_fixture_backend() {
    let error = AssetImporter::default()
        .registry()
        .descriptor_for_source(Path::new("legacy.v2.ui.toml"))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("legacy UI v2 suffix `.v2.ui.toml` has no registered importer"),
        "unexpected error: {error}"
    );
}

#[test]
fn importer_default_rejects_legacy_ui_toml_without_migration_fixture_backend() {
    let error = AssetImporter::default()
        .registry()
        .descriptor_for_source(Path::new("legacy.ui.toml"))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("typed toml asset suffix `.ui.toml` has no registered importer"),
        "unexpected error: {error}"
    );
}

#[test]
fn importer_registry_rejects_non_fixture_legacy_ui_toml_importer_registration() {
    let mut registry = AssetImporterRegistry::default();

    let error = registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "third_party.legacy_ui",
                "third_party",
                crate::asset::AssetKind::UiLayout,
                1,
            )
            .with_full_suffixes([".ui.toml"]),
            |context| test_data_outcome(context, "legacy"),
        ))
        .unwrap_err();

    assert_eq!(
        error,
        AssetImporterRegistryError::LegacyUiTomlImporter("third_party.legacy_ui".to_string())
    );
}

#[test]
fn importer_registry_rejects_non_fixture_legacy_v2_ui_toml_importer_registration() {
    let mut registry = AssetImporterRegistry::default();

    let error = registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "third_party.v2_ui",
                "third_party",
                crate::asset::AssetKind::UiLayout,
                2,
            )
            .with_full_suffixes([".v2.ui.toml"]),
            |context| test_data_outcome(context, "legacy-v2"),
        ))
        .unwrap_err();

    assert_eq!(
        error,
        AssetImporterRegistryError::LegacyV2UiTomlImporter("third_party.v2_ui".to_string())
    );
}

#[test]
fn importer_registry_rejects_unknown_typed_toml_instead_of_plain_data_fallback() {
    let root = unique_temp_project_root("unknown_typed_toml_registry");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("enemy.ability.toml");
    fs::write(&path, "name = \"Enemy\"").unwrap();

    let error = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://prefabs/enemy.ability.toml").unwrap(),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("typed toml asset suffix `.ability.toml` has no registered importer"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_default_reports_missing_first_wave_plugin_backend() {
    let root = unique_temp_project_root("missing_first_wave_plugin_importer");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("checker.png");
    fs::write(&path, b"not decoded by the diagnostic importer").unwrap();

    let error = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://textures/checker.png").unwrap(),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("texture image importer plugin is not installed"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_capability_report_marks_diagnostic_only_backends() {
    let importer = AssetImporter::default();
    let report = importer
        .capability_report_for_source(Path::new("asset.fbx"))
        .expect("fbx diagnostic importer report");

    assert_eq!(report.descriptor.id, "zircon.optional.model.fbx");
    match report.status {
        AssetImporterCapabilityStatus::DiagnosticOnly { message } => {
            assert!(message.contains("fbx model importer backend is not installed"));
        }
        other => panic!("expected diagnostic-only capability, got {other:?}"),
    }
    assert!(importer
        .capability_reports()
        .iter()
        .any(
            |report| report.descriptor.id == "zircon.builtin.zmesh" && report.status.is_available()
        ));
}

#[test]
fn importer_reports_ui_toml_schema_migration() {
    let root = unique_temp_project_root("ui_toml_migration");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("legacy.ui.toml");
    fs::write(&path, version_one_ui_layout_toml()).unwrap();

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &path,
            &AssetUri::parse("res://ui/legacy.ui.toml").unwrap(),
            Default::default(),
        )
        .unwrap();

    let root_entry = outcome.root_entry().expect("ui root asset entry");
    let migration = root_entry
        .migration_report
        .clone()
        .expect("ui importer should report source schema migration");
    assert_eq!(migration.source_schema_version, Some(1));
    assert_eq!(
        migration.target_schema_version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert!(migration.summary.contains("SourceVersionBumped"));
    match &root_entry.asset {
        ImportedAsset::UiLayout(layout) => {
            assert_eq!(
                layout.document.asset.version,
                UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_priority_overrides_duplicate_extension() {
    let root = unique_temp_project_root("registry_priority");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.testdata");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.testdata").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.low", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["testdata"])
                .with_priority(0),
            |context| test_data_outcome(context, "low"),
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.high", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["testdata"])
                .with_priority(10),
            |context| test_data_outcome(context, "high"),
        ))
        .unwrap();

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "high"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_prefers_available_extension_importer_over_higher_priority_diagnostic() {
    let root = unique_temp_project_root("registry_available_over_diagnostic_extension");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.profileaudio");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.profileaudio").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(DiagnosticOnlyAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.externalized.profileaudio",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["profileaudio"])
            .with_priority(100),
            "profile audio importer is externalized",
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.available.profileaudio",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["profileaudio"])
            .with_priority(10),
            |context| test_data_outcome(context, "available"),
        ))
        .unwrap();

    let report = registry
        .capability_report_for_source(&path)
        .expect("available importer report");
    assert_eq!(report.descriptor.id, "test.available.profileaudio");
    assert!(report.status.is_available());

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "available"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_prefers_available_full_suffix_importer_over_higher_priority_diagnostic() {
    let root = unique_temp_project_root("registry_available_over_diagnostic_suffix");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.profile.audio");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.profile.audio").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(DiagnosticOnlyAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.externalized.profile_audio_suffix",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_full_suffixes([".profile.audio"])
            .with_priority(100),
            "profile audio suffix importer is externalized",
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.available.profile_audio_suffix",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_full_suffixes([".profile.audio"])
            .with_priority(10),
            |context| test_data_outcome(context, "available_suffix"),
        ))
        .unwrap();

    let report = registry
        .capability_report_for_source(&path)
        .expect("available suffix importer report");
    assert_eq!(report.descriptor.id, "test.available.profile_audio_suffix");
    assert!(report.status.is_available());

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "available_suffix"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_rejects_same_priority_duplicate_matcher() {
    let mut registry = AssetImporterRegistry::default();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.first", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["dup"]),
            |context| test_data_outcome(context, "first"),
        ))
        .unwrap();

    let error = registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.second", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["dup"]),
            |context| test_data_outcome(context, "second"),
        ))
        .unwrap_err();

    assert!(error.to_string().contains("duplicate importer matcher"));
}

#[test]
fn importer_validates_wgsl_and_reports_errors() {
    let root = unique_temp_project_root("shader_import");
    fs::create_dir_all(&root).unwrap();
    let valid_path = root.join("pbr.wgsl");
    let invalid_path = root.join("broken.wgsl");
    fs::write(&valid_path, valid_wgsl()).unwrap();
    fs::write(&invalid_path, "@vertex fn vs_main( {").unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let valid = importer
        .import_from_source(
            &valid_path,
            &AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        )
        .unwrap();
    let invalid = importer.import_from_source(
        &invalid_path,
        &AssetUri::parse("res://shaders/broken.wgsl").unwrap(),
    );

    match valid {
        ImportedAsset::Shader(shader) => {
            assert!(shader.source.contains("vs_main"));
            assert_eq!(shader.uri.to_string(), "res://shaders/pbr.wgsl");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert!(invalid.is_err());
    assert!(invalid.unwrap_err().to_string().contains("wgsl"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_obj_and_gltf_into_model_assets() {
    let root = unique_temp_project_root("model_import");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();

    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();

    let obj = importer
        .import_from_source(
            &obj_path,
            &AssetUri::parse("res://models/triangle.obj").unwrap(),
        )
        .unwrap();
    let gltf = importer
        .import_from_source(
            &gltf_path,
            &AssetUri::parse("res://models/triangle.gltf").unwrap(),
        )
        .unwrap();

    match obj {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_cooked_virtual_geometry(&model.primitives[0], "res://models/triangle.obj");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_cooked_virtual_geometry(&model.primitives[0], "res://models/triangle.gltf");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_mesh_subassets_for_model_imports() {
    let root = unique_temp_project_root("model_import_mesh_subassets");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
",
    )
    .unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let outcome = importer
        .import_with_settings(
            &obj_path,
            &AssetUri::parse("res://models/triangle.obj").unwrap(),
            Default::default(),
        )
        .unwrap();
    let mesh_uri = AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0").unwrap();

    assert!(matches!(
        &outcome.root_entry().unwrap().asset,
        ImportedAsset::Model(_)
    ));
    assert!(outcome
        .root_entry()
        .unwrap()
        .dependencies
        .contains(&mesh_uri));
    let mesh_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == mesh_uri)
        .expect("mesh subasset entry");
    match &mesh_entry.asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected mesh subasset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_bevy_style_gltf_labeled_subassets() {
    let root = unique_temp_project_root("gltf_labeled_subassets");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let outcome = importer
        .import_with_settings(
            &gltf_path,
            &AssetUri::parse("res://models/triangle.gltf").unwrap(),
            Default::default(),
        )
        .unwrap();

    let root_entry = outcome.root_entry().expect("root gltf entry");
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
                .contains(&gltf_test_label_uri(label)),
            "root dependencies should include {label}"
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == gltf_test_label_uri(label)),
            "outcome should include {label}"
        );
    }

    match &gltf_entry_for_label(&outcome, "Texture0").asset {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 1);
            assert_eq!(texture.height, 1);
            assert_eq!(texture.rgba.len(), 4);
        }
        other => panic!("unexpected Texture0 asset: {other:?}"),
    }
    match &gltf_entry_for_label(&outcome, "Material0").asset {
        ImportedAsset::Material(material) => {
            assert_eq!(material.name.as_deref(), Some("TriangleMaterial"));
            assert_eq!(material.base_color, [0.2, 0.3, 0.4, 1.0]);
            assert_eq!(
                material.base_color_texture.as_ref().unwrap().locator,
                gltf_test_label_uri("Texture0")
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }
    match &gltf_entry_for_label(&outcome, "Mesh0/Primitive0").asset {
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
    match &gltf_entry_for_label(&outcome, "Node0").asset {
        ImportedAsset::Scene(scene) => {
            let entity = scene.entities.first().expect("node entity");
            assert_eq!(entity.name, "TriangleNode");
            let mesh = entity.mesh.as_ref().expect("node mesh");
            assert_eq!(mesh.model.locator, gltf_test_label_uri("Mesh0"));
            assert_eq!(mesh.material.locator, gltf_test_label_uri("Material0"));
        }
        other => panic!("unexpected Node0 asset: {other:?}"),
    }
    for label in ["Animation0", "Skin0", "Skin0/InverseBindMatrices"] {
        match &gltf_entry_for_label(&outcome, label).asset {
            ImportedAsset::Data(data) => assert!(
                data.text.contains("not implemented yet"),
                "{label} should carry a diagnostic placeholder"
            ),
            other => panic!("unexpected {label} asset: {other:?}"),
        }
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh() {
    let root = unique_temp_project_root("model_toml_virtual_geometry_import");
    fs::create_dir_all(&root).unwrap();
    let model_path = root.join("triangle.model.toml");
    let base_vertices = vec![
        MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO),
        MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
        MeshVertex::new(Vec3::Y, Vec3::Y, Vec2::Y),
    ];
    let base_indices = vec![0, 1, 2];
    let source_model = ModelAsset {
        uri: AssetUri::parse("res://models/triangle.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: base_vertices.clone(),
            indices: base_indices.clone(),
            virtual_geometry: None,
        }],
    };
    fs::write(&model_path, source_model.to_toml_string().unwrap()).unwrap();

    let imported = AssetImporter::default()
        .import_from_source(
            &model_path,
            &AssetUri::parse("res://models/triangle.model.toml").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices, base_vertices);
            assert_eq!(model.primitives[0].indices, base_indices);
            assert_cooked_virtual_geometry(
                &model.primitives[0],
                "res://models/triangle.model.toml",
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_skinning_channels_on_model_vertices() {
    let root = unique_temp_project_root("skinned_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_skinned_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();

    let gltf = importer
        .import_from_source(
            &gltf_path,
            &AssetUri::parse("res://models/skinned_triangle.gltf").unwrap(),
        )
        .unwrap();

    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
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

fn gltf_entry_for_label<'a>(
    outcome: &'a AssetImportOutcome,
    label: &str,
) -> &'a ImportedAssetEntry {
    let locator = gltf_test_label_uri(label);
    outcome
        .entries
        .iter()
        .find(|entry| entry.locator == locator)
        .unwrap_or_else(|| panic!("missing gltf subasset {locator}"))
}

fn gltf_test_label_uri(label: &str) -> AssetUri {
    AssetUri::parse(&format!("res://models/triangle.gltf#{label}")).unwrap()
}

fn identity_bind_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[test]
fn importer_decodes_physics_material_and_animation_sequence_assets() {
    let root = unique_temp_project_root("physics_animation_import");
    fs::create_dir_all(&root).unwrap();
    let physics_material_path = root.join("default.physics_material.toml");
    let sequence_path = root.join("hero.sequence.zranim");

    write_default_physics_material(physics_material_path.clone());
    write_default_animation_sequence(sequence_path.clone());

    let importer = AssetImporter::default();
    let physics_material = importer
        .import_from_source(
            &physics_material_path,
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
        )
        .unwrap();
    let sequence = importer
        .import_from_source(
            &sequence_path,
            &AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
        )
        .unwrap();

    assert_eq!(
        physics_material,
        ImportedAsset::PhysicsMaterial(sample_physics_material_asset())
    );
    assert_eq!(
        sequence,
        ImportedAsset::AnimationSequence(sample_animation_sequence_asset())
    );

    let _ = fs::remove_dir_all(root);
}

fn valid_wgsl() -> &'static str {
    r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#
}

fn version_one_ui_layout_toml() -> &'static str {
    r#"
[asset]
kind = "layout"
id = "legacy.layout"
version = 1
display_name = "Legacy Layout"

[root]
node_id = "legacy_root"
kind = "native"
type = "VerticalBox"
control_id = "LegacyRoot"
"#
}

fn minimal_zui_component_toml() -> &'static str {
    r#"
[asset]
kind = "component"
id = "runtime.ui.hud_overlay"
version = 2
display_name = "Runtime HUD Overlay"

[components.HudOverlay]
root = "root"

[nodes.root]
component = "Text"
control_id = "HudRoot"
props = { text = "HUD" }
"#
}

fn test_data_outcome(
    context: &AssetImportContext,
    winner: &'static str,
) -> Result<AssetImportOutcome, crate::asset::AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(crate::asset::DataAsset {
            uri: context.uri.clone(),
            format: crate::asset::DataAssetFormat::Json,
            text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
            canonical_json: serde_json::json!({ "winner": winner }),
        }),
    ))
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

fn write_triangle_gltf(root: &Path) -> std::path::PathBuf {
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

fn write_skinned_triangle_gltf(root: &Path) -> std::path::PathBuf {
    let buffer_path = root.join("skinned_triangle.bin");
    let gltf_path = root.join("skinned_triangle.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for joint in [
        0_u16, 1, 0, 0, //
        1, 0, 0, 0, //
        0, 0, 0, 0,
    ] {
        bytes.extend_from_slice(&joint.to_le_bytes());
    }
    for weight in [
        0.75_f32, 0.25, 0.0, 0.0, //
        1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 0.0,
    ] {
        bytes.extend_from_slice(&weight.to_le_bytes());
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
    { "uri": "skinned_triangle.bin", "byteLength": 114 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 48, "target": 34962 },
    { "buffer": 0, "byteOffset": 108, "byteLength": 6, "target": 34963 }
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
      "type": "VEC4"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC4"
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "JOINTS_0": 1,
            "WEIGHTS_0": 2
          },
          "indices": 3
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
