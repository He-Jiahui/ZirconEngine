use super::*;

#[test]
fn root_surface_avoids_wildcard_flatten_for_namespace_owned_domains() {
    let lib_source = include_str!("../../mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let layout_mod_source = include_str!("../../layout/mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let layout_mod_source = include_str!("../../layout/mod.rs");
    let interface_layout_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/layout/mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let tree_mod_source = include_str!("../../tree/mod.rs");
    let interface_tree_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/tree/mod.rs");
    let interface_ui_tree_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/tree/node/ui_tree.rs");

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
        "UiRuntimeTreeLayoutExt",
        "UiRuntimeTreeRoutingExt",
    ] {
        assert!(
            tree_mod_source.contains(required),
            "zircon_ui::tree should expose runtime behavior helper `{required}`"
        );
    }

    for required in [
        "pub fn new(",
        "pub fn insert_root(",
        "pub fn insert_child(",
        "pub fn node(",
        "pub fn node_mut(",
    ] {
        assert!(
            interface_ui_tree_source.contains(required),
            "zircon_runtime_interface::ui::tree::UiTree should own base tree access method `{required}`"
        );
        assert!(
            !tree_mod_source.contains("UiRuntimeTreeAccessExt"),
            "zircon_ui::tree should not keep the old runtime-only base tree access extension"
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
    let lib_source = include_str!("../../mod.rs");
    let tree_mod_source = include_str!("../../tree/mod.rs");
    let interface_tree_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/tree/mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let surface_mod_source = include_str!("../../surface/mod.rs");
    let interface_surface_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let surface_mod_source = include_str!("../../surface/mod.rs");
    let interface_surface_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/surface/mod.rs");

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
    let lib_source = include_str!("../../mod.rs");
    let dispatch_mod_source = include_str!("../../dispatch/mod.rs");
    let interface_dispatch_mod_source =
        include_str!("../../../../../zircon_runtime_interface/src/ui/dispatch/mod.rs");

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
