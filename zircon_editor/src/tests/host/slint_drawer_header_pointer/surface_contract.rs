fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_drawer_header_surfaces_use_rust_owned_pointer_callbacks() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");

    assert!(globals.contains("on_drawer_header_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_drawer_header_pointer_clicked("));
    assert!(pointer_layout.contains("build_host_drawer_header_pointer_layout("));
    assert!(!wiring.contains("on_toggle_drawer_tab"));
}
