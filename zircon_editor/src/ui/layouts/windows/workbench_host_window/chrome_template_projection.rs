use std::collections::BTreeMap;

use slint::{Model, ModelRc, SharedString};
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::{build_view_template_nodes, ViewTemplateNodeData};

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
pub(super) const MENU_SLOT_COUNT: usize = 6;
pub(super) const MENU_POPUP_ITEM_COUNT: usize = 16;
const MENU_POPUP_ITEM_LABEL_PREFIX: &str = "MenuPopupItemLabel";
const MENU_POPUP_ITEM_SHORTCUT_PREFIX: &str = "MenuPopupItemShortcut";
const MENU_POPUP_ITEM_ROW_PREFIX: &str = "MenuPopupItemRow";
const MENU_POPUP_ROW_STEP_FALLBACK_PX: f32 = 30.0;
const PAGE_TAB_PREFIX: &str = "PageTab";
const DOCK_TAB_PREFIX: &str = "DockTab";
const DOCK_TAB_CLOSE_PREFIX: &str = "DockTabClose";
const ACTIVITY_RAIL_BUTTON_PREFIX: &str = "ActivityRailButton";
const ACTIVITY_RAIL_BUTTON_LABEL_PREFIX: &str = "ActivityRailButtonLabel";
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
    let dock_header_height =
        control_frame_from_slice(&dock_nodes, DOCK_HEADER_BAR_CONTROL_ID).height;

    HostWindowSurfaceMetricsData {
        outer_margin_px: OUTER_MARGIN_PX,
        rail_width_px: RAIL_WIDTH_PX,
        top_bar_height_px: control_frame_from_slice(&menu_nodes, MENU_TOP_BAR_CONTROL_ID).height,
        host_bar_height_px: control_frame_from_slice(&page_nodes, PAGE_BAR_CONTROL_ID).height,
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

    template_nodes(
        "host.menu.chrome",
        MENU_CHROME_ASSET,
        width,
        height,
        &text_overrides,
        &[SlotFilter::new(MENU_SLOT_PREFIX, MENU_SLOT_COUNT)],
    )
}

pub(super) fn menu_control_frames(
    nodes: &ModelRc<ViewTemplateNodeData>,
    count: usize,
) -> ModelRc<HostChromeControlFrameData> {
    control_frames(nodes, MENU_SLOT_PREFIX, count)
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

    tab_template_nodes(
        "host.page.chrome",
        PAGE_CHROME_ASSET,
        width,
        height,
        &text_overrides,
        PAGE_TAB_PREFIX,
        tabs,
    )
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
        .map(|row| format!("{ACTIVITY_RAIL_BUTTON_LABEL_PREFIX}{row}").into())
        .unwrap_or_default()
}

fn expand_activity_rail_button_nodes(
    raw_nodes: Vec<ViewTemplateNodeData>,
    tabs: &ModelRc<TabData>,
) -> Vec<ViewTemplateNodeData> {
    let mut output_nodes = Vec::new();
    let mut button_templates = BTreeMap::new();
    let mut label_templates = BTreeMap::new();

    for node in raw_nodes {
        if let Some(row) = slot_index(node.control_id.as_str(), ACTIVITY_RAIL_BUTTON_LABEL_PREFIX) {
            label_templates.insert(row, node);
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
            output_nodes.push(button_node);
        }
        if let Some(mut label_node) = indexed_slot_node(
            &label_templates,
            ACTIVITY_RAIL_BUTTON_LABEL_PREFIX,
            ACTIVITY_RAIL_STENCIL_COUNT,
            item_index,
            row_step,
            Some(activity_rail_label_text(&tab).as_str()),
        ) {
            if tab.active {
                label_node.text_tone = "default".into();
                label_node.font_weight = 700;
            } else {
                label_node.text_tone = "muted".into();
                label_node.font_weight = 600;
            }
            output_nodes.push(label_node);
        }
    }

    output_nodes
}

fn activity_rail_label_text(tab: &TabData) -> String {
    let mut label = compact_ascii_label(tab.title.as_str());
    if label.is_empty() {
        label = compact_ascii_label(tab.icon_key.as_str());
    }
    if label.is_empty() {
        label.push('?');
    }
    label
}

fn compact_ascii_label(value: &str) -> String {
    let mut label: String = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(2)
        .collect();
    label.make_ascii_uppercase();
    label
}

pub(super) fn side_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    tab_template_nodes(
        "host.side.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    )
}

pub(super) fn document_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    subtitle: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), subtitle.to_string());
    tab_template_nodes(
        "host.document.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    )
}

pub(super) fn bottom_dock_header_nodes(
    tabs: &ModelRc<TabData>,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    tab_template_nodes(
        "host.bottom.dock.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    )
}

pub(super) fn floating_window_header_nodes(
    tabs: &ModelRc<TabData>,
    title: &SharedString,
    width: f32,
    height: f32,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = tab_text_overrides(DOCK_TAB_PREFIX, tabs);
    text_overrides.insert(DOCK_SUBTITLE_CONTROL_ID.to_string(), title.to_string());
    tab_template_nodes(
        "host.floating.window.header",
        DOCK_HEADER_ASSET,
        width,
        height,
        &text_overrides,
        DOCK_TAB_PREFIX,
        tabs,
    )
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
    let active_rows = active_tab_rows(tabs);
    let filters = [SlotFilter::new(slot_prefix, tabs.row_count())];
    let nodes = raw_template_nodes(document_tree_id, asset_path, width, height, text_overrides);
    model_rc(
        nodes
            .into_iter()
            .filter(|node| node_survives_filters(node, &filters))
            .map(|node| tab_node_with_state(node, slot_prefix, &active_rows))
            .collect(),
    )
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

fn active_tab_rows(tabs: &ModelRc<TabData>) -> Vec<usize> {
    (0..tabs.row_count())
        .filter(|row| tabs.row_data(*row).is_some_and(|tab| tab.active))
        .collect()
}

fn tab_node_with_state(
    mut node: ViewTemplateNodeData,
    prefix: &str,
    active_rows: &[usize],
) -> ViewTemplateNodeData {
    if let Some(row) = slot_index(node.control_id.as_str(), prefix) {
        if active_rows.contains(&row) {
            node.text_tone = "default".into();
            node.font_weight = 600;
        } else {
            node.text_tone = "subtle".into();
            node.font_weight = 400;
        }
    }
    node
}

fn node_survives_filters(node: &ViewTemplateNodeData, filters: &[SlotFilter]) -> bool {
    filters.iter().all(
        |filter| match slot_index(node.control_id.as_str(), filter.prefix) {
            Some(row) => row < filter.used_count,
            None => true,
        },
    )
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
    fn menu_popup_nodes_project_absolute_rows_beyond_authored_slots() {
        let items = model_rc(
            (0..18)
                .map(|index| HostMenuChromeItemData {
                    label: format!("Preset {index:02}").into(),
                    shortcut: "".into(),
                    action_id: format!("LoadPreset.Preset{index:02}").into(),
                    enabled: index != 17,
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
    }

    fn node(nodes: &ModelRc<ViewTemplateNodeData>, control_id: &str) -> ViewTemplateNodeData {
        (0..nodes.row_count())
            .filter_map(|row| nodes.row_data(row))
            .find(|node| node.control_id.as_str() == control_id)
            .unwrap_or_else(|| panic!("missing projected popup node {control_id}"))
    }
}
