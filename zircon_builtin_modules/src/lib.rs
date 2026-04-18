mod core_modules;
mod feature_modules;

use std::sync::Arc;

use zircon_module::EngineModule;

pub use core_modules::core_modules;
pub use feature_modules::feature_modules;

pub fn runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    let mut modules = core_modules();
    modules.extend(feature_modules());
    modules
}
