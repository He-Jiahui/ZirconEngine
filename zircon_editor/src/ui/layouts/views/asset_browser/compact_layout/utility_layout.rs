use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

const COMPACT_UTILITY_CONTENT_OFFSET_Y: f32 = 28.0;
const COMPACT_UTILITY_HEIGHT: f32 = 104.0;
const COMPACT_COLLAPSED_UTILITY_HEIGHT: f32 = 28.0;
pub(super) const COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD: f32 = 560.0;
const COMPACT_UTILITY_TAB_HEIGHT: f32 = 22.0;
const COMPACT_UTILITY_TAB_GAP: f32 = 6.0;
const COMPACT_UTILITY_TAB_WIDTHS: [(&str, f32); 4] = [
    ("AssetBrowserPreviewTabButton", 68.0),
    ("AssetBrowserReferencesTabButton", 92.0),
    ("AssetBrowserMetadataTabButton", 84.0),
    ("AssetBrowserPluginsTabButton", 72.0),
];
const COMPACT_UTILITY_LOCATOR_GAP: f32 = 12.0;
const COMPACT_UTILITY_LOCATOR_WIDTH: f32 = 156.0;

pub(super) fn apply_compact_utility_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let tabs_height = COMPACT_UTILITY_TAB_HEIGHT.min(height);
    let content_offset = COMPACT_UTILITY_CONTENT_OFFSET_Y.min(height);
    let content_y = y + content_offset;
    let content_height = finite_non_negative(height - content_offset);
    set_node_frame(
        nodes,
        "AssetBrowserUtilityTabsRow",
        x,
        y,
        width,
        tabs_height,
    );
    let tabs_end = apply_compact_utility_tab_button_layout(nodes, x, y, width, tabs_height);
    set_node_frame(
        nodes,
        "AssetBrowserUtilityDivider",
        x,
        y + 26.0,
        width,
        1.0_f32.min(finite_non_negative(height - 26.0)),
    );
    set_node_frame(
        nodes,
        "AssetBrowserUtilityContentPanel",
        x,
        content_y,
        width,
        content_height,
    );
    if content_height <= 1.0 {
        collapse_compact_utility_content(nodes, x, content_y, width);
        set_node_frame(nodes, "AssetBrowserSelectionLocatorText", x, y, 0.0, 0.0);
        return;
    }
    if node_frame(nodes, "AssetBrowserPreviewPanel").is_some() {
        apply_compact_preview_utility_layout(nodes, x, content_y, width, content_height);
    }
    apply_compact_utility_locator_layout(nodes, x, y, width, tabs_end, tabs_height);
}

fn apply_compact_utility_tab_button_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> f32 {
    let mut cursor_x = x;
    let panel_right = x + finite_non_negative(width);
    for (index, (control_id, width)) in COMPACT_UTILITY_TAB_WIDTHS.iter().enumerate() {
        if index > 0 {
            cursor_x += COMPACT_UTILITY_TAB_GAP;
        }
        if cursor_x + *width > panel_right {
            set_node_frame(nodes, control_id, panel_right, y, 0.0, 0.0);
            continue;
        }
        set_node_frame(nodes, control_id, cursor_x, y, *width, height);
        cursor_x += *width;
    }
    cursor_x.min(panel_right)
}

fn apply_compact_utility_locator_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    tabs_end: f32,
    height: f32,
) {
    let row_right = x + width;
    let locator_x = (row_right - COMPACT_UTILITY_LOCATOR_WIDTH)
        .max(tabs_end + COMPACT_UTILITY_LOCATOR_GAP)
        .min(row_right);
    let locator_width =
        finite_non_negative(row_right - locator_x).min(COMPACT_UTILITY_LOCATOR_WIDTH);
    set_node_frame(
        nodes,
        "AssetBrowserSelectionLocatorText",
        locator_x,
        y,
        locator_width,
        height,
    );
}

pub(super) fn compact_asset_browser_vertical_budget(
    viewport_height: f32,
    main_y: f32,
    panel_gap: f32,
) -> (f32, f32, f32) {
    let viewport_height = finite_non_negative(viewport_height);
    let main_y = finite_non_negative(main_y).min(viewport_height);
    let remaining_height = finite_non_negative(viewport_height - main_y);
    let utility_height =
        compact_asset_browser_utility_height_for_viewport(viewport_height).min(remaining_height);
    let utility_y = viewport_height - utility_height;
    let main_height = finite_non_negative(utility_y - main_y - finite_non_negative(panel_gap));
    (main_height, utility_y, utility_height)
}

fn compact_asset_browser_utility_height_for_viewport(viewport_height: f32) -> f32 {
    let viewport_height = finite_non_negative(viewport_height);
    if viewport_height < COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD {
        COMPACT_COLLAPSED_UTILITY_HEIGHT.min(viewport_height)
    } else {
        COMPACT_UTILITY_HEIGHT.min(viewport_height * 0.24)
    }
}

fn collapse_compact_utility_content(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
) {
    for control_id in [
        "AssetBrowserPreviewPanel",
        "AssetBrowserPreviewVisualPanel",
        "AssetBrowserPreviewNameText",
        "AssetBrowserPreviewLocatorText",
        "AssetBrowserPreviewKindText",
        "AssetBrowserPreviewIdentityText",
        "AssetBrowserPreviewToolkitText",
        "AssetBrowserPreviewMetaPathText",
        "AssetBrowserPreviewDiagnosticsText",
    ] {
        set_node_frame(nodes, control_id, x, y, width, 0.0);
    }
}

fn apply_compact_preview_utility_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let visual_width = 64.0_f32.min(width * 0.16);
    let text_x = x + visual_width + 22.0;
    let text_width = finite_non_negative(width - visual_width - 34.0);
    set_node_frame(nodes, "AssetBrowserPreviewPanel", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetBrowserPreviewVisualPanel",
        x + 8.0,
        y + 8.0,
        visual_width,
        finite_non_negative(height - 16.0),
    );
    for (control_id, offset_y, line_height) in [
        ("AssetBrowserPreviewNameText", 10.0, 14.0),
        ("AssetBrowserPreviewLocatorText", 27.0, 12.0),
        ("AssetBrowserPreviewKindText", 42.0, 12.0),
        ("AssetBrowserPreviewIdentityText", 56.0, 12.0),
        ("AssetBrowserPreviewToolkitText", 69.0, 10.0),
        ("AssetBrowserPreviewMetaPathText", 80.0, 10.0),
        ("AssetBrowserPreviewDiagnosticsText", 91.0, 10.0),
    ] {
        set_node_frame(
            nodes,
            control_id,
            text_x,
            y + offset_y,
            text_width,
            compact_line_height(height, offset_y, line_height),
        );
    }
}

pub(super) fn shift_asset_browser_utility_nodes(nodes: &mut [ViewTemplateNodeData], delta_y: f32) {
    if delta_y.abs() <= f32::EPSILON {
        return;
    }
    for node in nodes {
        if is_asset_browser_utility_control(node.control_id.as_str()) {
            node.frame.y = finite_coordinate(node.frame.y) + delta_y;
        }
    }
}

fn is_asset_browser_utility_control(control_id: &str) -> bool {
    control_id.starts_with("AssetBrowserUtility")
        || control_id.starts_with("AssetBrowserPreview")
        || control_id.starts_with("AssetBrowserReferences")
        || control_id.starts_with("AssetBrowserMetadata")
        || control_id.starts_with("AssetBrowserReference")
        || control_id.starts_with("AssetBrowserMetaPath")
        || control_id.starts_with("AssetBrowserToolkit")
        || control_id.starts_with("AssetBrowserDiagnostics")
        || control_id.starts_with("AssetBrowserPlugins")
        || control_id == "AssetBrowserSelectionLocatorText"
}

pub(super) fn compact_line_height(
    container_height: f32,
    offset_y: f32,
    preferred_height: f32,
) -> f32 {
    finite_non_negative(preferred_height).min(finite_non_negative(
        finite_non_negative(container_height) - finite_non_negative(offset_y),
    ))
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}

fn set_node_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id == control_id)
    {
        node.frame = ViewTemplateFrameData {
            x: finite_coordinate(x),
            y: finite_coordinate(y),
            width: finite_non_negative(width),
            height: finite_non_negative(height),
        };
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
