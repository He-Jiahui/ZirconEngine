mod core_modules;
mod feature_modules;

use std::sync::Arc;

use zircon_module::EngineModule;

pub(super) fn builtin_modules() -> Vec<Arc<dyn EngineModule>> {
    let mut modules = core_modules::core_modules();
    modules.extend(feature_modules::feature_modules());
    modules
}
