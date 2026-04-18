use super::super::{BuiltinEngineEntry, EngineEntry, EntryProfile, EntryRunMode};

#[test]
fn builtin_engine_entry_reports_run_mode_and_owned_modules() {
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime);
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
        .all(|descriptor| descriptor.name != zircon_editor::EDITOR_MODULE_NAME));
}
