use crate::prelude::*;

#[test]
fn app_prelude_exports_entry_and_plugin_group_types() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    let default_group = DefaultPlugins::default().build().unwrap().finish();
    let minimal_group = MinimalPlugins.build().unwrap().finish();
    let headless_group = HeadlessPlugins::default().build().unwrap().finish();
    let custom_group = PluginGroupBuilder::start("CustomPlugins").finish();
    let runtime_profile_config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d);

    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
    assert_eq!(
        runtime_profile_config.runtime_profile(),
        Some(RuntimeProfileId::Client3d)
    );
    assert_eq!(entry.plugin_group().name(), default_group.name());
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
