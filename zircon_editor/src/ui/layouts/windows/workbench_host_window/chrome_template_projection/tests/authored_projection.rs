use super::*;

#[test]
fn authored_chrome_assets_use_workbench_typography_baseline() {
    const BODY: &str = "font_size = 13.333333";
    const CAPTION: &str = "font_size = 10.666667";
    let menu = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/workbench_menu_chrome.zui"
    ));
    let page = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/workbench_page_chrome.zui"
    ));
    let dock = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/workbench_dock_header.zui"
    ));
    let status = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/ui/editor/workbench_status_bar.zui"
    ));

    assert_eq!(menu.matches(BODY).count(), 7);
    assert_eq!(page.matches(BODY).count(), 3);
    assert_eq!(page.matches(CAPTION).count(), 1);
    assert_eq!(dock.matches(BODY).count(), 3);
    assert_eq!(dock.matches(CAPTION).count(), 1);
    assert_eq!(status.matches(BODY).count(), 3);
    assert!(status.contains("height = { min = 16.0, preferred = 16.0, max = 16.0"));
}

#[test]
fn dock_header_nodes_hide_close_controls_for_non_closeable_tabs() {
    let tabs = model_rc(vec![test_tab("Welcome", true, false)]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 800.0, 31.0);

    assert!(
        maybe_node(&nodes, "DockTab0").is_some(),
        "the visible document tab should remain projected"
    );
    assert!(
        maybe_node(&nodes, "DockTabClose0").is_none(),
        "non-closeable tabs should not render an empty close-button surface"
    );
    assert!(
        maybe_node(&nodes, "DockTabClose1").is_none(),
        "unused close slots should be filtered with their tab slots"
    );
}

#[test]
fn dock_header_nodes_keep_close_controls_for_closeable_tabs() {
    let tabs = model_rc(vec![test_tab("Scene", true, true)]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 800.0, 31.0);

    assert!(
        maybe_node(&nodes, "DockTabClose0").is_some(),
        "closeable tabs should retain their close hit target"
    );
    assert!(
        maybe_node(&nodes, "DockTabClose1").is_none(),
        "close controls beyond the live tab count should still be filtered"
    );
}

#[test]
fn dock_header_nodes_keep_document_tabs_readable_and_close_control_clean() {
    let tabs = model_rc(vec![
        test_tab("Asset Browser", true, true),
        test_tab("Zircon M3 Visual", false, true),
    ]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 900.0, 31.0);
    let asset = node(&nodes, "DockTab0");
    let visual = node(&nodes, "DockTab1");
    let close = node(&nodes, "DockTabClose0");

    assert_eq!(asset.text.as_str(), "Asset Browser");
    assert!(asset.frame.width >= DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);
    assert!(visual.frame.width >= DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);
    assert_eq!(close.surface_variant.as_str(), "");
    assert_eq!(close.frame.width, DOCUMENT_TAB_CLOSE_EXTENT);
    assert!(close.frame.x + close.frame.width <= asset.frame.x + asset.frame.width);
}

#[test]
fn dock_header_nodes_measure_file_name_tabs_with_runtime_font_width() {
    let tabs = model_rc(vec![
        test_tab("editor base.zui", true, true),
        test_tab("folder-open-line.svg", false, true),
    ]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 900.0, 31.0);
    let editor = node(&nodes, "DockTab0");
    let folder = node(&nodes, "DockTab1");
    let folder_close = node(&nodes, "DockTabClose1");

    let expected_editor_width = document_tab_preferred_width_from_title_width(
        measure_runtime_text_width("editor base.zui", DOCUMENT_TAB_TITLE_FONT_SIZE),
        true,
    );
    let expected_folder_width = document_tab_preferred_width_from_title_width(
        measure_runtime_text_width("folder-open-line.svg", DOCUMENT_TAB_TITLE_FONT_SIZE),
        true,
    );

    assert_close(editor.frame.width, expected_editor_width);
    assert_close(folder.frame.width, expected_folder_width);
    assert_eq!(editor.font_size, DOCUMENT_TAB_TITLE_FONT_SIZE);
    assert_eq!(folder.font_size, DOCUMENT_TAB_TITLE_FONT_SIZE);
    assert!(
        folder_close.frame.x + folder_close.frame.width <= folder.frame.x + folder.frame.width,
        "close hitbox should stay inside the runtime-measured file-name tab"
    );
}

#[test]
fn dock_header_nodes_project_tabs_beyond_the_authored_stencil() {
    let tabs = model_rc(vec![
        test_tab("Scene", true, true),
        test_tab("Game", false, true),
        test_tab("Profiler", false, true),
        test_tab("Render Graph", false, true),
    ]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 900.0, 31.0);
    let fourth_tab = node(&nodes, "DockTab3");
    let fourth_close = node(&nodes, "DockTabClose3");

    assert_eq!(fourth_tab.text.as_str(), "Render Graph");
    assert!(fourth_tab.frame.width >= DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);
    assert!(fourth_tab.frame.x + fourth_tab.frame.width <= 900.0);
    assert!(fourth_close.frame.x >= fourth_tab.frame.x);
    assert!(
        fourth_close.frame.x + fourth_close.frame.width
            <= fourth_tab.frame.x + fourth_tab.frame.width
    );
}

#[test]
fn dock_header_nodes_compact_inactive_tabs_inside_narrow_document_lane() {
    let tabs = model_rc(vec![
        test_tab_with_icon("A Very Long Scene Graph", "hierarchy", false, true),
        test_tab_with_icon("A Very Long Asset Browser", "asset-browser", false, true),
        test_tab_with_icon("A Very Long Active Inspector", "inspector", true, true),
        test_tab_with_icon("A Very Long Render Graph", "render-graph", false, true),
    ]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 420.0, 31.0);
    let active = node(&nodes, "DockTab2");
    let compact_width = EditorControlTokens::workbench_dense().default_height;

    assert_eq!(active.text.as_str(), "A Very Long Active Inspector");
    assert!(active.frame.width >= DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);
    for row in [0, 1, 3] {
        let tab = node(&nodes, &format!("DockTab{row}"));
        assert_eq!(tab.text.as_str(), "");
        assert_close(tab.frame.width, compact_width);
        assert!(
            maybe_node(&nodes, &format!("DockTabClose{row}")).is_none(),
            "an icon-only compact tab must not keep a detached close target"
        );
    }
    let final_tab = node(&nodes, "DockTab3");
    assert!(
        final_tab.frame.x + final_tab.frame.width <= 420.0 - DOCUMENT_TAB_STRIP_X,
        "adaptive fallback tabs must stay inside the narrow document lane"
    );
    let active_close = node(&nodes, "DockTabClose2");
    assert!(active_close.frame.x >= active.frame.x);
    assert!(active_close.frame.x + active_close.frame.width <= active.frame.x + active.frame.width);
}

#[test]
fn dock_header_overflow_reserves_a_reachable_control_for_hidden_tabs() {
    let tabs = model_rc(vec![
        test_tab_with_icon("Scene", "scene", true, true),
        test_tab_with_icon("Game", "game", false, true),
        test_tab_with_icon("Profiler", "pulse-outline", false, true),
        test_tab_with_icon("Render Graph", "git-network-outline", false, true),
        test_tab_with_icon("Diagnostics", "grid-outline", false, true),
    ]);

    let nodes = document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 220.0, 31.0);
    let overflow = node(&nodes, DOCK_TAB_OVERFLOW_CONTROL_ID);
    let frames = dock_tab_frames(&nodes, &tabs);

    assert!(overflow.frame.width > 0.0);
    assert!(overflow.frame.x + overflow.frame.width <= 220.0 - DOCUMENT_TAB_STRIP_X + 0.01);
    assert_eq!(overflow.icon_name.as_str(), "ellipsis-horizontal-outline");
    assert!(
        frames.iter().any(|tab| tab.frame.width <= f32::EPSILON),
        "overflow is only published when at least one tab is hidden"
    );
    assert!(
        frames
            .iter()
            .filter(|tab| tab.frame.width > f32::EPSILON)
            .all(|tab| tab.frame.x + tab.frame.width + DOCUMENT_TAB_GAP <= overflow.frame.x + 0.01),
        "the overflow control owns reserved header space instead of overlapping visible tabs"
    );
}

#[test]
fn tab_chrome_falls_back_when_an_authored_tab_leaves_the_bar_bounds() {
    let nodes = model_rc(vec![
        ViewTemplateNodeData {
            control_id: DOCK_HEADER_BAR_CONTROL_ID.into(),
            frame: ViewTemplateFrameData {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 31.0,
            },
            ..ViewTemplateNodeData::default()
        },
        ViewTemplateNodeData {
            control_id: "DockTab0".into(),
            frame: ViewTemplateFrameData {
                x: 90.0,
                y: 4.0,
                width: 20.0,
                height: 27.0,
            },
            ..ViewTemplateNodeData::default()
        },
    ]);
    let tabs = model_rc(vec![test_tab("Scene", true, true)]);

    assert!(tab_chrome_needs_fallback(
        &nodes,
        DOCK_HEADER_BAR_CONTROL_ID,
        DOCK_TAB_PREFIX,
        &tabs,
    ));
}

#[test]
fn side_dock_header_nodes_compact_inactive_tabs_inside_narrow_panel() {
    let tabs = model_rc(vec![
        test_tab_with_icon("Project", "project", false, false),
        test_tab_with_icon("Assets", "assets", false, false),
        test_tab_with_icon("Asset Browser", "asset-browser", true, false),
        test_tab_with_icon("Hierarchy", "hierarchy", false, false),
        test_tab_with_icon("Components", "components", false, false),
    ]);

    let nodes = side_dock_header_nodes(&tabs, &"fyrox_panel".into(), 226.0, 31.0);
    let project = node(&nodes, "DockTab0");
    let assets = node(&nodes, "DockTab1");
    let asset_browser = node(&nodes, "DockTab2");

    assert_eq!(project.text.as_str(), "");
    assert_eq!(assets.text.as_str(), "");
    assert_eq!(asset_browser.text.as_str(), "Asset Browser");
    assert!(maybe_node(&nodes, "DockTab3").is_none());
    assert!(maybe_node(&nodes, "DockTab4").is_none());
    let controls = EditorControlTokens::workbench_dense();
    let density = EditorDensityTokens::workbench_dense();
    assert!(asset_browser.frame.width >= controls.default_height * 3.0 + density.gap_medium * 2.0);
    assert!(project.frame.x + project.frame.width <= assets.frame.x);
    assert!(assets.frame.x + assets.frame.width <= asset_browser.frame.x);
    assert!(
        asset_browser.frame.x + asset_browser.frame.width <= 226.0 - DOCUMENT_TAB_STRIP_X,
        "active side-dock tab should stay readable inside its panel: {asset_browser:?}"
    );
}

#[test]
fn side_dock_header_nodes_reuse_the_stable_projection_without_rebuilding_rows() {
    dock_header::clear_side_dock_header_projection_cache_for_tests();
    let tabs = model_rc(vec![test_tab("Asset Browser", true, true)]);

    let first = side_dock_header_nodes(&tabs, &"fyrox_panel".into(), 240.0, 31.0);
    let second = side_dock_header_nodes(&tabs, &"fyrox_panel".into(), 240.0, 31.0);

    assert!(first.shares_values_with(&second));
    assert_eq!(
        dock_header::side_dock_header_projection_builds_for_tests(),
        1,
        "unchanged side dock input must reuse its projected rows"
    );
}

#[test]
fn menu_popup_nodes_project_absolute_rows_beyond_authored_slots() {
    let items = model_rc(
        (0..18)
            .map(|index| HostMenuChromeItemData {
                label: format!("Preset {index:02}").into(),
                shortcut: "".into(),
                action_id: format!("workbench.layout.preset.load.preset_{index:02}").into(),
                enabled: index != 17,
                children: ModelRc::default(),
            })
            .collect(),
    );

    let nodes = menu_popup_nodes(&items, 224.0, 550.0);
    let label_0 = node(&nodes, "MenuPopupItemLabel0");
    let label_15 = node(&nodes, "MenuPopupItemLabel15");
    let label_16 = node(&nodes, "MenuPopupItemLabel16");
    let label_17 = node(&nodes, "MenuPopupItemLabel17");
    let shortcut_16 = node(&nodes, "MenuPopupItemShortcut16");
    let shortcut_17 = node(&nodes, "MenuPopupItemShortcut17");
    let row_17 = node(&nodes, "MenuPopupItemRow17");

    let row_step = (label_15.frame.y - label_0.frame.y) / 15.0;
    assert_eq!(label_16.text.as_str(), "Preset 16");
    assert_eq!(label_17.text.as_str(), "Preset 17");
    assert_eq!(
        label_17.text_tone.as_str(),
        "muted",
        "disabled overflow rows should not render with enabled text tone"
    );
    assert_eq!(
        shortcut_16.text_tone.as_str(),
        "muted",
        "enabled overflow shortcuts should preserve the TOML-authored shortcut tone"
    );
    assert_eq!(
        shortcut_17.text_tone.as_str(),
        "muted",
        "disabled overflow shortcuts should also render muted"
    );
    assert!(
        (label_16.frame.y - (label_0.frame.y + row_step * 16.0)).abs() < f32::EPSILON,
        "row 16 should keep the same TOML-derived row cadence as authored slots"
    );
    assert!(
        row_17.frame.y > label_15.frame.y,
        "absolute row 17 should be projected into the scrollable popup content instead of being truncated at slot 15"
    );
    assert_eq!(label_16.icon_name.as_str(), "folder-open-outline");
    assert!(
        label_16.has_preview_image,
        "overflow rows should keep menu action SVG projection when cloned beyond authored stencil slots"
    );
}

#[test]
fn menu_chrome_nodes_project_extension_slots_beyond_authored_stencil() {
    let menus = model_rc(
        (0..9)
            .map(|index| HostMenuChromeMenuData {
                label: format!("Plugin{index}").into(),
                popup_width_px: 224.0,
                popup_height_px: 72.0,
                popup_nodes: ModelRc::default(),
                items: ModelRc::default(),
            })
            .collect(),
    );

    let nodes = menu_chrome_nodes(&menus, 360.0, 29.0);
    let slot_6 = node(&nodes, "MenuSlot6");
    let slot_8 = node(&nodes, "MenuSlot8");

    assert_eq!(slot_8.text.as_str(), "Plugin8");
    assert!(
        slot_8.frame.x > slot_6.frame.x + slot_6.frame.width,
        "extension top-level menus should be projected after the authored menu stencil instead of being truncated at slot 6"
    );
}

#[test]
fn menu_chrome_nodes_measure_top_level_slots_with_runtime_font_width() {
    let menus = model_rc(vec![
        test_menu("iiiiiiii"),
        test_menu("WWWWWWWW"),
        test_menu("folder-open-line.svg"),
    ]);

    let nodes = menu_chrome_nodes(&menus, 420.0, 29.0);
    let narrow = node(&nodes, "MenuSlot0");
    let wide = node(&nodes, "MenuSlot1");
    let file_name = node(&nodes, "MenuSlot2");

    let expected_narrow = menu_slot_width_from_runtime_text("iiiiiiii");
    let expected_wide = menu_slot_width_from_runtime_text("WWWWWWWW");
    let expected_file_name = menu_slot_width_from_runtime_text("folder-open-line.svg");

    assert_close(narrow.frame.width, expected_narrow);
    assert_close(wide.frame.width, expected_wide);
    assert_close(file_name.frame.width, expected_file_name);
    assert_eq!(narrow.font_size, WORKBENCH_MENU_SLOT_FONT_SIZE);
    assert_eq!(wide.font_size, WORKBENCH_MENU_SLOT_FONT_SIZE);
    assert!(
        wide.frame.width > narrow.frame.width,
        "same character count should still use glyph-aware runtime measurement"
    );
    let first_gap = wide.frame.x - (narrow.frame.x + narrow.frame.width);
    let second_gap = file_name.frame.x - (wide.frame.x + wide.frame.width);
    assert!(first_gap > 0.0);
    assert_close(second_gap, first_gap);
}

#[test]
fn menu_popup_nodes_project_action_svg_icons() {
    let items = model_rc(vec![
        test_menu_item("Open Project", "Ctrl+O", "workbench.project.open", true),
        test_menu_item("Save Project", "Ctrl+S", "workbench.project.save", true),
        test_menu_item("Undo", "Ctrl+Z", "workbench.history.undo", false),
        test_menu_item(
            "Build Export",
            "",
            "workbench.view.open.editor.build_export_desktop",
            true,
        ),
        test_menu_item("Create Cube", "", "workbench.scene.node.create.cube", true),
        test_menu_item(
            "Create Rect Light",
            "",
            "workbench.scene.node.create.rect_light",
            true,
        ),
    ]);

    let nodes = menu_popup_nodes(&items, 224.0, 180.0);
    let open = node(&nodes, "MenuPopupItemLabel0");
    let save = node(&nodes, "MenuPopupItemLabel1");
    let undo = node(&nodes, "MenuPopupItemLabel2");
    let export = node(&nodes, "MenuPopupItemLabel3");
    let cube = node(&nodes, "MenuPopupItemLabel4");
    let rect_light = node(&nodes, "MenuPopupItemLabel5");

    assert_eq!(open.text.as_str(), "Open Project");
    assert_eq!(open.icon_name.as_str(), "folder-open-outline");
    assert!(open.has_preview_image);
    assert_eq!(save.icon_name.as_str(), "save-outline");
    assert!(save.has_preview_image);
    assert_eq!(undo.icon_name.as_str(), "chevron-back-outline");
    assert!(undo.has_preview_image);
    assert_eq!(
        undo.text_tone.as_str(),
        "muted",
        "disabled menu items should keep muted label tone while still carrying their action icon"
    );
    assert_eq!(export.icon_name.as_str(), "share-outline");
    assert_eq!(cube.icon_name.as_str(), "cube-outline");
    assert_eq!(rect_light.icon_name.as_str(), "color-fill-outline");
}

#[test]
fn activity_rail_nodes_current_drawer_tabs_svg_icons_and_selected_state() {
    let tabs = model_rc(vec![
        test_tab_with_icon("Hierarchy", "hierarchy", true, false),
        test_tab_with_icon("Assets", "assets", false, false),
    ]);

    let nodes = activity_rail_nodes(&tabs, &"jetbrains_shell".into(), 34.0, 96.0);
    let hierarchy_button = node(&nodes, "ActivityRailButton0");
    let hierarchy_icon = node(&nodes, "ActivityRailButtonIcon0");
    let assets_icon = node(&nodes, "ActivityRailButtonIcon1");

    assert_eq!(hierarchy_button.frame.x, 3.0);
    assert_eq!(hierarchy_button.frame.width, 28.0);
    assert!(hierarchy_button.frame.x + hierarchy_button.frame.width <= 34.0);
    assert_eq!(hierarchy_icon.frame.x, 8.0);
    assert_eq!(hierarchy_icon.frame.width, 18.0);
    assert!(hierarchy_icon.frame.x + hierarchy_icon.frame.width <= 34.0);
    assert_eq!(
        hierarchy_button.surface_variant.as_str(),
        "inset",
        "active activity rail button should project selected surface metadata"
    );
    assert!(hierarchy_button.selected);
    assert!(
        !hierarchy_button.focused,
        "active drawer selection must not impersonate keyboard focus"
    );
    assert_eq!(hierarchy_icon.icon_name.as_str(), "layers-outline");
    assert!(
        hierarchy_icon.has_preview_image,
        "activity rail icons should resolve SVG preview pixels during chrome projection"
    );
    assert_eq!(hierarchy_icon.text_tone.as_str(), "default");
    assert!(!hierarchy_icon.focused);
    assert_eq!(assets_icon.icon_name.as_str(), "folder-open-outline");
    assert_eq!(assets_icon.text_tone.as_str(), "subtle");
}

#[test]
fn control_frame_unions_all_render_primitives_for_the_same_control() {
    let nodes = model_rc(vec![
        ViewTemplateNodeData {
            control_id: "ActivityRailButton0".into(),
            frame: ViewTemplateFrameData {
                x: 30.0,
                y: 143.0,
                width: 1.0,
                height: 32.0,
            },
            ..ViewTemplateNodeData::default()
        },
        ViewTemplateNodeData {
            control_id: "ActivityRailButton0".into(),
            frame: ViewTemplateFrameData {
                x: 3.0,
                y: 143.0,
                width: 28.0,
                height: 32.0,
            },
            ..ViewTemplateNodeData::default()
        },
    ]);

    let frame = control_frame(&nodes, "ActivityRailButton0");
    assert_eq!(
        (frame.x, frame.y, frame.width, frame.height),
        (3.0, 143.0, 28.0, 32.0)
    );
}

#[test]
fn stencil_template_selection_is_independent_of_render_primitive_order() {
    let surface = ViewTemplateNodeData {
        control_id: "ActivityRailButton0".into(),
        role: "Button".into(),
        frame: ViewTemplateFrameData {
            x: 3.0,
            y: 143.0,
            width: 28.0,
            height: 32.0,
        },
        ..ViewTemplateNodeData::default()
    };
    let separator = ViewTemplateNodeData {
        control_id: "ActivityRailButton0".into(),
        role: "Button".into(),
        frame: ViewTemplateFrameData {
            x: 30.0,
            y: 143.0,
            width: 1.0,
            height: 32.0,
        },
        ..ViewTemplateNodeData::default()
    };

    for candidates in [
        [surface.clone(), separator.clone()],
        [separator.clone(), surface.clone()],
    ] {
        let mut templates = BTreeMap::new();
        for candidate in candidates {
            retain_dominant_control_template(&mut templates, 0, candidate);
        }
        let selected = templates.get(&0).expect("dominant template");
        assert_eq!(selected.frame.x, 3.0);
        assert_eq!(selected.frame.width, 28.0);
    }
}

#[test]
fn independent_activity_rails_do_not_evict_each_others_retained_surface() {
    crate::ui::layouts::views::clear_view_template_projection_caches_for_tests();
    let left_tabs = model_rc(vec![test_tab_with_icon(
        "Hierarchy",
        "hierarchy",
        true,
        false,
    )]);
    let right_tabs = model_rc(vec![test_tab_with_icon(
        "Inspector",
        "inspector",
        true,
        false,
    )]);

    let first_left = activity_rail_nodes_for_surface(
        "host.left.activity.rail",
        &left_tabs,
        &"jetbrains_shell".into(),
        34.0,
        420.0,
    );
    let _right = activity_rail_nodes_for_surface(
        "host.right.activity.rail",
        &right_tabs,
        &"jetbrains_shell".into(),
        34.0,
        520.0,
    );
    let second_left = activity_rail_nodes_for_surface(
        "host.left.activity.rail",
        &left_tabs,
        &"jetbrains_shell".into(),
        34.0,
        420.0,
    );

    assert!(first_left.shares_values_with(&second_left));
}

#[test]
fn document_and_bottom_headers_keep_independent_retained_surfaces() {
    crate::ui::layouts::views::clear_view_template_projection_caches_for_tests();
    let document_tabs = model_rc(vec![test_tab("Scene", true, true)]);
    let bottom_tabs = model_rc(vec![test_tab("Console", true, false)]);

    let first_document = document_dock_header_nodes(
        &document_tabs,
        &"Scene".into(),
        &"fyrox_panel".into(),
        720.0,
        31.0,
    );
    let _bottom = bottom_dock_header_nodes(&bottom_tabs, &"fyrox_panel".into(), 1280.0, 31.0);
    let second_document = document_dock_header_nodes(
        &document_tabs,
        &"Scene".into(),
        &"fyrox_panel".into(),
        720.0,
        31.0,
    );

    assert!(first_document.shares_values_with(&second_document));
}

#[test]
fn page_and_dock_tabs_project_svg_icons_and_close_button_icon() {
    let tabs = model_rc(vec![
        test_tab_with_icon("Scene", "scene", true, true),
        test_tab_with_icon("Assets", "asset-browser", false, true),
    ]);

    let page_nodes = page_chrome_nodes(
        &tabs,
        &"Demo".into(),
        &"jetbrains_shell".into(),
        640.0,
        64.0,
    );
    let page_scene = node(&page_nodes, "PageTab0");
    let page_assets = node(&page_nodes, "PageTab1");

    assert_eq!(page_scene.icon_name.as_str(), "cube-outline");
    assert_eq!(
        page_scene.media_source.as_str(),
        "icons/ionicons/cube-outline.svg"
    );
    assert!(page_scene.has_preview_image);
    assert!(page_scene.selected);
    assert!(
        !page_scene.focused,
        "active page selection must not impersonate keyboard focus"
    );
    assert_eq!(page_scene.text_tone.as_str(), "default");
    assert_eq!(page_assets.icon_name.as_str(), "folder-open-outline");
    assert!(page_assets.has_preview_image);
    assert!(!page_assets.selected);
    assert_eq!(page_assets.text_tone.as_str(), "subtle");
    assert!(
        maybe_node(&page_nodes, "PageTabClose0").is_some(),
        "the authored page template must provide the close control without appending a copied node model"
    );

    let dock_nodes =
        document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 640.0, 40.0);
    let dock_scene = node(&dock_nodes, "DockTab0");
    let dock_close = node(&dock_nodes, "DockTabClose0");

    assert_eq!(dock_scene.icon_name.as_str(), "cube-outline");
    assert!(dock_scene.has_preview_image);
    assert!(dock_scene.selected);
    assert!(
        !dock_scene.focused,
        "active dock selection must not impersonate keyboard focus"
    );
    assert_eq!(dock_close.role.as_str(), "IconButton");
    assert_eq!(dock_close.icon_name.as_str(), "close-outline");
    assert!(
        dock_close.has_preview_image,
        "dock close controls should render as SVG icon buttons instead of empty inset blocks"
    );
}

#[test]
fn page_chrome_reuses_the_template_model_when_tabs_are_stable() {
    let tabs = model_rc(vec![test_tab("Scene", true, true)]);

    let first = page_chrome_nodes(
        &tabs,
        &"Demo".into(),
        &"jetbrains_shell".into(),
        640.0,
        64.0,
    );
    let second = page_chrome_nodes(
        &tabs,
        &"Demo".into(),
        &"jetbrains_shell".into(),
        640.0,
        64.0,
    );

    assert!(first.shares_values_with(&second));
}

#[test]
fn page_chrome_switches_to_dynamic_overflow_when_the_stencil_is_too_small() {
    let tabs = model_rc(vec![
        test_tab("Welcome", false, false),
        test_tab("Asset Browser", false, false),
        test_tab("Scene", false, false),
        test_tab("Effects", false, false),
        test_tab("Animation State Machine", true, false),
    ]);

    let nodes = page_chrome_nodes(
        &tabs,
        &"ZirconProject4".into(),
        &"jetbrains_shell".into(),
        420.0,
        64.0,
    );
    let overflow = node(&nodes, "PageTabOverflow");
    let active = node(&nodes, "PageTab4");

    assert!(maybe_node(&nodes, "PageTab3").is_none());
    assert!(active.selected);
    assert_eq!(active.text.as_str(), "Animation State Machine");
    assert!(overflow.frame.x > active.frame.x);
}

#[test]
fn authored_tab_projection_keeps_selection_and_focus_independent() {
    let active_tabs = model_rc(vec![test_tab("Scene", true, false)]);
    let active = tab_node_with_state(
        ViewTemplateNodeData {
            control_id: "PageTab0".into(),
            ..ViewTemplateNodeData::default()
        },
        PAGE_TAB_PREFIX,
        &active_tabs,
    );
    let inactive_tabs = model_rc(vec![test_tab("Scene", false, false)]);
    let focused = tab_node_with_state(
        ViewTemplateNodeData {
            control_id: "PageTab0".into(),
            focused: true,
            ..ViewTemplateNodeData::default()
        },
        PAGE_TAB_PREFIX,
        &inactive_tabs,
    );

    assert!(active.selected);
    assert!(!active.focused);
    assert!(!focused.selected);
    assert!(focused.focused);
}

#[test]
fn status_bar_nodes_project_text_overrides_from_flat_asset() {
    let nodes = status_bar_nodes(
        &"Runtime ready".into(),
        &"2 warnings".into(),
        &"1920 x 1080".into(),
        &"material_dark".into(),
        800.0,
        22.0,
    );

    assert_eq!(
        node(&nodes, STATUS_PRIMARY_CONTROL_ID).text,
        "Runtime ready"
    );
    assert_eq!(node(&nodes, STATUS_SECONDARY_CONTROL_ID).text, "2 warnings");
    assert_eq!(node(&nodes, STATUS_VIEWPORT_CONTROL_ID).text, "1920 x 1080");
    assert_eq!(
        node(&nodes, STATUS_PRIMARY_CONTROL_ID).font_size,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
    assert!(
        maybe_node(&nodes, "WorkbenchStatusBarRoot").is_some(),
        "generic host status projection should preserve the authored responsive root instead of taking the procedural pixel fallback"
    );
    assert!(node(&nodes, "StatusBarPanel").frame.width > 0.0);
}
