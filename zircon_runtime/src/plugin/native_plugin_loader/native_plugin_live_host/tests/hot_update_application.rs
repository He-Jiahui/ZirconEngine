use super::*;

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
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("native plugin physics skipped because library is missing")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("plugin physics hot reload did not load a runtime native package")));
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
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("native plugin physics skipped because library is missing")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("plugin physics hot reload did not load a runtime native package")));

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
