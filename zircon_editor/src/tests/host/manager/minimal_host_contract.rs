use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::script::{
    VmPluginManager, VmPluginManifest, VmPluginPackage, VM_PLUGIN_MANAGER_NAME,
};

use crate::ui::host::minimal_host_contract::editor_host_minimal_contract;
use crate::ui::host::module::{self, module_descriptor, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorManager;
use crate::ui::host::{
    EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY,
};
use crate::ui::workbench::view::ViewDescriptorId;

use super::support::*;

#[test]
fn editor_host_minimal_contract_freezes_core_and_extension_capability_boundaries() {
    let contract = editor_host_minimal_contract();

    assert_eq!(
        contract.minimal_capability_ids(),
        vec![
            "editor.host.ui_shell",
            "editor.host.asset_core",
            "editor.host.scene_interaction",
            "editor.host.runtime_render_embed",
            "editor.host.plugin_management",
            "editor.host.capability_bridge",
        ]
    );
    assert!(contract.is_minimal("editor.host.ui_shell"));
    assert!(contract.is_extension_blacklisted("editor.extension.animation_authoring"));
    assert!(contract.is_extension_blacklisted("editor.extension.ui_asset_authoring"));
    assert!(contract
        .minimal_capability_ids()
        .iter()
        .all(|capability| !contract.is_extension_blacklisted(capability)));
}

#[test]
fn editor_manager_reports_minimal_host_capabilities_even_without_vm_bridge() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_minimal_host_no_vm");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let minimal = manager.minimal_host_report();
    assert!(minimal.missing_capabilities().is_empty());
    assert_eq!(
        minimal.loaded_capabilities(),
        editor_host_minimal_contract().minimal_capability_ids()
    );

    let bridge = manager.vm_extension_capability_report();
    assert!(bridge.loaded_capabilities().is_empty());
    assert!(bridge
        .diagnostics()
        .iter()
        .any(|message| message.contains("ScriptModule.Driver.PluginHostDriver")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_plugin_status_lists_owner_optional_feature_dependencies() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_optional_feature_status");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Optional Feature Status",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let blocked = manager
        .set_project_plugin_feature_enabled(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
            true,
        )
        .unwrap_err();
    assert!(blocked.contains("missing plugins"));

    let dependency_report = manager
        .enable_project_plugin_feature_dependencies(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
        )
        .expect("explicit dependency enable should update plugin selections");
    assert_eq!(
        dependency_report.enabled_dependency_plugins,
        vec!["sound".to_string(), "animation".to_string()]
    );

    let status = manager.plugin_status_report(&manifest);
    let sound = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "sound")
        .expect("sound plugin should be in the builtin catalog");
    let timeline = sound
        .optional_features
        .iter()
        .find(|feature| feature.id == "sound.timeline_animation_track")
        .expect("sound timeline animation optional feature should be projected");

    assert!(!timeline.enabled);
    assert!(timeline.available);
    assert_eq!(timeline.owner_plugin_id, "sound");
    assert!(timeline
        .provided_capabilities
        .contains(&"runtime.feature.sound.timeline_animation_track".to_string()));
    assert!(timeline.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "sound"
            && dependency.capability == "runtime.plugin.sound"
            && dependency.primary
            && dependency.plugin_enabled
            && dependency.capability_available
    }));
    assert!(timeline.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "animation"
            && dependency.capability == "runtime.feature.animation.timeline_event_track"
            && !dependency.primary
            && dependency.plugin_enabled
            && dependency.capability_available
    }));

    let feature_report = manager
        .set_project_plugin_feature_enabled(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
            true,
        )
        .expect("feature should enable after dependency plugins are enabled");
    assert!(feature_report.enabled);
    assert!(feature_report
        .project_selection
        .features
        .iter()
        .any(|feature| feature.id == "sound.timeline_animation_track" && feature.enabled));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_feature_dependency_enablement_turns_on_unique_provider_features() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_optional_feature_provider_dependencies");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Optional Feature Provider Dependencies",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let report = manager
        .enable_project_plugin_feature_dependencies(
            &mut manifest,
            "rendering",
            "rendering.vfx_graph",
        )
        .expect("explicit dependency enable should include provider features");

    assert!(report
        .enabled_dependency_plugins
        .contains(&"rendering".to_string()));
    assert!(report
        .enabled_dependency_plugins
        .contains(&"particles".to_string()));
    assert_eq!(
        report.enabled_dependency_features,
        vec!["rendering.shader_graph".to_string()]
    );

    let rendering_selection = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "rendering")
        .expect("rendering selection should be written back");
    assert!(rendering_selection
        .features
        .iter()
        .any(|feature| feature.id == "rendering.shader_graph" && feature.enabled));
    assert!(rendering_selection
        .features
        .iter()
        .any(|feature| feature.id == "rendering.vfx_graph" && !feature.enabled));

    let status = manager.plugin_status_report(&manifest);
    let vfx_graph = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "rendering")
        .and_then(|plugin| {
            plugin
                .optional_features
                .iter()
                .find(|feature| feature.id == "rendering.vfx_graph")
        })
        .expect("vfx graph optional feature should be projected");
    assert!(!vfx_graph.enabled);
    assert!(vfx_graph.available);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_plugin_status_lists_rendering_owner_features_and_defaults() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_rendering_optional_feature_status");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Rendering Optional Feature Status",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    manager
        .set_project_plugin_enabled(&mut manifest, "rendering", true)
        .expect("rendering plugin should be selectable from builtin catalogs");

    let status = manager.plugin_status_report(&manifest);
    let rendering = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "rendering")
        .expect("rendering plugin should be projected into the plugin manager status");

    assert!(rendering.enabled);
    assert_eq!(rendering.optional_features.len(), 8);
    assert!(rendering
        .editor_capabilities
        .contains(&"editor.extension.rendering_authoring".to_string()));
    assert_eq!(
        rendering
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.decals",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
            "rendering.ray_tracing_policy",
            "rendering.shader_graph",
            "rendering.vfx_graph",
        ]
    );
    assert_eq!(
        rendering
            .optional_features
            .iter()
            .filter(|feature| feature.enabled)
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
        ]
    );

    let shader_graph = rendering
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.shader_graph")
        .expect("shader graph feature status");
    assert!(!shader_graph.enabled);
    assert!(shader_graph.available);
    assert_eq!(
        shader_graph.runtime_crate.as_deref(),
        Some("zircon_plugin_rendering_shader_graph_runtime")
    );
    assert_eq!(
        shader_graph.editor_crate.as_deref(),
        Some("zircon_plugin_rendering_shader_graph_editor")
    );

    let vfx_graph = rendering
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.vfx_graph")
        .expect("vfx graph feature status");
    assert!(!vfx_graph.enabled);
    assert!(!vfx_graph.available);
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "particles"
            && dependency.capability == "runtime.plugin.particles"
            && !dependency.primary
            && !dependency.plugin_enabled
            && !dependency.capability_available
    }));
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "rendering"
            && dependency.capability == "runtime.feature.rendering.shader_graph"
            && !dependency.primary
            && dependency.plugin_enabled
            && !dependency.capability_available
    }));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

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
            .capability(handle)
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
        },
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

    let enabled = manager
        .set_editor_plugin_enabled("runtime_diagnostics", true)
        .unwrap();
    assert!(enabled.is_enabled("editor.extension.runtime_diagnostics"));
    assert!(manager
        .descriptors()
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.runtime_diagnostics"));

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
        .open_view(ViewDescriptorId::new("editor.runtime_diagnostics"), None,)
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
            zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        )
        .unwrap();
    assert_eq!(packaging.plugin_id, "runtime_diagnostics");
    assert_eq!(
        packaging.project_selection.packaging,
        zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic
    );

    let target_modes = manager
        .set_project_plugin_target_modes(
            &mut manifest,
            "runtime_diagnostics",
            [
                zircon_runtime::RuntimeTargetMode::EditorHost,
                zircon_runtime::RuntimeTargetMode::EditorHost,
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
            ],
        )
        .unwrap();
    assert_eq!(
        target_modes.project_selection.target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::EditorHost,
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
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
        zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(
        selection.target_modes,
        target_modes.project_selection.target_modes
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
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

    let status = manager.native_plugin_status_report(&project_root, &manifest);
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
    assert_eq!(
        native.target_modes,
        vec![zircon_runtime::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        native.packaging,
        zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic
    );
    assert_eq!(native.package_source, "native");
    assert_eq!(native.load_state, "missing library");
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
    assert!(status
        .diagnostics
        .iter()
        .any(|message| message.contains("library is missing")));
    assert!(native
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
            zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
        )
        .unwrap();
    assert_eq!(
        packaging.project_selection.packaging,
        zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed
    );
    let target_modes = manager
        .set_native_aware_project_plugin_target_modes(
            &project_root,
            &mut manifest,
            "native_tool",
            [zircon_runtime::RuntimeTargetMode::ServerRuntime],
        )
        .unwrap();
    assert_eq!(
        target_modes.project_selection.target_modes,
        vec![zircon_runtime::RuntimeTargetMode::ServerRuntime]
    );

    let status = manager.native_plugin_status_report(&project_root, &manifest);
    let native_status = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "native_tool")
        .expect("native plugin remains visible");
    assert!(native_status.enabled);
    assert_eq!(
        native_status.packaging,
        zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed
    );
    assert_eq!(
        native_status.target_modes,
        vec![zircon_runtime::RuntimeTargetMode::ServerRuntime]
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
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        selection.packaging,
        zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic
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
    manifest
        .plugins
        .selections
        .push(zircon_runtime::plugin::ProjectPluginSelection {
            id: "native_tool".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![zircon_runtime::RuntimeTargetMode::ClientRuntime],
            packaging: zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
            runtime_crate: Some("zircon_plugin_native_tool_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        });
    manifest.export_profiles = vec![zircon_runtime::plugin::ExportProfile::new(
        "client-native",
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::plugin::ExportTargetPlatform::Windows,
    )
    .with_strategies([zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic])];

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
        zircon_runtime::plugin::ProjectPluginSelection::runtime_plugin(
            zircon_runtime::RuntimePluginId::Sound,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime"),
    );
    manifest.export_profiles = vec![zircon_runtime::plugin::ExportProfile::new(
        "native-only",
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::plugin::ExportTargetPlatform::Windows,
    )
    .with_strategies([zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic])];

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
    manifest
        .plugins
        .selections
        .push(zircon_runtime::plugin::ProjectPluginSelection {
            id: "native_tool".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![zircon_runtime::RuntimeTargetMode::ClientRuntime],
            packaging: zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
            runtime_crate: Some("zircon_plugin_native_tool_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        });
    manifest.export_profiles = vec![zircon_runtime::plugin::ExportProfile::new(
        "client-native",
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::plugin::ExportTargetPlatform::Windows,
    )
    .with_strategies([zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic])];

    let report = manager
        .execute_native_aware_export_build(&project_root, &output_root, &manifest, "client-native")
        .unwrap();

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
        },
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
