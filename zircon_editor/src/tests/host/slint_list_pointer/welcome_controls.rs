#[test]
fn welcome_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
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
    let welcome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/welcome.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback welcome_project_name_edited(",
        "callback welcome_location_edited(",
        "callback welcome_create_project(",
        "callback welcome_open_existing_project(",
        "welcome_project_name_edited(value) =>",
        "welcome_location_edited(value) =>",
        "welcome_create_project() =>",
        "welcome_open_existing_project() =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy welcome business callback `{needle}`"
        );
    }

    for needle in [
        "callback project_name_edited(value: string);",
        "callback location_edited(value: string);",
        "callback create_project();",
        "callback open_existing_project();",
        "edited(value) => { root.project_name_edited(value); }",
        "edited(value) => { root.location_edited(value); }",
        "clicked => { root.create_project(); }",
        "clicked => { root.open_existing_project(); }",
    ] {
        assert!(
            !welcome.contains(needle),
            "welcome leaf surface still exposes legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_welcome_project_name_edited(",
        "ui.on_welcome_location_edited(",
        "ui.on_welcome_create_project(",
        "ui.on_welcome_open_existing_project(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy welcome control callback `{needle}`"
        );
    }

    for needle in [
        "callback welcome_control_changed(control_id: string, value: string);",
        "callback welcome_control_clicked(control_id: string);",
    ] {
        assert!(
            pane_surface_host_context.contains(needle),
            "pane surface host context is missing generic welcome control callback `{needle}`"
        );
        assert!(
            !pane_surface.contains(needle),
            "pane surface should not keep dead welcome callback bridge `{needle}`"
        );
    }

    for needle in [
        "control_changed(control_id, value) => { PaneSurfaceHostContext.welcome_control_changed(control_id, value); }",
        "control_clicked(control_id) => { PaneSurfaceHostContext.welcome_control_clicked(control_id); }",
    ] {
        assert!(
            pane_content.contains(needle),
            "pane content is missing generic welcome control route `{needle}`"
        );
    }

    for needle in [
        "callback control_changed(control_id: string, value: string);",
        "callback control_clicked(control_id: string);",
        "edited(value) => { root.control_changed(\"ProjectNameEdited\", value); }",
        "edited(value) => { root.control_changed(\"LocationEdited\", value); }",
        "clicked => { root.control_clicked(\"CreateProject\"); }",
        "clicked => { root.control_clicked(\"OpenExistingProject\"); }",
    ] {
        assert!(
            welcome.contains(needle),
            "welcome leaf surface is missing generic control route `{needle}`"
        );
    }

    for needle in [
        "pane_surface_host.on_welcome_control_changed(",
        "pane_surface_host.on_welcome_control_clicked(",
    ] {
        assert!(
            wiring.contains(needle),
            "slint host wiring is missing generic welcome control callback `{needle}`"
        );
    }
}
