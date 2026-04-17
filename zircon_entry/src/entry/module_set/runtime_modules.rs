use std::sync::Arc;

use zircon_module::EngineModule;

use super::builtin_registry::builtin_modules;

pub(super) fn runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    builtin_modules()
}
