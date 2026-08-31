use super::*;

#[test]
fn fallback_page_chrome_preserves_clickable_tab_and_project_path_frames() {
    let tabs = model_rc(vec![
        test_tab("Welcome", true, false),
        test_tab("Asset Browser", false, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"ZirconProject4".into(), 640.0, 0.0);
    let bar = node(&nodes, PAGE_BAR_CONTROL_ID);
    let first_tab = node(&nodes, "PageTab0");
    let second_tab = node(&nodes, "PageTab1");
    let project_path = node(&nodes, PAGE_PROJECT_PATH_CONTROL_ID);

    assert!(bar.frame.height >= PAGE_BAR_HEIGHT_PX);
    assert_eq!(bar.frame.y, MENU_TOP_BAR_HEIGHT_PX + 1.0);
    assert!(
        first_tab.frame.width > 0.0 && first_tab.frame.height > 0.0,
        "fallback page tabs must stay hit-testable when the template projection is unavailable"
    );
    assert!(
        first_tab.frame.y >= bar.frame.y
            && first_tab.frame.y + first_tab.frame.height <= bar.frame.y + bar.frame.height,
        "fallback page tabs must stay inside the host page bar instead of overlapping the menu bar"
    );
    assert!(
        project_path.frame.y >= bar.frame.y
            && project_path.frame.y + project_path.frame.height <= bar.frame.y + bar.frame.height,
        "fallback project path text must stay inside the host page bar"
    );
    assert_eq!(first_tab.surface_variant.as_str(), "inset");
    assert_eq!(second_tab.text_tone.as_str(), "subtle");
    assert_eq!(project_path.text.as_str(), "ZirconProject4");
    assert_eq!(
        project_path.font_size,
        EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    );
    assert!(project_path.frame.width > 0.0);
}

#[test]
fn fallback_page_chrome_projects_close_only_for_closeable_pages() {
    let tabs = model_rc(vec![
        test_tab("Workbench", true, false),
        test_tab("Prefab Editor", false, true),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"ZirconProject4".into(), 760.0, 0.0);
    let close = node(&nodes, "PageTabClose1");
    let closeable_tab = node(&nodes, "PageTab1");
    let frames = page_tab_frames(&nodes, &tabs);
    let closeable_frame = frames.row_data(1).expect("closeable page frame");
    let expected = main_page_tab_close_frame(UiFrame::new(
        closeable_tab.frame.x,
        closeable_tab.frame.y,
        closeable_tab.frame.width,
        closeable_tab.frame.height,
    ));

    assert!(maybe_node(&nodes, "PageTabClose0").is_none());
    assert_eq!(close.role.as_str(), "IconButton");
    assert_eq!(close.icon_name.as_str(), "close-outline");
    assert!(close.has_preview_image);
    assert_eq!(close.frame.width, MAIN_PAGE_TAB_CLOSE_EXTENT);
    assert_close(close.frame.x, expected.x);
    assert_close(close.frame.y, expected.y);
    assert_close(closeable_frame.close_frame.x, expected.x);
    assert_close(closeable_frame.close_frame.y, expected.y);
    assert_close(closeable_frame.close_frame.width, expected.width);
    assert_close(closeable_frame.close_frame.height, expected.height);
}

#[test]
fn fallback_page_chrome_measures_file_name_tabs_with_runtime_font_width() {
    let tabs = model_rc(vec![
        test_tab("editor base.zui", true, false),
        test_tab("folder-open-line.svg", false, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"ZirconProject4".into(), 1100.0, 0.0);
    let editor = node(&nodes, "PageTab0");
    let folder = node(&nodes, "PageTab1");

    let expected_editor_width = main_page_tab_preferred_width_from_title_width(
        measure_runtime_text_width("editor base.zui", MAIN_PAGE_TAB_TITLE_FONT_SIZE),
    );
    let expected_folder_width = main_page_tab_preferred_width_from_title_width(
        measure_runtime_text_width("folder-open-line.svg", MAIN_PAGE_TAB_TITLE_FONT_SIZE),
    );

    assert_close(editor.frame.width, expected_editor_width);
    assert_close(folder.frame.width, expected_folder_width);
    assert_eq!(editor.font_size, MAIN_PAGE_TAB_TITLE_FONT_SIZE);
    assert_eq!(folder.font_size, MAIN_PAGE_TAB_TITLE_FONT_SIZE);
}

#[test]
fn fallback_page_chrome_collapses_overflow_tabs_and_keeps_active_visible() {
    let tabs = model_rc(vec![
        test_tab("Welcome", false, false),
        test_tab("Asset Browser", false, false),
        test_tab("Scene", false, false),
        test_tab("Effects", false, false),
        test_tab("Animation State Machine", true, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"ZirconProject4".into(), 420.0, 0.0);
    let overflow = node(&nodes, "PageTabOverflow");
    let active = node(&nodes, "PageTab4");

    assert!(
        maybe_node(&nodes, "PageTab3").is_none(),
        "non-active tabs beyond the available page bar width should collapse behind overflow"
    );
    assert!(active.selected, "the active page tab should remain visible");
    assert_eq!(active.text.as_str(), "Animation State Machine");
    assert_eq!(overflow.role.as_str(), "IconButton");
    assert_eq!(overflow.icon_name.as_str(), "ellipsis-horizontal-outline");
    assert!(overflow.frame.x > active.frame.x);
}

#[test]
fn fallback_page_chrome_keeps_medium_width_tabs_readable_before_overflow() {
    let tabs = model_rc(vec![
        test_tab("Scene Editor", false, false),
        test_tab("Effects", false, false),
        test_tab("Abilities", true, false),
        test_tab("Tags", false, false),
        test_tab("Perception", false, false),
        test_tab("Material", false, false),
        test_tab("Behavior", false, false),
        test_tab("Rendering", false, false),
        test_tab("Assets", false, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"Zircon M3 Visual".into(), 900.0, 0.0);
    let visible_tabs = (0..tabs.row_count())
        .filter_map(|row| maybe_node(&nodes, &format!("PageTab{row}")))
        .collect::<Vec<_>>();

    assert!(
        visible_tabs.len() < tabs.row_count(),
        "medium-width host chrome should collapse extra main pages instead of compressing all tabs"
    );
    assert!(maybe_node(&nodes, "PageTabOverflow").is_some());
    assert!(
        visible_tabs
            .iter()
            .all(|tab| tab.frame.width >= MAIN_PAGE_TAB_MIN_WIDTH),
        "all visible main-page tabs should remain at the readable minimum width"
    );
    assert!(
        maybe_node(&nodes, "PageTab2").is_some_and(|tab| tab.selected),
        "the active tab should remain visible when overflow is active"
    );
}

#[test]
fn fallback_page_chrome_narrow_tier_caps_visible_tabs_before_project_path() {
    let tabs = model_rc(vec![
        test_tab("Scene Editor", false, false),
        test_tab("Effects", false, false),
        test_tab("Abilities", false, false),
        test_tab("Animation", true, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"Zircon M3 Visual".into(), 640.0, 0.0);
    let visible_tabs = (0..tabs.row_count())
        .filter_map(|row| maybe_node(&nodes, &format!("PageTab{row}")))
        .collect::<Vec<_>>();
    let overflow = node(&nodes, "PageTabOverflow");
    let project_path = node(&nodes, PAGE_PROJECT_PATH_CONTROL_ID);

    assert_eq!(
        visible_tabs.len(),
        2,
        "narrow host chrome should use the layout tier to keep only two readable page tabs"
    );
    assert!(
        maybe_node(&nodes, "PageTab3").is_some_and(|tab| tab.selected),
        "the active page tab should replace a non-active visible tab before overflow"
    );
    assert!(
        maybe_node(&nodes, "PageTab2").is_none(),
        "non-active rows beyond the narrow visible cap should stay behind overflow"
    );
    assert!(
        overflow.frame.x + overflow.frame.width <= project_path.frame.x,
        "overflow must stay inside the tab lane before the project path label"
    );
}

#[test]
fn fallback_page_chrome_collapses_project_path_before_primary_tabs() {
    let tabs = model_rc(vec![
        test_tab("Scene Editor", true, false),
        test_tab("Effects", false, false),
        test_tab("Assets", false, false),
    ]);
    let shell_width = 280.0;

    let nodes = fallback_page_chrome_nodes(&tabs, &"Zircon M3 Visual".into(), shell_width, 0.0);
    let project_path = node(&nodes, PAGE_PROJECT_PATH_CONTROL_ID);
    let primary_tabs = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .filter(|node| {
            node.control_id.as_str().starts_with(PAGE_TAB_PREFIX)
                || node.control_id.as_str() == "PageTabOverflow"
        })
        .collect::<Vec<_>>();

    assert_eq!(project_path.frame.width, 0.0);
    assert!(!primary_tabs.is_empty());
    assert!(primary_tabs
        .iter()
        .all(|node| { node.frame.x >= 0.0 && node.frame.x + node.frame.width <= shell_width }));
}

#[test]
fn fallback_page_chrome_wide_tier_does_not_force_overflow_when_tabs_fit() {
    let tabs = model_rc(vec![
        test_tab("Scene Editor", true, false),
        test_tab("Effects", false, false),
        test_tab("Abilities", false, false),
        test_tab("Tags", false, false),
        test_tab("Perception", false, false),
    ]);

    let nodes = fallback_page_chrome_nodes(&tabs, &"Zircon M3 Visual".into(), 1260.0, 0.0);
    let visible_tabs = (0..tabs.row_count())
        .filter_map(|row| maybe_node(&nodes, &format!("PageTab{row}")))
        .collect::<Vec<_>>();

    assert_eq!(
        visible_tabs.len(),
        tabs.row_count(),
        "wide host chrome should show every page tab when the tab lane can hold them"
    );
    assert!(
        maybe_node(&nodes, "PageTabOverflow").is_none(),
        "wide host chrome should not show overflow just because the component supports it"
    );
}

#[test]
fn fallback_dock_header_preserves_tab_drag_and_close_hit_frames() {
    let tabs = model_rc(vec![
        test_tab("Scene", true, true),
        test_tab("Game", false, false),
    ]);

    let nodes = fallback_dock_header_nodes(&tabs, &"Preview".into(), 480.0, 0.0);
    let header = node(&nodes, DOCK_HEADER_BAR_CONTROL_ID);
    let scene = node(&nodes, "DockTab0");
    let scene_close = node(&nodes, "DockTabClose0");
    let game = node(&nodes, "DockTab1");
    let subtitle = node(&nodes, DOCK_SUBTITLE_CONTROL_ID);

    assert!(header.frame.height >= DOCK_HEADER_HEIGHT_PX);
    assert!(
        scene.frame.width > 0.0 && scene.frame.height > 0.0,
        "fallback dock tabs must provide drag/click hit frames"
    );
    assert!(scene_close.frame.width > 0.0 && scene_close.frame.height > 0.0);
    assert!(maybe_node(&nodes, "DockTabClose1").is_none());
    assert_eq!(game.text_tone.as_str(), "subtle");
    assert_eq!(subtitle.text.as_str(), "Preview");
    assert_eq!(
        subtitle.font_size,
        EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    );
}

#[test]
fn fallback_chrome_keeps_quiet_controls_transparent_and_rounded() {
    let expected_radius = EditorControlTokens::workbench_dense().small_radius;
    let tabs = model_rc(vec![
        test_tab("Scene", true, true),
        test_tab("Game", false, false),
    ]);

    let page_nodes = fallback_page_chrome_nodes(&tabs, &"ZirconProject4".into(), 640.0, 0.0);
    let dock_nodes = fallback_dock_header_nodes(&tabs, &"Preview".into(), 480.0, 0.0);
    let side_dock_nodes = side_dock_header_nodes(&tabs, &"zircon_shell".into(), 480.0, 0.0);
    let activity_nodes =
        activity_rail::fallback_activity_rail_nodes(&tabs, &"zircon_shell".into(), 34.0, 96.0);
    let menu_nodes = menu_chrome::fallback_menu_chrome_nodes(
        &model_rc(vec![test_menu("File"), test_menu("Edit")]),
        320.0,
        24.0,
    );

    for (nodes, active_id, quiet_ids) in [
        (&page_nodes, "PageTab0", &["PageTab1", "PageTabClose0"][..]),
        (&dock_nodes, "DockTab0", &["DockTab1", "DockTabClose0"][..]),
        (
            &side_dock_nodes,
            "DockTab0",
            &["DockTab1", "DockTabClose0"][..],
        ),
        (
            &activity_nodes,
            "ActivityRailButton0",
            &["ActivityRailButton1"][..],
        ),
    ] {
        let active = node(nodes, active_id);
        assert_eq!(active.surface_variant.as_str(), "inset");
        assert_eq!(active.corner_radius, expected_radius);

        for control_id in quiet_ids {
            let quiet = node(nodes, control_id);
            assert_eq!(quiet.surface_variant.as_str(), "transparent");
            assert_eq!(quiet.corner_radius, expected_radius);
        }
    }

    for control_id in (0..MENU_SLOT_COUNT).map(|row| format!("MenuSlot{row}")) {
        let menu = node(&menu_nodes, &control_id);
        assert_eq!(menu.surface_variant.as_str(), "transparent");
        assert_eq!(menu.corner_radius, expected_radius);
    }
}

#[test]
fn tab_chrome_fallback_detects_zero_height_or_zero_width_hits() {
    let tabs = model_rc(vec![test_tab("Welcome", true, false)]);
    let zero_height_nodes = model_rc(vec![ViewTemplateNodeData {
        control_id: PAGE_BAR_CONTROL_ID.into(),
        frame: ViewTemplateFrameData {
            width: 640.0,
            ..ViewTemplateFrameData::default()
        },
        ..ViewTemplateNodeData::default()
    }]);
    let zero_tab_nodes = model_rc(vec![
        ViewTemplateNodeData {
            control_id: PAGE_BAR_CONTROL_ID.into(),
            frame: ViewTemplateFrameData {
                width: 640.0,
                height: 31.0,
                ..ViewTemplateFrameData::default()
            },
            ..ViewTemplateNodeData::default()
        },
        ViewTemplateNodeData {
            control_id: "PageTab0".into(),
            frame: ViewTemplateFrameData {
                height: 24.0,
                ..ViewTemplateFrameData::default()
            },
            ..ViewTemplateNodeData::default()
        },
    ]);

    assert!(tab_chrome_needs_fallback(
        &zero_height_nodes,
        PAGE_BAR_CONTROL_ID,
        PAGE_TAB_PREFIX,
        &tabs
    ));
    assert!(tab_chrome_needs_fallback(
        &zero_tab_nodes,
        PAGE_BAR_CONTROL_ID,
        PAGE_TAB_PREFIX,
        &tabs
    ));
}
