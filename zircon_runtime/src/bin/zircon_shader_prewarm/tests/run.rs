use super::*;
use zircon_runtime::core::resource::{ResourceKind, ResourceRecord, ResourceState};

#[test]
fn exported_resource_records_move_into_the_overlay() {
    let source = include_str!("../run.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let projection = source
        .split("let resource_registry =")
        .nth(1)
        .unwrap()
        .split("let cache_dir")
        .next()
        .unwrap();

    assert!(projection.contains("exported_resource_records.map("));
    assert!(!projection.contains("exported_resource_records\n            .clone()"));
}

#[test]
fn warm_unchanged_asset_roots_skip_projection_without_external_inputs() {
    assert!(!asset_root_requires_prewarm_projection(false, false, false));
    assert!(asset_root_requires_prewarm_projection(true, false, false));
    assert!(asset_root_requires_prewarm_projection(false, true, false));
    assert!(asset_root_requires_prewarm_projection(false, false, true));
}

#[test]
fn default_warm_asset_roots_skip_inventory_payload_hydration() {
    let source = include_str!("../run.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let inventory_loop = source
        .split("let inventory_snapshot_root")
        .nth(1)
        .expect("run should prepare an inventory snapshot root")
        .split("let exported_resource_records")
        .next()
        .expect("inventory collection should end before registry export");

    assert!(inventory_loop.contains("needs_unchanged_inventory_payload"));
    assert!(inventory_loop.contains("warm_snapshot_is_current_excluding"));
    assert!(inventory_loop.contains("continue;"));
    assert!(
        inventory_loop.find("warm_snapshot_is_current_excluding")
            < inventory_loop.find("collect_with_warm_snapshot_excluding"),
        "the compact index check must precede full inventory hydration"
    );
}

#[test]
fn invalid_execution_budget_emits_a_json_preflight_report_before_asset_io() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_invalid_budget_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("fixture root should be created");
    let project_root = root.join("missing_project");
    let asset_root = root.join("missing_assets");
    let report_path = root.join("preflight_report.json");

    let exit_code = run([
        "--project-root",
        project_root.to_str().expect("fixture path should be UTF-8"),
        "--asset-root",
        asset_root.to_str().expect("fixture path should be UTF-8"),
        "--report",
        report_path.to_str().expect("fixture path should be UTF-8"),
        "--max-resident-source-bytes",
        "0",
    ]
    .into_iter()
    .map(OsString::from))
    .expect("invalid budget should still produce a report");
    assert_eq!(exit_code, ExitCode::from(2));
    assert!(
        !project_root.exists() && !asset_root.exists(),
        "budget preflight must run before cache creation or asset inventory IO"
    );

    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(&report_path).expect("preflight report should be written"),
    )
    .expect("preflight report should be JSON");
    assert_eq!(report["requested_count"], 0);
    assert_eq!(report["written_count"], 0);
    assert_eq!(report["failed_count"], 0);
    assert_eq!(report["failures"], serde_json::json!([]));
    assert!(report["preflight_error"]
        .as_str()
        .is_some_and(|error| error.contains("max_resident_source_bytes must be non-zero")));

    fs::remove_dir_all(root).expect("fixture root should be removed");
}

#[test]
fn shader_prewarm_rejects_a_cache_root_equal_to_an_asset_root() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_cache_root_conflict_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&root).expect("fixture root should be created");
    let nested_cache_root = root.join("cache");
    fs::create_dir_all(&nested_cache_root).expect("nested cache root should be created");

    assert!(cache_root_matches_asset_root(&root, &root));
    assert!(
        !cache_root_matches_asset_root(&nested_cache_root, &root),
        "a nested cache root remains a valid asset-inventory exclusion"
    );

    fs::remove_dir_all(root).expect("fixture root should be removed");
}

#[test]
fn shader_prewarm_report_write_reports_typed_directory_error() {
    let report_parent = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_report_parent_{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&report_parent);
    let _ = fs::remove_dir_all(&report_parent);
    fs::write(&report_parent, "not a directory").unwrap();
    let report_path = report_parent.join("report.json");

    let error = write_shader_prewarm_report(&report_path, "{}").unwrap_err();

    match error {
        ShaderPrewarmReportError::CreateReportDirectory { path, source: _ } => {
            assert_eq!(path, report_parent);
        }
        other => panic!("expected typed report directory error, got {other:?}"),
    }

    let _ = fs::remove_file(report_parent);
}

#[test]
fn shader_prewarm_project_and_plugin_asset_roots_export_wrapped_resource_registry_file() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_project_plugin_export_file_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let project_root = root.join("project_assets");
    let plugin_root = root.join("plugin_assets");
    let export_path = root
        .join("ZirconEngine")
        .join("cache")
        .join("shader_resource_records.json");
    write_named_shader_with_meta(
        &project_root,
        "project",
        "00000000-0000-0000-0000-000000000056",
        "res://project/shaders/project",
        "source-hash-project-export-file",
    );
    write_named_shader_with_meta(
        &plugin_root,
        "plugin",
        "00000000-0000-0000-0000-000000000057",
        "package://virtual_geometry/shaders/plugin",
        "source-hash-plugin-export-file",
    );

    let returned_records = export_shader_resource_registry_for_asset_roots(
        &[project_root, plugin_root],
        Some(&export_path),
    )
    .unwrap()
    .unwrap();
    let exported_value: serde_json::Value =
        serde_json::from_slice(&fs::read(&export_path).unwrap()).unwrap();
    let exported_records =
        serde_json::from_value::<Vec<ResourceRecord>>(exported_value["resources"].clone()).unwrap();

    assert_eq!(returned_records.len(), exported_records.len());
    for (returned, exported) in returned_records.iter().zip(&exported_records) {
        assert_eq!(returned.id, exported.id);
        assert_eq!(returned.primary_locator, exported.primary_locator);
        assert_eq!(returned.revision, exported.revision);
    }
    assert_eq!(exported_records.len(), 2);
    assert_eq!(
        exported_records
            .iter()
            .map(|record| record.primary_locator.to_string())
            .collect::<Vec<_>>(),
        vec![
            "package://virtual_geometry/shaders/plugin".to_string(),
            "res://project/shaders/project".to_string()
        ]
    );
    assert!(exported_records.iter().all(|record| {
        record.kind == ResourceKind::Shader
            && record.state == ResourceState::Ready
            && record.revision != 0
    }));
    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(exported_records.clone());
    for record in &exported_records {
        let label = record.primary_locator.to_string();
        assert_eq!(
            overlay.revision_for(record.id, &label),
            Some(record.revision)
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_resource_registry_export_reports_typed_directory_error() {
    let export_parent = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_resource_registry_parent_{}",
        std::process::id()
    ));
    let _ = fs::remove_file(&export_parent);
    let _ = fs::remove_dir_all(&export_parent);
    fs::write(&export_parent, "not a directory").unwrap();
    let export_path = export_parent.join("shader_resource_records.json");

    let error =
        export_shader_resource_registry_for_asset_roots(&[], Some(&export_path)).unwrap_err();

    match error {
        ShaderPrewarmResourceRegistryError::CreateExportDirectory { path, source: _ } => {
            assert_eq!(path, export_parent);
        }
        other => panic!("expected typed resource registry directory error, got {other:?}"),
    }

    let _ = fs::remove_file(export_parent);
}

fn write_named_shader_with_meta(
    asset_root: &Path,
    name: &str,
    id: &str,
    locator: &str,
    source_hash: &str,
) {
    fs::create_dir_all(asset_root.join("shaders")).unwrap();
    fs::write(
        asset_root.join("shaders").join(format!("{name}.wgsl")),
        format!("fn {name}() {{}}\n"),
    )
    .unwrap();
    fs::write(
        asset_root
            .join("shaders")
            .join(format!("{name}.wgsl.zmeta")),
        format!(
            r#"format_version = 7
uuid = "{id}"
url = "{locator}"
asset_kind = "Shader"
unit = "single"
source_digest = "{source_hash}"
"#
        ),
    )
    .unwrap();
}
