use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::NativePluginLoader;

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

    let report = NativePluginLoader.load_from_load_manifest(&root);
    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].package_manifest.id, "weather");
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_calls_real_sample_descriptor_and_entries() {
    let sample_target = temp_export_root("native-dynamic-sample-target");
    let package_root = temp_export_root("native-dynamic-sample-package");
    let plugin_root = package_root.join("native_dynamic_sample");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_sample(&sample_target);
    fs::copy(
        &library_path,
        native_root.join(platform_library_file_name(
            "zircon_plugin_native_dynamic_sample_native",
        )),
    )
    .unwrap();
    fs::copy(
        repo_root().join("zircon_plugins/native_dynamic_sample/plugin.toml"),
        plugin_root.join("plugin.toml"),
    )
    .unwrap();

    let report = NativePluginLoader.load_discovered(&package_root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.loaded.len(), 1);
    let plugin = &report.loaded[0];
    assert_eq!(plugin.plugin_id, "native_dynamic_sample");
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .runtime_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_sample_runtime_entry_v1")
    );
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .editor_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_sample_editor_entry_v1")
    );
    assert_eq!(
        plugin.runtime_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_sample"
    );
    assert_eq!(
        plugin.editor_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_sample"
    );

    let registrations = report.runtime_plugin_registration_reports();
    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations[0].package_manifest.id,
        "native_dynamic_sample"
    );
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("runtime entry reached")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let _ = fs::remove_dir_all(sample_target);
    let _ = fs::remove_dir_all(package_root);
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

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

fn build_native_dynamic_sample(target_root: &std::path::Path) -> PathBuf {
    let manifest_path = repo_root().join("zircon_plugins/Cargo.toml");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_sample_native")
        .arg("--locked")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet")
        .status()
        .unwrap();
    assert!(
        status.success(),
        "native dynamic sample build failed: {status}"
    );
    target_root.join("debug").join(platform_library_file_name(
        "zircon_plugin_native_dynamic_sample_native",
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
