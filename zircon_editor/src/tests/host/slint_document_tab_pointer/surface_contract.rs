fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_document_tab_surfaces_use_rust_owned_pointer_callbacks() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");

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
    assert!(pointer_layout.contains("build_host_document_tab_pointer_layout("));
}
