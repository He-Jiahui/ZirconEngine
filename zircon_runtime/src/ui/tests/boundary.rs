use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn template_legacy_adapter_is_removed_from_formal_namespace_surface() {
    let lib_source = include_str!("../mod.rs");
    let template_mod_source = include_str!("../template/mod.rs");
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        lib_source.contains("pub mod template;"),
        "zircon_ui root should expose the template namespace directly"
    );

    assert!(
        interface_template_mod_source.contains("UiTemplateDocument"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiTemplateDocument`"
    );
    assert!(
        !template_mod_source.contains("UiTemplateDocument"),
        "zircon_ui::template should not re-export interface DTO `UiTemplateDocument`"
    );

    for required in ["UiTemplateLoader"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior service `{required}`"
        );
    }

    for forbidden in [
        "UiLegacyTemplateAdapter",
        "UiTemplateDocument",
        "UiTemplateLoader",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_ui root should stop flattening template boundary type `{forbidden}`"
        );
    }

    assert!(
        !template_mod_source.contains("UiLegacyTemplateAdapter"),
        "zircon_ui::template should drop the legacy template adapter from the formal surface"
    );
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiTemplateError"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for required in [
        "UiTemplateBuildError",
        "UiTemplateSurfaceBuilder",
        "UiTemplateTreeBuilder",
        "UiTemplateValidator",
    ] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior service `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiTemplateNode"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiTemplateNode`"
    );
    assert!(
        !template_mod_source.contains("UiTemplateNode"),
        "zircon_ui::template should not re-export interface DTO `UiTemplateNode`"
    );

    for required in ["UiTemplateInstance"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime behavior model `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiComponentDefinition",
        "UiComponentParamSchema",
        "UiNamedSlotSchema",
        "UiStyleScope",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiSelector", "UiSelectorToken"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral selector DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface selector DTO `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiActionRef",
        "UiBindingRef",
        "UiComponentTemplate",
        "UiSlotTemplate",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiAssetHeader", "UiAssetImports"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
        );
    }

    for required in ["UiAssetNodeIter", "UiNodeParent"] {
        assert!(
            template_mod_source.contains(required),
            "zircon_ui::template should expose runtime document helper `{required}`"
        );
    }

    for forbidden in [
        "UiAssetHeader",
        "UiAssetImports",
        "UiAssetNodeIter",
        "UiNodeParent",
    ] {
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiChildMount"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiChildMount`"
    );

    assert!(
        !template_mod_source.contains("UiChildMount"),
        "zircon_ui::template should not re-export interface DTO `UiChildMount`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in [
        "UiAssetError",
        "UiStyleDeclarationBlock",
        "UiStyleRule",
        "UiStyleSheet",
    ] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    for required in ["UiNodeDefinition", "UiNodeDefinitionKind"] {
        assert!(
            interface_template_mod_source.contains(required),
            "zircon_runtime_interface::ui::template should own neutral DTO `{required}`"
        );
        assert!(
            !template_mod_source.contains(required),
            "zircon_ui::template should not re-export interface DTO `{required}`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiAssetKind"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiAssetKind`"
    );

    assert!(
        !template_mod_source.contains("UiAssetKind"),
        "zircon_ui::template should not re-export interface DTO `UiAssetKind`"
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
    let interface_template_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/template/mod.rs");

    assert!(
        interface_template_mod_source.contains("UiAssetDocument"),
        "zircon_runtime_interface::ui::template should own neutral DTO `UiAssetDocument`"
    );

    assert!(
        !template_mod_source.contains("UiAssetDocument,"),
        "zircon_ui::template should not re-export interface DTO `UiAssetDocument`"
    );

    assert!(
        template_mod_source.contains("UiAssetDocumentRuntimeExt"),
        "zircon_ui::template should expose runtime document behavior `UiAssetDocumentRuntimeExt`"
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
    let interface_layout_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/layout/mod.rs");

    for required in ["AxisConstraint", "LayoutBoundary", "StretchMode"] {
        assert!(
            interface_layout_mod_source.contains(required),
            "zircon_runtime_interface::ui::layout should own neutral DTO `{required}`"
        );
        assert!(
            !layout_mod_source.contains(required),
            "zircon_ui::layout should not re-export interface DTO `{required}`"
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
    let interface_tree_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/tree/mod.rs");

    assert!(
        lib_source.contains("pub mod tree;"),
        "zircon_ui root should expose the tree namespace directly"
    );

    for required in [
        "UiTemplateNodeMetadata",
        "UiTreeError",
        "UiDirtyFlags",
        "UiLayoutCache",
        "UiTree",
        "UiTreeNode",
    ] {
        assert!(
            interface_tree_mod_source.contains(required),
            "zircon_runtime_interface::ui::tree should own neutral DTO `{required}`"
        );
        assert!(
            !tree_mod_source.contains(required),
            "zircon_ui::tree should not re-export interface DTO `{required}`"
        );
    }

    for required in [
        "UiHitTestIndex",
        "UiHitTestResult",
        "UiRuntimeTreeAccessExt",
        "UiRuntimeTreeLayoutExt",
        "UiRuntimeTreeRoutingExt",
    ] {
        assert!(
            tree_mod_source.contains(required),
            "zircon_ui::tree should expose runtime behavior helper `{required}`"
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
    let interface_tree_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/tree/mod.rs");

    assert!(
        interface_tree_mod_source.contains("UiInputPolicy"),
        "zircon_runtime_interface::ui::tree should own `UiInputPolicy`"
    );

    assert!(
        !tree_mod_source.contains("UiInputPolicy"),
        "zircon_ui::tree should not re-export interface input policy DTO"
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
    let interface_surface_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

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
            interface_surface_mod_source.contains(required),
            "zircon_runtime_interface::ui::surface should own neutral render DTO `{required}`"
        );
        assert!(
            !surface_mod_source.contains(required),
            "zircon_ui::surface should not re-export interface render DTO `{required}`"
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
    let interface_surface_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

    for required in ["UiFocusState", "UiNavigationState"] {
        assert!(
            interface_surface_mod_source.contains(required),
            "zircon_runtime_interface::ui::surface should own neutral state DTO `{required}`"
        );
        assert!(
            !surface_mod_source.contains(required),
            "zircon_ui::surface should not re-export interface state DTO `{required}`"
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
    let interface_dispatch_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/dispatch/mod.rs");

    assert!(
        lib_source.contains("pub mod dispatch;"),
        "zircon_ui root should expose the dispatch namespace directly"
    );

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
            "zircon_runtime_interface::ui::dispatch should own neutral DTO `{required}`"
        );
        assert!(
            !dispatch_mod_source.contains(required),
            "zircon_ui::dispatch should not re-export interface DTO `{required}`"
        );
    }

    for required in ["UiNavigationDispatcher", "UiPointerDispatcher"] {
        assert!(
            dispatch_mod_source.contains(required),
            "zircon_ui::dispatch should expose runtime behavior service `{required}`"
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
    let interface_binding_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/binding/mod.rs");

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
    let lib_source = include_str!("../mod.rs");
    let event_ui_mod_source = include_str!("../event_ui/mod.rs");
    let interface_event_ui_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/event_ui/mod.rs");

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

    for required in ["UiBindingCodec", "UiEventManager"] {
        assert!(
            event_ui_mod_source.contains(required),
            "zircon_ui::event_ui should expose runtime behavior helper `{required}`"
        );
    }

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
    let dispatch_mod_source = include_str!("../dispatch/mod.rs");
    let interface_dispatch_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/dispatch/mod.rs");

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
    let surface_mod_source = include_str!("../surface/mod.rs");
    let interface_surface_mod_source =
        include_str!("../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

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

#[test]
fn runtime_ui_entry_assets_do_not_live_under_src() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_root = manifest_dir.join("src");
    let offending = collect_ui_toml_files(&src_root);

    assert!(
        offending.is_empty(),
        "production runtime ui entry assets must not live under `src/`: {}",
        format_paths(&offending, manifest_dir)
    );
}

#[test]
fn legacy_runtime_fixture_source_directory_is_removed() {
    let legacy_fixture_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/runtime_ui/fixtures");
    assert!(
        !legacy_fixture_dir.exists(),
        "runtime fixture source directory must stay removed after the assets/ cutover: {}",
        legacy_fixture_dir.display()
    );
}

#[test]
fn runtime_fixture_assets_live_under_crate_assets() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest_dir.join("assets/ui/runtime/fixtures");
    let actual_files = collect_ui_toml_files(&fixture_root);

    let expected_files = vec![
        "assets/ui/runtime/fixtures/hud_overlay.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/inventory_list.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/pause_menu.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/quest_log_dialog.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/settings_dialog.ui.toml".to_string(),
    ];

    assert_eq!(
        rel_paths(&actual_files, manifest_dir),
        expected_files,
        "runtime fixtures should live exclusively under crate assets/"
    );
}

#[test]
fn runtime_fixture_loader_stays_on_asset_paths() {
    let fixture_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/runtime_ui/runtime_ui_fixture.rs"),
    )
    .expect("runtime_ui_fixture.rs should be readable");

    for required in [
        "fn relative_asset_path",
        "fn asset_path",
        "use crate::asset::runtime_asset_path",
        "runtime_asset_path(self.relative_asset_path())",
    ] {
        assert!(
            fixture_source.contains(required),
            "runtime fixture loader should keep asset-path helper `{required}`"
        );
    }

    for forbidden in ["fn source(", "include_str!"] {
        assert!(
            !fixture_source.contains(forbidden),
            "runtime fixture loader should not keep source-embedded entry helper `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_manager_loads_fixture_documents_from_asset_files() {
    let manager_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/runtime_ui/runtime_ui_manager.rs"),
    )
    .expect("runtime_ui_manager.rs should be readable");

    assert!(
        manager_source.contains("UiAssetLoader::load_toml_file(fixture.asset_path())"),
        "runtime ui manager should load fixture documents directly from asset files"
    );

    for forbidden in ["fixture.source()", "include_str!"] {
        assert!(
            !manager_source.contains(forbidden),
            "runtime ui manager should not regress to embedded fixture source `{forbidden}`"
        );
    }
}

fn collect_ui_toml_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_ui_toml_files_inner(root, &mut files);
    files.sort();
    files
}

fn collect_ui_toml_files_inner(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_ui_toml_files_inner(&path, files);
            continue;
        }

        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".ui.toml"))
        {
            files.push(path);
        }
    }
}

fn rel_paths(paths: &[PathBuf], base: &Path) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            relative_path(path, base)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

fn format_paths(paths: &[PathBuf], base: &Path) -> String {
    rel_paths(paths, base)
        .into_iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn relative_path(path: &Path, base: &Path) -> PathBuf {
    path.strip_prefix(base)
        .expect("path should stay under the manifest dir")
        .to_path_buf()
}
