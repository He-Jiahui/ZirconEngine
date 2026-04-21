#[test]
fn shared_document_tab_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let host_workbench_surfaces = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_workbench_surfaces.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "clicked => { root.activate_document_tab(tab.id); }",
        "close_clicked => { root.close_tab(tab.id); }",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy document tab callback `{needle}`"
        );
    }

    for needle in [
        "callback document_tab_pointer_clicked(",
        "callback document_tab_close_pointer_clicked(",
        "pointer_clicked(x, y) =>",
        "close_pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle)
                || host_context.contains(needle)
                || chrome.contains(needle)
                || host_components.contains(needle)
                || host_workbench_surfaces.contains(needle),
            "document tab shared pointer hook `{needle}` is missing"
        );
    }

    for needle in ["ui.on_activate_document_tab(", "ui.on_close_tab("] {
        assert!(
            !app.contains(needle),
            "slint host app should no longer register direct document tab callback `{needle}`"
        );
    }

    for needle in [
        "host_shell.on_document_tab_pointer_clicked(",
        "host_shell.on_document_tab_close_pointer_clicked(",
    ] {
        assert!(
            app.contains(needle) || wiring.contains(needle),
            "slint host app must register shared document tab callback `{needle}`"
        );
    }
}
