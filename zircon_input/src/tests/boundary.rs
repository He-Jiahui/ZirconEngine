#[test]
fn input_protocol_types_live_in_input_subsystem() {
    let manager_crate_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../zircon_manager");
    let input_lib_source = include_str!("../lib.rs");
    let input_runtime_source = include_str!("../runtime/default_input_manager.rs");
    let entry_runtime_handler_source =
        include_str!("../../../zircon_entry/src/entry/runtime_entry_app/application_handler.rs");
    let manager_lib_source = include_str!("../../../zircon_manager/src/lib.rs");
    let manager_records_mod_source = include_str!("../../../zircon_manager/src/records/mod.rs");
    let manager_input_source = manager_crate_root.join("src/records/input.rs");

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
        !input_runtime_source.contains("use zircon_manager::{"),
        "zircon_input runtime should not source input protocol types from zircon_manager"
    );
    assert!(
        !entry_runtime_handler_source.contains("use zircon_manager::{InputButton, InputEvent};"),
        "runtime entry should import input protocol types from zircon_input instead of zircon_manager"
    );

    for forbidden in ["InputButton", "InputEvent", "InputEventRecord", "InputSnapshot"] {
        assert!(
            !manager_lib_source.contains(forbidden),
            "zircon_manager lib.rs should not re-export {forbidden} after input protocol migration"
        );
        assert!(
            !manager_records_mod_source.contains(forbidden),
            "zircon_manager records mod should not re-export {forbidden} after input protocol migration"
        );
    }

    assert!(
        !manager_records_mod_source.contains("mod input;"),
        "zircon_manager records mod should not declare input records after input protocol migration"
    );
    assert!(
        !manager_input_source.exists(),
        "zircon_manager should delete src/records/input.rs after input protocol migration"
    );
}
