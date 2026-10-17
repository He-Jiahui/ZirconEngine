use super::*;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::RuntimeProfileId;

#[test]
fn native_aware_catalog_enables_external_feature_extension_provider() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_native_feature_extension_config");
    let project_root = unique_temp_dir("zircon_editor_native_feature_extension_project");
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    let owner_root = project_root.join("zircon_plugins/native_owner");
    let provider_root = project_root.join("zircon_plugins/native_owner_extension");
    std::fs::create_dir_all(&owner_root).unwrap();
    std::fs::create_dir_all(&provider_root).unwrap();
    std::fs::write(
        owner_root.join("plugin.toml"),
        r#"
id = "native_owner"
version = "0.1.0"
display_name = "Native Owner"
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_owner.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_owner_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.plugin.native_owner"]
"#,
    )
    .unwrap();
    std::fs::write(
        provider_root.join("plugin.toml"),
        r#"
id = "native_owner_extension"
version = "0.1.0"
package_kind = "feature_extension"
display_name = "Native Owner Timeline Provider"

[[feature_extensions]]
id = "native_owner.timeline"
display_name = "Native Owner Timeline"
owner_plugin_id = "native_owner"
capabilities = ["runtime.feature.native_owner.timeline"]
default_packaging = ["native_dynamic"]

[[feature_extensions.dependencies]]
plugin_id = "native_owner"
capability = "runtime.plugin.native_owner"
primary = true

[[feature_extensions.modules]]
name = "native_owner.timeline.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_owner_timeline_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.feature.native_owner.timeline"]
"#,
    )
    .unwrap();
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Native Feature Extension Test",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let dependencies = manager
        .enable_native_aware_project_plugin_feature_dependencies(
            &project_root,
            &mut manifest,
            "native_owner",
            "native_owner.timeline",
        )
        .expect("native feature-extension provider must enter the native-aware catalog");

    assert_eq!(
        dependencies.enabled_dependency_plugins,
        vec![
            "native_owner".to_string(),
            "native_owner_extension".to_string()
        ]
    );
    let provider = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "native_owner_extension")
        .expect("external native feature provider selection");
    assert!(provider.enabled);
    assert_eq!(
        provider.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(
        provider.runtime_crate.as_deref(),
        Some("zircon_plugin_native_owner_timeline_runtime")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn native_plugin_status_uses_manifest_when_library_is_missing() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_native_plugin_status");
    let project_root = unique_temp_dir("zircon_editor_native_plugin_project");
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    std::fs::create_dir_all(project_root.join("zircon_plugins/native_tool")).unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/native_tool/plugin.toml"),
        r#"
id = "native_tool"
version = "0.1.0"
display_name = "Native Tool"
description = "Native plugin status fixture."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_tool_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.plugin.native_tool"]

[[modules]]
name = "native_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_native_tool_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.native_tool"]

[[optional_features]]
id = "native_tool.timeline_bridge"
display_name = "Native Timeline Bridge"
owner_plugin_id = "native_tool"
capabilities = ["runtime.feature.native_tool.timeline_bridge"]
default_packaging = ["native_dynamic"]
enabled_by_default = false

[[optional_features.dependencies]]
plugin_id = "native_tool"
capability = "runtime.plugin.native_tool"
primary = true

[[optional_features.modules]]
name = "native_tool.timeline_bridge.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_tool_timeline_bridge_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.feature.native_tool.timeline_bridge"]
"#,
    )
    .unwrap();
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Native Tool Test",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let native_report = zircon_runtime::plugin::native::discovery::discover_native_plugins(
        manager.plugin_directory(&project_root),
    );
    let status = manager.native_plugin_status_report_from_load_report(&manifest, &native_report);
    let native = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "native_tool")
        .expect("native plugin appears from plugin.toml");
    assert!(!native.enabled);
    assert_eq!(
        native.editor_capabilities,
        vec!["editor.extension.native_tool".to_string()]
    );
    assert_eq!(
        native.runtime_capabilities,
        vec!["runtime.plugin.native_tool".to_string()]
    );
    assert_eq!(native.target_modes, vec![RuntimeTargetMode::EditorHost]);
    assert_eq!(
        native.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(native.package_source, "native");
    assert_eq!(native.load_state, "manifest only");
    let feature = native
        .optional_features
        .iter()
        .find(|feature| feature.id == "native_tool.timeline_bridge")
        .expect("native optional feature should be projected from plugin.toml");
    assert!(!feature.enabled);
    assert!(!feature.available);
    assert_eq!(
        feature.runtime_crate.as_deref(),
        Some("zircon_plugin_native_tool_timeline_bridge_runtime")
    );
    assert!(feature.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "native_tool"
            && dependency.capability == "runtime.plugin.native_tool"
            && dependency.primary
            && !dependency.plugin_enabled
            && !dependency.capability_available
    }));
    assert!(!status
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));
    assert!(!native
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));
    let registrations = manager.native_editor_plugin_registration_reports(&project_root);
    let registration = registrations
        .iter()
        .find(|registration| registration.package_manifest.id == "native_tool")
        .expect("native editor registration report");
    assert_eq!(
        registration.capabilities,
        vec!["editor.extension.native_tool".to_string()]
    );
    assert!(registration
        .package_manifest
        .modules
        .iter()
        .all(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor));
    assert!(registration
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));

    let dependency_report = manager
        .enable_native_aware_project_plugin_feature_dependencies(
            &project_root,
            &mut manifest,
            "native_tool",
            "native_tool.timeline_bridge",
        )
        .expect("native optional feature dependencies should use native catalog");
    assert_eq!(
        dependency_report.enabled_dependency_plugins,
        vec!["native_tool".to_string()]
    );
    assert!(dependency_report
        .project_selection
        .features
        .iter()
        .any(|feature| {
            feature.id == "native_tool.timeline_bridge"
                && !feature.enabled
                && feature.runtime_crate.as_deref()
                    == Some("zircon_plugin_native_tool_timeline_bridge_runtime")
        }));
    let dependency_status = manager.published_plugin_status_report();

    let feature_report = manager
        .set_native_aware_project_plugin_feature_enabled(
            &project_root,
            &mut manifest,
            "native_tool",
            "native_tool.timeline_bridge",
            true,
        )
        .expect("native optional feature should enable after dependencies");
    assert!(feature_report.enabled);
    assert!(feature_report
        .project_selection
        .features
        .iter()
        .any(|feature| feature.id == "native_tool.timeline_bridge" && feature.enabled));
    let feature_status = manager.published_plugin_status_report();
    assert!(!std::sync::Arc::ptr_eq(&dependency_status, &feature_status));

    let enabled = manager
        .set_native_aware_project_plugin_enabled(&project_root, &mut manifest, "native_tool", true)
        .unwrap();
    assert!(enabled.enabled);
    assert!(enabled
        .capability_snapshot
        .is_enabled("editor.extension.native_tool"));

    let packaging = manager
        .set_native_aware_project_plugin_packaging(
            &project_root,
            &mut manifest,
            "native_tool",
            zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed,
        )
        .unwrap();
    assert_eq!(
        packaging.project_selection.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed
    );
    let target_modes = manager
        .set_native_aware_project_plugin_target_modes(
            &project_root,
            &mut manifest,
            "native_tool",
            [RuntimeTargetMode::ServerRuntime],
        )
        .unwrap();
    assert_eq!(
        target_modes.project_selection.target_modes,
        vec![RuntimeTargetMode::ServerRuntime]
    );

    let status = manager.published_plugin_status_report();
    let native_status = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "native_tool")
        .expect("native plugin remains visible");
    assert!(native_status.enabled);
    assert_eq!(
        native_status.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed
    );
    assert_eq!(
        native_status.target_modes,
        vec![RuntimeTargetMode::ServerRuntime]
    );
    manifest
        .plugins
        .selections
        .iter_mut()
        .find(|selection| selection.id == "native_tool")
        .expect("native project selection")
        .required = true;
    let error = manager
        .set_native_aware_project_plugin_enabled(&project_root, &mut manifest, "native_tool", false)
        .unwrap_err();
    assert!(error.contains("required plugin native_tool cannot be disabled"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn native_aware_completion_aggregates_native_module_target_modes() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_native_split_target_config");
    let project_root = unique_temp_dir("zircon_editor_native_split_target_project");
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    std::fs::create_dir_all(project_root.join("zircon_plugins/split_target_tool")).unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/split_target_tool/plugin.toml"),
        r#"
id = "split_target_tool"
version = "0.1.0"
display_name = "Split Target Tool"
default_packaging = ["native_dynamic"]

[[modules]]
name = "split_target_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_split_target_tool_runtime"
target_modes = ["client_runtime"]
capabilities = ["runtime.plugin.split_target_tool"]

[[modules]]
name = "split_target_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_split_target_tool_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.split_target_tool"]
"#,
    )
    .unwrap();
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Split Target Native Tool Test",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let completed = manager.complete_native_aware_project_plugin_manifest(&project_root, &manifest);
    let selection = completed
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "split_target_tool")
        .expect("split-target native package selection");
    assert_eq!(
        selection.target_modes,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        selection.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(
        selection.editor_crate.as_deref(),
        Some("zircon_plugin_split_target_tool_editor")
    );
    assert_eq!(
        selection.runtime_crate.as_deref(),
        Some("zircon_plugin_split_target_tool_runtime")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn native_dynamic_export_without_source_template_skips_cargo_and_writes_loader_manifest() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_native_dynamic_export_config");
    let project_root = unique_temp_dir("zircon_editor_native_dynamic_export_project");
    let output_root = unique_temp_dir("zircon_editor_native_dynamic_export_output");
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    std::fs::create_dir_all(project_root.join("zircon_plugins/native_tool")).unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/native_tool/plugin.toml"),
        r#"
id = "native_tool"
version = "0.1.0"
display_name = "Native Tool"
description = "Native plugin export fixture."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_tool_runtime"
target_modes = ["client_runtime"]
"#,
    )
    .unwrap();
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Native Dynamic Export",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.plugins.selections.push(
        zircon_runtime::core::framework::project::ProjectPluginSelection {
            id: "native_tool".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging:
                zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
            runtime_crate: Some("zircon_plugin_native_tool_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        },
    );
    manifest.export_profiles = vec![
        zircon_runtime::core::framework::project::ExportProfile::new(
            "client-native",
            RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::project::ExportTargetPlatform::Windows,
            RuntimeProfileId::Minimal,
        )
        .with_strategies([
            zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
        ]),
    ];

    let editor_registrations = manager.native_editor_plugin_registration_reports(&project_root);
    assert!(
        editor_registrations
            .iter()
            .all(|registration| registration.package_manifest.id != "native_tool"),
        "runtime-only native packages must not enter editor extension registration"
    );

    let report = manager
        .execute_native_aware_export_build(&project_root, &output_root, &manifest, "client-native")
        .unwrap();

    assert!(report.fatal_diagnostics.is_empty());
    assert_eq!(report.plan.native_dynamic_packages, vec!["native_tool"]);
    assert!(!report.invoked_cargo);
    assert!(report.cargo_invocation.is_none());
    assert!(report.native_cargo_invocations.is_empty());
    assert!(report
        .generated_files
        .iter()
        .any(|path| path.ends_with("plugins/native_plugins.toml")));
    assert!(output_root.join("plugins/native_tool/plugin.toml").exists());
    assert!(!output_root.join(".native-dynamic-staging").exists());
    assert!(!output_root.join(".native-dynamic-build").exists());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("cargo build skipped")));
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));
    let diagnostics = std::fs::read_to_string(output_root.join("export-diagnostics.txt")).unwrap();
    assert!(diagnostics.contains("cargo build skipped"));
    assert!(diagnostics.contains("library is missing"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(project_root);
    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn export_build_report_includes_plan_diagnostics_when_no_files_are_generated() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_export_plan_diagnostics_config");
    let output_root = unique_temp_dir("zircon_editor_export_plan_diagnostics_output");
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Export Plan Diagnostics",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.plugins.selections.push(
        zircon_runtime::core::framework::project::ProjectPluginSelection::runtime_plugin(
            zircon_runtime::builtin::RuntimePluginId::Sound,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime"),
    );
    manifest.export_profiles = vec![
        zircon_runtime::core::framework::project::ExportProfile::new(
            "native-only",
            RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::project::ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        )
        .with_strategies([
            zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
        ]),
    ];

    let report = manager
        .execute_export_build(&output_root, &manifest, "native-only")
        .unwrap();

    assert!(!report.invoked_cargo);
    assert!(report.generated_files.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("sound") && message.contains("LibraryEmbed")));
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("cargo build skipped")));
    let diagnostics = std::fs::read_to_string(output_root.join("export-diagnostics.txt")).unwrap();
    assert!(diagnostics.contains("LibraryEmbed"));
    assert!(diagnostics.contains("cargo build skipped"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn native_dynamic_export_builds_native_cargo_package_before_materializing() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_native_dynamic_build_config");
    let project_root = unique_temp_dir("zircon_editor_native_dynamic_build_project");
    let output_root = unique_temp_dir("zircon_editor_native_dynamic_build_output");
    let fake_cargo = write_fake_native_cargo(&project_root, "zircon_plugin_native_tool_runtime");
    let _cargo_guard = EnvVarGuard::set("CARGO", &fake_cargo);
    std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
    std::fs::create_dir_all(project_root.join("zircon_plugins/native_tool/native/src")).unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/native_tool/plugin.toml"),
        r#"
id = "native_tool"
version = "0.1.0"
display_name = "Native Tool"
description = "Native plugin export fixture."
default_packaging = ["native_dynamic"]

[[modules]]
name = "native_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_tool_runtime"
target_modes = ["client_runtime"]
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/native_tool/native/Cargo.toml"),
        "[package]\nname = \"zircon_plugin_native_tool_runtime\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\ncrate-type = [\"cdylib\"]\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("zircon_plugins/native_tool/native/src/lib.rs"),
        "pub fn native_source_should_not_ship() {}\n",
    )
    .unwrap();
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Native Dynamic Build Export",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.plugins.selections.push(
        zircon_runtime::core::framework::project::ProjectPluginSelection {
            id: "native_tool".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging:
                zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
            runtime_crate: Some("zircon_plugin_native_tool_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        },
    );
    manifest.export_profiles = vec![
        zircon_runtime::core::framework::project::ExportProfile::new(
            "client-native",
            RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::project::ExportTargetPlatform::Windows,
            RuntimeProfileId::Minimal,
        )
        .with_strategies([
            zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
        ]),
    ];

    let report = manager
        .execute_native_aware_export_build(&project_root, &output_root, &manifest, "client-native")
        .unwrap();

    assert!(report.fatal_diagnostics.is_empty());
    assert_eq!(report.plan.native_dynamic_packages, vec!["native_tool"]);
    assert_eq!(report.native_cargo_invocations.len(), 1);
    assert!(report.native_cargo_invocations[0].success);
    assert!(!report.invoked_cargo);
    assert!(output_root
        .join("plugins/native_tool/native")
        .join(platform_library_file_name(
            "zircon_plugin_native_tool_runtime"
        ))
        .exists());
    assert!(!output_root
        .join("plugins/native_tool/native/Cargo.toml")
        .exists());
    assert!(!output_root
        .join("plugins/native_tool/native/src/lib.rs")
        .exists());
    assert!(!output_root.join(".native-dynamic-staging").exists());
    assert!(!output_root.join(".native-dynamic-build").exists());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(project_root);
    let _ = std::fs::remove_dir_all(output_root);
}

#[cfg(windows)]
mod durable_enablement {
    use super::*;
    use crate::ui::host::{
        PluginEnablementApplied, PluginEnablementCauseKind, PluginEnablementDurable,
        PluginEnablementRecovery, PluginEnablementRecoveryCommand,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
    };
    use zircon_runtime::asset::project::{plugin_apply::*, ProjectPaths};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    struct Fixture {
        root: PathBuf,
        project: PathBuf,
        config: PathBuf,
        old_config: Option<std::ffi::OsString>,
    }
    impl Fixture {
        fn new(label: &str) -> Self {
            use std::os::windows::fs::MetadataExt;
            let base = Path::new("E:/cargo-targets/zircon-local");
            for ancestor in base.ancestors() {
                assert_eq!(
                    fs::symlink_metadata(ancestor).unwrap().file_attributes() & 0x400,
                    0,
                    "fixture root must be physical before any write"
                );
            }
            let resolved = ProjectPaths::resolve_existing(base).unwrap();
            let physical = resolved.operation_path();
            let display = physical.to_str().unwrap().replace('\\', "/");
            assert!(display
                .strip_prefix("//?/")
                .unwrap_or(&display)
                .eq_ignore_ascii_case("E:/cargo-targets/zircon-local"));
            let root = physical.join(format!(
                "e06-host-{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let project = root.join("project");
            create_project_with_default_world(&project);
            let config = root.join("runtime-config.json");
            Self {
                root,
                project,
                config,
                old_config: std::env::var_os("ZIRCON_CONFIG_PATH"),
            }
        }
        fn register_project_metadata(&self, manager: &EditorManager, id: &str) {
            let package = self.project.join("zircon_plugins").join(id);
            fs::create_dir_all(&package).unwrap();
            fs::write(
                package.join("plugin.toml"),
                format!(
                    r#"
id = "{id}"
version = "0.1.0"
display_name = "E06 Metadata Only"
default_packaging = ["native_dynamic"]
[[modules]]
name = "{id}.runtime"
kind = "runtime"
crate_name = "{id}_runtime"
target_modes = ["editor_host"]
capabilities = ["runtime.e06.metadata_only"]
[[modules]]
name = "{id}.editor"
kind = "editor"
crate_name = "{id}_editor"
target_modes = ["editor_host"]
capabilities = ["editor.extension.e06_metadata_only"]
"#
                ),
            )
            .unwrap();
            // Real metadata discovery + Project registration, without a builtin Rust handle.
            manager
                .register_enablement_test_project_package(&self.project, id)
                .unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            match self.old_config.take() {
                Some(value) => std::env::set_var("ZIRCON_CONFIG_PATH", value),
                None => std::env::remove_var("ZIRCON_CONFIG_PATH"),
            }
            // Every test drops all reports/manager handles and CoreRuntime (joining its writer)
            // before this owned, uniquely created root. No foreign fixture is adopted.
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn manifest_replace_failure_preserves_live_owner_and_report_arc() {
        use std::os::windows::fs::OpenOptionsExt;
        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("disk-failure");
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        let before = fs::read(paths.manifest_path()).unwrap();
        let capabilities = manager.capability_snapshot();
        let generation = manager.enablement_test_generation();
        let status = manager.published_plugin_status_report();
        let result = manager.set_plugin_enabled_with_commit_guard(
            &fixture.project,
            "runtime_diagnostics",
            true,
            |paths| {
                fs::OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .open(paths.manifest_path())
                    .unwrap()
            },
        );
        let report = result
            .expect_err("a read handle denying DELETE must reject atomic manifest replacement");
        assert_eq!(report.applied, PluginEnablementApplied::NotApplied);
        assert_eq!(
            report.primary_cause.as_ref().unwrap().kind,
            PluginEnablementCauseKind::DurableCommit
        );
        assert_eq!(fs::read(paths.manifest_path()).unwrap(), before);
        assert_eq!(manager.capability_snapshot(), capabilities);
        assert_eq!(manager.enablement_test_generation(), generation);
        assert!(Arc::ptr_eq(
            &status,
            &manager.published_plugin_status_report()
        ));
        drop(report);
        drop(status);
        drop(manager);
        drop(runtime);
    }
    #[test]
    fn generation_change_during_prepare_cannot_stamp_an_old_plan() {
        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("prepare-generation");
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        fixture.register_project_metadata(&manager, "e06_prepare_generation");
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        let before = fs::read(paths.manifest_path()).unwrap();
        let status = manager.published_plugin_status_report();
        let report = manager
            .set_plugin_enabled_changing_generation(
                &fixture.project,
                "runtime_diagnostics",
                true,
                false,
            )
            .unwrap_err();
        assert_eq!(report.durable, PluginEnablementDurable::NotCommitted);
        assert_eq!(
            report.primary_cause.as_ref().unwrap().kind,
            PluginEnablementCauseKind::Prepare
        );
        assert_eq!(fs::read(paths.manifest_path()).unwrap(), before);
        assert!(pending_plugin_apply(&paths).unwrap().is_none());
        assert!(Arc::ptr_eq(
            &status,
            &manager.published_plugin_status_report()
        ));
        drop(report);
        drop(status);
        drop(manager);
        drop(runtime);
    }
    #[test]
    fn generation_change_after_commit_keeps_typed_pending_without_runtime_apply() {
        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("apply-generation");
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        fixture.register_project_metadata(&manager, "e06_apply_generation");
        let capabilities = manager.capability_snapshot();
        let status = manager.published_plugin_status_report();
        let report = manager
            .set_plugin_enabled_changing_generation(
                &fixture.project,
                "runtime_diagnostics",
                true,
                true,
            )
            .unwrap_err();
        assert_eq!(report.durable, PluginEnablementDurable::Durable);
        assert_eq!(report.applied, PluginEnablementApplied::NotApplied);
        assert_eq!(
            report.recovery,
            PluginEnablementRecovery::PendingRuntimeApply
        );
        assert_eq!(manager.capability_snapshot(), capabilities);
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        assert_eq!(
            pending_plugin_apply(&paths).unwrap().unwrap().operation_id,
            report.operation_id
        );
        let published = manager.published_plugin_status_report();
        assert!(!Arc::ptr_eq(&status, &published));
        assert_eq!(
            published.enablement.as_ref().unwrap().operation_id,
            report.operation_id
        );
        drop(published);
        drop(report);
        drop(status);
        drop(manager);
        drop(runtime);
    }
    #[test]
    fn registered_project_package_without_live_handle_requires_explicit_repair() {
        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("project-native");
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        fixture.register_project_metadata(&manager, "e06_metadata_only");
        let capabilities = manager.capability_snapshot();
        let generation = manager.enablement_test_generation();
        let report = manager
            .set_project_plugin_enabled_durable(&fixture.project, "e06_metadata_only", true)
            .unwrap_err();
        assert_eq!(report.durable, PluginEnablementDurable::Durable);
        assert_eq!(report.applied, PluginEnablementApplied::NotApplied);
        assert_eq!(
            report.recovery,
            PluginEnablementRecovery::RuntimeRepairRequired
        );
        assert_eq!(manager.capability_snapshot(), capabilities);
        assert_eq!(manager.enablement_test_generation(), generation);
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        let revision = report.committed_revision.clone().unwrap();
        assert!(pending_plugin_apply(&paths).unwrap().is_some());
        assert_eq!(
            manager
                .retry_project_plugin_enablement(&fixture.project, &report.operation_id, &revision)
                .unwrap_err()
                .recovery,
            PluginEnablementRecovery::RuntimeRepairRequired
        );
        assert_eq!(
            manager
                .revert_project_plugin_enablement(&fixture.project, &report.operation_id, &revision)
                .unwrap_err()
                .recovery,
            PluginEnablementRecovery::RuntimeRepairRequired
        );
        drop(report);
        drop(manager);
        drop(runtime);
    }
    #[test]
    fn inspect_preserves_unknown_runtime_fence_and_rejects_retry_revert_and_new_intent() {
        static CALLS: AtomicU64 = AtomicU64::new(0);
        fn panic_in_runtime_callback_boundary() {
            CALLS.fetch_add(1, Ordering::SeqCst);
            panic!("injected runtime callback uncertainty");
        }
        fn unexpected_runtime_apply() {
            CALLS.fetch_add(1, Ordering::SeqCst);
        }

        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("inspect-repair-fence");
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        let capabilities = manager.capability_snapshot();
        let generation = manager.enablement_test_generation();
        CALLS.store(0, Ordering::SeqCst);
        manager
            .set_enablement_runtime_checkpoint_for_test(Some(panic_in_runtime_callback_boundary));
        let original = manager
            .set_project_plugin_enabled_durable(&fixture.project, "runtime_diagnostics", true)
            .unwrap_err();
        assert_eq!(original.applied, PluginEnablementApplied::Unknown);
        assert_eq!(
            original.recovery,
            PluginEnablementRecovery::RuntimeRepairRequired
        );
        assert_eq!(
            original.primary_cause.as_ref().unwrap().kind,
            PluginEnablementCauseKind::Panic
        );
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        let revision = original.committed_revision.clone().unwrap();
        let manifest = fs::read(paths.manifest_path()).unwrap();
        let pending = pending_plugin_apply(&paths).unwrap().unwrap();
        assert_eq!(pending.operation_id, original.operation_id);
        let pending_path = paths.derived_root().join("plugin-enablement/pending.json");
        let before_pending = fs::read(&pending_path).unwrap();

        // Both ordinary status publishers and report eviction are presentation operations.
        manager.replace_then_clear_enablement_status_for_test();
        let inspected = manager
            .execute_plugin_enablement_recovery(
                &fixture.project,
                PluginEnablementRecoveryCommand::Inspect { admission: None },
            )
            .unwrap_err();
        assert_eq!(inspected, original);
        assert_eq!(
            manager.published_plugin_status_report().enablement.as_ref(),
            Some(&original)
        );
        manager.set_enablement_runtime_checkpoint_for_test(Some(unexpected_runtime_apply));
        let retry = manager
            .execute_plugin_enablement_recovery(
                &fixture.project,
                PluginEnablementRecoveryCommand::Retry {
                    operation_id: original.operation_id.clone(),
                    expected_revision: revision.clone(),
                },
            )
            .unwrap_err();
        assert_eq!(retry, original);
        let revert = manager
            .execute_plugin_enablement_recovery(
                &fixture.project,
                PluginEnablementRecoveryCommand::Revert {
                    operation_id: original.operation_id.clone(),
                    expected_revision: revision,
                    admission: None,
                },
            )
            .unwrap_err();
        assert_eq!(revert, original);
        // A new plugin/op/revision intention must hit the whole Ready-session fence first.
        let new_intent = manager
            .set_project_plugin_enabled_durable(&fixture.project, "e06_unrelated_new_intent", false)
            .unwrap_err();
        assert_eq!(new_intent, original);
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(manager.capability_snapshot(), capabilities);
        assert_eq!(manager.enablement_test_generation(), generation);
        assert_eq!(fs::read(paths.manifest_path()).unwrap(), manifest);
        assert_eq!(fs::read(&pending_path).unwrap(), before_pending);
        manager.set_enablement_runtime_checkpoint_for_test(None);
        drop(new_intent);
        drop(revert);
        drop(retry);
        drop(inspected);
        drop(original);
        drop(manager);
        drop(runtime);
    }
    #[test]
    fn cold_preflight_projects_admitted_revert_and_then_allows_a_fresh_open() {
        let _env = env_lock().lock().unwrap();
        let fixture = Fixture::new("cold-caller");
        let paths = ProjectPaths::from_root(&fixture.project).unwrap();
        let document = PluginApplyDocument::read(&paths).unwrap();
        let mut candidate = document.manifest.clone();
        let selection = &mut candidate.plugins.selections[0];
        let previous_enabled = selection.enabled;
        selection.enabled = !previous_enabled;
        let receipt = PendingPluginApply {
            version: 1,
            operation_id: "cold-operation".into(),
            plugin_id: selection.id.clone(),
            enabled: selection.enabled,
            previous_enabled,
            project_guid: candidate.project_guid.to_string(),
            manifest_digest: String::new(),
            session_instance: "closed-session".into(),
            session_generation: 1,
            catalog_generation: 1,
            acknowledged: false,
            resolved_before_activation: false,
        };
        let committed = commit_plugin_apply(&paths, &document, &candidate, receipt, None).unwrap();
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&fixture.config);
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        let capabilities = manager.capability_snapshot();
        let generation = manager.enablement_test_generation();
        assert!(manager.open_project_and_remember(&fixture.project).is_err());
        let status = manager.published_plugin_status_report();
        let pending = status
            .enablement
            .as_ref()
            .expect("failed cold preflight must publish an actionable typed operation");
        assert_eq!(pending.operation_id, "cold-operation");
        assert!(
            pending.recovery_admission.is_some(),
            "authenticated BuildSet admission must be retained before Ready"
        );
        assert_eq!(manager.capability_snapshot(), capabilities);
        assert_eq!(manager.enablement_test_generation(), generation);
        let resolved = manager
            .execute_plugin_enablement_recovery(
                &fixture.project,
                PluginEnablementRecoveryCommand::Revert {
                    operation_id: pending.operation_id.clone(),
                    expected_revision: committed.committed_revision.clone().unwrap(),
                    admission: pending.recovery_admission.clone(),
                },
            )
            .unwrap_err();
        assert_ne!(resolved.operation_id, "cold-operation");
        assert_eq!(resolved.recovery, PluginEnablementRecovery::RestartRequired);
        assert_eq!(resolved.applied, PluginEnablementApplied::NotApplied);
        assert_eq!(manager.capability_snapshot(), capabilities);
        assert_eq!(manager.enablement_test_generation(), generation);
        assert!(pending_plugin_apply(&paths).unwrap().is_none());
        crate::core::project::ProjectAuthority::default()
            .preflight_project(&fixture.project)
            .unwrap();
        manager
            .open_enablement_test_project(&fixture.project)
            .unwrap();
        drop(resolved);
        drop(status);
        drop(manager);
        drop(runtime);
    }
}
