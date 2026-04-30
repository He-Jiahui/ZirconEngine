use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::{
    NativePluginLoader, PluginModuleKind, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};

#[test]
fn native_loader_rejects_load_manifest_entries_outside_export_root() {
    let root = temp_export_root("native-load-manifest-escape");
    let outside_root = root.with_file_name(format!(
        "{}-outside",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(root.join("plugins")).unwrap();
    fs::create_dir_all(&outside_root).unwrap();
    fs::write(outside_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        format!(
            r#"
[[plugins]]
id = "weather"
path = "../{outside_name}"
manifest = "../{outside_name}/plugin.toml"
"#,
            outside_name = outside_root.file_name().unwrap().to_string_lossy()
        ),
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert!(report.discovered.is_empty(), "{:?}", report.discovered);
    assert!(report.diagnostics.iter().any(|message| message
        .contains("native plugin weather load manifest manifest escapes export root")));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside_root);
}

#[test]
fn native_loader_exposes_v2_behavior_boundary_from_real_fixture() {
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

    let runtime_report = plugin.runtime_entry_report.as_ref().unwrap();
    assert_eq!(runtime_report.plugin_id, "native_dynamic_fixture");
    assert!(runtime_report
        .negotiated_capabilities
        .iter()
        .any(|capability| capability == "runtime.plugin.native_dynamic_fixture"));
    assert!(runtime_report
        .diagnostics
        .iter()
        .any(|message| message.contains("denied capability runtime.plugin.denied_fixture")));
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

    let editor_report = plugin.editor_entry_report.as_ref().unwrap();
    assert_eq!(editor_report.plugin_id, "native_dynamic_fixture");
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
    assert!(!registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
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

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
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
