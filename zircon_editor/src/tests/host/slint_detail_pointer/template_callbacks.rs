#[test]
fn inspector_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback inspector_name_edited(",
        "callback inspector_parent_edited(",
        "callback inspector_x_edited(",
        "callback inspector_y_edited(",
        "callback inspector_z_edited(",
        "callback inspector_apply()",
        "callback delete_selected()",
        "inspector_name_edited(value) =>",
        "inspector_parent_edited(value) =>",
        "inspector_x_edited(value) =>",
        "inspector_y_edited(value) =>",
        "inspector_z_edited(value) =>",
        "inspector_apply() =>",
        "delete_selected() =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy inspector callback `{needle}`"
        );
    }

    for needle in [
        "callback inspector_name_edited(value: string);",
        "callback inspector_parent_edited(value: string);",
        "callback inspector_x_edited(value: string);",
        "callback inspector_y_edited(value: string);",
        "callback inspector_z_edited(value: string);",
        "callback inspector_apply();",
        "callback delete_selected();",
        "edited(value) => { root.inspector_name_edited(value); }",
        "edited(value) => { root.inspector_parent_edited(value); }",
        "edited(value) => { root.inspector_x_edited(value); }",
        "edited(value) => { root.inspector_y_edited(value); }",
        "edited(value) => { root.inspector_z_edited(value); }",
        "clicked => { root.inspector_apply(); }",
        "clicked => { root.delete_selected(); }",
    ] {
        assert!(
            !pane_content.contains(needle),
            "inspector pane still exposes legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_inspector_name_edited(",
        "ui.on_inspector_parent_edited(",
        "ui.on_inspector_x_edited(",
        "ui.on_inspector_y_edited(",
        "ui.on_inspector_z_edited(",
        "ui.on_inspector_apply(",
        "ui.on_delete_selected(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy inspector callback `{needle}`"
        );
    }
}
