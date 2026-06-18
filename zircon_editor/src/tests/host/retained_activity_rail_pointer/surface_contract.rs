fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_activity_rail_surfaces_use_rust_callbacks_and_toml_projection() {
    let globals = source("src/ui/retained_host/host_contract/globals.rs");
    let wiring = source("src/ui/retained_host/app/callback_wiring.rs");
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout.rs");
    let activity_rail_sync = pointer_layout
        .split("pub(super) fn sync_activity_rail_pointer_layout")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn sync_host_page_pointer_layout")
                .next()
        })
        .expect("activity rail pointer sync function should exist");
    let activity_rail_layout = source(
        "src/ui/retained_host/activity_rail_pointer/build_host_activity_rail_pointer_layout.rs",
    );
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let activity_asset = source("assets/ui/editor/workbench_activity_rail.v2.ui.toml");

    assert!(globals.contains("on_activity_rail_pointer_clicked"));
    assert!(wiring.contains("host_shell.on_activity_rail_pointer_clicked("));
    assert!(activity_rail_sync
        .contains("build_host_activity_rail_pointer_layout_with_workbench_layout_frames("));
    assert!(activity_rail_sync.contains("self.workbench_window_bridge.layout_frames()"));
    assert!(!activity_rail_sync.contains("self.template_bridge.root_shell_frames()"));
    assert!(activity_rail_sync.contains("workbench_layout_frames,"));
    assert!(!pointer_layout.contains("root_shell_frames_with_componentized_drawers();"));
    assert!(activity_rail_layout.contains(".activity_rail_frame"));
    assert!(activity_rail_layout.contains(".drawer_shell_frame(ShellRegionId::Right)"));
    assert!(!activity_rail_layout.contains("resolve_root_activity_rail_frame"));
    assert!(!pointer_layout.contains("workbench_layout_frames.left_region_frame"));
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
