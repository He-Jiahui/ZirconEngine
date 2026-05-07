use std::collections::BTreeMap;

use slint::{Model, ModelRc, SharedString};
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{
    build_view_template_nodes, load_preview_image, ViewTemplateFrameData, ViewTemplateNodeData,
};

use super::{
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostMenuChromeMenuData,
    HostWindowSurfaceMetricsData, TabData,
};

const MENU_CHROME_ASSET: &str = "/assets/ui/editor/workbench_menu_chrome.ui.toml";
const MENU_POPUP_ASSET: &str = "/assets/ui/editor/workbench_menu_popup.ui.toml";
const PAGE_CHROME_ASSET: &str = "/assets/ui/editor/workbench_page_chrome.ui.toml";
const DOCK_HEADER_ASSET: &str = "/assets/ui/editor/workbench_dock_header.ui.toml";
const STATUS_BAR_ASSET: &str = "/assets/ui/editor/workbench_status_bar.ui.toml";
const ACTIVITY_RAIL_ASSET: &str = "/assets/ui/editor/workbench_activity_rail.ui.toml";

const MENU_SLOT_PREFIX: &str = "MenuSlot";
pub(super) const MENU_SLOT_COUNT: usize = 7;
pub(super) const MENU_POPUP_ITEM_COUNT: usize = 16;
const MENU_POPUP_ITEM_LABEL_PREFIX: &str = "MenuPopupItemLabel";
const MENU_POPUP_ITEM_SHORTCUT_PREFIX: &str = "MenuPopupItemShortcut";
const MENU_POPUP_ITEM_ROW_PREFIX: &str = "MenuPopupItemRow";
const MENU_POPUP_ROW_STEP_FALLBACK_PX: f32 = 30.0;
const PAGE_TAB_PREFIX: &str = "PageTab";
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
const METRIC_PROBE_HEIGHT_PX: f32 = 96.0;
const MENU_TOP_BAR_HEIGHT_FALLBACK_PX: f32 = 28.0;
const PAGE_BAR_HEIGHT_FALLBACK_PX: f32 = 31.0;
const DOCK_HEADER_HEIGHT_FALLBACK_PX: f32 = 31.0;
const CHROME_TEXT_FONT_SIZE_PX: f32 = 12.0;
const CHROME_TAB_HEIGHT_INSET_PX: f32 = 4.0;

pub(super) fn surface_metrics_from_chrome_assets(shell_width: f32) -> HostWindowSurfaceMetricsData {
    let text_overrides = BTreeMap::new();
    let menu_nodes = raw_template_nodes(
        "host.menu.chrome.metrics",
        MENU_CHROME_ASSET,
        shell_width,
        METRIC_PROBE_HEIGHT_PX,
        &text_overrides,
    );
    let page_nodes = raw_template_nodes(
        "host.page.chrome.metrics",
        PAGE_CHROME_ASSET,
        shell_width,
        METRIC_PROBE_HEIGHT_PX,
        &text_overrides,
    );
    let dock_nodes = raw_template_nodes(
        "host.dock.header.metrics",
        DOCK_HEADER_ASSET,
        shell_width,
        METRIC_PROBE_HEIGHT_PX,
        &text_overrides,
    );
    let dock_header_height = measured_height_or(
        control_frame_from_slice(&dock_nodes, DOCK_HEADER_BAR_CONTROL_ID).height,
        DOCK_HEADER_HEIGHT_FALLBACK_PX,
    );

    HostWindowSurfaceMetricsData {
        outer_margin_px: OUTER_MARGIN_PX,
        rail_width_px: RAIL_WIDTH_PX,
        top_bar_height_px: measured_height_or(
            control_frame_from_slice(&menu_nodes, MENU_TOP_BAR_CONTROL_ID).height,
            MENU_TOP_BAR_HEIGHT_FALLBACK_PX,
        ),
        host_bar_height_px: measured_height_or(
            control_frame_from_slice(&page_nodes, PAGE_BAR_CONTROL_ID).height,
            PAGE_BAR_HEIGHT_FALLBACK_PX,
        ),
        panel_header_height_px: dock_header_height,
        document_header_height_px: dock_header_height,
    }
}

pub(super) fn menu_chrome_nodes(
    menus: &ModelRc<HostMenuChromeMenuData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    for row in 0..menus.row_count() {
        if let Some(menu) = menus.row_data(row) {
            text_overrides.insert(format!("{MENU_SLOT_PREFIX}{row}"), menu.label.to_string());
        }
    }

    let nodes = template_nodes(
        "host.menu.chrome",
        MENU_CHROME_ASSET,
        width,
        height,
        &text_overrides,
        &[SlotFilter::new(MENU_SLOT_PREFIX, MENU_SLOT_COUNT)],
    );
    if nodes.row_count() == 0 || control_frame(&nodes, "MenuSlot0").width <= 0.0 {
        return fallback_menu_chrome_nodes(menus, width, height);
    }
    model_rc(expand_menu_chrome_slot_nodes(model_nodes(&nodes), menus))
}

pub(super) fn menu_control_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    count: usize,
) -> ModelRc<HostChromeControlFrameData> {
    control_frames(nodes, MENU_SLOT_PREFIX, count)
}

fn fallback_menu_chrome_nodes(
    menus: &ModelRc<HostMenuChromeMenuData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let slot_count = menus.row_count().max(MENU_SLOT_COUNT);
    let mut nodes = Vec::with_capacity(slot_count + 1);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackWorkbenchMenuTopBar".into(),
        control_id: MENU_TOP_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: height.max(24.0),
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = 8.0;
    for row in 0..slot_count {
        let label = menus
            .row_data(row)
            .map(|menu| menu.label)
            .unwrap_or_default();
        let slot_width = menu_slot_width(label.as_str());
        nodes.push(ViewTemplateNodeData {
            node_id: format!("FallbackMenuSlot{row}").into(),
            control_id: format!("{MENU_SLOT_PREFIX}{row}").into(),
            role: "Button".into(),
            text: label,
            text_tone: "default".into(),
            font_size: 12.0,
            font_weight: 500,
            surface_variant: "".into(),
            button_variant: "ghost".into(),
            frame: ViewTemplateFrameData {
                x,
                y: 2.0,
                width: slot_width,
                height: (height - 4.0).clamp(20.0, 24.0),
            },
            ..ViewTemplateNodeData::default()
        });
        x += slot_width + 4.0;
    }

    model_rc(nodes)
}

fn expand_menu_chrome_slot_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    menus: &ModelRc<HostMenuChromeMenuData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut slot_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), MENU_SLOT_PREFIX) {
            slot_templates.insert(row, node);
        } else {
            output_nodes.push(node);
        }
    }
    if slot_templates.is_empty() {
        return output_nodes;
    }

    let slot_count = menus.row_count().max(MENU_SLOT_COUNT);
    let gap = menu_slot_gap(&slot_templates).unwrap_or(2.0);
    let mut projected_slots: Vec<ViewTemplateNodeData> = Vec::with_capacity(slot_count);
    for row in 0..slot_count {
        let template_index = row.min(MENU_SLOT_COUNT - 1);
        let Some(mut node) = slot_templates.get(&template_index).cloned() else {
            continue;
        };
        let label = menus
            .row_data(row)
            .map(|menu| menu.label.to_string())
            .unwrap_or_default();
        node.node_id = format!("{MENU_SLOT_PREFIX}{row}").into();
        node.control_id = format!("{MENU_SLOT_PREFIX}{row}").into();
        node.text = label.clone().into();
        if row >= MENU_SLOT_COUNT {
            if let Some(previous) = projected_slots.last() {
                node.frame.x = previous.frame.x + previous.frame.width + gap;
            }
            node.frame.width = menu_slot_width(&label);
        }
        projected_slots.push(node.clone());
        output_nodes.push(node);
    }
    output_nodes
}

fn model_nodes(nodes: &ModelRc<ViewTemplateNodeData>) -> Vec<ViewTemplateNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .collect()
}

fn menu_slot_width(label: &str) -> f32 {
    ((label.chars().count() as f32 * 7.0) + 24.0).clamp(40.0, 128.0)
}

fn menu_slot_gap(templates: &BTreeMap<usize, ViewTemplateNodeData>) -> Option<f32> {
    let ordered = templates.values().collect::<Vec<_>>();
    ordered
        .windows(2)
        .rev()
        .filter_map(|pair| {
            let gap = pair[1].frame.x - (pair[0].frame.x + pair[0].frame.width);
            (gap > 0.0).then_some(gap)
        })
        .next()
}

pub(super) fn menu_popup_nodes(
    items: &ModelRc<super::HostMenuChromeItemData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    for row in 0..items.row_count().min(MENU_POPUP_ITEM_COUNT) {
        if let Some(item) = items.row_data(row) {
            text_overrides.insert(
                format!("{MENU_POPUP_ITEM_LABEL_PREFIX}{row}"),
                item.label.to_string(),
            );
            text_overrides.insert(
                format!("{MENU_POPUP_ITEM_SHORTCUT_PREFIX}{row}"),
                item.shortcut.to_string(),
            );
        }
    }

    model_rc(expand_menu_popup_item_nodes(
        raw_template_nodes(
            "host.menu.popup",
            MENU_POPUP_ASSET,
            width,
            height,
            &text_overrides,
        ),
        items,
    ))
}

fn expand_menu_popup_item_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    items: &ModelRc<super::HostMenuChromeItemData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut row_templates = BTreeMap::new();
    let mut label_templates = BTreeMap::new();
    let mut shortcut_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_ROW_PREFIX) {
            row_templates.insert(row, node);
        } else if let Some(row) = slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_LABEL_PREFIX)
        {
            label_templates.insert(row, node);
        } else if let Some(row) =
            slot_index(node.control_id.as_str(), MENU_POPUP_ITEM_SHORTCUT_PREFIX)
        {
            shortcut_templates.insert(row, node);
        } else {
            output_nodes.push(node);
        }
    }

    let row_step = indexed_row_step(&row_templates, MENU_POPUP_ROW_STEP_FALLBACK_PX);
    for item_index in 0..items.row_count() {
        let Some(item) = items.row_data(item_index) else {
            continue;
        };

        if let Some(node) = indexed_slot_node(
            &row_templates,
            MENU_POPUP_ITEM_ROW_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            None,
        ) {
            output_nodes.push(node);
        }
        if let Some(mut label_node) = indexed_slot_node(
            &label_templates,
            MENU_POPUP_ITEM_LABEL_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            Some(item.label.as_str()),
        ) {
            apply_template_icon(&mut label_node, &menu_popup_item_icon_name(&item));
            if !item.enabled {
                label_node.text_tone = "muted".into();
            }
            output_nodes.push(label_node);
        }
        if let Some(mut shortcut_node) = indexed_slot_node(
            &shortcut_templates,
            MENU_POPUP_ITEM_SHORTCUT_PREFIX,
            MENU_POPUP_ITEM_COUNT,
            item_index,
            row_step,
            Some(item.shortcut.as_str()),
        ) {
            if !item.enabled {
                shortcut_node.text_tone = "muted".into();
            }
            output_nodes.push(shortcut_node);
        }
    }

    output_nodes
}

fn menu_popup_item_icon_name(item: &super::HostMenuChromeItemData) -> String {
    let action = item.action_id.as_str();
    if let Some(icon) = normalized_chrome_icon_key(action) {
        return icon;
    }
    match action {
        "OpenProject" => "folder-open-outline",
        "OpenScene" => "cube-outline",
        "CreateScene" => "add-outline",
        "SaveProject" | "SaveLayout" => "save-outline",
        "ResetLayout" => "sync-outline",
        "EnterPlayMode" => "play-outline",
        "ExitPlayMode" => "remove-outline",
        "Undo" => "chevron-back-outline",
        "Redo" => "chevron-forward-outline",
        "DeleteSelected" => "remove-outline",
        _ if action.starts_with("SavePreset.") => "save-outline",
        _ if action.starts_with("LoadPreset.") => "folder-open-outline",
        _ if action.starts_with("CreateNode.") => scene_create_menu_icon_name(action),
        _ if action.starts_with("OpenView.") => open_view_menu_icon_name(action),
        _ => menu_label_icon_name(item.label.as_str()),
    }
    .to_string()
}

fn scene_create_menu_icon_name(action: &str) -> &'static str {
    match action.strip_prefix("CreateNode.").unwrap_or_default() {
        "Cube" => "cube-outline",
        "Camera" => "scan-outline",
        "DirectionalLight" => "color-fill-outline",
        _ => "add-outline",
    }
}

fn open_view_menu_icon_name(action: &str) -> &'static str {
    let descriptor = action
        .strip_prefix("OpenView.")
        .unwrap_or_default()
        .replace('-', "_")
        .to_lowercase();
    match descriptor.as_str() {
        "editor.project" => "albums-outline",
        "editor.hierarchy" => "layers-outline",
        "editor.inspector" => "options-outline",
        "editor.scene" => "cube-outline",
        "editor.game" => "game-controller-outline",
        "editor.assets" | "editor.asset_browser" => "folder-open-outline",
        "editor.console" => "terminal-outline",
        "editor.runtime_diagnostics" => "grid-outline",
        "editor.module_plugins" => "git-network-outline",
        "editor.build_export_desktop" => "share-outline",
        "editor.prefab" => "cube-outline",
        _ => "ellipse-outline",
    }
}

fn menu_label_icon_name(label: &str) -> &'static str {
    let label = label.to_lowercase();
    if label.contains("open") {
        "folder-open-outline"
    } else if label.contains("save") {
        "save-outline"
    } else if label.contains("reset") || label.contains("reload") || label.contains("refresh") {
        "sync-outline"
    } else if label.contains("play") {
        "play-outline"
    } else if label.contains("delete") || label.contains("remove") {
        "remove-outline"
    } else if label.contains("guide") || label.contains("help") {
        "construct-outline"
    } else {
        "ellipse-outline"
    }
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
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
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
    nodes
}

pub(super) fn page_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    tab_frames(nodes, PAGE_TAB_PREFIX, None, tabs)
}

pub(super) fn page_tab_row_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, PAGE_BAR_CONTROL_ID)
}

pub(super) fn page_project_path_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, PAGE_PROJECT_PATH_CONTROL_ID)
}

pub(super) fn activity_rail_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    model_rc(expand_activity_rail_button_nodes(
        raw_template_nodes(
            "host.activity.rail",
            ACTIVITY_RAIL_ASSET,
            width,
            height,
            &BTreeMap::new(),
        ),
        tabs,
    ))
}

pub(super) fn activity_rail_button_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeControlFrameData> {
    control_frames(nodes, ACTIVITY_RAIL_BUTTON_PREFIX, tabs.row_count())
}

pub(super) fn activity_rail_active_control_id(tabs: &ModelRc<TabData>) -> SharedString {
    (0..tabs.row_count())
        .find(|row| tabs.row_data(*row).is_some_and(|tab| tab.active))
        .map(|row| format!("{ACTIVITY_RAIL_BUTTON_PREFIX}{row}").into())
        .unwrap_or_default()
}

fn expand_activity_rail_button_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut button_templates = BTreeMap::new();
    let mut icon_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), ACTIVITY_RAIL_BUTTON_ICON_PREFIX) {
            icon_templates.insert(row, node);
        } else if let Some(row) = slot_index(node.control_id.as_str(), ACTIVITY_RAIL_BUTTON_PREFIX)
        {
            button_templates.insert(row, node);
        } else {
            output_nodes.push(node);
        }
    }

    let row_step = indexed_row_step(&button_templates, ACTIVITY_RAIL_ROW_STEP_FALLBACK_PX);
    for item_index in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(item_index) else {
            continue;
        };

        if let Some(mut button_node) = indexed_slot_node(
            &button_templates,
            ACTIVITY_RAIL_BUTTON_PREFIX,
            ACTIVITY_RAIL_STENCIL_COUNT,
            item_index,
            row_step,
            None,
        ) {
            button_node.surface_variant = if tab.active { "inset" } else { "" }.into();
            button_node.selected = tab.active;
            button_node.focused = tab.active;
            output_nodes.push(button_node);
        }
        if let Some(mut icon_node) = indexed_slot_node(
            &icon_templates,
            ACTIVITY_RAIL_BUTTON_ICON_PREFIX,
            ACTIVITY_RAIL_STENCIL_COUNT,
            item_index,
            row_step,
            None,
        ) {
            let icon_name = chrome_tab_icon_name(&tab);
            icon_node.text = "".into();
            apply_template_icon(&mut icon_node, &icon_name);
            icon_node.selected = tab.active;
            icon_node.focused = tab.active;
            if tab.active {
                icon_node.text_tone = "default".into();
                icon_node.font_weight = 700;
            } else {
                icon_node.text_tone = "muted".into();
                icon_node.font_weight = 600;
            }
            output_nodes.push(icon_node);
        }
    }

    output_nodes
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
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    let nodes = tab_template_nodes(
        "host.side.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, &"".into(), width, height);
    }
    nodes
}

pub(super) fn document_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), subtitle.to_string());
    let nodes = tab_template_nodes(
        "host.document.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, subtitle, width, height);
    }
    nodes
}

pub(super) fn bottom_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    let nodes = tab_template_nodes(
        "host.bottom.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, &"".into(), width, height);
    }
    nodes
}

pub(super) fn floating_window_header_nodes(
    tabs: &ModelRc<TabData>,
    title: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), title.to_string());
    let nodes = tab_template_nodes(
        "host.floating.window.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    );
    if tab_chrome_needs_fallback(&nodes, DOCK_HEADER_BAR_CONTROL_ID, DOCK_TAB_PREFIX, tabs) {
        return fallback_dock_header_nodes(tabs, title, width, height);
    }
    nodes
}

pub(super) fn dock_header_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_HEADER_BAR_CONTROL_ID)
}

pub(super) fn dock_subtitle_frame(nodes: &ModelRc<ViewTemplateNodeData>) -> FrameRect {
    control_frame(nodes, DOCK_SUBTITLE_CONTROL_ID)
}

pub(super) fn dock_tab_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> ModelRc<HostChromeTabData> {
    tab_frames(nodes, DOCK_TAB_PREFIX, Some(DOCK_TAB_CLOSE_PREFIX), tabs)
}

pub(super) fn status_bar_nodes(
    status_primary: &SharedString,
    status_secondary: &SharedString,
    viewport_label: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        STATUS_PRIMARY_CONTROL_ID.to_string(),
        status_primary.to_string(),
    );
    text_overrides.insert(
        STATUS_SECONDARY_CONTROL_ID.to_string(),
        status_secondary.to_string(),
    );
    text_overrides.insert(
        STATUS_VIEWPORT_CONTROL_ID.to_string(),
        viewport_label.to_string(),
    );

    template_nodes(
        "host.status.bar",
        STATUS_BAR_ASSET,
        width,
        height,
        &text_overrides,
        &[],
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
            .filter(|node| node_survives_dock_tab_close_filter(node, tabs))
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
    let bar_height = height.max(PAGE_BAR_HEIGHT_FALLBACK_PX);
    let mut nodes = Vec::with_capacity(tabs.row_count() + 2);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackWorkbenchPageBar".into(),
        control_id: PAGE_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: bar_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = 12.0;
    let right_label_width = width.mul_add(0.28, 0.0).clamp(160.0, 320.0);
    let max_tab_right = (width - right_label_width - 12.0).max(12.0);
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let tab_width = ((tab.title.as_str().len() as f32 * 7.0) + 34.0).clamp(70.0, 180.0);
        let draw_width = tab_width.min((max_tab_right - x).max(42.0));
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let icon_name = chrome_tab_icon_name(&tab);
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackPageTab{row}").into(),
            control_id: format!("{PAGE_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: tab.title.clone(),
            text_tone: text_tone.into(),
            font_size: CHROME_TEXT_FONT_SIZE_PX,
            font_weight,
            surface_variant: if tab.active { "inset" } else { "" }.into(),
            button_variant: "ghost".into(),
            selected: tab.active,
            focused: tab.active,
            frame: ViewTemplateFrameData {
                x,
                y: CHROME_TAB_HEIGHT_INSET_PX,
                width: draw_width,
                height: (bar_height - CHROME_TAB_HEIGHT_INSET_PX).max(20.0),
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &icon_name);
        nodes.push(tab_node);
        x = (x + tab_width + 4.0).min(max_tab_right);
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
        font_size: 11.0,
        frame: ViewTemplateFrameData {
            x: (width - right_label_width - 12.0).max(12.0),
            y: 6.0,
            width: right_label_width,
            height: (bar_height - 12.0).max(16.0),
        },
        ..ViewTemplateNodeData::default()
    });

    model_rc(nodes)
}

fn fallback_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let header_height = height.max(DOCK_HEADER_HEIGHT_FALLBACK_PX);
    let mut nodes = Vec::with_capacity(tabs.row_count() * 2 + 2);
    nodes.push(ViewTemplateNodeData {
        node_id: "FallbackDockHeaderBar".into(),
        control_id: DOCK_HEADER_BAR_CONTROL_ID.into(),
        role: "Panel".into(),
        surface_variant: "panel".into(),
        frame: ViewTemplateFrameData {
            x: 0.0,
            y: 0.0,
            width: width.max(1.0),
            height: header_height,
        },
        ..ViewTemplateNodeData::default()
    });

    let mut x = 4.0;
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let tab_width = ((tab.title.as_str().len() as f32 * 7.0) + 36.0).clamp(56.0, 150.0);
        let text_tone = if tab.active { "default" } else { "subtle" };
        let font_weight = if tab.active { 600 } else { 400 };
        let icon_name = chrome_tab_icon_name(&tab);
        let mut tab_node = ViewTemplateNodeData {
            node_id: format!("FallbackDockTab{row}").into(),
            control_id: format!("{DOCK_TAB_PREFIX}{row}").into(),
            role: "Button".into(),
            text: tab.title.clone(),
            text_tone: text_tone.into(),
            font_size: CHROME_TEXT_FONT_SIZE_PX,
            font_weight,
            surface_variant: if tab.active { "inset" } else { "" }.into(),
            button_variant: "ghost".into(),
            selected: tab.active,
            focused: tab.active,
            frame: ViewTemplateFrameData {
                x,
                y: CHROME_TAB_HEIGHT_INSET_PX,
                width: tab_width,
                height: (header_height - CHROME_TAB_HEIGHT_INSET_PX).max(20.0),
            },
            ..ViewTemplateNodeData::default()
        };
        apply_template_icon(&mut tab_node, &icon_name);
        nodes.push(tab_node);
        if tab.closeable {
            let mut close_node = ViewTemplateNodeData {
                node_id: format!("FallbackDockTabClose{row}").into(),
                control_id: format!("{DOCK_TAB_CLOSE_PREFIX}{row}").into(),
                role: "IconButton".into(),
                text_tone: "muted".into(),
                font_size: 11.0,
                surface_variant: "inset".into(),
                button_variant: "ghost".into(),
                frame: ViewTemplateFrameData {
                    x: x + tab_width - 19.0,
                    y: 7.0,
                    width: 14.0,
                    height: 16.0,
                },
                ..ViewTemplateNodeData::default()
            };
            apply_template_icon(&mut close_node, DOCK_TAB_CLOSE_ICON);
            nodes.push(close_node);
        }
        x += tab_width + 3.0;
    }

    if !subtitle.is_empty() {
        nodes.push(ViewTemplateNodeData {
            node_id: "FallbackDockSubtitle".into(),
            control_id: DOCK_SUBTITLE_CONTROL_ID.into(),
            role: "Text".into(),
            text: subtitle.clone(),
            text_tone: "muted".into(),
            font_size: 10.0,
            frame: ViewTemplateFrameData {
                x: (x + 8.0).min(width.max(1.0)),
                y: 7.0,
                width: (width - x - 16.0).max(0.0),
                height: 16.0,
            },
            ..ViewTemplateNodeData::default()
        });
    }

    model_rc(nodes)
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

fn measured_height_or(measured_height: f32, fallback_height: f32) -> f32 {
    if measured_height > 0.0 {
        measured_height
    } else {
        fallback_height
    }
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
            node.focused = tab.active;
        }
        if tabs.row_data(row).is_some_and(|tab| tab.active) {
            node.text_tone = "default".into();
            node.font_weight = 600;
        } else {
            node.text_tone = "subtle".into();
            node.font_weight = 400;
        }
    } else if prefix == DOCK_TAB_PREFIX
        && slot_index(node.control_id.as_str(), DOCK_TAB_CLOSE_PREFIX).is_some()
    {
        node.role = "IconButton".into();
        node.text = "".into();
        node.text_tone = "muted".into();
        apply_template_icon(&mut node, DOCK_TAB_CLOSE_ICON);
    }
    node
}

fn apply_template_icon(node: &mut ViewTemplateNodeData, icon_name: &str) {
    node.icon_name = icon_name.into();
    node.media_source = format!("ionicons/{icon_name}.svg").into();
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

fn node_survives_dock_tab_close_filter(
    node: &ViewTemplateNodeData,
    tabs: &ModelRc<TabData>,
) -> bool {
    let Some(row) = slot_index(node.control_id.as_str(), DOCK_TAB_CLOSE_PREFIX) else {
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

fn control_frame_from_slice(nodes: &[ViewTemplateNodeData], control_id: &str) -> FrameRect {
    nodes
        .iter()
        .find(|node| node.control_id.as_str() == control_id)
        .map(frame_rect)
        .unwrap_or_default()
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
mod tests {
    use super::*;
    use slint::Model;

    use crate::ui::layouts::windows::workbench_host_window::HostMenuChromeItemData;

    #[test]
    fn dock_header_nodes_hide_close_controls_for_non_closeable_tabs() {
        let tabs = model_rc(vec![test_tab("Welcome", true, false)]);

        let nodes = document_dock_header_nodes(&tabs, &"".into(), 800.0, 31.0);

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

        let nodes = document_dock_header_nodes(&tabs, &"".into(), 800.0, 31.0);

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
    fn menu_popup_nodes_project_absolute_rows_beyond_authored_slots() {
        let items = model_rc(
            (0..18)
                .map(|index| HostMenuChromeItemData {
                    label: format!("Preset {index:02}").into(),
                    shortcut: "".into(),
                    action_id: format!("LoadPreset.Preset{index:02}").into(),
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
    fn menu_popup_nodes_project_action_svg_icons() {
        let items = model_rc(vec![
            test_menu_item("Open Project", "Ctrl+O", "OpenProject", true),
            test_menu_item("Save Project", "Ctrl+S", "SaveProject", true),
            test_menu_item("Undo", "Ctrl+Z", "Undo", false),
            test_menu_item(
                "Build Export",
                "",
                "OpenView.editor.build_export_desktop",
                true,
            ),
            test_menu_item("Create Cube", "", "CreateNode.Cube", true),
        ]);

        let nodes = menu_popup_nodes(&items, 224.0, 180.0);
        let open = node(&nodes, "MenuPopupItemLabel0");
        let save = node(&nodes, "MenuPopupItemLabel1");
        let undo = node(&nodes, "MenuPopupItemLabel2");
        let export = node(&nodes, "MenuPopupItemLabel3");
        let cube = node(&nodes, "MenuPopupItemLabel4");

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
    }

    #[test]
    fn activity_rail_nodes_project_tab_svg_icons_and_selected_state() {
        let tabs = model_rc(vec![
            test_tab_with_icon("Project", "project", true, false),
            test_tab_with_icon("Hierarchy", "hierarchy", false, false),
        ]);

        let nodes = activity_rail_nodes(&tabs, 34.0, 96.0);
        let project_button = node(&nodes, "ActivityRailButton0");
        let project_icon = node(&nodes, "ActivityRailButtonIcon0");
        let hierarchy_icon = node(&nodes, "ActivityRailButtonIcon1");

        assert_eq!(
            project_button.surface_variant.as_str(),
            "inset",
            "active activity rail button should project selected surface metadata"
        );
        assert_eq!(project_icon.icon_name.as_str(), "albums-outline");
        assert!(
            project_icon.has_preview_image,
            "activity rail icons should resolve SVG preview pixels during chrome projection"
        );
        assert_eq!(project_icon.text_tone.as_str(), "default");
        assert_eq!(hierarchy_icon.icon_name.as_str(), "layers-outline");
        assert_eq!(hierarchy_icon.text_tone.as_str(), "muted");
    }

    #[test]
    fn page_and_dock_tabs_project_svg_icons_and_close_button_icon() {
        let tabs = model_rc(vec![
            test_tab_with_icon("Scene", "scene", true, true),
            test_tab_with_icon("Assets", "asset-browser", false, true),
        ]);

        let page_nodes = page_chrome_nodes(&tabs, &"Demo".into(), 640.0, 64.0);
        let page_scene = node(&page_nodes, "PageTab0");
        let page_assets = node(&page_nodes, "PageTab1");

        assert_eq!(page_scene.icon_name.as_str(), "cube-outline");
        assert!(page_scene.has_preview_image);
        assert!(page_scene.selected);
        assert_eq!(page_scene.text_tone.as_str(), "default");
        assert_eq!(page_assets.icon_name.as_str(), "folder-open-outline");
        assert!(page_assets.has_preview_image);
        assert!(!page_assets.selected);
        assert_eq!(page_assets.text_tone.as_str(), "subtle");

        let dock_nodes = document_dock_header_nodes(&tabs, &"".into(), 640.0, 40.0);
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

        assert!(bar.frame.height >= PAGE_BAR_HEIGHT_FALLBACK_PX);
        assert!(
            first_tab.frame.width > 0.0 && first_tab.frame.height > 0.0,
            "fallback page tabs must stay hit-testable when the template projection is unavailable"
        );
        assert_eq!(first_tab.surface_variant.as_str(), "inset");
        assert_eq!(second_tab.text_tone.as_str(), "subtle");
        assert_eq!(project_path.text.as_str(), "ZirconProject4");
        assert!(project_path.frame.width > 0.0);
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

        assert!(header.frame.height >= DOCK_HEADER_HEIGHT_FALLBACK_PX);
        assert!(
            scene.frame.width > 0.0 && scene.frame.height > 0.0,
            "fallback dock tabs must provide drag/click hit frames"
        );
        assert!(scene_close.frame.width > 0.0 && scene_close.frame.height > 0.0);
        assert!(maybe_node(&nodes, "DockTabClose1").is_none());
        assert_eq!(game.text_tone.as_str(), "subtle");
        assert_eq!(subtitle.text.as_str(), "Preview");
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
}
