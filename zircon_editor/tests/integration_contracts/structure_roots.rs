#[test]
fn editor_crate_root_keeps_editor_module_out_of_lib_rs() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("editor crate root");

    for forbidden in [
        "pub struct EditorModule",
        "impl EngineModule for EditorModule",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor/src/lib.rs to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_root_stays_structural_after_helper_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("slint_host")
            .join("callback_dispatch")
            .join("template_bridge")
            .join("mod.rs"),
    )
    .expect("template bridge mod");

    for required in [
        "mod projection_support;",
        "pub(crate) use projection_support::{",
    ] {
        assert!(
            source.contains(required),
            "expected template bridge root to wire `{required}` after helper split"
        );
    }

    for forbidden in [
        "fn build_bindings_by_id(",
        "fn binding_for_control(",
        "fn project_builtin_surface(",
        "fn load_builtin_runtime_projection(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected template bridge root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_workbench_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(template_bridge_root.join("workbench").join("mod.rs"))
        .expect("workbench owner root");

    assert!(
        !template_bridge_root.join("workbench.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "mod host_projection;",
        "mod root_shell_frames;",
        "pub(crate) use bridge::BuiltinHostWindowTemplateBridge;",
        "pub(crate) use root_shell_frames::BuiltinHostRootShellFrames;",
    ] {
        assert!(
            source.contains(required),
            "expected workbench owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinHostWindowTemplateBridge {",
        "pub(crate) enum BuiltinHostWindowTemplateBridgeError {",
        "pub(crate) struct BuiltinHostRootShellFrames {",
        "fn build_builtin_host_window_projection(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected workbench owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_workbench_drawer_source_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(
        template_bridge_root
            .join("workbench_drawer_source")
            .join("mod.rs"),
    )
    .expect("workbench drawer source owner root");

    assert!(
        !template_bridge_root.join("workbench_drawer_source.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench_drawer_source.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod control_ids;",
        "mod error;",
        "mod layout;",
        "mod source_frames;",
        "pub(crate) use bridge::BuiltinHostDrawerSourceTemplateBridge;",
        "pub(super) use error::BuiltinHostDrawerSourceTemplateBridgeError;",
    ] {
        assert!(
            source.contains(required),
            "expected workbench drawer source owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinHostDrawerSourceTemplateBridge {",
        "pub(crate) enum BuiltinHostDrawerSourceTemplateBridgeError {",
        "pub(crate) struct BuiltinHostDrawerSourceFrames {",
        "fn build_builtin_host_drawer_source_surface(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected workbench drawer source owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_layout_floating_window_owner_stays_structural_after_folder_split() {
    let layout_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("layout");

    let source = std::fs::read_to_string(layout_root.join("floating_window").join("mod.rs"))
        .expect("floating window owner root");

    assert!(
        !layout_root.join("floating_window.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window.rs to be deleted after folder split"
    );

    for required in [
        "mod dispatch;",
        "mod resolution;",
        "#[cfg(test)]",
        "mod tests;",
        "pub(crate) use dispatch::{",
        "pub(crate) use resolution::resolve_builtin_floating_window_close_instances;",
    ] {
        assert!(
            source.contains(required),
            "expected floating window owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_builtin_floating_window_focus(",
        "pub(crate) fn dispatch_builtin_floating_window_focus_for_source(",
        "pub(crate) fn resolve_builtin_floating_window_close_instances(",
        "pub(super) fn resolve_floating_window_focus_instance(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected floating window owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_viewport_toolbar_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source =
        std::fs::read_to_string(template_bridge_root.join("viewport_toolbar").join("mod.rs"))
            .expect("viewport toolbar owner root");

    assert!(
        !template_bridge_root.join("viewport_toolbar.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar.rs to be deleted after folder split"
    );

    for required in [
        "mod action_control;",
        "mod bridge;",
        "mod error;",
        "mod host_projection;",
        "pub(crate) use bridge::BuiltinViewportToolbarTemplateBridge;",
    ] {
        assert!(
            source.contains(required),
            "expected viewport toolbar owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) enum BuiltinViewportToolbarTemplateBridgeError {",
        "pub(crate) struct BuiltinViewportToolbarTemplateBridge {",
        "fn build_builtin_viewport_toolbar_host_projection(",
        "fn projection_control_for_action(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected viewport toolbar owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_floating_window_source_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(
        template_bridge_root
            .join("floating_window_source")
            .join("mod.rs"),
    )
    .expect("floating window source owner root");

    assert!(
        !template_bridge_root.join("floating_window_source.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "mod source_frames;",
        "mod surface;",
        "pub(crate) use bridge::BuiltinFloatingWindowSourceTemplateBridge;",
        "pub(crate) use source_frames::BuiltinFloatingWindowSourceFrames;",
    ] {
        assert!(
            source.contains(required),
            "expected floating window source owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinFloatingWindowSourceTemplateBridge {",
        "pub(crate) enum BuiltinFloatingWindowSourceTemplateBridgeError {",
        "pub(crate) struct BuiltinFloatingWindowSourceFrames {",
        "fn build_builtin_floating_window_source_surface(",
        "fn source_frames_from_surface(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected floating window source owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_asset_surface_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(template_bridge_root.join("asset_surface").join("mod.rs"))
        .expect("asset surface owner root");

    assert!(
        !template_bridge_root.join("asset_surface.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/asset_surface.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "pub(crate) use bridge::BuiltinAssetSurfaceTemplateBridge;",
    ] {
        assert!(
            source.contains(required),
            "expected asset surface owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinAssetSurfaceTemplateBridge {",
        "pub(crate) enum BuiltinAssetSurfaceTemplateBridgeError {",
        "fn binding_for_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected asset surface owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_inspector_surface_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(
        template_bridge_root
            .join("inspector_surface")
            .join("mod.rs"),
    )
    .expect("inspector surface owner root");

    assert!(
        !template_bridge_root.join("inspector_surface.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/inspector_surface.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "pub(crate) use bridge::BuiltinInspectorSurfaceTemplateBridge;",
    ] {
        assert!(
            source.contains(required),
            "expected inspector surface owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinInspectorSurfaceTemplateBridge {",
        "pub(crate) enum BuiltinInspectorSurfaceTemplateBridgeError {",
        "fn binding_for_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected inspector surface owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_pane_surface_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source = std::fs::read_to_string(template_bridge_root.join("pane_surface").join("mod.rs"))
        .expect("pane surface owner root");

    assert!(
        !template_bridge_root.join("pane_surface.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/pane_surface.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "pub(crate) use bridge::BuiltinPaneSurfaceTemplateBridge;",
    ] {
        assert!(
            source.contains(required),
            "expected pane surface owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinPaneSurfaceTemplateBridge {",
        "pub(crate) enum BuiltinPaneSurfaceTemplateBridgeError {",
        "fn binding_for_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected pane surface owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_template_bridge_welcome_surface_owner_stays_structural_after_folder_split() {
    let template_bridge_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch")
        .join("template_bridge");

    let source =
        std::fs::read_to_string(template_bridge_root.join("welcome_surface").join("mod.rs"))
            .expect("welcome surface owner root");

    assert!(
        !template_bridge_root.join("welcome_surface.rs").exists(),
        "expected flat zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/welcome_surface.rs to be deleted after folder split"
    );

    for required in [
        "mod bridge;",
        "mod error;",
        "pub(crate) use bridge::BuiltinWelcomeSurfaceTemplateBridge;",
    ] {
        assert!(
            source.contains(required),
            "expected welcome surface owner root to wire `{required}` after folder split"
        );
    }

    for forbidden in [
        "pub(crate) struct BuiltinWelcomeSurfaceTemplateBridge {",
        "pub(crate) enum BuiltinWelcomeSurfaceTemplateBridgeError {",
        "fn binding_for_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected welcome surface owner root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_root_stays_structural_after_owner_split() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("mod.rs"))
        .expect("callback dispatch root");

    for required in [
        "mod asset;",
        "mod layout;",
        "mod shared_pointer;",
        "mod template_bridge;",
        "mod viewport;",
        "mod workbench;",
        "pub(crate) use asset::{dispatch_builtin_asset_surface_control, dispatch_mesh_import_path_edit};",
        "pub(crate) use layout::{",
        "pub(crate) use shared_pointer::{",
        "pub(crate) use template_bridge::{",
        "pub(crate) use viewport::{",
        "pub(crate) use workbench::dispatch_host_menu_action_with_template_fallback;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch root to wire `{required}` after owner split"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_builtin_asset_surface_control(",
        "pub(crate) fn dispatch_layout_command(",
        "pub(crate) fn dispatch_shared_menu_pointer_click(",
        "pub(crate) fn dispatch_viewport_event(",
        "pub(crate) struct BuiltinHostWindowTemplateBridge {",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_asset_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("asset").join("mod.rs"))
        .expect("callback dispatch asset root");

    for required in [
        "mod mesh_import_path;",
        "mod search;",
        "mod selection;",
        "mod surface_control;",
        "pub(crate) use mesh_import_path::dispatch_mesh_import_path_edit;",
        "pub(crate) use surface_control::dispatch_builtin_asset_surface_control;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch asset root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_mesh_import_path_edit(",
        "pub(crate) fn dispatch_asset_search(",
        "pub(crate) fn dispatch_asset_item_selection(",
        "pub(crate) fn dispatch_builtin_asset_surface_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch asset root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_inspector_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("inspector").join("mod.rs"))
        .expect("callback dispatch inspector root");

    for required in [
        "#[cfg(test)]",
        "mod apply;",
        "mod delete_selected;",
        "mod draft_field;",
        "mod surface_control;",
        "pub(crate) use surface_control::dispatch_builtin_inspector_surface_control;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch inspector root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_inspector_apply(",
        "pub(crate) fn dispatch_inspector_delete_selected(",
        "pub(crate) fn dispatch_inspector_draft_field(",
        "pub(crate) fn dispatch_builtin_inspector_surface_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch inspector root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_layout_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("layout").join("mod.rs"))
        .expect("callback dispatch layout root");

    for required in [
        "mod command;",
        "mod document_tab;",
        "mod drawer_toggle;",
        "mod floating_window;",
        "mod main_page;",
        "mod tab_drop;",
        "pub(crate) use command::dispatch_layout_command;",
        "pub(crate) use floating_window::{",
        "pub(crate) use tab_drop::dispatch_tab_drop;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch layout root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_layout_command(",
        "pub(crate) fn dispatch_builtin_host_document_tab_activation(",
        "pub(crate) fn dispatch_builtin_host_drawer_toggle(",
        "pub(crate) fn dispatch_builtin_host_page_activation(",
        "pub(crate) fn dispatch_tab_drop(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch layout root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_shared_pointer_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source =
        std::fs::read_to_string(callback_dispatch_root.join("shared_pointer").join("mod.rs"))
            .expect("callback dispatch shared pointer root");

    for required in [
        "mod activity_rail;",
        "mod asset_content;",
        "mod asset_reference;",
        "mod asset_tree;",
        "mod document_tab;",
        "mod drawer_header;",
        "mod hierarchy;",
        "mod host_page;",
        "mod menu;",
        "mod viewport_toolbar;",
        "mod welcome_recent;",
        "pub(crate) use menu::dispatch_shared_menu_pointer_click;",
        "pub(crate) use viewport_toolbar::dispatch_shared_viewport_toolbar_pointer_click;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch shared pointer root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_shared_activity_rail_pointer_click(",
        "pub(crate) fn dispatch_shared_document_tab_pointer_click(",
        "pub(crate) fn dispatch_shared_menu_pointer_click(",
        "pub(crate) fn dispatch_shared_viewport_toolbar_pointer_click(",
        "pub(crate) fn dispatch_shared_welcome_recent_pointer_click(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch shared pointer root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_viewport_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("viewport").join("mod.rs"))
        .expect("callback dispatch viewport root");

    for required in [
        "mod bridge;",
        "mod command_dispatch;",
        "mod pointer_dispatch;",
        "mod route_mapping;",
        "mod snap_cycle;",
        "mod toolbar_control;",
        "pub(crate) use bridge::SharedViewportPointerBridge;",
        "pub(crate) use command_dispatch::{dispatch_viewport_event, viewport_event_from_command};",
        "pub(crate) use pointer_dispatch::dispatch_viewport_pointer_event;",
        "pub(crate) use route_mapping::dispatch_viewport_toolbar_pointer_route;",
        "pub(crate) use toolbar_control::dispatch_builtin_viewport_toolbar_control;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch viewport root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) struct SharedViewportPointerBridge {",
        "pub(crate) fn dispatch_viewport_event(",
        "pub(crate) fn dispatch_viewport_pointer_event(",
        "pub(crate) fn dispatch_viewport_toolbar_pointer_route(",
        "pub(crate) fn dispatch_builtin_viewport_toolbar_control(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch viewport root to stay structural, found `{forbidden}`"
        );
    }
}

#[test]
fn slint_callback_dispatch_workbench_root_stays_structural() {
    let callback_dispatch_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("slint_host")
        .join("callback_dispatch");

    let source = std::fs::read_to_string(callback_dispatch_root.join("workbench").join("mod.rs"))
        .expect("callback dispatch workbench root");

    for required in [
        "mod control;",
        "mod menu_action;",
        "pub(crate) use menu_action::dispatch_host_menu_action_with_template_fallback;",
    ] {
        assert!(
            source.contains(required),
            "expected callback dispatch workbench root to wire `{required}`"
        );
    }

    for forbidden in [
        "pub(crate) fn dispatch_builtin_host_control(",
        "pub(crate) fn dispatch_builtin_host_menu_action(",
        "pub(crate) fn dispatch_menu_action(",
        "pub(crate) fn dispatch_host_menu_action_with_template_fallback(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected callback dispatch workbench root to stay structural, found `{forbidden}`"
        );
    }
}
