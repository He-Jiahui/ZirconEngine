#[test]
fn shared_list_surfaces_do_not_expose_legacy_direct_callback_routes() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let welcome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/welcome.slint"
    ));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app.rs"
    ));

    for needle in [
        "callback hierarchy_select(",
        "callback asset_select_folder(",
        "callback asset_select_item(",
        "callback asset_activate_reference(",
        "callback welcome_open_recent_project(",
        "callback welcome_remove_recent_project(",
        "hierarchy_select(node_id) =>",
        "asset_select_folder(folder_id) =>",
        "asset_select_item(asset_uuid) =>",
        "asset_activate_reference(asset_uuid) =>",
        "welcome_open_recent_project(path) =>",
        "welcome_remove_recent_project(path) =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback select(node_id: string);",
        "callback select_folder(folder_id: string);",
        "callback select_asset(asset_uuid: string);",
        "callback activate_reference(asset_uuid: string);",
        "select_folder(folder_id) =>",
        "select_asset(asset_uuid) =>",
        "activate_reference(asset_uuid) =>",
    ] {
        let found = assets.contains(needle) || pane_content.contains(needle);
        assert!(
            !found,
            "asset or pane leaf surface still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback open_recent_project(path: string);",
        "callback remove_recent_project(path: string);",
    ] {
        assert!(
            !welcome.contains(needle),
            "welcome leaf surface still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_hierarchy_select(",
        "ui.on_welcome_open_recent_project(",
        "ui.on_welcome_remove_recent_project(",
        "ui.on_asset_select_folder(",
        "ui.on_asset_select_item(",
        "ui.on_asset_activate_reference(",
        "fn select_hierarchy_node(",
        "fn select_asset_folder(",
        "fn select_asset_item(",
        "fn activate_asset_reference(",
    ] {
        assert!(
            !app.contains(needle),
            "slint host app still registers legacy direct callback `{needle}`"
        );
    }
}
