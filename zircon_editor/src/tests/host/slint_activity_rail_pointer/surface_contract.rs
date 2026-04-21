#[test]
fn shared_activity_rail_surfaces_replace_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_context.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    let direct_toggle_count = workbench
        .matches("clicked => { root.toggle_drawer_tab(tab.slot, tab.id); }")
        .count();
    assert!(
        direct_toggle_count <= 3,
        "workbench shell still exposes legacy rail direct callback sites ({direct_toggle_count})"
    );

    for needle in [
        "callback activity_rail_pointer_clicked(",
        "activity_rail_pointer_clicked(",
    ] {
        assert!(
            workbench.contains(needle) || host_context.contains(needle),
            "workbench shell is missing shared activity rail pointer hook `{needle}`"
        );
    }

    assert!(
        app.contains("host_shell.on_activity_rail_pointer_clicked(")
            || wiring.contains("host_shell.on_activity_rail_pointer_clicked("),
        "slint host app must register shared activity rail pointer clicks"
    );
}
