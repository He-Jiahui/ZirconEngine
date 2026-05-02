fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_activity_rail_surfaces_use_rust_callbacks_and_toml_projection() {
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let wiring = source("src/ui/slint_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let activity_asset = source("assets/ui/editor/workbench_activity_rail.ui.toml");

    assert!(globals.contains("on_activity_rail_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_activity_rail_pointer_clicked("));
    assert!(pointer_layout.contains("build_host_activity_rail_pointer_layout("));
    for required in [
        "activity_rail_nodes",
        "activity_rail_button_frames",
        "activity_rail_active_control_id",
    ] {
        assert!(chrome_projection.contains(required), "missing `{required}`");
    }
    for required in [
        "ActivityRailPanel",
        "ActivityRailButton0",
        "ActivityRailButton1",
    ] {
        assert!(activity_asset.contains(required), "missing `{required}`");
    }
}
