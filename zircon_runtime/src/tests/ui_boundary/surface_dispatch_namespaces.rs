#[test]
fn runtime_ui_surface_keeps_surface_state_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod surface;"),
        "zircon_runtime::ui should expose the surface namespace directly"
    );

    for forbidden in ["UiFocusState", "UiNavigationState"] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening surface state `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_input_policy_under_tree_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod tree;"),
        "zircon_runtime::ui should expose the tree namespace directly"
    );

    assert!(
        !ui_mod_source.contains("UiInputPolicy"),
        "zircon_runtime::ui should stop flattening tree input policy `UiInputPolicy`"
    );
}

#[test]
fn runtime_ui_surface_keeps_dispatch_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod dispatch;"),
        "zircon_runtime::ui should expose the dispatch namespace directly"
    );

    for (forbidden, needle) in [
        ("UiNavigationDispatchContext", "UiNavigationDispatchContext"),
        ("UiNavigationDispatchEffect", "UiNavigationDispatchEffect"),
        (
            "UiNavigationDispatchInvocation",
            "UiNavigationDispatchInvocation",
        ),
        ("UiNavigationDispatchResult", "UiNavigationDispatchResult"),
        ("UiNavigationDispatcher", "UiNavigationDispatcher"),
        ("UiPointerDispatchContext", "UiPointerDispatchContext"),
        ("UiPointerDispatchEffect", "UiPointerDispatchEffect"),
        ("UiPointerDispatchInvocation", "UiPointerDispatchInvocation"),
        ("UiPointerDispatchResult", "UiPointerDispatchResult"),
        ("UiPointerDispatcher", "UiPointerDispatcher"),
        ("UiPointerEvent", "UiPointerEvent,"),
    ] {
        assert!(
            !ui_mod_source.contains(needle),
            "zircon_runtime::ui should stop flattening dispatch specialist `{forbidden}`"
        );
    }
}
