#[test]
fn input_protocol_types_live_in_input_subsystem() {
    let manager_crate_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../zircon_manager");
    let input_lib_source = include_str!("../lib.rs");
    let input_runtime_source = include_str!("../runtime/default_input_manager.rs");
    let entry_runtime_handler_source =
        include_str!("../../../zircon_app/src/entry/runtime_entry_app/application_handler.rs");
    let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../zircon_manager/src/resolver.rs");

    for required in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            input_lib_source.contains(required),
            "zircon_input root should publicly export {required}"
        );
    }

    assert!(
        input_runtime_source.contains("zircon_framework::input"),
        "zircon_input runtime should source input contracts from zircon_framework"
    );
    assert!(
        !entry_runtime_handler_source.contains("use zircon_manager::{InputButton, InputEvent};"),
        "runtime entry should import input protocol types from zircon_input instead of zircon_manager"
    );

    for forbidden in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            !manager_lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after input protocol migration"
        );
        assert!(
            !manager_resolver_source.contains(forbidden),
            "zircon_manager resolver should not re-export {forbidden} after input protocol migration"
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
