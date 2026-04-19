#[test]
fn legacy_template_compat_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    assert!(
        lib_source.contains("pub mod template;"),
        "zircon_ui root should expose the template namespace directly"
    );

    for required in [
        "UiLegacyTemplateAdapter",
        "UiTemplateDocument",
        "UiTemplateLoader",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiLegacyTemplateAdapter",
        "UiTemplateDocument",
        "UiTemplateLoader",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template compat type `{forbidden}`"
        );
    }
}

#[test]
fn template_compiler_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in [
        "UiCompiledDocument",
        "UiDocumentCompiler",
        "UiStyleResolver",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiCompiledDocument",
        "UiDocumentCompiler",
        "UiStyleResolver",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template compiler type `{forbidden}`"
        );
    }
}

#[test]
fn template_runtime_builder_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in [
        "UiTemplateBuildError",
        "UiTemplateError",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiTemplateBuildError",
        "UiTemplateError",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template runtime specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_runtime_model_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in ["UiTemplateInstance", "UiTemplateNode"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in ["UiTemplateInstance", "UiTemplateNode"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template runtime model `{forbidden}`"
        );
    }
}

#[test]
fn template_component_schema_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in [
        "UiComponentDefinition",
        "UiComponentParamSchema",
        "UiNamedSlotSchema",
        "UiStyleScope",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiComponentDefinition",
        "UiComponentParamSchema",
        "UiNamedSlotSchema",
        "UiStyleScope",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template component-schema surface `{forbidden}`"
        );
    }
}

#[test]
fn template_selector_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in ["UiSelector", "UiSelectorToken"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in ["UiSelector", "UiSelectorToken"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template selector surface `{forbidden}`"
        );
    }
}

#[test]
fn template_binding_model_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in [
        "UiActionRef",
        "UiBindingRef",
        "UiComponentTemplate",
        "UiSlotTemplate",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiActionRef",
        "UiBindingRef",
        "UiComponentTemplate",
        "UiSlotTemplate",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template binding model `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_metadata_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in ["UiAssetHeader", "UiAssetImports", "UiAssetRoot"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in ["UiAssetHeader", "UiAssetImports", "UiAssetRoot"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset metadata `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_mount_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    assert!(
        template_mod_source.contains("UiChildMount"),
        "zircon_ui::template should own `UiChildMount`"
    );

    assert!(
        !lib_source.contains("UiChildMount"),
        "zircon_ui root should stop flattening template asset mount `UiChildMount`"
    );
}

#[test]
fn template_asset_loader_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    assert!(
        template_mod_source.contains("UiAssetLoader"),
        "zircon_ui::template should own `UiAssetLoader`"
    );

    assert!(
        !lib_source.contains("UiAssetLoader"),
        "zircon_ui root should stop flattening template asset loader `UiAssetLoader`"
    );
}

#[test]
fn template_asset_style_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in [
        "UiAssetError",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in [
        "UiAssetError",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset style specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_node_definition_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    for required in ["UiNodeDefinition", "UiNodeDefinitionKind"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should own `{required}`"
        );
    }

    for forbidden in ["UiNodeDefinition", "UiNodeDefinitionKind"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template asset node specialist `{forbidden}`"
        );
    }
}

#[test]
fn template_asset_kind_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    assert!(
        template_mod_source.contains("UiAssetKind"),
        "zircon_ui::template should own `UiAssetKind`"
    );

    assert!(
        !lib_source.contains("UiAssetKind"),
        "zircon_ui root should stop flattening template asset kind `UiAssetKind`"
    );
}

#[test]
fn template_asset_document_api_moves_under_template_namespace() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");

    assert!(
        template_mod_source.contains("UiAssetDocument"),
        "zircon_ui::template should own `UiAssetDocument`"
    );

    assert!(
        !lib_source.contains("UiAssetDocument"),
        "zircon_ui root should stop flattening template asset document `UiAssetDocument`"
    );
}

#[test]
fn root_surface_avoids_wildcard_flatten_for_namespace_owned_domains() {
    let lib_source = include_str!("../mod.rs");

    for forbidden in [
        "pub use dispatch::*;",
        "pub use layout::*;",
        "pub use surface::*;",
        "pub use template::*;",
        "pub use tree::*;",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop wildcard-flattening namespace-owned surface `{forbidden}`"
        );
    }
}

#[test]
fn layout_solver_api_moves_under_layout_namespace() {
    let lib_source = include_str!("../mod.rs");
    let layout_mod_source = include_str!("../layout/mod.rs");

    assert!(
        lib_source.contains("pub mod layout;"),
        "zircon_ui root should expose the layout namespace directly"
    );

    for required in [
        "solve_axis_constraints",
        "compute_layout_tree",
        "compute_virtual_list_window",
    ] {
        assert!(
            layout_mod_source.contains(required),
            "zircon_ui::layout should own `{required}`"
        );
    }

    for forbidden in [
        "compute_layout_tree",
        "compute_virtual_list_window",
        "solve_axis_constraints",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening layout solver `{forbidden}`"
        );
    }
}

#[test]
fn layout_constraint_model_api_moves_under_layout_namespace() {
    let lib_source = include_str!("../mod.rs");
    let layout_mod_source = include_str!("../layout/mod.rs");

    for required in ["AxisConstraint", "LayoutBoundary", "StretchMode"] {
        assert!(
            layout_mod_source.contains(required),
            "zircon_ui::layout should own `{required}`"
        );
    }

    for (forbidden, needle) in [
        ("AxisConstraint", " AxisConstraint,"),
        ("LayoutBoundary", " LayoutBoundary,"),
        ("StretchMode", " StretchMode,"),
    ] {
        assert!(
            !lib_source.contains(needle),
            "zircon_ui root should stop flattening layout constraint model `{forbidden}`"
        );
    }
}

#[test]
fn tree_specialist_api_moves_under_tree_namespace() {
    let lib_source = include_str!("../mod.rs");
    let tree_mod_source = include_str!("../tree/mod.rs");

    assert!(
        lib_source.contains("pub mod tree;"),
        "zircon_ui root should expose the tree namespace directly"
    );

    for required in [
        "UiTemplateNodeMetadata",
        "UiTreeError",
        "UiDirtyFlags",
        "UiLayoutCache",
        "UiHitTestIndex",
        "UiHitTestResult",
    ] {
        assert!(
            tree_mod_source.contains(required),
            "zircon_ui::tree should own `{required}`"
        );
    }

    for forbidden in [
        "UiTemplateNodeMetadata",
        "UiTreeError",
        "UiDirtyFlags",
        "UiLayoutCache",
        "UiHitTestIndex",
        "UiHitTestResult",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening tree specialist `{forbidden}`"
        );
    }
}

#[test]
fn tree_input_policy_api_moves_under_tree_namespace() {
    let lib_source = include_str!("../mod.rs");
    let tree_mod_source = include_str!("../tree/mod.rs");

    assert!(
        tree_mod_source.contains("UiInputPolicy"),
        "zircon_ui::tree should own `UiInputPolicy`"
    );

    assert!(
        !lib_source.contains("UiInputPolicy"),
        "zircon_ui root should stop flattening tree input policy `UiInputPolicy`"
    );
}

#[test]
fn surface_render_api_moves_under_surface_namespace() {
    let lib_source = include_str!("../mod.rs");
    let surface_mod_source = include_str!("../surface/mod.rs");

    assert!(
        lib_source.contains("pub mod surface;"),
        "zircon_ui root should expose the surface namespace directly"
    );

    for required in [
        "UiRenderCommand",
        "UiRenderCommandKind",
        "UiRenderExtract",
        "UiRenderList",
        "UiResolvedStyle",
        "UiVisualAssetRef",
    ] {
        assert!(
            surface_mod_source.contains(required),
            "zircon_ui::surface should own `{required}`"
        );
    }

    for forbidden in [
        "UiRenderCommand",
        "UiRenderCommandKind",
        "UiRenderExtract",
        "UiRenderList",
        "UiResolvedStyle",
        "UiVisualAssetRef",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening surface render specialist `{forbidden}`"
        );
    }
}

#[test]
fn surface_state_api_moves_under_surface_namespace() {
    let lib_source = include_str!("../mod.rs");
    let surface_mod_source = include_str!("../surface/mod.rs");

    for required in ["UiFocusState", "UiNavigationState"] {
        assert!(
            surface_mod_source.contains(required),
            "zircon_ui::surface should own `{required}`"
        );
    }

    for forbidden in ["UiFocusState", "UiNavigationState"] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening surface state `{forbidden}`"
        );
    }
}

#[test]
fn dispatch_api_moves_under_dispatch_namespace() {
    let lib_source = include_str!("../mod.rs");
    let dispatch_mod_source = include_str!("../dispatch/mod.rs");

    assert!(
        lib_source.contains("pub mod dispatch;"),
        "zircon_ui root should expose the dispatch namespace directly"
    );

    for required in [
        "UiNavigationDispatchContext",
        "UiNavigationDispatchEffect",
        "UiNavigationDispatchInvocation",
        "UiNavigationDispatchResult",
        "UiNavigationDispatcher",
        "UiPointerDispatchContext",
        "UiPointerDispatchEffect",
        "UiPointerDispatchInvocation",
        "UiPointerDispatchResult",
        "UiPointerDispatcher",
        "UiPointerEvent",
    ] {
        assert!(
            dispatch_mod_source.contains(required),
            "zircon_ui::dispatch should own `{required}`"
        );
    }

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
            !lib_source.contains(needle),
            "zircon_ui root should stop flattening dispatch specialist `{forbidden}`"
        );
    }
}

#[test]
fn binding_api_moves_under_binding_namespace() {
    let lib_source = include_str!("../mod.rs");
    let binding_mod_source = include_str!("../binding/mod.rs");

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
        "UiEventRouter",
    ] {
        assert!(
            binding_mod_source.contains(required),
            "zircon_ui::binding should own `{required}`"
        );
    }

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
    let lib_source = include_str!("../mod.rs");
    let event_ui_mod_source = include_str!("../event_ui/mod.rs");

    assert!(
        lib_source.contains("pub mod event_ui;"),
        "zircon_ui root should expose the event_ui namespace directly"
    );

    for required in [
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
            event_ui_mod_source.contains(required),
            "zircon_ui::event_ui should own `{required}`"
        );
    }

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
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening event_ui specialist `{forbidden}`"
        );
    }
}
