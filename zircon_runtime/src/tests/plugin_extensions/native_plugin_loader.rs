use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    NativePluginLoader, PluginModuleKind, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};

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
    assert!(report.diagnostics.iter().any(|message| message
        .contains("native plugin weather load manifest duplicate package id ignored")));

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

#[test]
fn native_loader_calls_real_fixture_descriptor_and_entries() {
    let fixture_target = temp_export_root("native-dynamic-fixture-target");
    let package_root = temp_export_root("native-dynamic-fixture-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_fixture(&fixture_target);
    fs::copy(
        &library_path,
        native_root.join(platform_library_file_name(
            "zircon_plugin_native_dynamic_fixture_native",
        )),
    )
    .unwrap();
    fs::copy(
        repo_root().join("zircon_plugins/native_dynamic_fixture/plugin.toml"),
        plugin_root.join("plugin.toml"),
    )
    .unwrap();

    let report = NativePluginLoader.load_discovered_all(&package_root);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.loaded.len(), 1);
    let plugin = &report.loaded[0];
    assert_eq!(plugin.plugin_id, "native_dynamic_fixture");
    assert_eq!(plugin.descriptor.as_ref().unwrap().abi_version, 2);
    assert!(plugin
        .descriptor
        .as_ref()
        .unwrap()
        .requested_capabilities
        .iter()
        .any(|capability| capability == "runtime.plugin.native_dynamic_fixture"));
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .runtime_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_fixture_runtime_entry_v2")
    );
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .editor_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_fixture_editor_entry_v2")
    );
    assert_eq!(
        plugin.runtime_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_fixture"
    );
    assert!(plugin
        .runtime_entry_report
        .as_ref()
        .unwrap()
        .negotiated_capabilities
        .iter()
        .any(|capability| capability == "runtime.plugin.native_dynamic_fixture"));
    assert_eq!(plugin.runtime_behavior_is_stateless(), Some(false));
    assert!(plugin
        .runtime_command_manifest()
        .is_some_and(|manifest| manifest.contains("command=echo;payload=bytes")));
    assert!(plugin
        .runtime_command_manifest()
        .is_some_and(|manifest| manifest.contains("command=mismatched_buffer;payload=bytes")));
    assert!(plugin
        .runtime_event_manifest()
        .is_some_and(|manifest| manifest.contains("event=native_dynamic_fixture.echoed")));
    let echo_report = plugin.invoke_runtime_command("echo", b"hello");
    assert_eq!(echo_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(echo_report.payload.as_deref(), Some(&b"echo:hello"[..]));
    assert!(echo_report
        .diagnostics
        .iter()
        .any(|message| message.contains("serialized command echo completed")));
    let denied_report = plugin.invoke_runtime_command("unknown", b"hello");
    assert_eq!(
        denied_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED
    );
    assert!(denied_report
        .diagnostics
        .iter()
        .any(|message| message.contains("denied native command unknown")));
    let panic_report = plugin.invoke_runtime_command("panic", b"hello");
    assert_eq!(panic_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
    assert!(panic_report
        .diagnostics
        .iter()
        .any(|message| message.contains("caught panic")));
    let mismatch_report = plugin.invoke_runtime_command("mismatched_buffer", b"hello");
    assert_eq!(mismatch_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(
        mismatch_report.payload.as_deref(),
        Some(&b"mismatch:hello"[..])
    );
    assert!(mismatch_report.diagnostics.iter().any(
        |message| message.contains("owned buffer free failed: allocation/free owner mismatch")
    ));
    let state_report = plugin.save_runtime_state();
    assert_eq!(state_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(
        state_report.payload.as_deref(),
        Some(&b"state:v2:native_dynamic_fixture"[..])
    );
    let restore_report = plugin.restore_runtime_state(state_report.payload.as_ref().unwrap());
    assert_eq!(restore_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert!(restore_report
        .diagnostics
        .iter()
        .any(|message| message.contains("state restore accepted")));
    let invalid_restore_report = plugin.restore_runtime_state(b"invalid");
    assert_eq!(
        invalid_restore_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert!(invalid_restore_report
        .diagnostics
        .iter()
        .any(|message| message.contains("state restore rejected invalid blob")));
    let unload_report = plugin.unload_runtime_behavior();
    assert_eq!(unload_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert!(unload_report
        .diagnostics
        .iter()
        .any(|message| message.contains("unload callback reached")));
    assert_eq!(
        plugin.editor_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_fixture"
    );
    assert_eq!(plugin.editor_behavior_is_stateless(), Some(true));
    let editor_state_report = plugin.save_editor_state();
    assert_eq!(
        editor_state_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert!(editor_state_report
        .diagnostics
        .iter()
        .any(|message| message.contains("save_state is missing")));
    let editor_unload_report = plugin.unload_editor_behavior();
    assert_eq!(
        editor_unload_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
    assert!(editor_unload_report
        .diagnostics
        .iter()
        .any(|message| message.contains("stateless unload callback reached")));

    let registrations = report.runtime_plugin_registration_reports();
    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations[0].package_manifest.id,
        "native_dynamic_fixture"
    );
    assert!(registrations[0]
        .package_manifest
        .modules
        .iter()
        .all(|module| module.kind == PluginModuleKind::Runtime));
    assert!(registrations[0].project_selection.editor_crate.is_none());
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("negotiated runtime.plugin.native_dynamic_fixture")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("denied capability runtime.plugin.denied_fixture")));
    assert!(!registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_runtime_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));
    assert!(!report
        .diagnostics_for_runtime_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_editor_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(!report
        .diagnostics_for_editor_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));

    let runtime_report = NativePluginLoader.load_discovered_runtime(&package_root);
    assert!(
        runtime_report.diagnostics.is_empty(),
        "{:?}",
        runtime_report.diagnostics
    );
    assert_eq!(runtime_report.loaded.len(), 1);
    let runtime_loaded = &runtime_report.loaded[0];
    assert!(runtime_loaded.runtime_entry_report.is_some());
    assert!(runtime_loaded.editor_entry_report.is_none());
    assert_eq!(runtime_loaded.runtime_behavior_is_stateless(), Some(false));
    let runtime_registrations = runtime_report.runtime_plugin_registration_reports();
    assert!(runtime_registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));
    assert!(!runtime_registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let editor_report = NativePluginLoader.load_discovered_editor(&package_root);
    assert!(
        editor_report.diagnostics.is_empty(),
        "{:?}",
        editor_report.diagnostics
    );
    assert_eq!(editor_report.loaded.len(), 1);
    let editor_loaded = &editor_report.loaded[0];
    assert!(editor_loaded.runtime_entry_report.is_none());
    assert!(editor_loaded.editor_entry_report.is_some());
    assert_eq!(editor_loaded.editor_behavior_is_stateless(), Some(true));
    assert!(editor_report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(!editor_report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));

    let _ = fs::remove_dir_all(fixture_target);
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

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

fn build_native_dynamic_fixture(target_root: &std::path::Path) -> PathBuf {
    let manifest_path = repo_root().join("zircon_plugins/Cargo.toml");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_fixture_native")
        .arg("--locked")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet")
        .status()
        .unwrap();
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
