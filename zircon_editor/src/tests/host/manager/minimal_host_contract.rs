use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::script::{
    VmPluginManagementPolicy, VmPluginManager, VmPluginManifest, VmPluginPackage,
    VM_PLUGIN_MANAGER_NAME,
};

use crate::ui::host::minimal_host_contract::editor_host_minimal_contract;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::host::{
    EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
};
use crate::ui::workbench::view::ViewDescriptorId;

use super::support::*;

#[path = "minimal_host_contract/core_contract.rs"]
mod core_contract;
#[path = "minimal_host_contract/native_plugins.rs"]
mod native_plugins;
#[path = "minimal_host_contract/optional_features.rs"]
mod optional_features;

#[test]
fn editor_manager_registers_minimal_host_capabilities_as_vm_handles_when_script_is_available() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_minimal_host_vm");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = editor_runtime_with_script_module();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let vm_manager = runtime
        .handle()
        .resolve_manager::<VmPluginManager>(VM_PLUGIN_MANAGER_NAME)
        .unwrap();

    let bridge = manager.vm_extension_capability_report();
    assert!(bridge.diagnostics().is_empty());

    for capability in editor_host_minimal_contract().minimal_capability_ids() {
        let handle = bridge
            .handle_for(&capability)
            .expect("registered capability handle");
        let record = vm_manager
            .host_registry()
            .resolve(handle)
            .expect("host registry record");
        assert_eq!(record.label, capability);
    }

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_vm_extension_load_failure_is_reported_without_breaking_host() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_vm_extension_failure");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = editor_runtime_with_script_module();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let report = manager.load_vm_extension_package(VmPluginPackage {
        manifest: VmPluginManifest {
            name: "broken-tool".to_string(),
            version: "0.1.0".to_string(),
            entry: "main".to_string(),
            capabilities: Default::default(),
            management: VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    });

    assert!(report.loaded_slot().is_none());
    assert!(report
        .diagnostics()
        .iter()
        .any(|message| message.contains("BackendUnavailable")));
    assert!(manager
        .minimal_host_report()
        .missing_capabilities()
        .is_empty());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_honors_subsystem_toggles_without_disabling_minimal_shell() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_subsystem_toggles");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!(["editor.extension.runtime_diagnostics"]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let minimal = manager.minimal_host_report();
    assert!(minimal.missing_capabilities().is_empty());
    let subsystem_report = manager.subsystem_report();
    assert!(subsystem_report.is_enabled("editor.extension.runtime_diagnostics"));
    assert!(!subsystem_report.is_enabled("editor.extension.animation_authoring"));
    assert!(!subsystem_report.is_enabled("editor.extension.ui_asset_authoring"));
    let descriptors = manager.descriptors();
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.runtime_diagnostics"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.debug_observatory"));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.animation_sequence"));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.ui_asset"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_exposes_capability_snapshot_for_view_filtering() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_capability_snapshot");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!(["editor.extension.runtime_diagnostics"]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let snapshot = manager.capability_snapshot();
    assert!(snapshot.is_enabled("editor.host.ui_shell"));
    assert!(snapshot.is_enabled("editor.extension.runtime_diagnostics"));
    assert!(!snapshot.is_enabled("editor.extension.animation_authoring"));

    let diagnostics = manager
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "editor.runtime_diagnostics")
        .expect("runtime diagnostics descriptor");
    assert_eq!(
        diagnostics.required_capabilities,
        vec!["editor.extension.runtime_diagnostics".to_string()]
    );
    let debug_observatory = manager
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "editor.debug_observatory")
        .expect("debug observatory descriptor");
    assert_eq!(
        debug_observatory.required_capabilities,
        vec!["editor.extension.runtime_diagnostics".to_string()]
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_plugin_toggle_refreshes_snapshot_and_view_gate() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_plugin_toggle");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(manager
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.module_plugins"));
    assert!(manager
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.debug_observatory"));

    let enabled = manager
        .set_editor_plugin_enabled("runtime_diagnostics", true)
        .unwrap();
    assert!(enabled.is_enabled("editor.extension.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.debug_observatory"));

    manager
        .set_editor_plugin_enabled("runtime_diagnostics", false)
        .unwrap();
    assert!(!manager
        .capability_snapshot()
        .is_enabled("editor.extension.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "editor.debug_observatory"));
    assert!(manager
        .open_view(ViewDescriptorId::new("editor.runtime_diagnostics"), None,)
        .is_err());
    assert!(manager
        .open_view(ViewDescriptorId::new("editor.debug_observatory"), None,)
        .is_err());

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn required_builtin_plugin_cannot_be_disabled_through_manager_api() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_required_builtin_plugin");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Required Builtin Plugin Test",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    manager
        .set_project_plugin_enabled(&mut manifest, "runtime_diagnostics", true)
        .unwrap();
    manifest
        .plugins
        .selections
        .iter_mut()
        .find(|selection| selection.id == "runtime_diagnostics")
        .expect("runtime diagnostics project selection")
        .required = true;

    let error = manager
        .set_project_plugin_enabled(&mut manifest, "runtime_diagnostics", false)
        .unwrap_err();

    assert!(error.contains("required plugin runtime_diagnostics cannot be disabled"));
    assert!(
        manifest
            .plugins
            .selections
            .iter()
            .find(|selection| selection.id == "runtime_diagnostics")
            .expect("runtime diagnostics project selection remains")
            .enabled
    );
    assert!(
        manifest
            .plugins
            .selections
            .iter()
            .find(|selection| selection.id == "runtime_diagnostics")
            .expect("runtime diagnostics project selection remains")
            .required
    );
    assert!(manager
        .capability_snapshot()
        .is_enabled("editor.extension.runtime_diagnostics"));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn project_plugin_packaging_and_target_modes_are_editable_through_manager_api() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_plugin_selection_policy");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Plugin Selection Policy Test",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let packaging = manager
        .set_project_plugin_packaging(
            &mut manifest,
            "runtime_diagnostics",
            zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
        )
        .unwrap();
    assert_eq!(packaging.plugin_id, "runtime_diagnostics");
    assert_eq!(
        packaging.project_selection.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    );

    let target_modes = manager
        .set_project_plugin_target_modes(
            &mut manifest,
            "runtime_diagnostics",
            [
                zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
                zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
                zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            ],
        )
        .unwrap();
    assert_eq!(
        target_modes.project_selection.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        ]
    );
    let selection = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "runtime_diagnostics")
        .expect("runtime diagnostics selection");
    assert_eq!(
        selection.packaging,
        zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(
        selection.target_modes,
        target_modes.project_selection.target_modes
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_runtime_sandbox_can_be_disabled_before_vm_bridge_registration() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_sandbox_disabled");
    std::env::set_var("ZIRCON_CONFIG_PATH", &path);
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
        serde_json::json!(false),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::script::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let bridge = manager.vm_extension_capability_report();
    assert!(!bridge.sandbox_enabled());
    assert!(bridge.loaded_capabilities().is_empty());
    assert!(bridge
        .diagnostics()
        .iter()
        .any(|message| message.contains("runtime sandbox disabled")));

    let report = manager.load_vm_extension_package(VmPluginPackage {
        manifest: VmPluginManifest {
            name: "sandbox-disabled-tool".to_string(),
            version: "0.1.0".to_string(),
            entry: "main".to_string(),
            capabilities: Default::default(),
            management: VmPluginManagementPolicy::default(),
        },
        zr_vm_project: None,
        bytecode: vec![1, 2, 3],
    });
    assert!(report.loaded_slot().is_none());
    assert!(report
        .diagnostics()
        .iter()
        .any(|message| message.contains("runtime sandbox disabled")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

fn editor_runtime_with_script_module() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::script::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();

    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(zircon_runtime::script::SCRIPT_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

fn write_fake_native_cargo(project_root: &std::path::Path, crate_name: &str) -> std::path::PathBuf {
    let tools_root = project_root.join("tools");
    std::fs::create_dir_all(&tools_root).unwrap();
    let library_name = platform_library_file_name(crate_name);
    let cargo_path = tools_root.join(if cfg!(target_os = "windows") {
        "fake-cargo.cmd"
    } else {
        "fake-cargo.sh"
    });
    if cfg!(target_os = "windows") {
        std::fs::write(
            &cargo_path,
            format!(
                "@echo off\r\nset \"TARGET_DIR=\"\r\n:loop\r\nif \"%~1\"==\"\" goto done\r\nif \"%~1\"==\"--target-dir\" goto capture_target\r\nshift\r\ngoto loop\r\n:capture_target\r\nshift\r\nset \"TARGET_DIR=%~1\"\r\nshift\r\ngoto loop\r\n:done\r\nif \"%TARGET_DIR%\"==\"\" exit /b 2\r\nmkdir \"%TARGET_DIR%\\debug\" 2>NUL\r\necho fake-native-library>\"%TARGET_DIR%\\debug\\{library_name}\"\r\nexit /b 0\r\n"
            ),
        )
        .unwrap();
    } else {
        std::fs::write(
            &cargo_path,
            format!(
                "#!/bin/sh\nTARGET_DIR=\"\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"--target-dir\" ]; then\n    shift\n    TARGET_DIR=\"$1\"\n  fi\n  shift\ndone\nif [ -z \"$TARGET_DIR\" ]; then exit 2; fi\nmkdir -p \"$TARGET_DIR/debug\"\nprintf 'fake-native-library' > \"$TARGET_DIR/debug/{library_name}\"\n"
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&cargo_path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&cargo_path, permissions).unwrap();
        }
    }
    cargo_path
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

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}
