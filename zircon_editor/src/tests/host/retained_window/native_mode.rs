fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn native_floating_window_mode_uses_rust_owned_host_window_contract() {
    let native_presentation = source("src/ui/retained_host/app/native_windows/presentation.rs");
    let native_store = source("src/ui/retained_host/app/native_windows/store.rs");
    let window_handle = source("src/ui/retained_host/host_contract/window/handle.rs");
    let window_snapshot =
        source("src/ui/retained_host/host_contract/window/presentation/snapshot.rs");
    let floating_data =
        source("src/ui/retained_host/host_contract/data/host_components/floating.rs");

    for required in [
        "configure_native_floating_window_presentation",
        "native_floating_window_mode = true",
        "native_floating_window_id",
        "native_surface_tree_id",
        "native_window_title",
        "native_window_bounds",
    ] {
        assert!(
            native_presentation.contains(required),
            "native window path missing `{required}`"
        );
    }
    for required in ["UiHostWindow::new()", "UiHostWindow::clone_strong"] {
        assert!(
            native_store.contains(required),
            "native window store missing `{required}`"
        );
    }
    for required in ["set_size", "is_maximized", "set_maximized"] {
        assert!(
            window_handle.contains(required),
            "UiHostWindow contract missing `{required}`"
        );
    }
    assert!(window_snapshot.contains("get_host_window_bootstrap"));
    assert!(floating_data.contains("pub(crate) struct HostNativeFloatingWindowSurfaceData"));
}
