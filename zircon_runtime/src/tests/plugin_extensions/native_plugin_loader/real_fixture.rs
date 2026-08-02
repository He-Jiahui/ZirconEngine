use std::sync::Arc;

use super::*;
use crate::asset::{
    AssetImportContext, AssetImporterDescriptor, AssetImporterHandler, AssetKind,
    AssetSchemaMigrationReport, DataAssetFormat, ImportedAsset, NativeAssetImporterHandler,
};
use crate::plugin::native::{
    NativePluginBehaviorHealth, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED,
    ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
    ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};
use crate::plugin::PluginModuleKind;

#[test]
fn native_loader_loads_real_fixture_from_export_load_manifest_payload() {
    let fixture_target = temp_export_root("native-dynamic-export-fixture-target");
    let export_root = temp_export_root("native-dynamic-export-fixture-root");
    let plugin_root = export_root.join("plugins").join("native_dynamic_fixture");
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
    fs::write(
        plugin_root.join("native_dynamic_package.toml"),
        r#"
format_version = 1
package_id = "native_dynamic_fixture"
path = "plugins/native_dynamic_fixture"
"#,
    )
    .unwrap();
    fs::write(
        export_root.join("plugins").join("native_plugins.toml"),
        native_dynamic_fixture_load_manifest(),
    )
    .unwrap();

    let report = NativePluginLoader.load_all_from_load_manifest(&export_root);

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.loaded().len(), 1);
    let plugin = &report.loaded()[0];
    assert_eq!(plugin.plugin_id, "native_dynamic_fixture");
    assert_eq!(plugin.descriptor.as_ref().unwrap().abi_version, 3);
    assert!(plugin.runtime_entry_report.is_some());
    assert!(plugin.editor_entry_report.is_some());
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached with v3 host ABI table")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(export_root);
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
    assert_eq!(plugin.runtime_state_schema_version(), Some(3));
    assert_eq!(
        plugin.runtime_command_manifest_schema(),
        Some("zircon.native.command-manifest/4")
    );
    assert_eq!(
        plugin.runtime_event_manifest_schema(),
        Some("zircon.native.event-manifest/3")
    );
    assert!(plugin
        .runtime_command_manifest()
        .is_some_and(|manifest| manifest.contains("name = \"echo\"")));
    assert!(plugin
        .runtime_command_manifest()
        .is_some_and(|manifest| manifest.contains("slot = 0")));
    assert!(plugin
        .runtime_event_manifest()
        .is_some_and(|manifest| manifest.contains("event=native_dynamic_fixture.echoed")));
    assert_eq!(
        plugin.runtime_behavior_health(),
        Some(NativePluginBehaviorHealth::Clean)
    );
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
    let bounded_output_report = plugin.invoke_runtime_command("bounded_overflow", b"hello");
    assert_eq!(
        bounded_output_report.status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert!(bounded_output_report.diagnostics.iter().any(|message| {
        message.contains("output exceeds") || message.contains("declared 4 byte limit")
    }));
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
    assert_eq!(
        plugin.editor_entry_report.as_ref().unwrap().plugin_id,
        "native_dynamic_fixture"
    );
    assert_eq!(plugin.editor_behavior_is_stateless(), Some(true));
    assert_eq!(
        plugin.editor_behavior_health(),
        Some(NativePluginBehaviorHealth::Clean)
    );
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
        .any(|message| message.contains("negotiated runtime.plugin.native_dynamic_fixture")));
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("denied capability runtime.plugin.denied_fixture")));
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
    assert!(report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .entry_diagnostics()
        .iter()
        .any(|message| { message.contains("editor entry reached with v3 host ABI table") }));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached with v3 host ABI table")));
    assert!(report
        .diagnostics_for_runtime_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));
    assert!(!report
        .diagnostics_for_runtime_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_editor_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(report
        .diagnostics_for_editor_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("editor entry reached with v3 host ABI table")));
    assert!(!report
        .diagnostics_for_editor_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));

    let runtime_report = NativePluginLoader.load_discovered_runtime(&package_root);
    assert!(
        runtime_report.diagnostics().is_empty(),
        "{:?}",
        runtime_report.diagnostics()
    );
    assert_eq!(runtime_report.loaded().len(), 1);
    let runtime_loaded = &runtime_report.loaded()[0];
    assert!(runtime_loaded.runtime_entry_report.is_some());
    assert!(runtime_loaded.editor_entry_report.is_none());
    assert_eq!(runtime_loaded.runtime_behavior_is_stateless(), Some(false));
    let runtime_registrations = runtime_report.runtime_plugin_registration_reports();
    assert!(runtime_registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));
    assert!(!runtime_registrations[0]
        .diagnostics
        .iter()
        .any(|message| message.contains("editor entry reached")));

    let editor_report = NativePluginLoader.load_discovered_editor(&package_root);
    assert!(
        editor_report.diagnostics().is_empty(),
        "{:?}",
        editor_report.diagnostics()
    );
    assert_eq!(editor_report.loaded().len(), 1);
    let editor_loaded = &editor_report.loaded()[0];
    assert!(editor_loaded.runtime_entry_report.is_none());
    assert!(editor_loaded.editor_entry_report.is_some());
    assert_eq!(editor_loaded.editor_behavior_is_stateless(), Some(true));
    assert!(editor_report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("editor entry reached")));
    assert!(editor_report
        .entry_diagnostics()
        .iter()
        .any(|message| { message.contains("editor entry reached with v3 host ABI table") }));
    assert!(!editor_report
        .entry_diagnostics()
        .iter()
        .any(|message| message.contains("runtime v3 entry reached with host ABI table")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_falls_back_to_v2_when_v3_descriptor_is_absent() {
    let fixture_target = temp_export_root("native-dynamic-fixture-v2-target");
    let package_root = temp_export_root("native-dynamic-fixture-v2-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path =
        build_native_dynamic_fixture_with_features(&fixture_target, &["abi_v2_only"]);
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

    let report = NativePluginLoader.load_discovered_runtime(&package_root);

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.loaded().len(), 1);
    let plugin = &report.loaded()[0];
    assert_eq!(plugin.descriptor.as_ref().unwrap().abi_version, 2);
    assert_eq!(
        plugin
            .descriptor
            .as_ref()
            .unwrap()
            .runtime_entry_name
            .as_deref(),
        Some("zircon_native_dynamic_fixture_runtime_entry_v2")
    );
    assert_eq!(plugin.runtime_behavior_is_stateless(), Some(false));
    assert_eq!(plugin.runtime_state_schema_version(), Some(0));
    assert_eq!(
        plugin.runtime_behavior_health(),
        Some(NativePluginBehaviorHealth::Degraded)
    );
    assert!(plugin
        .runtime_behavior_validation_report()
        .unwrap()
        .diagnostics
        .iter()
        .any(|message| message.contains("legacy ABI v2 behavior is metadata-only")));
    let command = plugin.invoke_runtime_command("echo", b"hello");
    assert_eq!(command.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert!(command
        .diagnostics
        .iter()
        .any(|message| message.contains("legacy ABI v2 behavior cannot invoke commands")));
    assert!(report
        .diagnostics_for_runtime_plugin("native_dynamic_fixture")
        .iter()
        .any(|message| message.contains("runtime v2 entry reached with host ABI table")));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_rejects_unknown_abi_version_with_explicit_report() {
    let fixture_target = temp_export_root("native-dynamic-fixture-unknown-abi-target");
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
    fs::copy(
        repo_root().join("zircon_plugins/native_dynamic_fixture/plugin.toml"),
        plugin_root.join("plugin.toml"),
    )
    .unwrap();

    let report = NativePluginLoader.load_discovered_runtime(&package_root);

    assert!(report
        .diagnostics()
        .iter()
        .any(|message| message.contains("descriptor-probe contract abi_version mismatch")));
    assert!(report
        .diagnostics()
        .iter()
        .any(|message| message.contains("expected 3, actual 99")));
    assert!(report.loaded().is_empty());

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_rejects_library_without_descriptor_export() {
    let fixture_target = temp_export_root("native-dynamic-fixture-missing-descriptor-target");
    let package_root = temp_export_root("native-dynamic-fixture-missing-descriptor-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path =
        build_native_dynamic_fixture_with_features(&fixture_target, &["descriptor_export_missing"]);
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

    let report = NativePluginLoader.load_discovered_runtime(&package_root);

    assert!(report.loaded().is_empty());
    assert!(report.diagnostics().iter().any(|message| {
        message.contains("descriptor-probe")
            && message.contains("expected zircon_native_plugin_descriptor_v3")
            && message.contains("actual symbol not exported")
            && message.contains("native_dist_runtime_plugin_v3!")
    }));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_rejects_library_when_requested_runtime_entry_is_missing() {
    let fixture_target = temp_export_root("native-dynamic-fixture-missing-entry-target");
    let package_root = temp_export_root("native-dynamic-fixture-missing-entry-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_fixture_with_features(
        &fixture_target,
        &["runtime_entry_export_missing"],
    );
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

    let report = NativePluginLoader.load_discovered_runtime(&package_root);

    assert!(report.loaded().is_empty());
    assert!(report.diagnostics().iter().any(|message| {
        message.contains("runtime-entry")
            && message.contains("expected zircon_native_dynamic_fixture_runtime_entry_missing_v3")
            && message.contains("actual symbol not exported")
            && message.contains("same entry symbol")
    }));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_reports_structured_missing_required_capability() {
    let fixture_target = temp_export_root("native-dynamic-fixture-missing-capability-target");
    let package_root = temp_export_root("native-dynamic-fixture-missing-capability-package");
    let plugin_root = package_root.join("native_dynamic_fixture");
    let native_root = plugin_root.join("native");
    fs::create_dir_all(&native_root).unwrap();

    let library_path = build_native_dynamic_fixture_with_features(
        &fixture_target,
        &["required_capability_missing"],
    );
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

    let report = NativePluginLoader.load_discovered_runtime(&package_root);

    assert!(report.loaded().is_empty());
    assert!(report.diagnostics().iter().any(|message| {
        message.contains("runtime-entry capability negotiation failed")
            && message.contains("missing_required=[\"runtime.plugin.native_dynamic_fixture\"]")
            && message.contains("denied=[]")
            && message.contains("native v3 entry missing negotiated host ABI table")
    }));

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_loader_fixture_can_import_data_asset_through_native_importer_handler() {
    let fixture_target = temp_export_root("native-dynamic-fixture-import-target");
    let package_root = temp_export_root("native-dynamic-fixture-import-package");
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

    let mut report = NativePluginLoader.load_discovered_runtime(&package_root);
    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    let plugin = Arc::new(
        report
            .into_loaded()
            .pop()
            .expect("fixture native plugin loaded"),
    );
    let descriptor = AssetImporterDescriptor::new(
        "native_dynamic_fixture.data_json",
        "native_dynamic_fixture",
        AssetKind::Data,
        1,
    )
    .with_source_extensions(["nativejson"]);
    let importer = NativeAssetImporterHandler::new(descriptor, plugin);
    let context = AssetImportContext::new(
        "weather.nativejson".into(),
        crate::asset::AssetUri::parse("res://assets/weather.nativejson").unwrap(),
        br#"{"temperature":21,"condition":"clear"}"#.to_vec(),
        Default::default(),
    );

    let outcome = importer.import(&context).expect("native data import");
    let entry = outcome.root_entry().expect("native root asset entry");

    match &entry.asset {
        ImportedAsset::Data(data) => {
            assert_eq!(data.format, DataAssetFormat::Json);
            assert_eq!(data.text, r#"{"temperature":21,"condition":"clear"}"#);
            assert_eq!(data.canonical_json["temperature"], 21);
            assert_eq!(data.canonical_json["condition"], "clear");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert!(entry
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("weather.nativejson")));
    assert_eq!(
        entry.migration_report,
        Some(AssetSchemaMigrationReport {
            source_schema_version: Some(1),
            target_schema_version: 2,
            summary: "native fixture migrated weather.nativejson".to_string(),
        })
    );

    let _ = fs::remove_dir_all(fixture_target);
    let _ = fs::remove_dir_all(package_root);
}
