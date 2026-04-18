use std::sync::Arc;

use zircon_module::EngineModule;

pub(super) fn core_modules() -> Vec<Arc<dyn EngineModule>> {
    zircon_builtin_modules::core_modules()
}
