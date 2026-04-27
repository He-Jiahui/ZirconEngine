use super::super::{BuiltinEngineEntry, EngineEntry, EntryConfig, EntryProfile, EntryRunMode};
use zircon_runtime::{RuntimePluginId, RuntimeTargetMode};

const EDITOR_MODULE_NAME: &str = "EditorModule";

#[test]
fn builtin_engine_entry_reports_run_mode_and_owned_modules() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime).unwrap();
    let descriptors = entry.module_descriptors();

    assert_eq!(entry.profile(), EntryProfile::Runtime);
    assert_eq!(entry.run_mode(), EntryRunMode::Runtime);
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
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert!(descriptors
        .iter()
        .all(|descriptor| descriptor.name != zircon_runtime::ui::UI_MODULE_NAME));
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
