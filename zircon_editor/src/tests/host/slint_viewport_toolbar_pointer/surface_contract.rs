#[test]
fn shared_viewport_toolbar_surface_replaces_legacy_direct_click_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let pane_surface_host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let scene_viewport_toolbar = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/scene_viewport_toolbar.slint"
    ));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback set_tool(tool: string);",
        "callback set_transform_space(space: string);",
        "callback set_projection_mode(mode: string);",
        "callback align_view(orientation: string);",
        "callback set_display_mode(mode: string);",
        "callback set_grid_mode(mode: string);",
        "callback set_translate_snap(step: float);",
        "callback set_rotate_snap_degrees(step: float);",
        "callback set_scale_snap(step: float);",
        "callback set_preview_lighting(enabled: bool);",
        "callback set_preview_skybox(enabled: bool);",
        "callback set_gizmos_enabled(enabled: bool);",
        "callback frame_selection();",
        "callback viewport_set_tool(",
        "callback viewport_set_transform_space(",
        "callback viewport_set_projection_mode(",
        "callback viewport_align_view(",
        "callback viewport_set_display_mode(",
        "callback viewport_set_grid_mode(",
        "callback viewport_set_translate_snap(",
        "callback viewport_set_rotate_snap_degrees(",
        "callback viewport_set_scale_snap(",
        "callback viewport_set_preview_lighting(",
        "callback viewport_set_preview_skybox(",
        "callback viewport_set_gizmos_enabled(",
        "callback viewport_frame_selection(",
        "viewport_set_tool(tool) =>",
        "viewport_set_transform_space(space) =>",
        "viewport_set_projection_mode(mode) =>",
        "viewport_align_view(orientation) =>",
        "viewport_set_display_mode(mode) =>",
        "viewport_set_grid_mode(mode) =>",
        "viewport_set_translate_snap(step) =>",
        "viewport_set_rotate_snap_degrees(step) =>",
        "viewport_set_scale_snap(step) =>",
        "viewport_set_preview_lighting(enabled) =>",
        "viewport_set_preview_skybox(enabled) =>",
        "viewport_set_gizmos_enabled(enabled) =>",
        "viewport_frame_selection() =>",
        "clicked => { root.set_tool(\"Drag\"); }",
        "clicked => { root.set_transform_space(\"Local\"); }",
        "clicked => { root.frame_selection(); }",
    ] {
        assert!(
            !workbench.contains(needle),
            "viewport toolbar still exposes legacy direct control callback `{needle}`"
        );
    }

    assert!(
        pane_surface_host_context.contains("callback viewport_toolbar_pointer_clicked("),
        "pane surface host context should own the shared viewport toolbar callback"
    );
    assert!(
        scene_viewport_toolbar.contains("callback pointer_clicked("),
        "scene viewport toolbar should expose the shared pointer callback contract"
    );
    assert!(
        pane_surface.contains(
            "pointer_clicked(control_id, control_x, control_y, control_width, control_height, point_x, point_y) => { PaneSurfaceHostContext.viewport_toolbar_pointer_clicked(root.pane.id, control_id, control_x, control_y, control_width, control_height, point_x, point_y); }"
        ),
        "pane surface should route toolbar pointer clicks directly through PaneSurfaceHostContext"
    );
    assert!(
        chrome.contains("pointer_clicked(x, y) =>")
            || scene_viewport_toolbar.contains("pointer_clicked(x, y) =>"),
        "viewport toolbar leaf controls should still emit shared pointer clicks"
    );

    for needle in [
        "ui.on_viewport_set_tool(",
        "ui.on_viewport_set_transform_space(",
        "ui.on_viewport_set_projection_mode(",
        "ui.on_viewport_frame_selection(",
    ] {
        assert!(
            !app.contains(needle),
            "slint host app should no longer register direct viewport toolbar callback `{needle}`"
        );
    }

    assert!(
        wiring.contains("pane_surface_host.on_viewport_toolbar_pointer_clicked("),
        "slint host app must register shared viewport toolbar callback through PaneSurfaceHostContext"
    );
}
