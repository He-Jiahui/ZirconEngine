fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn host_side_activity_rails_use_projected_toml_template_nodes() {
    let host_components = source("src/ui/slint_host/host_contract/data/host_components.rs");
    let chrome_projection = source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let scene_projection = source("src/ui/layouts/windows/workbench_host_window/scene_projection.rs");
    let activity_asset = source("assets/ui/editor/workbench_activity_rail.ui.toml");

    for required in ["rail_nodes", "rail_button_frames", "rail_active_control_id"] {
        assert!(host_components.contains(required), "side dock DTO missing `{required}`");
    }
    for required in [
        "activity_rail_nodes",
        "activity_rail_button_frames",
        "activity_rail_active_control_id",
    ] {
        assert!(chrome_projection.contains(required), "chrome projection missing `{required}`");
    }
    for required in ["activity_rail_nodes(", "activity_rail_button_frames("] {
        assert!(scene_projection.contains(required), "scene projection missing `{required}`");
    }
    for required in [
        "ActivityRailPanel",
        "ActivityRailButton0",
        "ActivityRailButtonLabel0",
        "ActivityRailButton1",
        "ActivityRailButtonLabel1",
    ] {
        assert!(activity_asset.contains(required), "activity rail asset missing `{required}`");
    }
}
