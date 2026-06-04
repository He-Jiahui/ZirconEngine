use zircon_runtime::core::manager::resolve_ai_manager;
use zircon_runtime::core::CoreRuntime;

use crate::{module_descriptor, AI_MODULE_NAME};

#[test]
fn ai_module_resolves_neutral_manager_handle() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(AI_MODULE_NAME).unwrap();

    resolve_ai_manager(&runtime.handle()).expect("AI manager handle");
}
