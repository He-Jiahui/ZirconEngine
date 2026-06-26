use super::*;

#[test]
fn binding_api_moves_under_binding_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let binding_mod_source = include_str!("../../binding/mod.rs");
    let interface_binding_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/binding/mod.rs");

    assert!(
        lib_source.contains("pub mod binding;"),
        "zircon_ui root should expose the binding namespace directly"
    );

    for required in [
        "UiBindingCall",
        "UiBindingParseError",
        "UiBindingValue",
        "UiEventBinding",
        "UiEventKind",
        "UiEventPath",
    ] {
        assert!(
            interface_binding_mod_source.contains(required),
            "zircon_runtime_interface::ui::binding should own neutral DTO `{required}`"
        );
        assert!(
            !binding_mod_source.contains(required),
            "zircon_ui::binding should not re-export interface DTO `{required}`"
        );
    }

    assert!(
        binding_mod_source.contains("UiEventRouter"),
        "zircon_ui::binding should expose runtime behavior service `UiEventRouter`"
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
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening binding specialist `{forbidden}`"
        );
    }
}

#[test]
fn event_ui_api_moves_under_event_ui_namespace() {
    let lib_source = include_str!("../../mod.rs");
    let event_ui_mod_source = include_str!("../../event_ui/mod.rs");
    let interface_event_ui_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/event_ui/mod.rs");

    assert!(
        lib_source.contains("pub mod event_ui;"),
        "zircon_ui root should expose the event_ui namespace directly"
    );

    for required in [
        "UiActionDescriptor",
        "UiControlRequest",
        "UiControlResponse",
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
            interface_event_ui_mod_source.contains(required),
            "zircon_runtime_interface::ui::event_ui should own neutral DTO `{required}`"
        );
        assert!(
            !event_ui_mod_source.contains(required),
            "zircon_ui::event_ui should not re-export interface DTO `{required}`"
        );
    }

    for required in ["UiEventManager"] {
        assert!(
            event_ui_mod_source.contains(required),
            "zircon_ui::event_ui should expose runtime behavior helper `{required}`"
        );
    }

    assert!(
        interface_event_ui_mod_source.contains("UiBindingCodec"),
        "zircon_runtime_interface::ui::event_ui should own neutral binding codec helper `UiBindingCodec`"
    );
    assert!(
        !event_ui_mod_source.contains("UiBindingCodec"),
        "zircon_ui::event_ui should not re-export interface binding codec helper `UiBindingCodec`"
    );

    for forbidden in [
        "UiActionDescriptor",
        "UiControlRequest",
        "UiControlResponse",
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
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening event_ui specialist `{forbidden}`"
        );
    }
}

#[test]
fn dispatch_root_stays_structural_after_folder_split() {
    let dispatch_mod_source = include_str!("../../dispatch/mod.rs");
    let interface_dispatch_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/dispatch/mod.rs");

    for required in ["mod navigation;", "mod pointer;"] {
        assert!(
            dispatch_mod_source.contains(required),
            "zircon_ui::dispatch root should keep structural module entry `{required}`"
        );
    }

    for required in [
        "UiNavigationDispatchContext",
        "UiNavigationDispatchEffect",
        "UiNavigationDispatchInvocation",
        "UiNavigationDispatchResult",
        "UiPointerDispatchContext",
        "UiPointerDispatchEffect",
        "UiPointerDispatchInvocation",
        "UiPointerDispatchResult",
        "UiPointerEvent",
    ] {
        assert!(
            interface_dispatch_mod_source.contains(required),
            "zircon_runtime_interface::ui::dispatch should keep neutral DTO export `{required}`"
        );
        assert!(
            !dispatch_mod_source.contains(required),
            "zircon_ui::dispatch root should not keep old-path DTO export `{required}`"
        );
    }

    for required in ["UiNavigationDispatcher", "UiPointerDispatcher"] {
        assert!(
            dispatch_mod_source.contains(required),
            "zircon_ui::dispatch root should keep runtime behavior export `{required}`"
        );
    }

    for forbidden in [
        "impl UiPointerDispatcher",
        "impl UiNavigationDispatcher",
        "type PointerHandler",
        "type NavigationHandler",
    ] {
        assert!(
            !dispatch_mod_source.contains(forbidden),
            "zircon_ui::dispatch root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn surface_root_stays_structural_after_folder_split() {
    let surface_mod_source = include_str!("../../surface/mod.rs");
    let interface_surface_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

    for required in ["mod render;", "mod surface;"] {
        assert!(
            surface_mod_source.contains(required),
            "zircon_ui::surface root should keep structural module entry `{required}`"
        );
    }

    for required in [
        "UiFocusState",
        "UiNavigationEventKind",
        "UiNavigationRoute",
        "UiNavigationState",
        "UiPointerButton",
        "UiPointerEventKind",
        "UiPointerRoute",
        "UiRenderCommand",
        "UiRenderCommandKind",
        "UiRenderExtract",
        "UiRenderList",
        "UiResolvedStyle",
        "UiVisualAssetRef",
    ] {
        assert!(
            interface_surface_mod_source.contains(required),
            "zircon_runtime_interface::ui::surface should keep neutral DTO export `{required}`"
        );
        assert!(
            !surface_mod_source.contains(required),
            "zircon_ui::surface root should not keep old-path DTO export `{required}`"
        );
    }

    for required in ["extract_ui_render_tree", "layout_text", "UiSurface"] {
        assert!(
            surface_mod_source.contains(required),
            "zircon_ui::surface root should keep runtime behavior export `{required}`"
        );
    }

    for forbidden in [
        "impl UiSurface",
        "fn resolve_command_kind",
        "struct UiNodeVisualData",
        "fn diff_nodes",
    ] {
        assert!(
            !surface_mod_source.contains(forbidden),
            "zircon_ui::surface root should not keep implementation detail `{forbidden}`"
        );
    }
}
