use super::*;

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
