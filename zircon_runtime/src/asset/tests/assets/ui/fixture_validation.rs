use super::*;

#[test]
fn fixture_v2_toml_importer_rejects_component_kind_in_favor_of_zui() {
    let root = unique_temp_project_root("v2_component_rejected_in_fixture_importer");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("legacy_component.v2.ui.toml");
    fs::write(&path, legacy_v2_component_toml()).unwrap();

    let error = importer_with_first_wave_plugin_fixtures()
        .import_from_source(
            &path,
            &AssetUri::parse("res://ui/legacy_component.v2.ui.toml").unwrap(),
        )
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("component documents must use `.zui`, not `.v2.ui.toml`"),
        "unexpected error: {error}"
    );

    let _ = fs::remove_dir_all(root);
}
