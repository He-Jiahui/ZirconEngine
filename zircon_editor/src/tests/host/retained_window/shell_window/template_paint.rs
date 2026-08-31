use super::support::*;

#[test]
fn rust_owned_host_window_snapshot_renders_template_node_styles() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 248.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 248.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 248.0, 85.0),
        pane: pane_with_nodes(
            "Inspector",
            vec![
                selected_template_node(
                    "SelectedPanel",
                    "Panel",
                    "Selected",
                    10.0,
                    10.0,
                    80.0,
                    24.0,
                ),
                primary_template_node("PrimaryButton", "Button", "Apply", 102.0, 10.0, 70.0, 24.0),
                disabled_template_node(
                    "DisabledPanel",
                    "Panel",
                    "Disabled",
                    10.0,
                    44.0,
                    80.0,
                    24.0,
                ),
                muted_label_node("MutedLabel", "Label", "Muted text", 102.0, 48.0, 96.0, 18.0),
            ],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("styled template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 84, 102),
        [15, 101, 116, 255],
        "selected template panel should use active surface color"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 176, 102),
        [53, 199, 208, 255],
        "primary button variant should use primary surface color"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 84, 136),
        [25, 29, 34, 255],
        "disabled template node should use disabled surface color"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 180, 148),
        [24, 29, 37, 255],
        "label-only template nodes should render deterministic text bars"
    );
}

#[test]
fn rust_owned_host_window_snapshot_renders_template_icon_states() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(260, 160));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(260.0, 160.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(260.0, 160.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(40.0, 58.0, 180.0, 78.0),
        header_frame: host_frame(0.0, 0.0, 180.0, 0.0),
        content_frame: host_frame(0.0, 0.0, 180.0, 78.0),
        pane: pane_with_nodes(
            "Inspector",
            vec![
                icon_state_node("HoveredIcon", 18.0, 16.0, false, true, false, false),
                icon_state_node("PressedIcon", 70.0, 16.0, false, false, true, false),
                icon_state_node("SelectedIcon", 122.0, 16.0, true, false, false, false),
                icon_state_node("DisabledIcon", 18.0, 50.0, false, false, false, true),
            ],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("icon-state template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 59, 75),
        [128, 234, 255, 255],
        "hovered icon controls should paint the Material hover state layer"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 111, 75),
        [128, 234, 255, 255],
        "pressed icon controls should paint the Material interaction state layer"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 163, 75),
        [128, 234, 255, 255],
        "selected icon controls should paint the Material interaction state layer"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 59, 109),
        [51, 72, 82, 255],
        "disabled icon controls should paint the disabled surface"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 178, 90),
        [100, 201, 220, 255],
        "selected icons should use the active icon tint"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 178, 90),
        pixel(snapshot.width(), snapshot.as_bytes(), 74, 124),
        "selected and disabled icon glyphs should be visually distinguishable"
    );
}

#[test]
fn rust_owned_host_window_snapshot_respects_template_node_order_and_clip() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut baseline = ui.get_host_presentation();
    baseline.host_layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    baseline.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 120.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 120.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 120.0, 85.0),
        pane: pane_with_nodes("Inspector", Vec::new()),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(baseline.clone());
    let baseline_snapshot = ui
        .window()
        .take_snapshot()
        .expect("baseline order/clip snapshot should render");

    let mut with_nodes = baseline;
    with_nodes.host_scene_data.document_dock.pane = std::sync::Arc::new(pane_with_nodes(
        "Inspector",
        vec![
            disabled_template_node("BackPanel", "Panel", "Back", 10.0, 10.0, 58.0, 28.0),
            selected_template_node("FrontPanel", "Panel", "Front", 10.0, 10.0, 58.0, 28.0),
            primary_template_node("ClippedPanel", "Panel", "Clip", 100.0, 44.0, 80.0, 24.0),
        ],
    ));
    ui.set_host_presentation(with_nodes);
    let snapshot = ui
        .window()
        .take_snapshot()
        .expect("order/clip template node snapshot should render");

    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 92, 106),
        [15, 101, 116, 255],
        "later overlapping template nodes should paint over earlier nodes"
    );
    assert_ne!(
        pixel(snapshot.width(), snapshot.as_bytes(), 182, 138),
        pixel(
            baseline_snapshot.width(),
            baseline_snapshot.as_bytes(),
            182,
            138
        ),
        "node portion inside pane clip should paint"
    );
    assert_eq!(
        pixel(snapshot.width(), snapshot.as_bytes(), 202, 138),
        pixel(
            baseline_snapshot.width(),
            baseline_snapshot.as_bytes(),
            202,
            138
        ),
        "node portion outside pane clip should not paint"
    );
}

#[test]
fn rust_owned_host_painter_does_not_render_structural_control_ids_as_text() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut anonymous = ui.get_host_presentation();
    anonymous.host_layout = host_window_layout_for_test(320.0, 200.0);
    anonymous.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    anonymous.host_scene_data.page_chrome.template_nodes =
        model_rc(vec![template_node("", "Panel", "", 0.0, 26.0, 320.0, 30.0)]);
    ui.set_host_presentation(anonymous.clone());
    let anonymous_snapshot = ui
        .window()
        .take_snapshot()
        .expect("anonymous structural panel snapshot should render");

    let mut with_control_id = anonymous;
    with_control_id.host_scene_data.page_chrome.template_nodes = model_rc(vec![template_node(
        "WorkbenchPageBar",
        "Panel",
        "",
        0.0,
        26.0,
        320.0,
        30.0,
    )]);
    ui.set_host_presentation(with_control_id.clone());
    let control_id_snapshot = ui
        .window()
        .take_snapshot()
        .expect("structural panel snapshot should render");

    let mut with_text = with_control_id;
    with_text.host_scene_data.page_chrome.template_nodes = model_rc(vec![template_node(
        "WorkbenchPageBar",
        "Panel",
        "Workbench",
        0.0,
        26.0,
        320.0,
        30.0,
    )]);
    ui.set_host_presentation(with_text);
    let text_snapshot = ui
        .window()
        .take_snapshot()
        .expect("labeled structural panel snapshot should render");

    assert_eq!(
        changed_pixel_count(
            anonymous_snapshot.width(),
            anonymous_snapshot.as_bytes(),
            control_id_snapshot.as_bytes(),
            0,
            26,
            160,
            30,
        ),
        0,
        "empty structural panel text should not fall back to control_id/node_id glyphs"
    );
    assert!(
        changed_pixel_count(
            text_snapshot.width(),
            anonymous_snapshot.as_bytes(),
            text_snapshot.as_bytes(),
            0,
            26,
            160,
            30,
        ) > 0,
        "explicit text should still render glyphs for labeled nodes"
    );
}
