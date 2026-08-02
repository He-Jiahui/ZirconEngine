use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cargo_zircon::plugin::check::check_plugin_workspace;
use cargo_zircon::plugin::scaffold::{scaffold_plugin, NewPluginOptions, PluginKind};
use cargo_zircon::plugin::validate::{validate_native_artifact, validate_plugin_manifest};

#[test]
fn validate_reports_typed_manifest_diagnostics_with_repair_hints() {
    let diagnostics = validate_plugin_manifest(
        r#"id = "Demo Probe"
version = "0.1"
sdk_api_version = "0.2.0"
display_name = ""
category = "runtime"
description = "Demo"
supported_targets = ["desktop"]
supported_platforms = ["windows"]
capabilities = []
maturity = "stable"
default_packaging = ["native_dynamic"]

[distribution]
forms = ["dist"]
abi_version = 3
dist_crate = "zircon_plugin_demo_probe_dist"

[[modules]]
name = "demo_probe.runtime"
kind = "runtime"
crate_name = "zircon_plugin_demo_probe_runtime"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.demo_probe"]

[[modules]]
name = "demo_probe.editor"
kind = "editor"
crate_name = "zircon_plugin_demo_probe_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.demo_probe"]
"#,
        None,
    );

    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"plugin.id.invalid"));
    assert!(codes.contains(&"plugin.version.invalid"));
    assert!(codes.contains(&"plugin.display_name.empty"));
    assert!(codes.contains(&"plugin.target.unknown"));
    assert!(codes.contains(&"plugin.capabilities.empty"));
    assert!(codes.contains(&"plugin.distribution.runtime_entry.missing"));
    assert!(codes.contains(&"plugin.distribution.editor_entry.missing"));
    assert!(diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.hint.trim().is_empty()));
}

#[test]
fn validate_rejects_non_string_arrays_and_undeclared_module_capabilities() {
    let diagnostics = validate_plugin_manifest(
        r#"id = "demo_probe"
version = "0.1.0"
sdk_api_version = "0.2.0"
display_name = "Demo Probe"
category = "runtime"
description = "Demo"
supported_targets = ["client_runtime", 7]
supported_platforms = ["windows"]
capabilities = ["runtime.plugin.demo_probe", 9]
maturity = "experimental"
default_packaging = ["source_template"]

[[modules]]
name = "demo_probe.runtime"
kind = "unknown"
crate_name = ""
target_modes = ["server_runtime", false]
capabilities = ["runtime.plugin.not_declared"]

[[modules]]
name = "demo_probe.editor"
kind = "editor"
crate_name = "zircon_plugin_demo_probe_editor"
target_modes = ["editor_host"]
capabilities = ["runtime.plugin.wrong_kind"]
"#,
        None,
    );

    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"plugin.target.invalid_type"));
    assert!(codes.contains(&"plugin.capability.invalid_type"));
    assert!(codes.contains(&"plugin.module.kind.unknown"));
    assert!(codes.contains(&"plugin.module.crate_name.empty"));
    assert!(codes.contains(&"plugin.module.target_mode.undeclared"));
    assert!(codes.contains(&"plugin.module.target_mode.invalid_type"));
    assert!(codes.contains(&"plugin.module.capability.invalid_prefix"));
}

#[test]
fn validate_rejects_module_identity_and_editor_target_drift() {
    let diagnostics = validate_plugin_manifest(
        r#"id = "demo_probe"
version = "0.1.0"
sdk_api_version = "0.2.0"
display_name = "Demo Probe"
category = "runtime"
description = "Demo"
supported_targets = ["client_runtime", "editor_host"]
supported_platforms = ["windows"]
capabilities = ["runtime.plugin.demo_probe"]
maturity = "experimental"
default_packaging = ["source_template"]

[[modules]]
name = "foreign.runtime"
kind = "runtime"
crate_name = "bad-Crate"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.demo_probe"]

[[modules]]
name = "demo_probe.worker"
kind = "editor"
crate_name = "zircon_plugin_demo_probe_editor"
target_modes = ["client_runtime"]
capabilities = ["editor.extension.demo_probe"]

[[modules]]
name = "demo_probe.worker"
kind = "editor"
crate_name = "zircon_plugin_demo_probe_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.demo_probe"]
"#,
        None,
    );

    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"plugin.module.name.outside_namespace"));
    assert!(codes.contains(&"plugin.module.crate_name.invalid"));
    assert!(codes.contains(&"plugin.module.name.invalid_for_kind"));
    assert!(codes.contains(&"plugin.module.target_mode.invalid_for_kind"));
    assert!(codes.contains(&"plugin.module.name.duplicate"));
}

#[test]
fn validate_rejects_a_stale_distribution_without_native_packaging() {
    let diagnostics = validate_plugin_manifest(
        r#"id = "demo_probe"
version = "0.1.0"
sdk_api_version = "0.2.0"
display_name = "Demo Probe"
category = "runtime"
description = "Demo"
supported_targets = ["client_runtime"]
supported_platforms = ["windows"]
capabilities = ["runtime.plugin.demo_probe"]
maturity = "experimental"
default_packaging = ["source_template"]

[distribution]
dist_crate = "zircon_plugin_demo_probe_dist"

[[modules]]
name = "demo_probe.runtime"
kind = "runtime"
crate_name = "zircon_plugin_demo_probe_runtime"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.demo_probe"]
"#,
        None,
    );

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plugin.distribution.unexpected"));
}

#[test]
fn native_artifact_validation_reports_a_missing_build_with_a_repair_hint() {
    let diagnostics = validate_native_artifact(
        r#"id = "demo_probe"
[distribution]
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
"#,
        Path::new("missing-demo-probe.dll"),
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "plugin.native_artifact.missing");
    assert!(diagnostics[0].hint.contains("--artifact"));
}

#[test]
fn validate_accepts_an_inline_native_distribution_crate() {
    let package = unique_test_directory();
    write(
        &package.join("native/Cargo.toml"),
        "[package]\nname = \"zircon_plugin_inline_probe_native\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    let diagnostics = validate_plugin_manifest(
        r#"id = "inline_probe"
version = "0.1.0"
sdk_api_version = "0.2.0"
display_name = "Inline Probe"
category = "sdk"
description = "Inline native validation fixture."
supported_targets = ["client_runtime"]
supported_platforms = ["windows"]
capabilities = ["runtime.plugin.inline_probe"]
maturity = "experimental"
default_packaging = ["native_dynamic"]

[distribution]
forms = ["dist"]
default_packaging = ["native_dynamic"]
abi_version = 3
engine_compat = ">=0.1, <0.2"
dist_crate = "zircon_plugin_inline_probe_native"
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
runtime_entry = "zircon_plugin_inline_probe_runtime_entry_v3"

[[modules]]
name = "inline_probe.runtime"
kind = "runtime"
crate_name = "zircon_plugin_inline_probe_native"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.inline_probe"]
"#,
        Some(&package),
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "plugin.distribution.crate.missing"),
        "{diagnostics:#?}"
    );
    fs::remove_dir_all(package).unwrap();
}

#[test]
fn new_system_plugin_generates_native_skeleton_and_catalog_wiring() {
    let repo = create_repository_fixture();
    let report = scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "demo_probe",
        kind: PluginKind::System,
        native: true,
    })
    .unwrap();

    assert_eq!(report.package_id, "demo_probe");
    let package = repo.join("zircon_plugins/demo_probe");
    for relative in [
        "plugin.toml",
        "runtime/Cargo.toml",
        "runtime/src/capability.rs",
        "runtime/src/lib.rs",
        "runtime/src/plugin.rs",
        "dist/Cargo.toml",
        "dist/src/lib.rs",
    ] {
        assert!(package.join(relative).is_file(), "missing {relative}");
    }
    let workspace = fs::read_to_string(repo.join("zircon_plugins/Cargo.toml")).unwrap();
    assert!(workspace.contains("# keep-workspace-comment"));
    assert!(workspace.contains("demo_probe/runtime"));
    assert!(workspace.contains("demo_probe/dist"));
    let catalog =
        fs::read_to_string(repo.join("zircon_plugins/first_party_runtime_catalog/Cargo.toml"))
            .unwrap();
    assert!(catalog.contains("# keep-runtime-catalog-comment"));
    assert!(catalog.contains("demo-probe-runtime-plugin"));
    assert!(catalog.contains("dep:zircon_plugin_demo_probe_runtime"));
    assert!(catalog.contains("../demo_probe/runtime"));
    let registry =
        fs::read_to_string(repo.join("zircon_plugins/first_party_runtime_catalog/src/lib.rs"))
            .unwrap();
    assert!(registry.contains("zircon_plugin_demo_probe_runtime::plugin_registration()"));
    assert!(registry.contains("../../demo_probe/plugin.toml"));
    let runtime_plugin = fs::read_to_string(package.join("runtime/src/plugin.rs")).unwrap();
    assert!(runtime_plugin.contains("pub struct DemoProbeRuntimePlugin"));
    assert!(!runtime_plugin.contains("DEMO_PROBERuntimePlugin"));
    assert!(runtime_plugin.contains(".with_native_module(native_dist_module_manifest())"));
    assert!(runtime_plugin.contains("NATIVE_RUNTIME_ENTRY.name().to_string()"));
    assert!(runtime_plugin.contains(".with_target_modes(DEMO_PROBE_DECLARATION.target_modes())"));
    assert!(runtime_plugin.contains("DIST_ENGINE_COMPAT: &str = \">=0.1, <0.2\""));
    assert!(runtime_plugin.contains("manifest.version = env!(\"CARGO_PKG_VERSION\")"));
    assert!(runtime_plugin.contains("zircon_plugin_sdk::SDK_API_VERSION"));
    let app = fs::read_to_string(repo.join("zircon_app/Cargo.toml")).unwrap();
    assert!(app.contains("# keep-app-comment"));
    assert!(app.contains("first-party-demo-probe-runtime-plugin"));
    assert!(app.contains("zircon_first_party_runtime_catalog/demo-probe-runtime-plugin"));

    let second = scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "demo_probe",
        kind: PluginKind::System,
        native: true,
    });
    assert!(second.is_err(), "scaffolding must not overwrite a package");

    fs::remove_dir_all(repo).unwrap();
}

#[test]
fn new_editor_plugin_generates_editor_catalog_and_app_wiring() {
    let repo = create_repository_fixture();
    scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "scene_notes",
        kind: PluginKind::Editor,
        native: true,
    })
    .unwrap();

    let package = repo.join("zircon_plugins/scene_notes");
    assert!(package.join("editor/src/plugin.rs").is_file());
    assert!(package.join("dist/src/lib.rs").is_file());
    let editor_plugin = fs::read_to_string(package.join("editor/src/plugin.rs")).unwrap();
    assert!(editor_plugin.contains("let declaration = SCENE_NOTES_DECLARATION;"));
    assert!(editor_plugin.contains("declaration.display_name()"));
    assert!(editor_plugin.contains("declaration.default_packaging()"));
    assert!(editor_plugin.contains(".with_native_module(native_dist_module_manifest())"));
    assert!(editor_plugin.contains("NATIVE_EDITOR_ENTRY.name().to_string()"));
    assert!(editor_plugin.contains("manifest.version = env!(\"CARGO_PKG_VERSION\")"));
    assert!(editor_plugin.contains("zircon_plugin_sdk::SDK_API_VERSION"));
    let editor_catalog =
        fs::read_to_string(repo.join("zircon_plugins/first_party_editor_catalog/Cargo.toml"))
            .unwrap();
    assert!(editor_catalog.contains("scene-notes-editor-plugin"));
    assert!(editor_catalog.contains("dep:zircon_plugin_scene_notes_editor"));
    assert!(editor_catalog.contains("../scene_notes/editor"));
    let editor_registry =
        fs::read_to_string(repo.join("zircon_plugins/first_party_editor_catalog/src/catalog.rs"))
            .unwrap();
    assert!(editor_registry.contains("zircon_plugin_scene_notes_editor::plugin_registration()"));
    let runtime_registry =
        fs::read_to_string(repo.join("zircon_plugins/first_party_runtime_catalog/src/lib.rs"))
            .unwrap();
    assert!(runtime_registry.contains("../../scene_notes/plugin.toml"));
    let app = fs::read_to_string(repo.join("zircon_app/Cargo.toml")).unwrap();
    assert!(app.contains("first-party-scene-notes-editor-plugin"));
    assert!(app.contains("zircon_first_party_editor_catalog/scene-notes-editor-plugin"));

    fs::remove_dir_all(repo).unwrap();
}

#[test]
fn new_importer_plugin_generates_a_registered_importer_skeleton() {
    let repo = create_repository_fixture();
    scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "demo_importer",
        kind: PluginKind::Importer,
        native: true,
    })
    .unwrap();

    let package = repo.join("zircon_plugins/demo_importer");
    let manifest = fs::read_to_string(package.join("plugin.toml")).unwrap();
    assert!(manifest.contains("supported_targets = [\"client_runtime\", \"editor_host\"]"));
    assert!(manifest.contains("[[asset_importers]]"));
    assert!(manifest.contains("id = \"demo_importer.demo_importer\""));
    assert!(manifest.contains("source_extensions = [\"demo_importer\"]"));
    assert!(package.join("dist/src/lib.rs").is_file());

    let declaration = fs::read_to_string(package.join("runtime/src/capability.rs")).unwrap();
    assert!(declaration.contains("point: \"runtime.asset.importer.data\""));
    assert!(declaration.contains("contribution: \"plugin.demo_importer.runtime\""));

    let plugin = fs::read_to_string(package.join("runtime/src/plugin.rs")).unwrap();
    assert!(plugin.contains("AssetImporterDescriptor::new"));
    assert!(plugin.contains("AssetKind::Data"));
    assert!(plugin.contains("register_asset_importer_descriptor"));
    assert!(plugin.contains("with_asset_importer(asset_importer_descriptor())"));
    assert!(plugin.contains(".with_native_module(native_dist_module_manifest())"));
    assert!(plugin.contains("NATIVE_RUNTIME_ENTRY.name().to_string()"));
    assert!(plugin.contains("manifest.version = env!(\"CARGO_PKG_VERSION\")"));
    assert!(plugin.contains("zircon_plugin_sdk::SDK_API_VERSION"));

    fs::remove_dir_all(repo).unwrap();
}

#[test]
fn check_accepts_scaffolded_package_and_detects_workspace_member_drift() {
    let repo = create_repository_fixture();
    scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "demo_probe",
        kind: PluginKind::System,
        native: false,
    })
    .unwrap();

    let clean = check_plugin_workspace(&repo).unwrap();
    assert!(clean.diagnostics.is_empty(), "{:?}", clean.diagnostics);

    let workspace_path = repo.join("zircon_plugins/Cargo.toml");
    let mut workspace: toml::Value = fs::read_to_string(&workspace_path)
        .unwrap()
        .parse()
        .unwrap();
    workspace["workspace"]["members"]
        .as_array_mut()
        .unwrap()
        .retain(|member| member.as_str() != Some("demo_probe/runtime"));
    fs::write(&workspace_path, toml::to_string_pretty(&workspace).unwrap()).unwrap();
    let drift = check_plugin_workspace(&repo).unwrap();
    assert!(drift
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plugin.workspace.member.missing"));

    fs::remove_dir_all(repo).unwrap();
}

#[test]
fn check_detects_catalog_registration_and_app_feature_drift() {
    let repo = create_repository_fixture();
    scaffold_plugin(&NewPluginOptions {
        repo_root: &repo,
        id: "demo_probe",
        kind: PluginKind::System,
        native: false,
    })
    .unwrap();

    let registry_path = repo.join("zircon_plugins/first_party_runtime_catalog/src/lib.rs");
    let registry = fs::read_to_string(&registry_path).unwrap().replace(
        "zircon_plugin_demo_probe_runtime::plugin_registration()",
        "missing_demo_probe_registration()",
    );
    fs::write(&registry_path, registry).unwrap();
    let app_path = repo.join("zircon_app/Cargo.toml");
    let mut app: toml::Value = fs::read_to_string(&app_path).unwrap().parse().unwrap();
    app["features"]
        .as_table_mut()
        .unwrap()
        .remove("first-party-demo-probe-runtime-plugin");
    fs::write(&app_path, toml::to_string_pretty(&app).unwrap()).unwrap();

    let drift = check_plugin_workspace(&repo).unwrap();
    let codes = drift
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"plugin.catalog.registration.missing"));
    assert!(codes.contains(&"plugin.app.catalog_feature.missing"));

    fs::remove_dir_all(repo).unwrap();
}

fn create_repository_fixture() -> PathBuf {
    let root = unique_test_directory();
    write(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = []

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
"#,
    );
    write(
        &root.join("zircon_plugins/Cargo.toml"),
        r#"[workspace]
# keep-workspace-comment
members = [
    "plugin_sdk",
    "first_party_runtime_catalog",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
zircon_plugin_sdk = { path = "plugin_sdk", default-features = false }
zircon_runtime = { path = "../zircon_runtime", default-features = false }
"#,
    );
    write(
        &root.join("zircon_plugins/plugin_sdk/src/manifest/defaults.rs"),
        "pub const SDK_API_VERSION: &str = \"0.2.0\";\n",
    );
    write(
        &root.join("zircon_plugins/first_party_runtime_catalog/Cargo.toml"),
        r#"[package]
name = "zircon_first_party_runtime_catalog"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
# keep-runtime-catalog-comment
default = []
base-runtime-plugins = []

[dependencies]
zircon_runtime = { workspace = true }
"#,
    );
    write(
        &root.join("zircon_plugins/first_party_runtime_catalog/src/lib.rs"),
        r#"pub fn first_party_registration_for_runtime_plugin(
    id: &str,
) -> Option<()> {
    // @cargo-zircon:runtime-registration-begin
    // @cargo-zircon:runtime-registration-end
    let _ = id;
    None
}

#[cfg(test)]
const STATIC_PLUGIN_MANIFESTS: &[(&str, &str)] = &[
    // @cargo-zircon:static-manifest-begin
    // @cargo-zircon:static-manifest-end
];
"#,
    );
    write(
        &root.join("zircon_plugins/first_party_editor_catalog/Cargo.toml"),
        r#"[package]
name = "zircon_first_party_editor_catalog"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
# keep-app-comment
default = []

[dependencies]
zircon_editor = { path = "../../zircon_editor" }
zircon_runtime = { workspace = true }
"#,
    );
    write(
        &root.join("zircon_plugins/first_party_editor_catalog/src/catalog.rs"),
        r#"pub fn first_party_registration_for_editor_plugin(
    _plugin_id: RuntimePluginId,
) -> Option<EditorPluginRegistrationReport> {
    // @cargo-zircon:editor-registration-begin
    // @cargo-zircon:editor-registration-end
    None
}
"#,
    );
    write(
        &root.join("zircon_app/Cargo.toml"),
        r#"[package]
name = "zircon_app"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
default = []

[dependencies]
zircon_first_party_editor_catalog = { path = "../zircon_plugins/first_party_editor_catalog", optional = true }
zircon_first_party_runtime_catalog = { path = "../zircon_plugins/first_party_runtime_catalog", optional = true }
"#,
    );
    root
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn unique_test_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("cargo-zircon-plugin-commands-{nonce}"))
}
