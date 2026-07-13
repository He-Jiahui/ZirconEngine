use super::*;

#[test]
fn asset_manager_service_reports_importer_capabilities_before_and_after_project_open() {
    let root = unique_temp_project_root("asset_manager_importer_capabilities");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let service: &dyn AssetManager = &manager;

    assert!(service
        .asset_importer_capability_reports()
        .iter()
        .any(|report| {
            report.descriptor.id == "texture_importer.image" && report.status.is_available()
        }));
    let before_open = service
        .asset_importer_capability_report_for_source("checker.png")
        .expect("png importer capability before project open");
    assert_eq!(before_open.descriptor.id, "texture_importer.image");
    assert!(before_open.status.is_available());

    service
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let after_open = service
        .asset_importer_capability_report_for_source("checker.png")
        .expect("png importer capability after project open");
    assert_eq!(after_open.descriptor.id, "texture_importer.image");
    assert!(after_open.status.is_available());

    let diagnostic = service
        .asset_importer_capability_report_for_source("legacy.fbx")
        .expect("fbx diagnostic importer capability");
    assert_eq!(diagnostic.descriptor.id, "zircon.optional.model.fbx");
    match diagnostic.status {
        AssetImporterCapabilityStatus::DiagnosticOnly { message } => {
            assert!(message.contains("fbx model importer backend is not installed"));
        }
        other => panic!("expected diagnostic-only fbx importer, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
