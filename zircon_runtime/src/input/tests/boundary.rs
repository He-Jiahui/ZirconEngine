#[test]
fn input_protocol_types_live_in_runtime_input_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_mod_source = include_str!("../mod.rs");
    let input_runtime_source = include_str!("../runtime/default_input_manager.rs");
    let app_runtime_handler_source =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/application_handler.rs");
    let manager_mod_source = include_str!("../../core/manager/mod.rs");
    let manager_resolver_source = include_str!("../../core/manager/resolver.rs");

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
        input_runtime_source.contains("crate::core::framework::input"),
        "runtime input manager should source input contracts from zircon_framework"
    );
    assert!(
        !app_runtime_handler_source.contains("use crate::core::manager::{InputButton, InputEvent};"),
        "runtime app should import input protocol types from zircon_runtime::input instead of zircon_manager"
    );

    for forbidden in [
        "InputButton",
        "InputEvent",
        "InputEventRecord",
        "InputSnapshot",
    ] {
        assert!(
            !manager_mod_source.contains(forbidden),
            "core manager mod.rs should not re-export {forbidden} after framework migration"
        );
        assert!(
            !manager_resolver_source.contains(forbidden),
            "core manager resolver should not re-export {forbidden} after framework migration"
        );
    }

    assert!(
        manager_resolver_source.contains("crate::core::framework::input"),
        "core manager resolver should source input manager contracts from crate::core::framework"
    );
    assert!(
        !runtime_root.join("src/manager").exists(),
        "runtime root should delete the legacy manager module after framework extraction"
    );
}
