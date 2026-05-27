use super::super::{
    BuiltinEngineEntry, EngineEntry, EntryConfig, EntryProfile, EntryRunMode, EntryRunner,
};
use crate::plugins::{DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup};
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowResolution};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginAvailabilityCategory,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport, RuntimeProfileId,
};
use zircon_runtime::{RuntimePluginId, RuntimeTargetMode};

const EDITOR_MODULE_NAME: &str = "EditorModule";

#[test]
fn builtin_engine_entry_reports_run_mode_and_owned_modules() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    let descriptors = entry.module_descriptors();

    assert_eq!(entry.profile(), EntryProfile::Runtime);
    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
    assert_eq!(entry.plugin_group().name(), "DefaultPlugins");
    assert_eq!(
        entry.plugin_group().module_keys(),
        DefaultPlugins::default()
            .build()
            .unwrap()
            .finish()
            .module_keys()
    );
    assert_eq!(entry.modules().len(), descriptors.len());
    assert!(entry
        .modules()
        .iter()
        .all(|module| !module.module_name().is_empty()));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != EDITOR_MODULE_NAME));
}

#[test]
fn entry_config_selects_runtime_modules_explicitly_for_client_runtime() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::Ui])
        .with_optional_runtime_plugins([RuntimePluginId::Physics]);
    let entry = BuiltinEngineEntry::for_config(&config).unwrap();
    let descriptors = entry.module_descriptors();

    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == zircon_runtime::ui::UI_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != EDITOR_MODULE_NAME));
}

#[test]
fn entry_config_can_define_headless_target_without_client_plugins() {
    let config = EntryConfig::new(EntryProfile::Headless)
        .with_target_mode(RuntimeTargetMode::ServerRuntime)
        .with_runtime_plugins([], []);
    let entry = BuiltinEngineEntry::for_config(&config).unwrap();
    let descriptors = entry.module_descriptors();

    assert_eq!(entry.run_mode(), EntryRunMode::Headless);
    assert_eq!(
        entry.plugin_group().module_keys(),
        HeadlessPlugins::default()
            .build()
            .unwrap()
            .finish()
            .module_keys()
    );
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::ui::UI_MODULE_NAME));
    assert_eq!(
        entry
            .module_selection_report()
            .window_descriptor
            .primary_window,
        None
    );
}

#[test]
fn minimal_runtime_profile_selects_minimal_plugin_group() {
    let entry = BuiltinEngineEntry::for_runtime_profile(RuntimeProfileId::Minimal).unwrap();
    let expected = MinimalPlugins.build().unwrap().finish();
    let descriptors = entry.module_descriptors();
    let report = entry.module_selection_report();

    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
    assert_eq!(entry.plugin_group().name(), "MinimalPlugins");
    assert_eq!(entry.plugin_group().module_keys(), expected.module_keys());
    assert_eq!(report.runtime_profile, Some(RuntimeProfileId::Minimal));
    assert_eq!(report.target_mode, RuntimeTargetMode::ClientRuntime);
    assert_eq!(report.plugin_group, "MinimalPlugins");
    assert_eq!(report.module_keys(), expected.module_keys());
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == zircon_runtime::foundation::FOUNDATION_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::platform::PLATFORM_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::input::INPUT_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
}

#[test]
fn dev_runtime_profile_selects_dev_plugin_group() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Dev);
    let entry = BuiltinEngineEntry::for_config_with_available_runtime_plugins(
        &config,
        [
            RuntimePluginId::Sound.key().to_string(),
            RuntimePluginId::Rendering.key().to_string(),
        ],
    )
    .unwrap();
    let expected = DevPlugins::default().build().unwrap().finish();
    let report = entry.module_selection_report();

    assert_eq!(entry.run_mode(), EntryRunMode::Editor);
    assert_eq!(entry.plugin_group().name(), "DevPlugins");
    assert_eq!(entry.plugin_group().module_keys(), expected.module_keys());
    assert_eq!(report.runtime_profile, Some(RuntimeProfileId::Dev));
    assert_eq!(report.target_mode, RuntimeTargetMode::EditorHost);
    assert_eq!(report.plugin_group, "DevPlugins");
    assert!(report
        .module_keys()
        .contains(&zircon_runtime::core::modules::LOG_DIAGNOSTICS_MODULE_NAME));
}

#[test]
fn module_selection_report_formats_diagnostic_summary() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal)
        .with_window_descriptor(
            WindowDescriptor::default()
                .with_title("Diagnostic Window")
                .with_resolution(WindowResolution::new(1440, 900)),
        );
    let entry = BuiltinEngineEntry::for_config(&config).unwrap();
    let report = entry.module_selection_report();
    let diagnostics = report.format_diagnostics();

    for expected in [
        "entry.profile=Runtime",
        "entry.run_mode=Runtime",
        "entry.runtime_profile=Minimal",
        "entry.target_mode=ClientRuntime",
        "platform.monitor_inventory=",
        "platform.window_events=",
        "platform.window_lifecycle=",
        "platform.window_metrics=",
        "platform.ime=",
        "platform.keyboard_events=",
        "platform.cursor_boundary=",
        "platform.cursor_options=",
        "platform.mouse_buttons=",
        "platform.mouse_wheel=",
        "platform.touch_events=",
        "platform.gesture_events=",
        "platform.pointer_position=",
        "platform.raw_mouse_motion=",
        "platform.gamepad_events=",
        "platform.gamepad_rumble=",
        "entry.plugin_group=MinimalPlugins",
        "entry.modules=",
        "window.title=Diagnostic Window",
        "window.physical_size=1440x900",
        zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
        "drivers=",
        "managers=",
        "plugins=",
    ] {
        assert!(
            diagnostics.contains(expected),
            "module selection diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn entry_runner_exposes_module_selection_diagnostics_without_bootstrap() {
    let diagnostics = EntryRunner::module_selection_diagnostics(EntryConfig::for_runtime_profile(
        RuntimeProfileId::Minimal,
    ))
    .unwrap();

    for expected in [
        "entry.profile=Runtime",
        "entry.runtime_profile=Minimal",
        "platform.monitor_inventory=",
        "platform.window_events=",
        "platform.window_lifecycle=",
        "platform.window_metrics=",
        "platform.ime=",
        "platform.keyboard_events=",
        "platform.cursor_boundary=",
        "platform.cursor_options=",
        "platform.mouse_buttons=",
        "platform.mouse_wheel=",
        "platform.touch_events=",
        "platform.gesture_events=",
        "platform.pointer_position=",
        "platform.raw_mouse_motion=",
        "platform.gamepad_events=",
        "platform.gamepad_rumble=",
        "entry.plugin_group=MinimalPlugins",
        zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
    ] {
        assert!(
            diagnostics.contains(expected),
            "entry runner module selection diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn entry_runner_exposes_first_party_module_selection_diagnostics_without_bootstrap() {
    let diagnostics =
        EntryRunner::module_selection_diagnostics_with_first_party_runtime_plugin_registrations(
            EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal),
        )
        .unwrap();

    for expected in [
        "entry.profile=Runtime",
        "entry.runtime_profile=Minimal",
        "entry.plugin_group=MinimalPlugins",
        zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
    ] {
        assert!(
            diagnostics.contains(expected),
            "first-party provider module selection diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn entry_runner_module_selection_diagnostics_include_linked_runtime_plugin_registrations() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(
        &config,
        [linked_virtual_geometry_registration()],
    )
    .unwrap();
    let availability = entry.runtime_plugin_availability();
    let diagnostics = entry.module_selection_report().format_diagnostics();

    assert!(availability.contains(
        RuntimePluginAvailabilityCategory::Linked,
        RuntimePluginId::VirtualGeometry
    ));
    assert!(!availability.has_missing_required());

    for expected in [
        "entry.profile=Runtime",
        "entry.plugin_group=DefaultPlugins",
        "runtime_plugin_availability.linked=virtual_geometry",
        "runtime_plugin_availability.missing_required.count=0",
        "module=VirtualGeometryPlugin",
        "description=Linked virtual geometry plugin module",
    ] {
        assert!(
            diagnostics.contains(expected),
            "linked runtime plugin diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn entry_runner_bootstrap_with_report_preserves_runtime_plugin_availability() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let bootstrap = EntryRunner::bootstrap_with_runtime_plugin_registrations_and_report(
        config,
        [linked_virtual_geometry_registration()],
    )
    .unwrap();

    assert!(bootstrap
        .module_selection_report()
        .runtime_plugin_availability
        .contains(
            RuntimePluginAvailabilityCategory::Linked,
            RuntimePluginId::VirtualGeometry
        ));
    assert!(!bootstrap
        .module_selection_report()
        .runtime_plugin_availability
        .has_missing_required());

    let (_core, report) = bootstrap.into_parts();
    assert!(report.module_keys().contains(&"VirtualGeometryPlugin"));
}

#[test]
fn entry_runner_feature_aware_module_selection_diagnostics_accept_linked_plugin_registrations() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_required_runtime_plugins([RuntimePluginId::VirtualGeometry]);
    let diagnostics =
        EntryRunner::module_selection_diagnostics_with_runtime_plugin_and_feature_registrations(
            config,
            [linked_virtual_geometry_registration()],
            [],
        )
        .unwrap();

    assert!(diagnostics.contains("module=VirtualGeometryPlugin"));
    assert!(diagnostics.contains("runtime_plugin_availability.linked=virtual_geometry"));
    assert!(diagnostics.contains("runtime_plugin_availability.missing_required.count=0"));
}

fn linked_virtual_geometry_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_plugin(&LinkedVirtualGeometryPlugin {
        descriptor: RuntimePluginDescriptor::new(
            "virtual_geometry",
            "Virtual Geometry",
            RuntimePluginId::VirtualGeometry,
            "zircon_plugin_virtual_geometry_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    })
}

#[derive(Debug)]
struct LinkedVirtualGeometryPlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for LinkedVirtualGeometryPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "VirtualGeometryPlugin",
            "Linked virtual geometry plugin module",
        ))
    }
}

#[cfg(feature = "target-editor-host")]
#[test]
fn entry_config_keeps_editor_shell_while_runtime_plugins_are_explicit() {
    let config = EntryConfig::new(EntryProfile::Editor).with_runtime_plugins([], []);
    let entry = BuiltinEngineEntry::for_config(&config).unwrap();
    let descriptors = entry.module_descriptors();

    assert_eq!(entry.run_mode(), EntryRunMode::Editor);
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == EDITOR_MODULE_NAME));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
}
