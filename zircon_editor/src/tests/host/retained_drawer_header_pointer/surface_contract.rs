fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_drawer_header_callbacks_consume_the_committed_native_receipt() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");

    assert!(globals.contains("on_drawer_header_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_drawer_header_pointer_clicked("));
    assert!(pointer_layout.contains("build_host_drawer_header_pointer_layout(model)"));
    assert!(!wiring.contains("on_toggle_drawer_tab"));
}

#[test]
fn drawer_header_pointer_consumes_native_action_without_a_mirror_hit_surface() {
    let native =
        source("src/ui/retained_host/host_contract/native_pointer/routing/chrome/tabs/drawer.rs");
    let bridge =
        source("src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_bridge.rs");
    let click = source("src/ui/retained_host/drawer_header_pointer/handle_click.rs");

    assert!(native.contains("ChromePointerRoute::DrawerHeaderTab"));
    assert!(native.contains("contains(&tab_frame, x, y)"));
    assert!(bridge.contains("route_for_receipt"));
    assert!(bridge.contains("target_for_route"));
    for forbidden in [
        "UiSurface",
        "UiPointerDispatcher",
        "measured_frames",
        "route_intents",
        "UiPointerEvent",
        "dispatch_event",
    ] {
        assert!(
            !bridge.contains(forbidden) && !click.contains(forbidden),
            "drawer header receipt path must not retain mirror marker `{forbidden}`"
        );
    }
}

#[test]
fn drawer_content_consumers_use_componentized_workbench_layout_frames() {
    let helpers = source(
        "src/ui/retained_host/app/helpers/callback_surface/surface_size/host_frames/workbench.rs",
    );
    let viewport = source("src/ui/retained_host/app/viewport/toolbar_pointer/size.rs");
    let layout_frames =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/layout_frames.rs");
    let root_shell_frames = source(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs",
    );
    let componentized_window = source(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs",
    );

    assert!(layout_frames.contains("left_drawer_content_frame"));
    assert!(layout_frames.contains("pub(crate) fn drawer_content_frame(&self"));
    assert!(!root_shell_frames.contains("left_drawer_content_frame"));
    assert!(!root_shell_frames.contains("fn drawer_content_frame"));
    assert!(componentized_window.contains("LEFT_DRAWER_CONTENT_CONTROL_ID"));
    assert!(helpers.contains("host.workbench_window_bridge.layout_frames()"));
    assert!(helpers.contains(".drawer_content_frame(region)"));
    assert!(viewport.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(viewport.contains(".drawer_content_frame(region)"));
    assert!(viewport.contains(".drawer_shell_frame(region)"));
    assert!(!viewport.contains("root_shell_frames_with_componentized_drawers();"));
}

#[test]
fn drawer_header_receipt_route_is_compact_and_identity_remains_typed() {
    let route =
        source("src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_route.rs");
    let item =
        source("src/ui/retained_host/drawer_header_pointer/host_drawer_header_pointer_item.rs");

    assert!(route.contains("Clone, Copy"));
    assert!(!route.contains("String"));
    assert!(item.contains("slot: ActivityDrawerSlot"));
    assert!(item.contains("instance_id: ViewInstanceId"));
}
