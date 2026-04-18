use std::sync::Arc;

use zircon_module::EngineModule;

use super::entry_profile::EntryProfile;

pub(super) fn builtin_modules_for_profile(profile: EntryProfile) -> Vec<Arc<dyn EngineModule>> {
    let mut modules = zircon_runtime::builtin_runtime_modules();
    if matches!(profile, EntryProfile::Editor) {
        modules.push(Arc::new(zircon_editor::EditorModule));
    }

    modules
}
