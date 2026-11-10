//! Source locks for the dynamic runtime image seal.
//! The refcount harness itself lives beside `plan_module_release_for`.

#[test]
fn dynamic_load_seals_the_module_that_exported_the_v8_entry() {
    let source = include_str!("../loaded_runtime.rs");
    let load = source
        .split("fn load_dynamic_library(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn runtime_instance(").next())
        .expect("dynamic load body");
    let address = load
        .find("let code_address = (*get_api) as *const ()")
        .expect("pin address is the loaded export");
    let seal = load
        .find("SealedDynamicLibrary::seal(library, code_address)")
        .expect("validated load seals the dynamic image");
    let validated = load.find("validate_v8_api(api)").unwrap();
    assert!(
        validated < seal,
        "a rejected table must be able to drop its library before any device exists"
    );
    assert!(address < seal);
}

#[test]
fn windows_pin_uses_the_loader_pin_flag_and_leaks_only_when_that_call_fails() {
    let source = include_str!("../loaded_runtime/module_pin.rs");
    let seal = source
        .split("fn seal(")
        .nth(1)
        .and_then(|body| body.split("fn receipt_label(").next())
        .expect("seal body");
    let retain = seal.find("if plan.retain_library_handle").unwrap();
    let forget = seal.find("std::mem::forget(library)").unwrap();
    assert!(retain < forget);
    assert!(seal.contains("plan_module_release(pin_succeeded)"));
    assert!(
        source.contains("RemoveVectoredExceptionHandler(output_debug_string_handler as *mut _)")
    );
    assert!(source.contains("GET_MODULE_HANDLE_EX_FLAG_PIN"));
    let pin = source
        .split("fn pin_module_containing(")
        .nth(1)
        .expect("windows pin helper");
    assert!(pin.contains("0x0000_0001"));
    assert!(pin.contains("0x0000_0004"));
    assert!(pin.contains("GetModuleHandleExW"));
    assert!(
        !pin.contains("FreeLibrary"),
        "the pinned HMODULE stays mapped until process termination"
    );
}

#[test]
fn default_editor_host_still_loads_play_through_a_second_dynamic_library() {
    let cargo = include_str!("../../../../Cargo.toml");
    let features = cargo
        .split("target-editor-host = [")
        .nth(1)
        .and_then(|block| block.split("]\n").next())
        .expect("target-editor-host feature block");
    assert!(
        !features.contains("editor-scripting"),
        "enabling editor-scripting is a separate host decision, not this teardown seal"
    );
    let factory = include_str!("../../entry_runner/editor/play_session_factory.rs");
    assert!(factory.contains(".load_after_preflight()"));
    assert!(factory.contains("fn create_linked("));
}
