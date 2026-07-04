use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::plugin::native::NativePluginLoader;

#[path = "native_plugin_loader/real_fixture.rs"]
mod real_fixture;

#[test]
fn native_loader_discovers_candidates_from_export_load_manifest() {
    let root = temp_export_root("native-load-manifest");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.discovered.len(), 1);
    assert_eq!(report.discovered[0].plugin_id, "weather");
    assert_eq!(
        report.discovered[0].manifest_path,
        plugin_root.join("plugin.toml")
    );
    assert!(report.discovered[0]
        .library_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("zircon_plugin_weather_runtime"));
    assert_eq!(
        report.discovered[0].library_path,
        plugin_root
            .join("native")
            .join(platform_library_file_name("zircon_plugin_weather_runtime"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_reports_load_manifest_entry_mismatches() {
    let root = temp_export_root("native-load-manifest-mismatch");
    let declared_root = root.join("plugins").join("declared_weather");
    let actual_root = root.join("plugins").join("actual_weather");
    fs::create_dir_all(&declared_root).unwrap();
    fs::create_dir_all(&actual_root).unwrap();
    fs::write(actual_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "declared_weather"
path = "plugins/declared_weather"
manifest = "plugins/actual_weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert_eq!(report.discovered.len(), 1);
    assert!(report.diagnostics.iter().any(|message| message
        .contains("native plugin weather load manifest id mismatch: entry id declared_weather")));
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("native plugin weather load manifest path mismatch")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_deduplicates_load_manifest_package_ids() {
    let root = temp_export_root("native-load-manifest-duplicate");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"

[[plugins]]
id = "weather-alias"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert_eq!(report.discovered.len(), 1);
    assert_eq!(report.discovered[0].plugin_id, "weather");
    assert!(report.diagnostics.iter().any(|message| {
        message.contains("native plugin weather load manifest duplicate package id ignored")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_discovers_editor_only_native_package() {
    let root = temp_export_root("native-editor-only");
    let plugin_root = root.join("native_window_hosting");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        editor_only_plugin_manifest(),
    )
    .unwrap();

    let report = NativePluginLoader.discover(&root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.discovered.len(), 1);
    assert_eq!(report.discovered[0].plugin_id, "native_window_hosting");
    assert!(report.discovered[0]
        .library_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("zircon_plugin_native_window_hosting_editor"));
    assert!(
        report.runtime_plugin_registration_reports().is_empty(),
        "editor-only native packages must not enter runtime plugin registration"
    );
    let runtime_report = NativePluginLoader.load_discovered_runtime(&root);
    assert!(
        runtime_report.diagnostics.is_empty(),
        "runtime-only loading should skip editor-only packages without probing editor libraries: {:?}",
        runtime_report.diagnostics
    );
    assert!(runtime_report.loaded.is_empty());
    assert!(runtime_report
        .runtime_plugin_registration_reports()
        .is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_discovers_feature_extension_package_from_feature_runtime_module() {
    let root = temp_export_root("native-feature-extension");
    let plugin_root = root.join("sound_timeline_animation_track");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        feature_extension_plugin_manifest(),
    )
    .unwrap();

    let report = NativePluginLoader.discover(&root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.discovered.len(), 1);
    assert_eq!(
        report.discovered[0].plugin_id,
        "sound_timeline_animation_track"
    );
    assert!(report.discovered[0]
        .library_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("zircon_plugin_sound_timeline_animation_runtime"));
    assert!(report.runtime_plugin_registration_reports().is_empty());

    let feature_reports = report.runtime_plugin_feature_registration_reports();
    assert_eq!(feature_reports.len(), 1);
    assert_eq!(
        feature_reports[0].provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(feature_reports[0].manifest.owner_plugin_id, "sound");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_uses_target_module_crate_for_split_native_package_loading() {
    let root = temp_export_root("native-split-target-library");
    let plugin_root = root.join("split_tool");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        split_native_plugin_manifest(),
    )
    .unwrap();

    let runtime_report = NativePluginLoader.load_discovered_runtime(&root);
    assert!(runtime_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));
    assert!(!runtime_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));

    let editor_report = NativePluginLoader.load_discovered_editor(&root);
    assert!(editor_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));
    assert!(!editor_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));

    let full_report = NativePluginLoader.load_discovered_all(&root);
    assert!(full_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));
    assert!(full_report.diagnostics.iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_registration_reports_preserve_per_plugin_loader_diagnostics() {
    let root = temp_export_root("native-registration-diagnostics");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.load_all_from_load_manifest(&root);
    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].package_manifest.id, "weather");
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));

    let _ = fs::remove_dir_all(root);
}

fn runtime_plugin_manifest() -> &'static str {
    r#"
id = "weather"
version = "0.1.0"
display_name = "Weather"

[[modules]]
name = "weather.runtime"
kind = "runtime"
crate_name = "zircon_plugin_weather_runtime"
"#
}

fn editor_only_plugin_manifest() -> &'static str {
    r#"
id = "native_window_hosting"
version = "0.1.0"
display_name = "Native Window Hosting"

[[modules]]
name = "native_window_hosting.editor"
kind = "editor"
crate_name = "zircon_plugin_native_window_hosting_editor"
"#
}

fn split_native_plugin_manifest() -> &'static str {
    r#"
id = "split_tool"
version = "0.1.0"
display_name = "Split Tool"

[[modules]]
name = "split_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_split_tool_runtime"

[[modules]]
name = "split_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_split_tool_editor"
"#
}

fn feature_extension_plugin_manifest() -> &'static str {
    r#"
id = "sound_timeline_animation_track"
version = "0.1.0"
package_kind = "feature_extension"
display_name = "Sound Timeline Animation Track Provider"

[[feature_extensions]]
id = "sound.timeline_animation_track"
display_name = "Sound Timeline Animation Track"
owner_plugin_id = "sound"
capabilities = ["runtime.feature.sound.timeline_animation_track"]

[[feature_extensions.dependencies]]
plugin_id = "sound"
capability = "runtime.plugin.sound"
primary = true

[[feature_extensions.modules]]
name = "sound.timeline_animation_track.runtime"
kind = "runtime"
crate_name = "zircon_plugin_sound_timeline_animation_runtime"
target_modes = ["client_runtime"]
capabilities = ["runtime.feature.sound.timeline_animation_track"]
"#
}

fn native_dynamic_fixture_load_manifest() -> &'static str {
    r#"
[[plugins]]
id = "native_dynamic_fixture"
path = "plugins/native_dynamic_fixture"
manifest = "plugins/native_dynamic_fixture/plugin.toml"
package_report = "plugins/native_dynamic_fixture/native_dynamic_package.toml"

[plugins.abi]
abi_version = 3
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
descriptor_contract = "NativePluginAbiV3"
runtime_entry_source = "NativePluginAbiV3.runtime_entry_name"
editor_entry_source = "NativePluginAbiV3.editor_entry_name"
host_function_table = "NativePluginHostFunctionTableV3"
entry_report_contract = "NativePluginEntryReportV3"
behavior_contract = "NativePluginBehaviorV3"
state_snapshot_contract = "NativePluginBehaviorV3.save_state/restore_state"
bridge_method_table = "NativePluginBridgeMethodTableV3"
"#
}

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

fn build_native_dynamic_fixture(target_root: &std::path::Path) -> PathBuf {
    build_native_dynamic_fixture_with_features(target_root, &[])
}

fn build_native_dynamic_fixture_with_features(
    target_root: &std::path::Path,
    features: &[&str],
) -> PathBuf {
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(repo_root().join("zircon_plugins/Cargo.toml"))
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_fixture_native")
        .arg("--locked")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet");
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    let status = command.status().unwrap();
    assert!(
        status.success(),
        "native dynamic fixture build failed: {status}"
    );
    target_root.join("debug").join(platform_library_file_name(
        "zircon_plugin_native_dynamic_fixture_native",
    ))
}

fn platform_library_file_name(crate_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{crate_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{crate_name}.dylib")
    } else {
        format!("lib{crate_name}.so")
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
