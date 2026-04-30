fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn pane_surface_actions_use_generic_rust_control_callbacks() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let actions = source("src/ui/slint_host/app/pane_surface_actions.rs");

    assert!(globals.contains("on_surface_control_clicked"));
    assert!(wiring.contains("pane_surface_host.on_surface_control_clicked("));
    assert!(actions.contains("dispatch_pane_surface_control_clicked"));
    for legacy in ["on_menu_action", "handle_menu_action"] {
        assert!(!wiring.contains(legacy) && !actions.contains(legacy));
    }
}
