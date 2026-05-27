use super::sources::runtime_application_handler_source;

#[test]
fn runtime_viewport_interaction_is_owned_by_dynamic_runtime_session() {
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let runtime_construct_source = include_str!("../../runtime_entry_app/construct.rs");
    let runtime_handler_source = runtime_application_handler_source();

    assert!(
        runtime_app_source.contains("RuntimeSession"),
        "runtime entry app should own a runtime session wrapper"
    );
    assert!(
        !runtime_app_source.contains("mod camera_controller;"),
        "runtime camera control should live in zircon_runtime dynamic session state"
    );
    assert!(
        runtime_construct_source.contains("ZrRuntimeEventV1::viewport_resized"),
        "runtime entry construction should forward viewport changes through ABI events"
    );
    assert!(
        !runtime_app_source.contains("zircon_graphics::ViewportController"),
        "runtime entry app should not depend on zircon_graphics::ViewportController"
    );
    assert!(
        !runtime_construct_source
            .contains("zircon_graphics::{ViewportController, ViewportInput, ViewportState}"),
        "runtime construction should not import graphics viewport interaction types"
    );
    assert!(
        !runtime_handler_source.contains("use zircon_graphics::ViewportInput;"),
        "runtime window event handling should not import graphics viewport input types"
    );
}
