#[test]
fn shared_host_page_surface_replaces_legacy_direct_click_route() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    assert!(
        !workbench.contains("clicked => { root.activate_host_page(page.id); }"),
        "workbench shell still exposes legacy direct host page callback"
    );
    assert!(
        workbench.contains("callback host_page_pointer_clicked(")
            || host_context.contains("callback host_page_pointer_clicked("),
        "workbench shell must expose shared host page pointer callback"
    );
    assert!(
        chrome.contains("callback pointer_pressed("),
        "DockTabButton should expose raw pointer coordinates for shared hit-test"
    );
    assert!(
        !app.contains("ui.on_activate_host_page("),
        "slint host app should no longer register direct host page activation callback"
    );
    assert!(
        app.contains("host_shell.on_host_page_pointer_clicked(")
            || wiring.contains("host_shell.on_host_page_pointer_clicked("),
        "slint host app must register shared host page pointer callback"
    );
}
