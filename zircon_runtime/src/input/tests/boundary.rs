#[test]
fn input_protocol_types_live_in_runtime_input_surface() {
    let manager_crate_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../zircon_manager");
    let input_mod_source = include_str!("../mod.rs");
    let input_runtime_source = include_str!("../runtime/default_input_manager.rs");
    let app_runtime_handler_source =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/application_handler.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../../zircon_manager/src/resolver.rs");

    for required in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            input_mod_source.contains(required),
            "zircon_runtime::input root should publicly export {required}"
        );
    }

    assert!(
        input_runtime_source.contains("zircon_framework::input"),
        "runtime input manager should source input contracts from zircon_framework"
    );
    assert!(
        !app_runtime_handler_source.contains("use zircon_manager::{InputButton, InputEvent};"),
        "runtime app should import input protocol types from zircon_runtime::input instead of zircon_manager"
    );

    for forbidden in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            !manager_lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after framework migration"
        );
        assert!(
            !manager_resolver_source.contains(forbidden),
            "zircon_manager resolver should not re-export {forbidden} after framework migration"
        );
    }

    assert!(
        manager_resolver_source.contains("zircon_framework"),
        "zircon_manager resolver should source input manager contracts from zircon_framework"
    );
    assert!(
        !manager_crate_root.join("src/records").exists(),
        "zircon_manager should delete records after framework extraction"
    );
}
