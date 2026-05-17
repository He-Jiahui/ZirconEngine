use crate::prelude::*;

#[test]
fn app_prelude_exports_entry_and_plugin_group_types() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    let default_group = DefaultPlugins::default().build().unwrap().finish();
    let dev_group = DevPlugins::default().build().unwrap().finish();
    let minimal_group = MinimalPlugins.build().unwrap().finish();
    let headless_group = HeadlessPlugins::default().build().unwrap().finish();
    let custom_group = PluginGroupBuilder::start("CustomPlugins").finish();
    let runtime_profile_config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d);
    let selection_report: EntryModuleSelectionReport = entry.module_selection_report();
    let runner_diagnostics = EntryRunner::module_selection_diagnostics(
        EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal),
    )
    .unwrap();
    let provider_runner_diagnostics =
        EntryRunner::module_selection_diagnostics_with_first_party_runtime_plugin_registrations(
            EntryConfig::for_runtime_profile(RuntimeProfileId::Minimal),
        )
        .unwrap();

    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
    assert_eq!(selection_report.plugin_group, "DefaultPlugins");
    assert!(runner_diagnostics.contains("entry.plugin_group=MinimalPlugins"));
    assert!(runner_diagnostics.contains("platform.enabled=false"));
    assert!(provider_runner_diagnostics.contains("entry.plugin_group=MinimalPlugins"));
    assert!(provider_runner_diagnostics.contains("platform.target_mode=client_runtime"));
    assert!(selection_report
        .module_keys()
        .contains(&zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert_eq!(
        runtime_profile_config.runtime_profile(),
        Some(RuntimeProfileId::Client3d)
    );
    assert_eq!(entry.plugin_group().name(), default_group.name());
    assert!(dev_group
        .module_keys()
        .contains(&zircon_runtime::core::modules::LOG_DIAGNOSTICS_MODULE_NAME));
    assert!(minimal_group
        .module_keys()
        .contains(&zircon_runtime::foundation::FOUNDATION_MODULE_NAME));
    assert!(!headless_group
        .module_keys()
        .contains(&zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert_eq!(custom_group.name(), "CustomPlugins");
}

#[test]
fn app_prelude_includes_runtime_prelude_foundations() {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    enum AppPreludeState {
        #[default]
        Boot,
        Ready,
    }

    let runtime = CoreRuntime::new();
    let descriptor = ModuleDescriptor::new("PreludeAppModule", "app prelude smoke module");

    runtime.register_module(descriptor).unwrap();
    runtime.init_state::<AppPreludeState>();
    runtime.set_next_state(AppPreludeState::Ready);

    assert_eq!(RuntimeProfileId::Client3d, RuntimeProfileId::Client3d);
    assert_eq!(
        runtime
            .apply_state_transition::<AppPreludeState>()
            .unwrap()
            .entered,
        Some(AppPreludeState::Ready)
    );
}
