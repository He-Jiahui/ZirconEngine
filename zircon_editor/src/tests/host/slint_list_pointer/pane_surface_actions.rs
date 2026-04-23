#[test]
fn pane_surface_actions_use_generic_template_callbacks_instead_of_legacy_menu_action_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_surface_owners = concat!(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_side_dock_surface.slint"
        )),
        "\n",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_document_dock_surface.slint"
        )),
        "\n",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_bottom_dock_surface.slint"
        )),
        "\n",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_floating_window_layer.slint"
        )),
        "\n",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ui/workbench/host_native_floating_window_surface.slint"
        ))
    );
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
    let pane_surface_host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let lifecycle = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/host_lifecycle.rs"
    ));
    let pane_content_normalized = pane_content.split_whitespace().collect::<String>();
    let template_pane_normalized = template_pane.split_whitespace().collect::<String>();

    for needle in [
        "callback menu_action(action_id: string);",
        "callback empty_action(action_id: string);",
        "trigger_action(action_id) => { root.empty_action(action_id); }",
        "open_assets() => { root.empty_action(\"OpenView.editor.assets\"); }",
    ] {
        let found =
            workbench.contains(needle) || assets.contains(needle) || pane_content.contains(needle);
        assert!(
            !found,
            "pane surfaces still expose legacy menu-action callback `{needle}`"
        );
    }

    for needle in ["ui.on_menu_action(", "fn handle_menu_action("] {
        let found = wiring.contains(needle) || lifecycle.contains(needle);
        assert!(
            !found,
            "slint host still wires legacy menu-action callback `{needle}`"
        );
    }

    assert!(
        pane_surface_host_context
            .contains("callback surface_control_clicked(control_id: string, action_id: string);"),
        "pane surface host context is missing generic pane-surface control callback"
    );
    assert!(
        workbench.contains(
            "export { PaneSurfaceHostContext } from \"workbench/pane_surface_host_context.slint\";"
        ),
        "workbench shell should still export the shared PaneSurfaceHostContext for host wiring"
    );
    assert!(!host_surface_owners.contains("export { PaneSurfaceHostContext }"));

    for needle in [
        "trigger_action(action_id) => { PaneSurfaceHostContext.surface_control_clicked(\"TriggerAction\", action_id); }",
    ] {
        let found = pane_surface.contains(needle) || pane_content.contains(needle);
        assert!(
            found,
            "pane leaf surfaces are missing generic control route `{needle}`"
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
            "project template route is missing generic pane-surface control bridge `{needle}`"
        );
    }
    assert!(
        pane_content.contains(
            "trigger_action(action_id) => { PaneSurfaceHostContext.surface_control_clicked(\"TriggerAction\", action_id); }"
        ),
        "pane content is missing the generic empty-state trigger action bridge"
    );

    assert!(
        wiring.contains("let pane_surface_host = ui.global::<PaneSurfaceHostContext>();"),
        "slint host wiring must access the shared PaneSurfaceHostContext global"
    );
    assert!(
        wiring.contains("pane_surface_host.on_surface_control_clicked("),
        "slint host wiring is missing generic pane-surface control global callback"
    );
}
