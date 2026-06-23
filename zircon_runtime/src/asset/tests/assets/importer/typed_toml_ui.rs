use super::*;

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
fn importer_default_rejects_v2_ui_toml_without_source_fixture_backend() {
    let error = AssetImporter::default()
        .registry()
        .descriptor_for_source(Path::new("source.v2.ui.toml"))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("UI v2 source-template suffix `.v2.ui.toml` has no registered importer"),
        "unexpected error: {error}"
    );
}

#[test]
fn importer_default_rejects_ui_toml_without_source_fixture_backend() {
    let error = AssetImporter::default()
        .registry()
        .descriptor_for_source(Path::new("source.ui.toml"))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("typed toml asset suffix `.ui.toml` has no registered importer"),
        "unexpected error: {error}"
    );
}

#[test]
fn importer_registry_rejects_non_fixture_ui_toml_source_importer_registration() {
    let mut registry = AssetImporterRegistry::default();

    let error = registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "third_party.ui_source",
                "third_party",
                crate::asset::AssetKind::UiLayout,
                1,
            )
            .with_full_suffixes([".ui.toml"]),
            |context| test_data_outcome(context, "source-template"),
        ))
        .unwrap_err();

    assert_eq!(
        error,
        AssetImporterRegistryError::UiTomlSourceImporter("third_party.ui_source".to_string())
    );
}

#[test]
fn importer_registry_rejects_non_fixture_v2_ui_toml_source_importer_registration() {
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
            |context| test_data_outcome(context, "source-template-v2"),
        ))
        .unwrap_err();

    assert_eq!(
        error,
        AssetImporterRegistryError::V2UiTomlSourceImporter("third_party.v2_ui".to_string())
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
fn importer_reports_ui_toml_schema_migration() {
    let root = unique_temp_project_root("ui_toml_migration");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("source_fixture.ui.toml");
    fs::write(&path, version_one_ui_layout_toml()).unwrap();

    let outcome = importer_with_first_wave_plugin_fixtures()
        .import_with_settings(
            &path,
            &AssetUri::parse("res://ui/source_fixture.ui.toml").unwrap(),
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
