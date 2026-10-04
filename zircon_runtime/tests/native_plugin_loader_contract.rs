use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::plugin::native::{
    discovery::{
        discover_native_plugins_from_load_manifest, load_discovered_native_plugins_with_authority,
        load_discovered_native_runtime_plugins_with_authority,
    },
    host::NativePluginHostHandle,
    NativePluginArtifactAuthority, NativePluginArtifactDigest, NativePluginArtifactExpectation,
    NativePluginArtifactTarget, NativePluginBehaviorHealth, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
use zircon_runtime::plugin::{PluginModuleKind, PluginPackageManifest};

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

    let report = discover_native_plugins_from_load_manifest(&root);

    assert!(report.discovered().is_empty(), "{:?}", report.discovered());
    assert!(report.diagnostics().iter().any(|message| message
        .contains("native plugin weather load manifest manifest escapes export root")));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside_root);
}

#[test]
#[cfg(windows)]
fn native_runtime_hot_update_loads_real_fixture_from_export_manifest() {
    let fixture_target = temp_native_fixture_target("native-dynamic-fixture-export-target");
    let export_root = temp_export_root("native-dynamic-fixture-export-root");

    let authority = materialize_native_dynamic_fixture_export_root(&fixture_target, &export_root);

    let host = NativePluginHostHandle::with_artifact_authority(authority);
    let report = host
        .hot_reload_runtime_plugins_from_export_root(&export_root)
        .expect("manifest-driven hot update should load the real runtime fixture");

    assert_eq!(
        report.manifest_plugin_ids,
        vec!["native_dynamic_fixture".to_string()]
    );
    assert_eq!(
        report.runtime_plugin_ids,
        vec!["native_dynamic_fixture".to_string()]
    );
    assert_eq!(
        report.loaded_plugin_ids,
        vec!["native_dynamic_fixture".to_string()]
    );
    assert!(report.skipped_plugin_ids.is_empty());
    assert_eq!(report.outcomes.len(), 1);
    assert_eq!(report.outcomes[0].plugin_id, "native_dynamic_fixture");
    assert_eq!(report.outcomes[0].module_kind, PluginModuleKind::Runtime);
    assert!(report.diagnostics.iter().any(|message| {
        message.contains("native plugin native_dynamic_fixture: runtime v3 entry reached")
    }));
    assert!(
        report
            .diagnostics
            .iter()
            .all(|message| !message.contains("editor entry reached")),
        "runtime hot update must not call the editor entry: {:?}",
        report.diagnostics
    );

    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime).unwrap(),
        vec!["native_dynamic_fixture".to_string()]
    );
    let echo_report = host
        .invoke_runtime_plugin_command("native_dynamic_fixture", "echo", b"manifest")
        .unwrap();
    assert_eq!(echo_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(echo_report.payload.as_deref(), Some(&b"echo:manifest"[..]));

    let source_library = export_root
        .join("plugins/native_dynamic_fixture/native")
        .join(platform_library_file_name(
            "zircon_plugin_native_dynamic_fixture_native",
        ));
    fs::write(source_library, b"tampered replacement DLL").unwrap();
    let rejected = host
        .hot_reload_runtime_plugins_from_export_root(&export_root)
        .unwrap();
    assert!(rejected.loaded_plugin_ids.is_empty());
    assert!(rejected
        .diagnostics
        .iter()
        .any(|message| message.contains("admission") || message.contains("mismatch")));
    let preserved = host
        .invoke_runtime_plugin_command("native_dynamic_fixture", "echo", b"preserved")
        .unwrap();
    assert_eq!(preserved.payload.as_deref(), Some(&b"echo:preserved"[..]));

    drop(host);
    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(export_root);
}

#[test]
#[cfg(windows)]
fn native_loader_exposes_v3_behavior_boundary_from_real_fixture() {
    let fixture_target = temp_native_fixture_target("native-dynamic-fixture-target");
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
    let manifest_path = write_product_fixture_manifest(&plugin_root.join("plugin.toml"));

    let report = load_discovered_native_plugins_with_authority(
        &package_root,
        &fixture_authority(&library_path, &manifest_path),
    );

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.loaded().len(), 1);
    let plugin = &report.loaded()[0];
    assert_eq!(plugin.plugin_id, "native_dynamic_fixture");
    assert_eq!(plugin.descriptor.as_ref().unwrap().abi_version, 3);
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
        Some("zircon_native_dynamic_fixture_runtime_entry_v3")
    );
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .editor_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_fixture_editor_entry_v3")
    );

    let runtime_report = plugin.runtime_entry_report.as_ref().unwrap();
    assert_eq!(runtime_report.plugin_id, "native_dynamic_fixture");
    assert_eq!(
        runtime_report.behavior_validation.health,
        NativePluginBehaviorHealth::Clean
    );
    assert!(runtime_report.behavior_validation.diagnostics.is_empty());
    assert_eq!(runtime_report.behavior_validation.is_stateless, Some(false));
    assert_eq!(
        runtime_report.behavior_validation.state_schema_version,
        Some(3)
    );
    assert_eq!(
        runtime_report
            .behavior_validation
            .command_manifest_schema
            .as_deref(),
        Some("zircon.native.command-manifest/4")
    );
    assert_eq!(
        runtime_report
            .behavior_validation
            .event_manifest_schema
            .as_deref(),
        Some("zircon.native.event-manifest/3")
    );
    assert!(runtime_report.behavior_validation.has_command_manifest);
    assert!(runtime_report.behavior_validation.has_event_manifest);
    assert!(runtime_report.behavior_validation.has_invoke_command);
    assert!(runtime_report.behavior_validation.has_save_state);
    assert!(runtime_report.behavior_validation.has_restore_state);
    assert!(runtime_report.behavior_validation.has_unload);
    assert!(runtime_report
        .negotiated_capabilities
        .iter()
        .any(|capability| capability == "runtime.plugin.native_dynamic_fixture"));
    assert!(runtime_report
        .diagnostics
        .iter()
        .any(|message| message.contains("denied capability runtime.plugin.denied_fixture")));
    assert_eq!(plugin.runtime_behavior_is_stateless(), Some(false));
    assert_eq!(plugin.runtime_state_schema_version(), Some(3));
    assert_eq!(
        plugin.runtime_command_manifest_schema(),
        Some("zircon.native.command-manifest/4")
    );
    assert_eq!(
        plugin.runtime_event_manifest_schema(),
        Some("zircon.native.event-manifest/3")
    );
    assert!(plugin.runtime_command_manifest().is_some_and(|manifest| {
        manifest.contains("name = \"echo\"\nslot = 0\npayload_schema = \"bytes\"")
    }));
    assert!(plugin
        .runtime_command_manifest()
        .is_some_and(|manifest| {
            manifest.contains(
                "name = \"bounded_overflow\"\nslot = 1\npayload_schema = \"bytes\"\nmax_output_bytes = 4",
            )
        }));
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

    let bounded_overflow_report = plugin.invoke_runtime_command("bounded_overflow", b"hello");
    assert_eq!(
        bounded_overflow_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert!(bounded_overflow_report.payload.is_none());
    assert!(bounded_overflow_report
        .diagnostics
        .iter()
        .any(|message| message.contains("exceeded its declared 4 byte limit")));

    let state_report = plugin.save_runtime_state();
    assert_eq!(state_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(
        state_report.payload.as_deref(),
        Some(&b"state:v3:native_dynamic_fixture"[..])
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
    assert_eq!(
        editor_report.behavior_validation.health,
        NativePluginBehaviorHealth::Clean
    );
    assert!(editor_report.behavior_validation.diagnostics.is_empty());
    assert_eq!(editor_report.behavior_validation.is_stateless, Some(true));
    assert!(!editor_report.behavior_validation.has_save_state);
    assert!(!editor_report.behavior_validation.has_restore_state);
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
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("host log level=2 target=native_dynamic_fixture.runtime")));
    assert!(registrations[0].diagnostics.iter().any(|message| message.contains(
        "host diagnostic plugin.native_dynamic_fixture.runtime.entry=1 count tags=plugin,native,runtime"
    )));
    assert!(!registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
#[cfg(windows)]
fn native_loader_rejects_unknown_abi_version_with_explicit_report() {
    let fixture_target = temp_native_fixture_target("native-dynamic-fixture-unknown-abi-target");
    let package_root = temp_export_root("native-dynamic-fixture-unknown-abi-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path =
        build_native_dynamic_fixture_with_features(&fixture_target, &["abi_unknown_version"]);
    fs::copy(
        &library_path,
        native_root.join(platform_library_file_name(
            "zircon_plugin_native_dynamic_fixture_native",
        )),
    )
    .unwrap();
    let manifest_path = write_product_fixture_manifest(&plugin_root.join("plugin.toml"));

    let report = load_discovered_native_runtime_plugins_with_authority(
        &package_root,
        &fixture_authority(&library_path, &manifest_path),
    );

    assert!(report.diagnostics().iter().any(|message| message.contains(
        "native plugin native_dynamic_fixture descriptor-probe contract abi_version mismatch"
    )));
    assert!(report
        .diagnostics()
        .iter()
        .any(|message| message.contains("expected 3, actual 99")));
    assert!(report.loaded().is_empty());
    let runtime_diagnostics = report.diagnostics_for_runtime_plugin("native_dynamic_fixture");
    assert!(runtime_diagnostics
        .iter()
        .any(|message| message.contains("descriptor-probe contract abi_version mismatch")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

fn build_native_dynamic_fixture(target_root: &std::path::Path) -> PathBuf {
    build_native_dynamic_fixture_with_features(target_root, &[])
}

fn build_native_dynamic_fixture_with_features(
    target_root: &std::path::Path,
    features: &[&str],
) -> PathBuf {
    let manifest_path = prepare_native_dynamic_fixture_workspace(target_root);
    let linker_temp = target_root.join("t");
    fs::create_dir_all(&linker_temp).unwrap();
    let lock_status = Command::new("cargo")
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--offline")
        .status()
        .unwrap();
    assert!(
        lock_status.success(),
        "native dynamic fixture lockfile generation failed: {lock_status}"
    );
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_fixture_native")
        .arg("--locked")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet");
    // The managed dev-dynamic lane exports `prefer-dynamic` to its Cargo
    // process. A product native DLL must not inherit that test-lane-only
    // linkage, otherwise its Rust `std-*.dll` import is already bound from
    // the host process and correctly rejected by native staging.
    command
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_BUILD_RUSTFLAGS")
        .env_remove("CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("CARGO_BUILD_RUSTC_WRAPPER")
        .env_remove("CARGO_TARGET_DIR")
        .env("TEMP", &linker_temp)
        .env("TMP", &linker_temp)
        .env("TMPDIR", &linker_temp);
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

fn prepare_native_dynamic_fixture_workspace(target_root: &std::path::Path) -> PathBuf {
    let source_root = repo_root().join("zircon_plugins/native_dynamic_fixture");
    let workspace_root = target_root.join("w");
    let package_root = workspace_root.join("n");
    let crate_root = package_root.join("native");
    let source_destination = crate_root.join("src");
    fs::create_dir_all(&source_destination).unwrap();
    fs::copy(
        source_root.join("native/src/lib.rs"),
        source_destination.join("lib.rs"),
    )
    .unwrap();
    fs::copy(
        source_root.join("native/src/tests/cases.rs"),
        source_destination.join("tests.rs"),
    )
    .unwrap();
    write_product_fixture_manifest(&package_root.join("plugin.toml"));

    let sdk_path = repo_root()
        .join("zircon_plugins/plugin_sdk")
        .to_string_lossy()
        .replace('\\', "/");
    let manifest = format!(
        r#"[package]
name = "zircon_plugin_native_dynamic_fixture_native"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[lib]
crate-type = ["cdylib"]

[features]
default = ["dist"]
dist = []
abi_unknown_version = []
descriptor_export_missing = []
runtime_entry_export_missing = []
required_capability_missing = []

[dependencies]
serde = {{ version = "1.0.228", features = ["derive"] }}
serde_json = "1.0.149"
zircon_plugin_sdk = {{ path = "{sdk_path}", default-features = false, features = ["native"] }}
"#
    );
    let manifest_path = crate_root.join("Cargo.toml");
    fs::write(&manifest_path, manifest).unwrap();
    manifest_path
}

fn materialize_native_dynamic_fixture_export_root(
    fixture_target: &std::path::Path,
    export_root: &std::path::Path,
) -> NativePluginArtifactAuthority {
    let plugin_root = export_root.join("plugins").join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_fixture(fixture_target);
    fs::copy(
        &library_path,
        native_root.join(platform_library_file_name(
            "zircon_plugin_native_dynamic_fixture_native",
        )),
    )
    .unwrap();
    let manifest_path = write_product_fixture_manifest(&plugin_root.join("plugin.toml"));
    fs::write(
        export_root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "native_dynamic_fixture"
path = "plugins/native_dynamic_fixture"
manifest = "plugins/native_dynamic_fixture/plugin.toml"
"#
        .trim_start(),
    )
    .unwrap();
    fixture_authority(&library_path, &manifest_path)
}

fn fixture_authority(
    library_path: &std::path::Path,
    manifest_path: &std::path::Path,
) -> NativePluginArtifactAuthority {
    use zircon_runtime::core::framework::{
        platform::RuntimeTargetMode, project::ExportTargetPlatform,
    };
    let manifest: PluginPackageManifest =
        toml::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let expectation = NativePluginArtifactExpectation::trusted_local_first_party(
        "native_dynamic_fixture",
        manifest.package_id(),
        NativePluginArtifactDigest::capture(manifest_path).unwrap(),
        NativePluginArtifactDigest::capture(library_path).unwrap(),
        "fixture-build-authority",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime, PluginModuleKind::Editor],
        manifest.capabilities,
    );
    NativePluginArtifactAuthority::from_expectations([expectation]).unwrap()
}

fn write_product_fixture_manifest(destination: &std::path::Path) -> PathBuf {
    let source = repo_root().join("zircon_plugins/native_dynamic_fixture/plugin.toml");
    let source_text = fs::read_to_string(source).unwrap();
    let role_marker = "package_role = \"test_fixture\"";
    let replacements = source_text.matches(role_marker).count();
    let manifest_text = source_text.replace(role_marker, "package_role = \"production\"");
    assert_eq!(replacements, 1, "native fixture role marker must be unique");
    fs::write(destination, manifest_text).unwrap();
    destination.to_path_buf()
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

fn temp_native_fixture_target(_label: &str) -> PathBuf {
    let target_root = std::env::current_exe()
        .unwrap()
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .unwrap()
        .to_path_buf();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    target_root.join(format!("fx-{stamp}"))
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
