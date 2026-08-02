use std::collections::BTreeMap;

use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::{
    measure_runtime_text_width,
    primitives::{ModelRc, SharedString},
};

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    build_view_template_nodes, load_preview_image, ViewTemplateFrameData, ViewTemplateNodeData,
};
use crate::ui::workbench::page_tabs::{
    main_page_project_path_width, main_page_tab_preferred_width_from_title_width_with_close,
    main_page_tab_visible_cap_for_width, MAIN_PAGE_TAB_CHROME_SIDE_INSET, MAIN_PAGE_TAB_GAP,
    MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_OVERFLOW_WIDTH,
    MAIN_PAGE_TAB_TITLE_FONT_SIZE,
};

use super::{
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostMenuChromeMenuData,
    HostWindowSurfaceMetricsData, TabData,
};

mod activity_rail;
mod dock_header;
mod menu_chrome;
mod page_tabs;
mod status_bar;

const MENU_CHROME_ASSET: &str = "/assets/ui/editor/workbench_menu_chrome.zui";
#[cfg(test)]
const MENU_POPUP_ASSET: &str = "/assets/ui/editor/workbench_menu_popup.zui";
const PAGE_CHROME_ASSET: &str = "/assets/ui/editor/workbench_page_chrome.zui";
const DOCK_HEADER_ASSET: &str = "/assets/ui/editor/workbench_dock_header.zui";
const STATUS_BAR_ASSET: &str = "/assets/ui/editor/workbench_status_bar.zui";
const ACTIVITY_RAIL_ASSET: &str = "/assets/ui/editor/workbench_activity_rail.zui";

const MENU_SLOT_PREFIX: &str = "MenuSlot";
pub(super) const MENU_SLOT_COUNT: usize = 7;
#[cfg(test)]
pub(super) const MENU_POPUP_ITEM_COUNT: usize = 16;
#[cfg(test)]
const MENU_POPUP_ITEM_LABEL_PREFIX: &str = "MenuPopupItemLabel";
#[cfg(test)]
const MENU_POPUP_ITEM_SHORTCUT_PREFIX: &str = "MenuPopupItemShortcut";
#[cfg(test)]
const MENU_POPUP_ITEM_ROW_PREFIX: &str = "MenuPopupItemRow";
#[cfg(test)]
const MENU_POPUP_ROW_STEP_FALLBACK_PX: f32 = 30.0;
const PAGE_TAB_PREFIX: &str = "PageTab";
const PAGE_TAB_CLOSE_PREFIX: &str = "PageTabClose";
const PAGE_TAB_CLOSE_ICON: &str = "close-outline";
const DOCK_TAB_PREFIX: &str = "DockTab";
const DOCK_TAB_CLOSE_PREFIX: &str = "DockTabClose";
const DOCK_TAB_CLOSE_ICON: &str = "close-outline";
const ACTIVITY_RAIL_BUTTON_PREFIX: &str = "ActivityRailButton";
const ACTIVITY_RAIL_BUTTON_ICON_PREFIX: &str = "ActivityRailButtonIcon";
const ACTIVITY_RAIL_STENCIL_COUNT: usize = 2;
const ACTIVITY_RAIL_ROW_STEP_FALLBACK_PX: f32 = 32.0;
const MENU_TOP_BAR_CONTROL_ID: &str = "WorkbenchMenuTopBar";
const PAGE_BAR_CONTROL_ID: &str = "WorkbenchPageBar";
const PAGE_PROJECT_PATH_CONTROL_ID: &str = "PageProjectPath";
const DOCK_HEADER_BAR_CONTROL_ID: &str = "DockHeaderBar";
const DOCK_SUBTITLE_CONTROL_ID: &str = "DockSubtitle";
const STATUS_PRIMARY_CONTROL_ID: &str = "StatusPrimaryLabel";
const STATUS_SECONDARY_CONTROL_ID: &str = "StatusSecondaryLabel";
const STATUS_VIEWPORT_CONTROL_ID: &str = "StatusViewportLabel";
const OUTER_MARGIN_PX: f32 = 0.0;
const RAIL_WIDTH_PX: f32 = 34.0;
const MENU_TOP_BAR_HEIGHT_PX: f32 = 24.0;
const PAGE_BAR_HEIGHT_PX: f32 = 32.0;
const DOCK_HEADER_HEIGHT_PX: f32 = 31.0;
const CHROME_TAB_HEIGHT_INSET_PX: f32 = 4.0;
const FAST_PROCEDURAL_CHROME_NODES: bool = true;

pub(super) fn surface_metrics_from_chrome_assets(
    _shell_width: f32,
) -> HostWindowSurfaceMetricsData {
    // Startup must not instantiate template assets just to discover stable shell
    // heights. The authored v2 chrome assets still own node projection and hit
    // frames; these constants mirror their fixed metric controls.
    HostWindowSurfaceMetricsData {
        outer_margin_px: OUTER_MARGIN_PX,
        rail_width_px: RAIL_WIDTH_PX,
        top_bar_height_px: MENU_TOP_BAR_HEIGHT_PX,
        host_bar_height_px: PAGE_BAR_HEIGHT_PX,
        panel_header_height_px: DOCK_HEADER_HEIGHT_PX,
        document_header_height_px: DOCK_HEADER_HEIGHT_PX,
    }
}

pub(super) fn menu_chrome_nodes(
    menus: &ModelRc<HostMenuChromeMenuData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    menu_chrome::menu_chrome_nodes(menus, width, height)
}

pub(super) fn menu_control_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    count: usize,
) -> ModelRc<HostChromeControlFrameData> {
    menu_chrome::menu_control_frames(nodes, count)
}

#[cfg(test)]
pub(super) fn menu_popup_nodes(
    items: &ModelRc<super::HostMenuChromeItemData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    menu_chrome::menu_popup_nodes(items, width, height)
}

fn indexed_slot_node(
    templates: &BTreeMap<usize, ViewTemplateNodeData>,
    prefix: &str,
    stencil_count: usize,
    item_index: usize,
    row_step: f32,
    text: Option<&str>,
) -> Option<ViewTemplateNodeData> {
    let template_index = item_index % stencil_count;
    let mut node = templates.get(&template_index)?.clone();
    let absolute_control_id = format!("{prefix}{item_index}");
    node.node_id = absolute_control_id.clone().into();
    node.control_id = absolute_control_id.into();
    if let Some(text) = text {
        node.text = text.into();
    }
    node.frame.y += (item_index - template_index) as f32 * row_step;
    Some(node)
}

fn indexed_row_step(row_templates: &BTreeMap<usize, ViewTemplateNodeData>, fallback: f32) -> f32 {
    row_templates
        .get(&0)
        .zip(row_templates.get(&1))
        .map(|(first, second)| second.frame.y - first.frame.y)
        .filter(|step| *step > 0.0)
        .unwrap_or(fallback)
}

pub(super) fn page_chrome_nodes(
    tabs: &ModelRc<TabData>,
    project_path: &SharedString,
    shell_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    if FAST_PROCEDURAL_CHROME_NODES {
        return fallback_page_chrome_nodes(tabs, project_path, width, height);
    }

    let mut text_overrides = tab_text_overrides(PAGE_TAB_PREFIX, tabs);
    text_overrides.insert(
        "PageProjectPath".to_string(),
        if project_path.is_empty() {
            "No project open".to_string()
        } else {
            project_path.to_string()
        },
    );

    let nodes = tab_template_nodes(
        "host.page.chrome",
        PAGE_CHROME_ASSET,
        width,
        height,
        &text_overrides,
        PAGE_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, PAGE_BAR_CONTROL_ID, PAGE_TAB_PREFIX, tabs) {
        return fallback_page_chrome_nodes(tabs, project_path, width, height);
    }
    let nodes = page_tabs::append_missing_close_nodes(nodes, tabs);
    if shell_preset_id.as_str() == "jetbrains_shell" {
        return model_rc(
            (0..nodes.row_count())
                .filter_map(|row| nodes.row_data(row))
                .map(|mut node| {
                    if node.control_id == PAGE_PROJECT_PATH_CONTROL_ID {
                        node.text_tone = "muted".into();
                    }
                    node
                })
                .collect(),
        );
    }
    nodes
}

pub(super) fn page_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    tab_frames(nodes, PAGE_TAB_PREFIX, Some(PAGE_TAB_CLOSE_PREFIX), tabs)
}

pub(super) fn page_tab_row_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, PAGE_BAR_CONTROL_ID)
}

pub(super) fn page_overflow_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, "PageTabOverflow")
}

pub(super) fn page_overflow_hidden_tab_indices(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> Vec<usize> {
    (0..tabs.row_count())
        .filter(|row| !has_control_frame(nodes, &format!("{PAGE_TAB_PREFIX}{row}")))
        .collect()
}

pub(super) fn page_project_path_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, PAGE_PROJECT_PATH_CONTROL_ID)
}

pub(super) fn activity_rail_nodes(
    tabs: &ModelRc<TabData>,
    shell_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    activity_rail::activity_rail_nodes(tabs, shell_preset_id, width, height)
}

pub(super) fn activity_rail_button_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeControlFrameData> {
    activity_rail::activity_rail_button_frames(nodes, tabs)
}

pub(super) fn activity_rail_active_control_id(tabs: &ModelRc<TabData>) -> SharedString {
    activity_rail::activity_rail_active_control_id(tabs)
}

fn chrome_tab_icon_name(tab: &TabData) -> String {
    let key = tab.icon_key.as_str();
    if let Some(icon) = normalized_chrome_icon_key(key) {
        return icon;
    }
    let title = tab.title.to_lowercase();
    match key {
        "project" | "projects" => "albums-outline",
        "hierarchy" | "tree" => "layers-outline",
        "console" | "terminal" => "terminal-outline",
        "asset-browser" | "asset_browser" | "assets" => "folder-open-outline",
        "build-export" | "build_export" | "export" => "share-outline",
        "module-plugins" | "module_plugins" | "plugins" => "git-network-outline",
        "runtime-diagnostics" | "runtime_diagnostics" | "diagnostics" => "grid-outline",
        "scene" | "scene-view" | "scene_view" => "cube-outline",
        "game" | "game-view" | "game_view" => "game-controller-outline",
        "prefab" | "prefabs" => "cube-outline",
        "ui" | "widgets" => "construct-outline",
        "grid" => "grid-outline",
        _ if title.contains("project") => "albums-outline",
        _ if title.contains("hierarchy") => "layers-outline",
        _ if title.contains("console") || title.contains("terminal") => "terminal-outline",
        _ if title.contains("asset") => "folder-open-outline",
        _ if title.contains("export") => "share-outline",
        _ if title.contains("plugin") => "git-network-outline",
        _ if title.contains("diagnostic") => "grid-outline",
        _ if title.contains("scene") => "cube-outline",
        _ if title.contains("game") => "game-controller-outline",
        _ => "ellipse-outline",
    }
    .to_string()
}

fn normalized_chrome_icon_key(value: &str) -> Option<String> {
    let file_name = value
        .rsplit(|character| character == '/' || character == '\\')
        .next()
        .unwrap_or(value);
    let icon_name = file_name.strip_suffix(".svg").unwrap_or(file_name);
    (!icon_name.is_empty() && icon_name.ends_with("-outline")).then(|| icon_name.to_string())
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header::side_dock_header_nodes(tabs, panel_preset_id, width, height)
}

pub(super) fn document_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header::document_dock_header_nodes(tabs, subtitle, panel_preset_id, width, height)
}

pub(super) fn bottom_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    panel_preset_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header::bottom_dock_header_nodes(tabs, panel_preset_id, width, height)
}

pub(super) fn floating_window_header_nodes(
    tabs: &ModelRc<TabData>,
    title: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header::floating_window_header_nodes(tabs, title, width, height)
}

pub(super) fn dock_header_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    dock_header::dock_header_frame(nodes)
}

pub(super) fn dock_subtitle_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    dock_header::dock_subtitle_frame(nodes)
}

pub(super) fn dock_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    dock_header::dock_tab_frames(nodes, tabs)
}

pub(super) fn status_bar_nodes(
    status_primary: &SharedString,
    status_secondary: &SharedString,
    viewport_label: &SharedString,
    skin_id: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    status_bar::status_bar_nodes(
        status_primary,
        status_secondary,
        viewport_label,
        skin_id,
        width,
        height,
    )
}

fn tab_template_nodes(
    document_tree_id: &str,
    asset_path: &str,
    width: f32,
    height: f32,
    text_overrides: &BTreeMap<String, String>,
    slot_prefix: &'static str,
    tabs: &ModelRc<TabData>,
) -> ModelRc<ViewTemplateNodeData> {
    let filters = [SlotFilter::new(slot_prefix, tabs.row_count())];
    let nodes = raw_template_nodes(document_tree_id, asset_path, width, height, text_overrides);
    model_rc(
        nodes
            .into_iter()
            .filter(|node| node_survives_filters(node, &filters))
            .filter(|node| node_survives_tab_close_filter(node, tabs))
            .map(|node| tab_node_with_state(node, slot_prefix, tabs))
            .collect(),
    )
}

fn fallback_page_chrome_nodes(
    tabs: &ModelRc<TabData>,
    project_path: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let page_bar_y = MENU_TOP_BAR_HEIGHT_PX + 1.0;
    let bar_height = (height - page_bar_y).max(PAGE_BAR_HEIGHT_PX);
    let visible_tab_indices = visible_page_tab_indices(tabs, width);
    let has_overflow = visible_tab_indices.len() < tabs.row_count();
    let close_count = visible_tab_indices
        .iter()
        .filter(|row| tabs.row_data(**row).is_some_and(|tab| tab.closeable))
        .count();
    let mut nodes =
        Vec::with_capacity(visible_tab_indices.len() + close_count + usize::from(has_overflow) + 2);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackWorkbenchPageBar".into(),
        control_id: PAGE_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: page_bar_y,
            width: width.max(1.0),
            height: bar_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = MAIN_PAGE_TAB_CHROME_SIDE_INSET;
    let right_label_width = main_page_project_path_width(width);
    let max_tab_right = (width - right_label_width - MAIN_PAGE_TAB_CHROME_SIDE_INSET)
        .max(MAIN_PAGE_TAB_CHROME_SIDE_INSET);
    let tab_right_limit = if has_overflow {
        (max_tab_right - MAIN_PAGE_TAB_OVERFLOW_WIDTH - MAIN_PAGE_TAB_GAP).max(12.0)
    } else {
        max_tab_right
    };
    for row in visible_tab_indices.iter().copied() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let tab_width = page_tab_width(&tab);
        let draw_width = tab_width
            .min((tab_right_limit - x).max(MAIN_PAGE_TAB_MIN_WIDTH))
            .clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH);
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let icon_name = chrome_tab_icon_name(&tab);
        let tab_frame = ViewTemplateFrameData {
            x,
            y: page_bar_y + CHROME_TAB_HEIGHT_INSET_PX,
            width: draw_width,
            height: (bar_height - CHROME_TAB_HEIGHT_INSET_PX).max(20.0),
        };
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackPageTab{row}").into(),
            control_id: format!("{PAGE_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: tab.title.clone(),
            text_tone: text_tone.into(),
            font_size: MAIN_PAGE_TAB_TITLE_FONT_SIZE,
            font_weight,
            surface_variant: if tab.active { "inset" } else { "" }.into(),
            button_variant: "ghost".into(),
            selected: tab.active,
            focused: false,
            frame: tab_frame.clone(),
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &icon_name);
        nodes.push(tab_node);
        if tab.closeable {
            nodes.push(page_tabs::close_node(
                row,
                page_tabs::close_view_frame(&tab_frame),
            ));
        }
        x = (x + draw_width + MAIN_PAGE_TAB_GAP).min(tab_right_limit);
    }

    if has_overflow {
        let mut overflow_node = ViewTemplateNodeData {
            node_id: "FallbackPageTabOverflow".into(),
            control_id: "PageTabOverflow".into(),
            role: "IconButton".into(),
            text: "".into(),
            text_tone: "subtle".into(),
            font_size: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
            font_weight: 600,
            button_variant: "ghost".into(),
            frame: ViewTemplateFrameData {
                x: x.min(
                    (max_tab_right - MAIN_PAGE_TAB_OVERFLOW_WIDTH)
                        .max(MAIN_PAGE_TAB_CHROME_SIDE_INSET),
                ),
                y: page_bar_y + CHROME_TAB_HEIGHT_INSET_PX,
                width: MAIN_PAGE_TAB_OVERFLOW_WIDTH,
                height: (bar_height - CHROME_TAB_HEIGHT_INSET_PX).max(20.0),
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut overflow_node, "ellipsis-horizontal-outline");
        nodes.push(overflow_node);
    }

    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackPageProjectPath".into(),
        control_id: PAGE_PROJECT_PATH_CONTROL_ID.into(),
        role: "Text".into(),
        text: if project_path.is_empty() {
            "No project open".into()
        } else {
            project_path.clone()
        },
        text_tone: "muted".into(),
        font_size: EditorTypographyTokens::WORKBENCH_CAPTION_SIZE,
        frame: ViewTemplateFrameData {
            x: (width - right_label_width - MAIN_PAGE_TAB_CHROME_SIDE_INSET)
                .max(MAIN_PAGE_TAB_CHROME_SIDE_INSET),
            y: page_bar_y + 6.0,
            width: right_label_width,
            height: (bar_height - 12.0).max(16.0),
        },
        ..ViewTemplateNodeData::default()
    });

    model_rc(nodes)
}

fn visible_page_tab_indices(tabs: &ModelRc<TabData>, width: f32) -> Vec<usize> {
    let tab_count = tabs.row_count();
    if tab_count == 0 {
        return Vec::new();
    }

    let right_label_width = main_page_project_path_width(width);
    let max_tab_right = (width - right_label_width - MAIN_PAGE_TAB_CHROME_SIDE_INSET)
        .max(MAIN_PAGE_TAB_CHROME_SIDE_INSET);
    let visible_cap = main_page_tab_visible_cap_for_width(width, tab_count);
    let force_overflow = visible_cap < tab_count;
    let mut x = MAIN_PAGE_TAB_CHROME_SIDE_INSET;
    let mut visible = Vec::new();
    for row in 0..tab_count {
        if visible.len() >= visible_cap {
            break;
        }
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let remaining_after_row = tab_count.saturating_sub(row + 1);
        let overflow_reserve = if remaining_after_row > 0 || force_overflow {
            MAIN_PAGE_TAB_OVERFLOW_WIDTH + MAIN_PAGE_TAB_GAP
        } else {
            0.0
        };
        let tab_width = page_tab_width(&tab);
        if !visible.is_empty() && x + tab_width + overflow_reserve > max_tab_right {
            break;
        }
        visible.push(row);
        x += tab_width + MAIN_PAGE_TAB_GAP;
    }

    if let Some(active_row) = active_tab_row(tabs) {
        if !visible.contains(&active_row) {
            if let Some(last_visible) = visible.last_mut() {
                *last_visible = active_row;
            } else {
                visible.push(active_row);
            }
        }
    }

    visible.dedup();
    visible
}

fn page_tab_width(tab: &TabData) -> f32 {
    let title_width = measure_runtime_text_width(tab.title.as_str(), MAIN_PAGE_TAB_TITLE_FONT_SIZE);
    main_page_tab_preferred_width_from_title_width_with_close(title_width, tab.closeable)
}

fn active_tab_row(tabs: &ModelRc<TabData>) -> Option<usize> {
    (0..tabs.row_count()).find(|row| tabs.row_data(*row).is_some_and(|tab| tab.active))
}

fn fallback_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    dock_header::fallback_dock_header_nodes(tabs, subtitle, width, height)
}

fn tab_chrome_needs_fallback(
    nodes: &ModelRc<ViewTemplateNodeData>,
    bar_control_id: &str,
    tab_prefix: &str,
    tabs: &ModelRc<TabData>,
) -> bool {
    control_frame(nodes, bar_control_id).height <= 0.0
        || (tabs.row_count() > 0 && control_frame(nodes, &format!("{tab_prefix}0")).width <= 0.0)
}

fn template_nodes(
    document_tree_id: &str,
    asset_path: &str,
    width: f32,
    height: f32,
    text_overrides: &BTreeMap<String, String>,
    filters: &[SlotFilter],
) -> ModelRc<ViewTemplateNodeData> {
    model_rc(
        raw_template_nodes(document_tree_id, asset_path, width, height, text_overrides)
            .into_iter()
            .filter(|node| node_survives_filters(node, filters))
            .collect(),
    )
}

fn raw_template_nodes(
    document_tree_id: &str,
    asset_path: &str,
    width: f32,
    height: f32,
    text_overrides: &BTreeMap<String, String>,
) -> Vec<ViewTemplateNodeData> {
    build_view_template_nodes(
        document_tree_id,
        asset_path,
        &[],
        UiSize::new(width.max(0.0), height.max(0.0)),
        text_overrides,
    )
    .unwrap_or_default()
}

fn tab_text_overrides(prefix: &str, tabs: &ModelRc<TabData>) -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    for row in 0..tabs.row_count() {
        if let Some(tab) = tabs.row_data(row) {
            overrides.insert(format!("{prefix}{row}"), tab.title.to_string());
        }
    }
    overrides
}

fn tab_node_with_state(
    mut node: ViewTemplateNodeData,
    prefix: &str,
    tabs: &ModelRc<TabData>,
) -> ViewTemplateNodeData {
    if let Some(row) = slot_index(node.control_id.as_str(), prefix) {
        if let Some(tab) = tabs.row_data(row) {
            let icon_name = chrome_tab_icon_name(&tab);
            apply_template_icon(&mut node, &icon_name);
            node.selected = tab.active;
        }
        if tabs.row_data(row).is_some_and(|tab| tab.active) {
            node.text_tone = "default".into();
            node.font_weight = 600;
        } else {
            node.text_tone = "subtle".into();
            node.font_weight = 400;
        }
    } else if (prefix == DOCK_TAB_PREFIX
        && slot_index(node.control_id.as_str(), DOCK_TAB_CLOSE_PREFIX).is_some())
        || (prefix == PAGE_TAB_PREFIX
            && slot_index(node.control_id.as_str(), PAGE_TAB_CLOSE_PREFIX).is_some())
    {
        node.role = "IconButton".into();
        node.text = "".into();
        node.text_tone = "muted".into();
        let close_icon = if prefix == PAGE_TAB_PREFIX {
            PAGE_TAB_CLOSE_ICON
        } else {
            DOCK_TAB_CLOSE_ICON
        };
        apply_template_icon(&mut node, close_icon);
    }
    node
}

fn apply_template_icon(node: &mut ViewTemplateNodeData, icon_name: &str) {
    node.icon_name = icon_name.into();
    node.media_source = format!("icons/ionicons/{icon_name}.svg").into();
    node.preview_image = load_preview_image("", icon_name);
    let preview_size = node.preview_image.size();
    node.has_preview_image = preview_size.width > 0 && preview_size.height > 0;
}

fn node_survives_filters(node: &ViewTemplateNodeData, filters: &[SlotFilter]) -> bool {
    filters.iter().all(
        |filter| match slot_index(node.control_id.as_str(), filter.prefix) {
            Some(row) => row < filter.used_count,
            None => true,
        },
    )
}

fn node_survives_tab_close_filter(node: &ViewTemplateNodeData, tabs: &ModelRc<TabData>) -> bool {
    let row = slot_index(node.control_id.as_str(), PAGE_TAB_CLOSE_PREFIX)
        .or_else(|| slot_index(node.control_id.as_str(), DOCK_TAB_CLOSE_PREFIX));
    let Some(row) = row else {
        return true;
    };
    tabs.row_data(row).is_some_and(|tab| tab.closeable)
}

fn slot_index(control_id: &str, prefix: &str) -> Option<usize> {
    control_id.strip_prefix(prefix)?.parse().ok()
}

fn control_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    prefix: &str,
    count: usize,
) -> ModelRc<HostChromeControlFrameData> {
    model_rc(
        (0..count)
            .map(|row| {
                let control_id = format!("{prefix}{row}");
                HostChromeControlFrameData {
                    frame: control_frame(nodes, &control_id),
                    control_id: control_id.into(),
                }
            })
            .collect(),
    )
}

fn tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    prefix: &str,
    close_prefix: Option<&str>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    model_rc(
        (0..tabs.row_count())
            .filter_map(|row| {
                let tab = tabs.row_data(row)?;
                let control_id = format!("{prefix}{row}");
                let close_frame = close_prefix
                    .map(|prefix| control_frame(nodes, &format!("{prefix}{row}")))
                    .unwrap_or_default();
                Some(HostChromeTabData {
                    frame: control_frame(nodes, &control_id),
                    close_frame,
                    control_id: control_id.into(),
                    tab,
                })
            })
            .collect(),
    )
}

fn control_frame(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> FrameRect {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .map(|node| frame_rect(&node))
        .unwrap_or_default()
}

fn has_control_frame(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> bool {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .any(|node| node.control_id.as_str() == control_id && node.frame.width > 0.0)
}

fn frame_rect(node: &ViewTemplateNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

struct SlotFilter {
    prefix: &'static str,
    used_count: usize,
}

impl SlotFilter {
    const fn new(prefix: &'static str, used_count: usize) -> Self {
        Self { prefix, used_count }
    }
}

#[cfg(test)]
mod tests;
