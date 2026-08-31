fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_document_tab_callbacks_consume_the_committed_receipt_projection() {
    let globals = source("src/ui/retained_host/host_contract/globals.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/shell_chrome.rs");
    let document_tab_sync = pointer_layout
        .split("fn sync_document_tab_pointer_layout")
        .nth(1)
        .and_then(|source| source.split("fn sync_drawer_header_pointer_layout").next())
        .expect("document tab receipt sync function should exist");

    for required in [
        "on_document_tab_pointer_clicked",
        "on_document_tab_close_pointer_clicked",
        "document_tab_pointer_clicked",
        "document_tab_close_pointer_clicked",
    ] {
        assert!(
            globals.contains(required) || wiring.contains(required),
            "missing `{required}`"
        );
    }
    assert!(document_tab_sync.contains("build_host_document_tab_pointer_layout(model)"));
    assert!(!document_tab_sync.contains("workbench_window_bridge.layout_frames()"));
    assert!(!document_tab_sync.contains("floating_window_projection_bundle"));
}

#[test]
fn document_tab_pointer_consumes_native_action_without_a_mirror_hit_surface() {
    let native_body = source(
        "src/ui/retained_host/host_contract/native_pointer/routing/chrome/tabs/document/body.rs",
    );
    let native_close = source(
        "src/ui/retained_host/host_contract/native_pointer/routing/chrome/tabs/document/close.rs",
    );
    let bridge =
        source("src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge.rs");
    let activate = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_activate.rs",
    );
    let close = source(
        "src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_bridge_close.rs",
    );

    assert!(native_body.contains("ChromePointerRoute::DocumentTab"));
    assert!(native_body.contains("close: false"));
    assert!(native_close.contains("ChromePointerRoute::DocumentTab"));
    assert!(native_close.contains("close: true"));
    assert!(bridge.contains("route_for_receipt"));
    assert!(bridge.contains("target_for_route"));
    for retired in [
        "UiSurface",
        "UiPointerDispatcher",
        "measured_frames",
        "route_intents",
        "UiPointerEvent",
        "dispatch_event",
    ] {
        assert!(
            !bridge.contains(retired) && !activate.contains(retired) && !close.contains(retired),
            "document tab receipt path must not retain mirror marker `{retired}`"
        );
    }
}

#[test]
fn document_tab_receipt_route_is_compact_and_identity_remains_typed() {
    let route =
        source("src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_route.rs");
    let item =
        source("src/ui/retained_host/document_tab_pointer/host_document_tab_pointer_item.rs");

    assert!(route.contains("Clone, Copy"));
    assert!(!route.contains("String"));
    assert!(item.contains("instance_id: ViewInstanceId"));
    assert!(!item.contains("instance_id: String"));
}
