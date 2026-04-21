#[test]
fn runtime_ui_surface_keeps_binding_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod binding;"),
        "zircon_runtime::ui should expose the binding namespace directly"
    );
    assert!(
        !ui_mod_source.contains("pub use binding::*;"),
        "zircon_runtime::ui should stop wildcard-flattening the binding namespace"
    );

    for forbidden in [
        "UiBindingCall",
        "UiBindingParseError",
        "UiBindingValue",
        "UiEventBinding",
        "UiEventKind",
        "UiEventPath",
        "UiEventRouter",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening binding specialist `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_surface_keeps_event_ui_under_namespace() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_mod_source =
        std::fs::read_to_string(runtime_root.join("src/ui/mod.rs")).unwrap_or_default();

    assert!(
        ui_mod_source.contains("pub mod event_ui;"),
        "zircon_runtime::ui should expose the event_ui namespace directly"
    );
    assert!(
        !ui_mod_source.contains("pub use event_ui::*;"),
        "zircon_runtime::ui should stop wildcard-flattening the event_ui namespace"
    );

    for forbidden in [
        "UiActionDescriptor",
        "UiBindingCodec",
        "UiControlRequest",
        "UiControlResponse",
        "UiEventManager",
        "UiInvocationContext",
        "UiInvocationError",
        "UiInvocationRequest",
        "UiInvocationResponse",
        "UiInvocationResult",
        "UiInvocationSource",
        "UiNodeDescriptor",
        "UiNodeId",
        "UiNodePath",
        "UiNotification",
        "UiParameterDescriptor",
        "UiPropertyDescriptor",
        "UiReflectionDiff",
        "UiReflectionSnapshot",
        "UiRouteId",
        "UiStateFlags",
        "UiSubscriptionId",
        "UiTreeId",
        "UiValueType",
    ] {
        assert!(
            !ui_mod_source.contains(forbidden),
            "zircon_runtime::ui should stop flattening event_ui specialist `{forbidden}`"
        );
    }
}
