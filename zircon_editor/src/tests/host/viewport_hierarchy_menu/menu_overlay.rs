use super::super::support::*;

#[test]
fn rust_owned_host_painter_draws_open_menu_popup_above_pane_surfaces() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut closed = ui.get_host_presentation();
    closed.host_layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "File".into(),
            popup_width_px: 144.0,
            popup_height_px: 66.0,
            items: model_rc(vec![
                HostMenuChromeItemData {
                    label: "Open".into(),
                    action_id: "workbench.project.open".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Reset Layout".into(),
                    action_id: "workbench.layout.reset".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 66.0),
                template_node("MenuPopupItemRow0", "Panel", "Open", 6.0, 6.0, 132.0, 26.0),
                template_node(
                    "MenuPopupItemRow1",
                    "Panel",
                    "Reset",
                    6.0,
                    34.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    closed.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    closed.menu_state = HostMenuStateData {
        open_menu_index: -1,
        ..HostMenuStateData::default()
    };
    ui.set_host_presentation(closed.clone());
    let closed_snapshot = ui
        .window()
        .take_snapshot()
        .expect("closed menu snapshot should render");

    let mut open = closed;
    open.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let open_menu_state = open.menu_state.clone();
    ui.set_host_presentation(open);
    ui.global::<UiHostContext>().set_menu_state(open_menu_state);
    let open_snapshot = ui
        .window()
        .take_snapshot()
        .expect("open menu snapshot should render");

    assert!(
        changed_pixel_count(
            open_snapshot.width(),
            closed_snapshot.as_bytes(),
            open_snapshot.as_bytes(),
            8,
            27,
            144,
            66,
        ) > 200,
        "open menu popup should paint over the document/viewport surface below the menu bar"
    );
}

#[test]
fn rust_owned_host_painter_draws_open_nested_menu_popup() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "Tools".into(),
            popup_width_px: 144.0,
            popup_height_px: 38.0,
            items: model_rc(vec![HostMenuChromeItemData {
                label: "Weather".into(),
                shortcut: ">".into(),
                enabled: true,
                children: model_rc(vec![HostMenuChromeItemData {
                    label: "Refresh Clouds".into(),
                    action_id: "weather.cloud_layer.refresh".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                }]),
                ..HostMenuChromeItemData::default()
            }]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 38.0),
                template_node(
                    "MenuPopupItemRow0",
                    "Panel",
                    "Weather",
                    6.0,
                    6.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let root_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation.clone());
    ui.global::<UiHostContext>().set_menu_state(root_menu_state);
    let root_only = ui
        .window()
        .take_snapshot()
        .expect("root menu snapshot should render");

    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        open_submenu_path: vec![0],
        hovered_menu_item_path: vec![0, 0],
        hovered_menu_item_index: 1,
        ..HostMenuStateData::default()
    };
    let nested_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_menu_state(nested_menu_state);
    let nested = ui
        .window()
        .take_snapshot()
        .expect("nested menu snapshot should render");

    assert!(
        changed_pixel_count(
            nested.width(),
            root_only.as_bytes(),
            nested.as_bytes(),
            148,
            33,
            150,
            42,
        ) > 140,
        "opening a submenu branch should paint a visible child popup beside the root menu"
    );
}
