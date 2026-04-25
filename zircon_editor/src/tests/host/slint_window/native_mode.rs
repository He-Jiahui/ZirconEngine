#[test]
fn native_floating_window_mode_forwards_tabs_header_and_pane_callbacks_to_root() {
    let scaffold = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_scaffold.slint"
    ));
    let host_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_surface.slint"
    ));
    let host_native_floating_window_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_native_floating_window_surface.slint"
    ));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let template_pane = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));
    let pane_content_normalized = pane_content.split_whitespace().collect::<String>();
    let template_pane_normalized = template_pane.split_whitespace().collect::<String>();
    assert!(
        scaffold.contains("HostWindowSurfaceHost {"),
        "host scaffold should delegate window chrome switching to HostWindowSurfaceHost"
    );
    let native_wrapper_start = host_surface
        .find("export component HostNativeWindowSurface inherits Rectangle {")
        .expect("native floating wrapper should exist");
    let native_wrapper = &host_surface[native_wrapper_start..];
    let native_start = host_native_floating_window_surface
        .find("export component HostNativeFloatingWindowSurface inherits Rectangle {")
        .expect("native floating window component should exist");
    let native_mode = &host_native_floating_window_surface[native_start..];

    for needle in [
        "HostNativeFloatingWindowSurface {",
        "surface_data: root.surface_data;",
    ] {
        assert!(
            native_wrapper.contains(needle),
            "native floating wrapper is missing host-context forwarding `{needle}`"
        );
    }
    for removed_wrapper_forwarder in [
        "UiHostContext.document_tab_pointer_clicked(",
        "UiHostContext.document_tab_close_pointer_clicked(",
        "UiHostContext.floating_window_header_pointer_clicked(",
    ] {
        assert!(
            !native_wrapper.contains(removed_wrapper_forwarder),
            "native floating wrapper should stop forwarding behavior `{removed_wrapper_forwarder}`"
        );
    }

    for needle in [
        "closeable: tab.closeable;",
        "pointer_clicked(x, y) => {",
        "close_pointer_clicked(x, y) => {",
        "UiHostContext.document_tab_pointer_clicked(",
        "UiHostContext.document_tab_close_pointer_clicked(",
        "UiHostContext.floating_window_header_pointer_clicked(",
        "root.surface_data.native_window_bounds.x + self.x / 1px + self.mouse-x / 1px",
        "root.surface_data.native_window_bounds.y + self.mouse-y / 1px",
        "PaneSurface {",
    ] {
        assert!(
            native_mode.contains(needle),
            "native floating window mode is missing shared tab/header host behavior `{needle}`"
        );
    }

    assert!(
        !native_mode.contains("closeable: false;"),
        "native floating window mode should not hardcode tabs as non-closeable"
    );
    for removed_local_forwarder in [
        "root.document_tab_pointer_clicked(",
        "root.document_tab_close_pointer_clicked(",
        "root.floating_window_header_pointer_clicked(",
    ] {
        assert!(
            !native_mode.contains(removed_local_forwarder),
            "native floating window mode should stop using local forwarding callback `{removed_local_forwarder}`"
        );
    }
    for removed_forwarder in [
        "surface_control_clicked(control_id, action_id) => { root.pane_surface_control_clicked(control_id, action_id); }",
        "asset_tree_pointer_clicked(surface_mode, x, y, width, height) => { root.asset_tree_pointer_clicked(surface_mode, x, y, width, height); }",
        "hierarchy_pointer_scrolled(x, y, delta, width, height) => { root.hierarchy_pointer_scrolled(x, y, delta, width, height); }",
        "console_pointer_scrolled(x, y, delta, width, height) => { root.console_pointer_scrolled(x, y, delta, width, height); }",
        "inspector_pointer_scrolled(x, y, delta, width, height) => { root.inspector_pointer_scrolled(x, y, delta, width, height); }",
        "viewport_pointer_event(kind, button, x, y, delta) => { root.viewport_pointer_event(kind, button, x, y, delta); }",
        "ui_asset_action(instance_id, action_id) => { root.ui_asset_action(instance_id, action_id); }",
    ] {
        assert!(
            !native_mode.contains(removed_forwarder),
            "native floating window mode should not keep legacy pane callback forwarding `{removed_forwarder}`"
        );
    }
    for (owner, global_route) in [
        (
            "pane_content",
            "tree_pointer_clicked(x, y, width, height) => { PaneSurfaceHostContext.asset_tree_pointer_clicked(",
        ),
        (
            "pane_content",
            "pointer_scrolled(x, y, delta, width, height) => { PaneSurfaceHostContext.hierarchy_pointer_scrolled(x, y, delta, width, height); }",
        ),
        (
            "pane_content",
            "pointer_scrolled(x, y, delta, width, height) => { PaneSurfaceHostContext.console_pointer_scrolled(x, y, delta, width, height); }",
        ),
        ("pane_surface", "PaneSurfaceHostContext.viewport_pointer_event("),
        (
            "pane_content",
            "action(action_id) => { PaneSurfaceHostContext.ui_asset_action(root.pane.id, action_id); }",
        ),
    ] {
        assert!(
            match owner {
                "pane_surface" => pane_surface.contains(global_route),
                "pane_content" => pane_content.contains(global_route),
                _ => false,
            },
            "{owner} should route native floating pane interactions through PaneSurfaceHostContext via `{global_route}`"
        );
    }
    for needle in [
        "exportcomponentTemplatePaneinheritsRectangle{",
        "callbacknode_dispatched(control_id:string,dispatch_kind:string,action_id:string);",
        "if!root.pane.show_empty&&root.pane.kind==\"Project\":TemplatePane{",
        "node_dispatched(control_id,dispatch_kind,action_id)=>{",
        "if(dispatch_kind==\"surface\"){PaneSurfaceHostContext.surface_control_clicked(control_id,action_id);}",
    ] {
        let found = template_pane_normalized.contains(needle)
            || pane_content_normalized.contains(needle);
        assert!(
            found,
            "project template pane should route native floating interactions through PaneSurfaceHostContext via `{needle}`"
        );
    }
}
