use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cargo_zircon::plugin::manifest_sync::{
    discover_manifest_declarations, synchronize_manifest, synchronize_manifest_file,
    PluginDeclarationProjection, SyncMode, SyncOutcome, GENERATED_MANIFEST_HEADER,
};

const DECLARATION_SOURCE: &str = r#"
zircon_plugin_sdk::declare_plugin! {
    pub DEMO_DECLARATION {
        id: PLUGIN_ID = "demo_probe",
        display_name: "Demo Probe",
        category: runtime,
        module: MODULE_NAME = "demo_probe.runtime",
        crate_name: RUNTIME_CRATE_NAME = "zircon_plugin_demo_probe_runtime",
        module_description: "Demo runtime services",
        targets: [client_runtime, editor_host],
        platforms: [windows, linux],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.demo_probe" => runtime_registration,
            AUTHORING_CAPABILITY = "editor.extension.demo_probe" => editor_registration,
        ],
        maturity: experimental,
        packaging: [source_template, native_dynamic],
        native_projection: {
            plugin_id: NATIVE_PLUGIN_ID,
            requested_capabilities: NATIVE_REQUESTED_CAPABILITIES,
            runtime: {
                entry: NATIVE_RUNTIME_ENTRY = "zircon_plugin_demo_probe_runtime_entry_v3",
                registration_manifest: NATIVE_RUNTIME_REGISTRATION_MANIFEST,
                modules: [{ name: "runtime", kind: "runtime" }],
                systems: [],
                events: [],
                extensions: [],
            },
        },
    }
}
"#;

const STALE_MANIFEST: &str = r#"# old generated header
id = "stale"
version = "9.9.9"
sdk_api_version = "0.1.0"
display_name = "Stale"
category = "sdk"
description = "Package description stays manifest-specific."
supported_targets = ["server_runtime"]
supported_platforms = ["macos"]
capabilities = ["runtime.plugin.stale"]
maturity = "stable"
default_packaging = ["library_embed"]

[distribution]
forms = ["dist"]
engine_compat = ">=0.1, <0.2"
runtime_entry = "stale_entry"

[[dependencies]]
id = "physics"
required = false
interfaces = ["physics.query.v1"]

[[modules]]
name = "stale.runtime"
kind = "runtime"
crate_name = "zircon_plugin_demo_probe_runtime"
target_modes = ["server_runtime"]
capabilities = ["runtime.plugin.stale"]
system_sets = ["demo.main"]

[[modules]]
name = "demo_probe.dist"
kind = "native"
crate_name = "zircon_plugin_demo_probe_dist"
target_modes = ["server_runtime"]
capabilities = ["runtime.plugin.stale"]

[[modules]]
name = "demo_probe.editor"
kind = "editor"
crate_name = "zircon_plugin_demo_probe_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.demo_probe"]
"#;

#[test]
fn parses_declare_plugin_as_the_manifest_authority() {
    let declaration = PluginDeclarationProjection::parse(DECLARATION_SOURCE).unwrap();

    assert_eq!(declaration.id(), "demo_probe");
    assert_eq!(declaration.display_name(), "Demo Probe");
    assert_eq!(declaration.category(), "runtime");
    assert_eq!(declaration.module_name(), "demo_probe.runtime");
    assert_eq!(declaration.crate_name(), "zircon_plugin_demo_probe_runtime");
    assert_eq!(declaration.targets(), ["client_runtime", "editor_host"]);
    assert_eq!(declaration.platforms(), ["windows", "linux"]);
    assert_eq!(
        declaration.capabilities(),
        ["runtime.plugin.demo_probe", "editor.extension.demo_probe"]
    );
    assert_eq!(
        declaration.runtime_capabilities(),
        ["runtime.plugin.demo_probe"]
    );
    assert_eq!(
        declaration.editor_capabilities(),
        ["editor.extension.demo_probe"]
    );
    assert_eq!(declaration.maturity(), "experimental");
    assert_eq!(
        declaration.packaging(),
        ["source_template", "native_dynamic"]
    );
    assert_eq!(
        declaration.runtime_entry(),
        Some("zircon_plugin_demo_probe_runtime_entry_v3")
    );
}

#[test]
fn synchronization_rewrites_owned_fields_and_preserves_extensions() {
    let declaration = PluginDeclarationProjection::parse(DECLARATION_SOURCE).unwrap();
    let synchronized =
        synchronize_manifest(STALE_MANIFEST, &declaration, "0.1.0", "0.2.0").unwrap();

    assert!(synchronized.starts_with(GENERATED_MANIFEST_HEADER));
    let manifest: toml::Value = synchronized.parse().unwrap();
    let root = manifest.as_table().unwrap();
    assert_eq!(root["id"].as_str(), Some("demo_probe"));
    assert_eq!(root["version"].as_str(), Some("0.1.0"));
    assert_eq!(root["sdk_api_version"].as_str(), Some("0.2.0"));
    assert_eq!(root["display_name"].as_str(), Some("Demo Probe"));
    assert_eq!(
        root["description"].as_str(),
        Some("Package description stays manifest-specific.")
    );
    assert_eq!(root["dependencies"].as_array().unwrap().len(), 1);

    let distribution = root["distribution"].as_table().unwrap();
    assert_eq!(
        distribution["runtime_entry"].as_str(),
        Some("zircon_plugin_demo_probe_runtime_entry_v3")
    );
    assert_eq!(distribution["forms"].as_array().unwrap().len(), 1);
    assert_eq!(distribution["abi_version"].as_integer(), Some(3));
    assert_eq!(
        distribution["descriptor_symbol"].as_str(),
        Some("zircon_native_plugin_descriptor_v3")
    );
    assert_eq!(
        distribution["dist_crate"].as_str(),
        Some("zircon_plugin_demo_probe_dist")
    );
    assert_eq!(distribution["engine_compat"].as_str(), Some(">=0.1, <0.2"));

    let modules = root["modules"].as_array().unwrap();
    let runtime = modules[0].as_table().unwrap();
    assert_eq!(runtime["name"].as_str(), Some("demo_probe.runtime"));
    assert_eq!(
        runtime["capabilities"].as_array().unwrap(),
        &[toml::Value::String("runtime.plugin.demo_probe".to_string())]
    );
    assert_eq!(runtime["system_sets"].as_array().unwrap().len(), 1);
    let native = modules[1].as_table().unwrap();
    assert_eq!(native["target_modes"], root["supported_targets"]);
    assert_eq!(native["capabilities"], runtime["capabilities"]);
    let editor = modules[2].as_table().unwrap();
    assert_eq!(editor["name"].as_str(), Some("demo_probe.editor"));
}

#[test]
fn synchronization_converges_native_distribution_modules_from_the_declaration() {
    let declaration = PluginDeclarationProjection::parse(DECLARATION_SOURCE).unwrap();
    let mut missing_dist: toml::Value = STALE_MANIFEST.parse().unwrap();
    missing_dist["distribution"]["dist_crate"] =
        toml::Value::String("zircon_plugin_obsolete_dist".to_string());
    missing_dist["modules"]
        .as_array_mut()
        .unwrap()
        .retain(|module| module.get("kind").and_then(toml::Value::as_str) != Some("native"));

    let synchronized = synchronize_manifest(
        &toml::to_string(&missing_dist).unwrap(),
        &declaration,
        "0.1.0",
        "0.2.0",
    )
    .unwrap();
    let manifest: toml::Value = synchronized.parse().unwrap();
    assert_eq!(
        manifest["distribution"]["dist_crate"].as_str(),
        Some("zircon_plugin_demo_probe_dist")
    );
    let native_modules = manifest["modules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|module| module.get("kind").and_then(toml::Value::as_str) == Some("native"))
        .collect::<Vec<_>>();
    assert_eq!(native_modules.len(), 1);
    assert_eq!(native_modules[0]["name"].as_str(), Some("demo_probe.dist"));
    assert_eq!(
        native_modules[0]["crate_name"].as_str(),
        Some("zircon_plugin_demo_probe_dist")
    );

    let mut duplicate_dist: toml::Value = STALE_MANIFEST.parse().unwrap();
    let duplicate = duplicate_dist["modules"].as_array().unwrap()[1].clone();
    duplicate_dist["modules"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let synchronized = synchronize_manifest(
        &toml::to_string(&duplicate_dist).unwrap(),
        &declaration,
        "0.1.0",
        "0.2.0",
    )
    .unwrap();
    let manifest: toml::Value = synchronized.parse().unwrap();
    assert_eq!(
        manifest["modules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|module| module.get("kind").and_then(toml::Value::as_str) == Some("native"))
            .count(),
        1
    );

    let library_only = PluginDeclarationProjection::parse(&DECLARATION_SOURCE.replace(
        "packaging: [source_template, native_dynamic]",
        "packaging: [source_template, library_embed]",
    ))
    .unwrap();
    let synchronized =
        synchronize_manifest(STALE_MANIFEST, &library_only, "0.1.0", "0.2.0").unwrap();
    let manifest: toml::Value = synchronized.parse().unwrap();
    assert!(manifest.get("distribution").is_none());
    assert!(manifest["modules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|module| { module.get("kind").and_then(toml::Value::as_str) != Some("native") }));

    let inline_native = PluginDeclarationProjection::parse(&DECLARATION_SOURCE.replace(
        "zircon_plugin_demo_probe_runtime",
        "zircon_plugin_demo_probe_native",
    ))
    .unwrap();
    let synchronized =
        synchronize_manifest(STALE_MANIFEST, &inline_native, "0.1.0", "0.2.0").unwrap();
    let manifest: toml::Value = synchronized.parse().unwrap();
    assert_eq!(
        manifest["distribution"]["dist_crate"].as_str(),
        Some("zircon_plugin_demo_probe_native")
    );
    assert!(manifest["modules"]
        .as_array()
        .unwrap()
        .iter()
        .all(|module| { module.get("kind").and_then(toml::Value::as_str) != Some("native") }));
}

#[test]
fn check_mode_reports_drift_without_writing_and_write_mode_is_idempotent() {
    let directory = unique_test_directory();
    fs::create_dir_all(&directory).unwrap();
    let declaration_path = directory.join("capability.rs");
    let manifest_path = directory.join("plugin.toml");
    fs::write(&declaration_path, DECLARATION_SOURCE).unwrap();
    fs::write(&manifest_path, STALE_MANIFEST).unwrap();

    let drift = synchronize_manifest_file(
        &declaration_path,
        &manifest_path,
        "0.1.0",
        "0.2.0",
        SyncMode::Check,
    )
    .unwrap();
    assert_eq!(drift, SyncOutcome::Drift);
    assert_eq!(fs::read_to_string(&manifest_path).unwrap(), STALE_MANIFEST);

    let updated = synchronize_manifest_file(
        &declaration_path,
        &manifest_path,
        "0.1.0",
        "0.2.0",
        SyncMode::Write,
    )
    .unwrap();
    assert_eq!(updated, SyncOutcome::Updated);
    let unchanged = synchronize_manifest_file(
        &declaration_path,
        &manifest_path,
        "0.1.0",
        "0.2.0",
        SyncMode::Check,
    )
    .unwrap();
    assert_eq!(unchanged, SyncOutcome::Unchanged);

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn discovery_selects_package_declarations_and_skips_nested_feature_providers() {
    let directory = unique_test_directory();
    let package = directory.join("zircon_plugins/package_folder_is_not_identity");
    let nested_feature = package.join("features/optional/runtime/src");
    fs::create_dir_all(package.join("runtime/src")).unwrap();
    fs::create_dir_all(&nested_feature).unwrap();
    fs::write(package.join("plugin.toml"), STALE_MANIFEST).unwrap();
    fs::write(
        package.join("runtime/src/capability.rs"),
        DECLARATION_SOURCE,
    )
    .unwrap();
    fs::write(nested_feature.join("capability.rs"), DECLARATION_SOURCE).unwrap();
    let inline_package = directory.join("zircon_plugins/inline_package_folder");
    fs::create_dir_all(inline_package.join("native/src")).unwrap();
    fs::write(inline_package.join("plugin.toml"), STALE_MANIFEST).unwrap();
    fs::write(
        inline_package.join("native/src/lib.rs"),
        DECLARATION_SOURCE
            .replace("demo_probe", "inline_native")
            .replace("Demo Probe", "Inline Native"),
    )
    .unwrap();

    let declarations = discover_manifest_declarations(&directory).unwrap();

    assert_eq!(declarations.len(), 2);
    assert!(declarations.iter().any(|declaration| {
        declaration.package_id() == "demo_probe"
            && declaration.manifest_path() == package.join("plugin.toml")
    }));
    assert!(declarations.iter().any(|declaration| {
        declaration.package_id() == "inline_native"
            && declaration.manifest_path() == inline_package.join("plugin.toml")
            && declaration.declaration_path() == inline_package.join("native/src/lib.rs")
    }));

    fs::remove_dir_all(directory).unwrap();
}

fn unique_test_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("cargo-zircon-manifest-sync-{nonce}"))
}
