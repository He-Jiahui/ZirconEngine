use std::sync::Arc;

use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::graphics::GraphicsModule;

use super::entry_profile::EntryProfile;

pub(super) fn builtin_modules_for_profile(profile: EntryProfile) -> Vec<Arc<dyn EngineModule>> {
    let mut modules = zircon_runtime::builtin_runtime_modules();
    // Keep the historical ordering: graphics registers immediately after the asset module.
    modules.insert(4, Arc::new(GraphicsModule));
    if matches!(profile, EntryProfile::Editor) {
        modules.push(Arc::new(zircon_editor::EditorModule));
    }

    modules
}
