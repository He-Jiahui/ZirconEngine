use std::sync::Arc;

use zircon_module::EngineModule;

pub(super) fn editor_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![Arc::new(zircon_editor::EditorModule)]
}
