#[test]
fn pane_surface_actions_use_generic_template_callbacks_instead_of_legacy_menu_action_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let host_workbench_surfaces = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_workbench_surfaces.slint"
    ));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let pane_surface_host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let asset_panes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/asset_panes.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let lifecycle = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/host_lifecycle.rs"
    ));

    for needle in [
        "callback menu_action(action_id: string);",
        "callback empty_action(action_id: string);",
        "trigger_action(action_id) => { root.empty_action(action_id); }",
        "open_assets() => { root.empty_action(\"OpenView.editor.assets\"); }",
    ] {
        let found =
            workbench.contains(needle) || assets.contains(needle) || asset_panes.contains(needle);
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
        pane_surface_host_context.contains("export global PaneSurfaceHostContext {"),
        "pane surface host context owner must keep the shared PaneSurfaceHostContext global"
    );
    assert!(
        !workbench.contains(
            "export { PaneSurfaceHostContext } from \"workbench/pane_surface_host_context.slint\";"
        ),
        "workbench shell should stop re-exporting the shared PaneSurfaceHostContext for host wiring"
    );
    assert!(!host_workbench_surfaces.contains("export { PaneSurfaceHostContext }"));

    for needle in [
        "trigger_action(action_id) => { PaneSurfaceHostContext.surface_control_clicked(\"TriggerAction\", action_id); }",
    ] {
        let found = pane_surface.contains(needle) || pane_content.contains(needle);
        assert!(
            found,
            "pane leaf surfaces are missing generic control route `{needle}`"
        );
    }
    assert!(
        pane_content.contains(
            "surface_control_clicked(control_id, action_id) => { PaneSurfaceHostContext.surface_control_clicked(control_id, action_id); }"
        ),
        "pane content is missing the direct project surface control route"
    );
    assert!(
        asset_panes.contains(
            "clicked => { root.surface_control_clicked(\"TriggerAction\", \"OpenView.editor.assets\"); }"
        ),
        "asset pane surfaces are missing the generic empty-state trigger action"
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
