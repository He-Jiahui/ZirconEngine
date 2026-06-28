fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn welcome_surface_controls_use_generic_rust_callbacks_and_toml_controls() {
    let globals = source("src/ui/retained_host/host_contract/globals/pane_context/callbacks.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring/pane_surface/welcome.rs");
    let controls = source("assets/ui/editor/host/startup_welcome_controls.zui");

    for required in ["on_welcome_control_changed", "on_welcome_control_clicked"] {
        assert!(
            globals.contains(required),
            "host globals missing `{required}`"
        );
        assert!(
            wiring.contains(required),
            "callback wiring missing `{required}`"
        );
    }
    for required in [
        "ProjectNameEdited",
        "LocationEdited",
        "CreateProject",
        "OpenExistingProject",
        "OpenRecentProject",
        "RemoveRecentProject",
    ] {
        assert!(
            controls.contains(required),
            "welcome controls asset missing `{required}`"
        );
    }
}
