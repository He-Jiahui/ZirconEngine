fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn host_page_native_callback_carries_explicit_action_without_geometry() {
    let globals = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs");
    let native_route = source(
        "src/ui/retained_host/host_contract/native_pointer/routing/chrome/tabs/host_page.rs",
    );

    let callback_line = globals
        .lines()
        .find(|line| line.contains("on_host_page_pointer_clicked"))
        .expect("host page callback declaration");
    assert!(callback_line.contains("close: bool"));
    for forbidden in ["tab_x", "tab_width", "point_x", "point_y"] {
        assert!(!callback_line.contains(forbidden));
    }
    assert!(wiring.contains("host_page_pointer_clicked(tab_index, close)"));
    assert!(native_route.contains("contains(&tab.close_frame, x, y)"));
    assert!(native_route.contains("close: true"));
    assert!(native_route.contains("close: false"));
}

#[test]
fn host_page_pointer_bridge_is_a_typed_receipt_projection_only() {
    let bridge = source("src/ui/retained_host/host_page_pointer/host_page_pointer_bridge.rs");
    let builder =
        source("src/ui/retained_host/host_page_pointer/build_host_page_pointer_layout.rs");
    let item = source("src/ui/retained_host/host_page_pointer/host_page_pointer_item.rs");

    for forbidden in [
        "UiSurface",
        "UiPointerDispatcher",
        "EditorRouteIntentMap",
        "UiFrame",
        "measured_frames",
    ] {
        assert!(!bridge.contains(forbidden));
    }
    for forbidden in [
        "WorkbenchChromeMetrics",
        "UiFrame",
        "title",
        "allocate_host_page_tabs",
    ] {
        assert!(!builder.contains(forbidden));
    }
    assert!(item.contains("page_id: MainPageId"));
    assert!(item.contains("close_instance_id: Option<ViewInstanceId>"));
}
