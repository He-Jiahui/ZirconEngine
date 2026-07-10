use super::*;

#[test]
fn importer_registry_routes_zui_to_document_backend() {
    let root = unique_temp_project_root("zui_registry");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("hud_overlay.zui");
    fs::write(&path, minimal_zui_component_toml()).unwrap();

    let default_error = AssetImporter::default()
        .registry()
        .descriptor_for_source(&path)
        .unwrap_err();
    assert!(
        default_error
            .to_string()
            .contains("no asset importer registered"),
        "unexpected error: {default_error}"
    );

    let fixture_importer = importer_with_first_wave_plugin_fixtures();
    let fixture_descriptor = fixture_importer
        .registry()
        .descriptor_for_source(&path)
        .unwrap();
    assert_eq!(fixture_descriptor.id, "ui_document_importer.zui_document");
    assert_eq!(fixture_descriptor.importer_version, 2);
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
            .contains("typed toml asset suffix `.ui.toml` has no registered importer"),
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
        AssetImporterRegistryError::DeprecatedUiDocumentSuffixImporter {
            importer_id: "third_party.ui_source".to_string(),
            suffix: ".ui.toml".to_string(),
        }
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
        AssetImporterRegistryError::DeprecatedUiDocumentSuffixImporter {
            importer_id: "third_party.v2_ui".to_string(),
            suffix: ".v2.ui.toml".to_string(),
        }
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
