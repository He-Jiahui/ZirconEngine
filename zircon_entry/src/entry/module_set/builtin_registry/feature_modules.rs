use std::sync::Arc;

use zircon_module::EngineModule;

pub(super) fn feature_modules() -> Vec<Arc<dyn EngineModule>> {
    zircon_builtin_modules::feature_modules()
}
