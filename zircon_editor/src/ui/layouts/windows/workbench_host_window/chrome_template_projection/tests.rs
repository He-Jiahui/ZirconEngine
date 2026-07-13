use super::*;

use crate::ui::layouts::windows::workbench_host_window::HostMenuChromeItemData;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::document_tabs::{
    document_tab_preferred_width_from_title_width, DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH,
    DOCUMENT_TAB_CLOSE_EXTENT, DOCUMENT_TAB_STRIP_X, DOCUMENT_TAB_TITLE_FONT_SIZE,
};
use crate::ui::workbench::menu_bar::{
    workbench_menu_slot_width_from_label_width, WORKBENCH_MENU_SLOT_FONT_SIZE,
};
use crate::ui::workbench::page_tabs::{
    main_page_tab_preferred_width_from_title_width, MAIN_PAGE_TAB_TITLE_FONT_SIZE,
};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};

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

    assert_eq!(
        hierarchy_button.surface_variant.as_str(),
        "inset",
        "active activity rail button should project selected surface metadata"
    );
    assert_eq!(hierarchy_icon.icon_name.as_str(), "layers-outline");
    assert!(
        hierarchy_icon.has_preview_image,
        "activity rail icons should resolve SVG preview pixels during chrome projection"
    );
    assert_eq!(hierarchy_icon.text_tone.as_str(), "default");
    assert_eq!(assets_icon.icon_name.as_str(), "folder-open-outline");
    assert_eq!(assets_icon.text_tone.as_str(), "subtle");
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
    assert_eq!(page_scene.text_tone.as_str(), "default");
    assert_eq!(page_assets.icon_name.as_str(), "folder-open-outline");
    assert!(page_assets.has_preview_image);
    assert!(!page_assets.selected);
    assert_eq!(page_assets.text_tone.as_str(), "subtle");

    let dock_nodes =
        document_dock_header_nodes(&tabs, &"".into(), &"fyrox_panel".into(), 640.0, 40.0);
    let dock_scene = node(&dock_nodes, "DockTab0");
    let dock_close = node(&dock_nodes, "DockTabClose0");

    assert_eq!(dock_scene.icon_name.as_str(), "cube-outline");
    assert!(dock_scene.has_preview_image);
    assert_eq!(dock_close.role.as_str(), "IconButton");
    assert_eq!(dock_close.icon_name.as_str(), "close-outline");
    assert!(
        dock_close.has_preview_image,
        "dock close controls should render as SVG icon buttons instead of empty inset blocks"
    );
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
    assert!(node(&nodes, "StatusBarPanel").frame.width > 0.0);
}

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

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "expected {expected:.3}, got {actual:.3}",
    );
}

fn menu_slot_width_from_runtime_text(label: &str) -> f32 {
    workbench_menu_slot_width_from_label_width(measure_runtime_text_width(
        label,
        WORKBENCH_MENU_SLOT_FONT_SIZE,
    ))
}

fn node(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> ViewTemplateNodeData {
    maybe_node(nodes, control_id)
        .unwrap_or_else(|| panic!("missing projected popup node {control_id}"))
}

fn maybe_node(
    nodes: &ModelRc<ViewTemplateNodeData>,
    control_id: &str,
) -> Option<ViewTemplateNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
}

fn test_tab(title: &str, active: bool, closeable: bool) -> TabData {
    test_tab_with_icon(title, "", active, closeable)
}

fn test_tab_with_icon(title: &str, icon_key: &str, active: bool, closeable: bool) -> TabData {
    TabData {
        id: title.into(),
        slot: "document".into(),
        title: title.into(),
        icon_key: icon_key.into(),
        active,
        closeable,
    }
}

fn test_menu_item(
    label: &str,
    shortcut: &str,
    action_id: &str,
    enabled: bool,
) -> HostMenuChromeItemData {
    HostMenuChromeItemData {
        label: label.into(),
        shortcut: shortcut.into(),
        action_id: action_id.into(),
        enabled,
        children: ModelRc::default(),
    }
}

fn test_menu(label: &str) -> HostMenuChromeMenuData {
    HostMenuChromeMenuData {
        label: label.into(),
        popup_width_px: 224.0,
        popup_height_px: 72.0,
        popup_nodes: ModelRc::default(),
        items: ModelRc::default(),
    }
}
