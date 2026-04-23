#[test]
fn shared_drawer_header_surfaces_replace_legacy_direct_click_routes() {
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
    let host_surface_owners = concat!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_side_dock_surface.slint"
        )),
        "\n",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_bottom_dock_surface.slint"
        ))
    );
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    assert!(
        !workbench.contains("clicked => { root.toggle_drawer_tab(tab.slot, tab.id); }"),
        "workbench shell still exposes legacy direct drawer header callback"
    );
    for needle in [
        "callback drawer_header_pointer_clicked(",
        "pointer_clicked(x, y) =>",
    ] {
        assert!(
            workbench.contains(needle)
                || host_context.contains(needle)
                || chrome.contains(needle)
                || host_components.contains(needle)
                || host_surface_owners.contains(needle),
            "drawer header shared pointer hook `{needle}` is missing"
        );
    }
    assert!(
        !app.contains("ui.on_toggle_drawer_tab("),
        "slint host app should no longer register direct drawer toggle callback"
    );
    assert!(
        app.contains("host_shell.on_drawer_header_pointer_clicked(")
            || wiring.contains("host_shell.on_drawer_header_pointer_clicked("),
        "slint host app must register shared drawer header callback"
    );
}
