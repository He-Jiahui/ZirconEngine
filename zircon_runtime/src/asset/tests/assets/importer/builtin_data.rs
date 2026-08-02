use super::*;

#[test]
fn importer_default_decodes_builtin_png_texture_without_plugin_backend() {
    let root = unique_temp_project_root("builtin_png_texture_importer");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("checker.png");
    write_checker_png(path.clone());

    let report = AssetImporter::default()
        .capability_report_for_source(&path)
        .expect("png importer report");
    assert_eq!(report.descriptor.id, "zircon.builtin.texture.image");
    assert_eq!(report.status, AssetImporterCapabilityStatus::Available);

    let imported = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://textures/checker.png").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_default_decodes_txt_as_text_data() {
    let root = unique_temp_project_root("builtin_text_data_importer");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("license.txt");
    fs::write(&path, "CC0 fixture").unwrap();

    let imported = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://licenses/license.txt").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Text);
            assert_eq!(data.text, "CC0 fixture");
            assert_eq!(data.canonical_json, serde_json::Value::Null);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

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
    assert!(
        importer
            .capability_reports()
            .iter()
            .any(|report| report.descriptor.id == "zircon.builtin.zmesh"
                && report.status.is_available())
    );
    let cube_lut_report = importer
        .capability_report_for_source(Path::new("grade.cube"))
        .expect("cube LUT importer report");
    assert_eq!(
        cube_lut_report.descriptor.id,
        "zircon.builtin.texture.cube_lut"
    );
    assert!(cube_lut_report.status.is_available());
}
