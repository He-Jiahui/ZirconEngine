use super::*;

use crate::asset::pack::{
    ZrPackInputAsset, ZrPackPromotionMethod, ZrPackReader, ZrPackWriter,
    ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION,
};
use crate::plugin::BridgeOwnerTransitionMode;

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn native_runtime_hot_update_uses_export_load_manifest_package_set() {
    let export_root = unique_hot_update_temp_dir("manifest-package-set");
    write_runtime_package(&export_root, "physics", "physics_runtime");
    write_runtime_package(&export_root, "weather", "weather_runtime");
    write_load_manifest(
        &export_root,
        r#"
[[plugins]]
id = "physics"
path = "plugins/physics"
manifest = "plugins/physics/plugin.toml"
"#,
    );

    let report = NativePluginLiveHost::default()
        .hot_reload_runtime_plugins_from_export_root(&export_root)
        .expect("manifest-driven hot update should report package failures without host failure");

    assert!(report.loaded_plugin_ids.is_empty());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("native plugin physics skipped because library is missing")
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("plugin physics hot reload did not load a runtime native package")
    }));
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.contains("weather")),
        "unlisted plugin packages must not be considered during manifest-driven hot update"
    );

    let _ = fs::remove_dir_all(export_root);
}

#[test]
fn native_runtime_hot_update_reports_non_runtime_manifest_entries_as_skipped() {
    let export_root = unique_hot_update_temp_dir("editor-entry-skip");
    write_editor_package(&export_root, "editor_tools", "editor_tools");
    write_load_manifest(
        &export_root,
        r#"
[[plugins]]
id = "editor_tools"
path = "plugins/editor_tools"
manifest = "plugins/editor_tools/plugin.toml"
"#,
    );

    let report = NativePluginLiveHost::default()
        .hot_reload_runtime_plugins_from_export_root(&export_root)
        .expect("editor-only packages should be skipped without host failure");

    assert!(report.loaded_plugin_ids.is_empty());
    assert_eq!(
        report.diagnostics,
        vec!["native runtime hot update skipped non-runtime plugin package(s): editor_tools"]
    );

    let _ = fs::remove_dir_all(export_root);
}

#[test]
fn native_runtime_hot_update_accepts_runtime_feature_extension_modules() {
    let export_root = unique_hot_update_temp_dir("feature-runtime-entry");
    write_runtime_feature_extension_package(&export_root, "physics", "physics_runtime");
    write_load_manifest(
        &export_root,
        r#"
[[plugins]]
id = "physics"
path = "plugins/physics"
manifest = "plugins/physics/plugin.toml"
"#,
    );

    let report = NativePluginLiveHost::default()
        .hot_reload_runtime_plugins_from_export_root(&export_root)
        .expect("runtime feature-extension packages should be considered for hot update");

    assert_eq!(report.runtime_plugin_ids, vec!["physics"]);
    assert!(report.loaded_plugin_ids.is_empty());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("native plugin physics skipped because library is missing")
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("plugin physics hot reload did not load a runtime native package")
    }));

    let _ = fs::remove_dir_all(export_root);
}

#[test]
fn native_runtime_hot_update_report_applies_bridge_lifecycle_to_loaded_outcomes() {
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let mut report = NativePluginRuntimeHotUpdateReport {
        export_root: PathBuf::from("export-root"),
        manifest_plugin_ids: vec!["physics".to_string(), "editor_tools".to_string()],
        runtime_plugin_ids: vec!["physics".to_string()],
        loaded_plugin_ids: vec!["physics".to_string()],
        skipped_plugin_ids: vec!["editor_tools".to_string()],
        outcomes: vec![NativePluginLiveHostOutcome {
            plugin_id: "physics".to_string(),
            module_kind: PluginModuleKind::Runtime,
            command: NativePluginLiveHostCommand::HotReload,
            bridge_lifecycle_report: None,
            diagnostics: vec!["native.live_host.bridge_bindings_discovered".to_string()],
        }],
        diagnostics: vec!["native.live_host.bridge_bindings_discovered".to_string()],
    };

    report.apply_runtime_bridge_lifecycle(&lifecycle);

    let outcome = report
        .outcomes
        .first()
        .expect("hot update report should retain the loaded runtime outcome");
    let bridge_report = outcome
        .bridge_lifecycle_report
        .as_ref()
        .expect("loaded runtime hot update outcome should attach bridge lifecycle report");
    assert_eq!(bridge_report.event.mode, BridgeOwnerTransitionMode::Reload);
    assert!(bridge_report.is_applied());
    assert!(outcome
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("native.live_host.bridge_lifecycle")));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("native.live_host.bridge_lifecycle")));
}

#[test]
fn native_runtime_delta_hot_update_installs_pack_then_runs_manifest_hot_reload() {
    let export_root = unique_hot_update_temp_dir("delta-pack-runtime");
    write_runtime_package(&export_root, "physics", "physics_runtime");
    write_load_manifest(
        &export_root,
        r#"
[[plugins]]
id = "physics"
path = "plugins/physics"
manifest = "plugins/physics/plugin.toml"
"#,
    );
    let base_pack = export_root.join("packs").join("assets.zrpack");
    let delta_pack = export_root.join("downloads").join("assets.delta.zrpd");
    let staged_pack = export_root.join("staging").join("assets.zrpack");
    let backup_pack = export_root.join("backup").join("assets.previous.zrpack");
    let receipt_path = export_root.join("receipts").join("assets.install.json");
    write_delta_update_fixture(&base_pack, &delta_pack);

    let request = NativePluginRuntimeDeltaHotUpdateRequest::new(
        &export_root,
        &base_pack,
        &delta_pack,
        &staged_pack,
        &base_pack,
    )
    .with_backup_pack(&backup_pack)
    .with_receipt_path(&receipt_path);
    let report = NativePluginLiveHost::default()
        .hot_reload_runtime_plugins_after_delta_pack_install(request)
        .expect("delta install should complete before manifest-driven hot update diagnostics");

    assert_eq!(report.pack_install.base_pack, base_pack);
    assert_eq!(report.pack_install.delta_pack, delta_pack);
    assert_eq!(report.pack_promotion.backup_pack, Some(backup_pack));
    assert_eq!(
        report.pack_promotion.promotion_method,
        ZrPackPromotionMethod::Renamed
    );
    assert_eq!(
        report
            .pack_install_receipt
            .as_ref()
            .expect("install receipt should be written")
            .format_version,
        ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION
    );
    assert_eq!(report.plugin_hot_update.runtime_plugin_ids, vec!["physics"]);
    assert!(report.plugin_hot_update.loaded_plugin_ids.is_empty());
    assert!(report
        .plugin_hot_update
        .diagnostics
        .iter()
        .any(|diagnostic| {
            diagnostic.contains("native plugin physics skipped because library is missing")
        }));
    assert_eq!(
        ZrPackReader::from_bytes(fs::read(&base_pack).unwrap())
            .unwrap()
            .read_asset("textures/changed.bin")
            .unwrap(),
        b"new"
    );
    assert!(receipt_path.exists());

    let _ = fs::remove_dir_all(export_root);
}

fn write_runtime_package(export_root: &Path, plugin_id: &str, crate_name: &str) {
    write_plugin_package(export_root, plugin_id, crate_name, "runtime");
}

fn write_editor_package(export_root: &Path, plugin_id: &str, crate_name: &str) {
    write_plugin_package(export_root, plugin_id, crate_name, "editor");
}

fn write_runtime_feature_extension_package(export_root: &Path, plugin_id: &str, crate_name: &str) {
    let package_dir = export_root.join("plugins").join(plugin_id);
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("plugin.toml"),
        format!(
            r#"
id = "{plugin_id}"
version = "0.1.0"
display_name = "{plugin_id}"

[[feature_extensions]]
id = "physics_core"
display_name = "Physics Core"
owner_plugin_id = "{plugin_id}"

[[feature_extensions.modules]]
name = "runtime"
kind = "runtime"
crate_name = "{crate_name}"
"#
        ),
    )
    .unwrap();
}

fn write_plugin_package(export_root: &Path, plugin_id: &str, crate_name: &str, kind: &str) {
    let package_dir = export_root.join("plugins").join(plugin_id);
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(
        package_dir.join("plugin.toml"),
        format!(
            r#"
id = "{plugin_id}"
version = "0.1.0"
display_name = "{plugin_id}"

[[modules]]
name = "{kind}"
kind = "{kind}"
crate_name = "{crate_name}"
"#
        ),
    )
    .unwrap();
}

fn write_load_manifest(export_root: &Path, manifest: &str) {
    let manifest_path = export_root.join("plugins").join("native_plugins.toml");
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, manifest.trim_start()).unwrap();
}

fn write_delta_update_fixture(base_path: &Path, delta_path: &Path) {
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = crate::asset::pack::ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(base_path, base.bytes).unwrap();
    fs::write(delta_path, delta.bytes).unwrap();
}

fn unique_hot_update_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "zircon-native-hot-update-{label}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
