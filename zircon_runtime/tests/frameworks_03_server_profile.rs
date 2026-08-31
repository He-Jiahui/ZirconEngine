#![cfg(feature = "script")]

use zircon_runtime::{
    builtin::runtime_modules_for_target, core::framework::platform::RuntimeTargetMode,
};

#[test]
fn server_runtime_selection_excludes_script_from_a_client_compiled_binary() {
    let report =
        runtime_modules_for_target(RuntimeTargetMode::ServerRuntime, Some(&Default::default()))
            .expect("server module composition should compile");
    let module_names = report
        .modules()
        .iter()
        .map(|module| module.module_name())
        .collect::<Vec<_>>();

    assert!(
        !module_names.contains(&zircon_runtime::script::SCRIPT_MODULE_NAME),
        "server module selection must exclude ScriptModule even when script is compiled: {module_names:?}"
    );
}
