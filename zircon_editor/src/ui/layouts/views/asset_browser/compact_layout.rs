use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::workbench::snapshot::AssetViewMode;
use zircon_runtime_interface::ui::layout::UiSize;

use super::compact_table_layout::{
    apply_compact_table_layout, asset_table_row_count, collapse_compact_table_nodes,
    compact_table_stack_height,
};
use super::summary_layout::apply_compact_content_preview_summary_layout;
use super::thumbnail_layout::{apply_compact_thumbnail_grid_layout, has_thumbnail_grid};

const COMPACT_LAYOUT_HEIGHT_THRESHOLD: f32 = 760.0;
const COMPACT_PANEL_GAP: f32 = 6.0;
const COMPACT_CONTENT_GAP: f32 = 8.0;
const COMPACT_CONTENT_HEADER_HEIGHT: f32 = 20.0;
const COMPACT_HEADER_TABLE_GAP: f32 = 4.0;
const COMPACT_HEADER_HORIZONTAL_PADDING: f32 = 8.0;
const COMPACT_UTILITY_CONTENT_OFFSET_Y: f32 = 28.0;
const COMPACT_UTILITY_HEIGHT: f32 = 104.0;
const COMPACT_COLLAPSED_UTILITY_HEIGHT: f32 = 28.0;
const COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD: f32 = 560.0;
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
const COMPACT_COLLAPSED_SOURCES_WIDTH_THRESHOLD: f32 = 900.0;
const COMPACT_COLLAPSED_DETAILS_MAIN_HEIGHT_THRESHOLD: f32 = 300.0;
const COMPACT_PREVIEW_CARD_HEIGHT: f32 = 50.0;
const COMPACT_DETAILS_HEADER_HEIGHT: f32 = 42.0;
const COMPACT_DETAILS_DIVIDER_HEIGHT: f32 = 1.0;
const COMPACT_DETAILS_PREVIEW_HEIGHT: f32 = 96.0;
const COMPACT_DETAILS_FIELD_HEIGHT: f32 = 42.0;
const COMPACT_DETAILS_IDENTITY_HEIGHT: f32 = 50.0;
const COMPACT_DETAILS_METADATA_HEIGHT: f32 = 52.0;
const COMPACT_DETAILS_DIAGNOSTICS_HEIGHT: f32 = 54.0;

pub(super) fn apply_asset_browser_compact_layout(
    nodes: &mut [ViewTemplateNodeData],
    size: UiSize,
    view_mode: AssetViewMode,
    toolbar_main_y: Option<f32>,
) {
    if size.height >= COMPACT_LAYOUT_HEIGHT_THRESHOLD {
        return;
    }

    let Some(root) = node_frame(nodes, "AssetBrowserRoot") else {
        return;
    };
    let Some(main) = node_frame(nodes, "AssetBrowserMainPanel") else {
        return;
    };
    let Some(utility) = node_frame(nodes, "AssetBrowserUtilityPanel") else {
        return;
    };

    let viewport_height = size.height.max(360.0);
    let viewport_width = size.width.max(root.width);
    set_node_frame(
        nodes,
        "AssetBrowserRoot",
        root.x,
        root.y,
        viewport_width,
        viewport_height,
    );

    let main_y = toolbar_main_y.unwrap_or(main.y);

    let compact_utility_height = compact_asset_browser_utility_height_for_viewport(viewport_height);
    let utility_y = (viewport_height - compact_utility_height)
        .max(main_y + 152.0 + COMPACT_PANEL_GAP)
        .min(viewport_height - 64.0);
    let main_height = (utility_y - COMPACT_PANEL_GAP - main_y).max(152.0);
    let utility_delta_y = utility_y - utility.y;

    shift_asset_browser_utility_nodes(nodes, utility_delta_y);
    set_node_frame(
        nodes,
        "AssetBrowserMainPanel",
        main.x,
        main_y,
        main.width,
        main_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserUtilityPanel",
        utility.x,
        utility_y,
        utility.width,
        compact_utility_height,
    );

    let collapse_sources = viewport_width < COMPACT_COLLAPSED_SOURCES_WIDTH_THRESHOLD;
    let collapse_details = main_height < COMPACT_COLLAPSED_DETAILS_MAIN_HEIGHT_THRESHOLD
        || viewport_height < COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD;
    apply_compact_main_panel_layout(
        nodes,
        main_y,
        main_height,
        collapse_sources,
        collapse_details,
        view_mode,
    );
    apply_compact_utility_panel_layout(
        nodes,
        utility.x,
        utility_y,
        utility.width,
        compact_utility_height,
    );
}

fn apply_compact_main_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    main_y: f32,
    main_height: f32,
    collapse_sources: bool,
    collapse_details: bool,
    view_mode: AssetViewMode,
) {
    let sources_frame = node_frame(nodes, "AssetBrowserSourcesPanel");
    if let Some(sources) = sources_frame.as_ref() {
        if collapse_sources {
            collapse_compact_sources_nodes(nodes, sources.x, main_y);
        } else {
            set_node_frame(
                nodes,
                "AssetBrowserSourcesPanel",
                sources.x,
                main_y,
                sources.width,
                main_height,
            );
            set_node_height(
                nodes,
                "AssetBrowserSourcesScrollBody",
                (main_height - 49.0).max(24.0),
            );
        }
    }

    let details_frame = node_frame(nodes, "AssetBrowserDetailsPanel");
    if let Some(content) = node_frame(nodes, "AssetBrowserContentPanel") {
        let content_x = if collapse_sources {
            sources_frame
                .as_ref()
                .map(|sources| sources.x)
                .unwrap_or(content.x)
        } else {
            content.x
        };
        let content_right = if collapse_details {
            details_frame
                .as_ref()
                .map(|details| details.x + details.width)
                .unwrap_or(content.x + content.width)
        } else {
            content.x + content.width
        };
        let content_height = main_height;
        let content_width = (content_right - content_x).max(content.width);
        set_node_frame(
            nodes,
            "AssetBrowserContentPanel",
            content_x,
            main_y,
            content_width,
            content_height,
        );
        apply_compact_content_panel_layout(
            nodes,
            content_x,
            main_y,
            content_width,
            content_height,
            view_mode,
        );
    }

    if let Some(details) = details_frame {
        if collapse_details {
            collapse_compact_details_nodes(nodes, details.x, main_y);
            return;
        }
        set_node_frame(
            nodes,
            "AssetBrowserDetailsPanel",
            details.x,
            main_y,
            details.width,
            main_height,
        );
        apply_compact_details_panel_layout(nodes, details.x, main_y, details.width, main_height);
    }
}

fn collapse_compact_sources_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id.starts_with("AssetBrowserSources") {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = 0.0;
            node.frame.height = 0.0;
        }
    }
}

fn collapse_compact_details_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id.starts_with("AssetBrowserDetails") {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = 0.0;
            node.frame.height = 0.0;
        }
    }
}

fn apply_compact_content_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    view_mode: AssetViewMode,
) {
    let header_height = COMPACT_CONTENT_HEADER_HEIGHT;
    let has_thumbnail_view = view_mode == AssetViewMode::Thumbnail && has_thumbnail_grid(nodes);
    let preview_height = if has_thumbnail_view {
        0.0
    } else {
        COMPACT_PREVIEW_CARD_HEIGHT.min((height * 0.28).max(42.0))
    };
    let row_count = asset_table_row_count(nodes);
    let table_height = compact_table_stack_height(
        height - header_height - preview_height - COMPACT_HEADER_TABLE_GAP - COMPACT_CONTENT_GAP,
        row_count,
    );
    let table_y = y + header_height + COMPACT_HEADER_TABLE_GAP;
    let preview_y = (y + height - preview_height).max(table_y + table_height + COMPACT_CONTENT_GAP);

    set_node_frame(
        nodes,
        "AssetBrowserContentHeaderRow",
        x,
        y,
        width,
        header_height,
    );
    apply_compact_content_header_layout(nodes, x, y, width, header_height);
    set_node_frame(
        nodes,
        "AssetBrowserAssetTablePanel",
        x,
        table_y,
        width,
        table_height,
    );
    if has_thumbnail_view {
        collapse_compact_table_nodes(nodes, x, table_y);
        let grid_height = (height - header_height - COMPACT_HEADER_TABLE_GAP).max(0.0);
        apply_compact_thumbnail_grid_layout(nodes, x, table_y, width, grid_height);
    } else {
        apply_compact_table_layout(nodes, x, table_y, width, row_count);
        apply_compact_content_preview_summary_layout(nodes, x, preview_y, width, preview_height);
    }
    collapse_duplicate_compact_container_nodes(
        nodes,
        &["AssetBrowserContentPanel", "AssetBrowserAssetTablePanel"],
    );
}

fn apply_compact_content_header_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let inner_x = x + COMPACT_HEADER_HORIZONTAL_PADDING;
    let inner_width = (width - COMPACT_HEADER_HORIZONTAL_PADDING * 2.0).max(0.0);
    let path_width = (inner_width * 0.32).min(220.0).max(96.0).min(inner_width);
    let title_width = (inner_width - path_width - COMPACT_HEADER_HORIZONTAL_PADDING).max(0.0);
    let text_y = y + ((height - 12.0) * 0.5).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserContentHeaderTitleText",
        inner_x,
        text_y,
        title_width,
        12.0,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentHeaderPathText",
        x + width - COMPACT_HEADER_HORIZONTAL_PADDING - path_width,
        text_y + 1.0,
        path_width,
        10.0,
    );
}

fn apply_compact_details_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let scroll_y = y + COMPACT_DETAILS_HEADER_HEIGHT + COMPACT_DETAILS_DIVIDER_HEIGHT;
    let scroll_height = (height - COMPACT_DETAILS_HEADER_HEIGHT - COMPACT_DETAILS_DIVIDER_HEIGHT)
        .max(COMPACT_DETAILS_PREVIEW_HEIGHT);
    let content_x = x + 8.0;
    let content_width = (width - 16.0).max(80.0);

    set_node_frame(
        nodes,
        "AssetBrowserDetailsHeaderPanel",
        x,
        y,
        width,
        COMPACT_DETAILS_HEADER_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserDetailsDivider",
        x,
        y + COMPACT_DETAILS_HEADER_HEIGHT,
        width,
        COMPACT_DETAILS_DIVIDER_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserDetailsScrollBody",
        x,
        scroll_y,
        width,
        scroll_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserDetailsContentPanel",
        content_x,
        scroll_y + 8.0,
        content_width,
        scroll_height,
    );

    let mut field_y = scroll_y + 8.0;
    apply_compact_details_preview_layout(nodes, content_x, field_y, content_width);
    field_y += COMPACT_DETAILS_PREVIEW_HEIGHT + COMPACT_CONTENT_GAP;
    for (control_id, field_height) in [
        (
            "AssetBrowserDetailsLocatorPanel",
            COMPACT_DETAILS_FIELD_HEIGHT,
        ),
        ("AssetBrowserDetailsTypePanel", COMPACT_DETAILS_FIELD_HEIGHT),
        (
            "AssetBrowserDetailsIdentityPanel",
            COMPACT_DETAILS_IDENTITY_HEIGHT,
        ),
        (
            "AssetBrowserDetailsMetadataPanel",
            COMPACT_DETAILS_METADATA_HEIGHT,
        ),
        (
            "AssetBrowserDetailsDiagnosticsPanel",
            COMPACT_DETAILS_DIAGNOSTICS_HEIGHT,
        ),
    ] {
        set_node_frame(
            nodes,
            control_id,
            content_x,
            field_y,
            content_width,
            field_height,
        );
        apply_compact_details_field_layout(
            nodes,
            control_id,
            content_x,
            field_y,
            content_width,
            field_height,
        );
        field_y += field_height + COMPACT_CONTENT_GAP;
    }
}

fn apply_compact_details_field_layout(
    nodes: &mut [ViewTemplateNodeData],
    panel_control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let label_x = x + 10.0;
    let value_x = label_x;
    let text_width = (width - 20.0).max(24.0);
    match panel_control_id {
        "AssetBrowserDetailsLocatorPanel" => {
            layout_label_value_field(
                nodes,
                "AssetBrowserDetailsLocatorLabel",
                "AssetBrowserDetailsLocatorValue",
                label_x,
                value_x,
                y,
                text_width,
            );
        }
        "AssetBrowserDetailsTypePanel" => {
            layout_label_value_field(
                nodes,
                "AssetBrowserDetailsTypeLabel",
                "AssetBrowserDetailsTypeValue",
                label_x,
                value_x,
                y,
                text_width,
            );
        }
        "AssetBrowserDetailsIdentityPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityLabel",
                label_x,
                y + 6.0,
                text_width,
                10.0,
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityUuidValue",
                value_x,
                y + 20.0,
                text_width,
                12.0,
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityRevisionValue",
                value_x,
                y + 34.0,
                text_width,
                10.0,
            );
        }
        "AssetBrowserDetailsMetadataPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataLabel",
                label_x,
                y + 6.0,
                text_width,
                10.0,
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataMetaPathValue",
                value_x,
                y + 20.0,
                text_width,
                12.0,
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataToolkitValue",
                value_x,
                y + 34.0,
                text_width,
                10.0,
            );
        }
        "AssetBrowserDetailsDiagnosticsPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsDiagnosticsLabel",
                label_x,
                y + 6.0,
                text_width,
                10.0,
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsDiagnosticsText",
                value_x,
                y + 20.0,
                text_width,
                (height - 28.0).max(10.0),
            );
        }
        _ => {}
    }
}

fn layout_label_value_field(
    nodes: &mut [ViewTemplateNodeData],
    label_control_id: &str,
    value_control_id: &str,
    label_x: f32,
    value_x: f32,
    y: f32,
    text_width: f32,
) {
    set_node_frame(nodes, label_control_id, label_x, y + 6.0, text_width, 10.0);
    set_node_frame(nodes, value_control_id, value_x, y + 20.0, text_width, 12.0);
}

fn apply_compact_details_preview_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
) {
    let visual_width = 48.0_f32.min((width * 0.34).max(36.0));
    let text_x = x + visual_width + 14.0;
    let text_width = (x + width - text_x - 8.0).max(32.0);
    set_node_frame(
        nodes,
        "AssetBrowserDetailsPreviewPanel",
        x,
        y,
        width,
        COMPACT_DETAILS_PREVIEW_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserDetailsPreviewVisualPanel",
        x + 8.0,
        y + 8.0,
        visual_width,
        COMPACT_DETAILS_PREVIEW_HEIGHT - 16.0,
    );
    for (control_id, offset_y, height) in [
        ("AssetBrowserDetailsPreviewNameText", 10.0, 14.0),
        ("AssetBrowserDetailsPreviewLocatorText", 27.0, 12.0),
        ("AssetBrowserDetailsPreviewKindText", 42.0, 12.0),
        ("AssetBrowserDetailsPreviewIdentityText", 56.0, 12.0),
        ("AssetBrowserDetailsPreviewToolkitText", 69.0, 12.0),
        ("AssetBrowserDetailsPreviewMetaPathText", 82.0, 10.0),
        ("AssetBrowserDetailsPreviewDiagnosticsText", 94.0, 10.0),
    ] {
        set_node_frame(nodes, control_id, text_x, y + offset_y, text_width, height);
    }
}

fn apply_compact_utility_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let content_y = y + COMPACT_UTILITY_CONTENT_OFFSET_Y;
    let content_height = (height - COMPACT_UTILITY_CONTENT_OFFSET_Y).max(0.0);
    set_node_frame(
        nodes,
        "AssetBrowserUtilityTabsRow",
        x,
        y,
        width,
        COMPACT_UTILITY_TAB_HEIGHT,
    );
    let tabs_end = apply_compact_utility_tab_button_layout(nodes, x, y);
    set_node_frame(nodes, "AssetBrowserUtilityDivider", x, y + 26.0, width, 1.0);
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
        return;
    }
    if node_frame(nodes, "AssetBrowserPreviewPanel").is_some() {
        apply_compact_preview_utility_layout(nodes, x, content_y, width, content_height);
    }
    apply_compact_utility_locator_layout(nodes, x, y, width, tabs_end);
}

fn apply_compact_utility_tab_button_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
) -> f32 {
    let mut cursor_x = x;
    for (index, (control_id, width)) in COMPACT_UTILITY_TAB_WIDTHS.iter().enumerate() {
        if index > 0 {
            cursor_x += COMPACT_UTILITY_TAB_GAP;
        }
        set_node_frame(
            nodes,
            control_id,
            cursor_x,
            y,
            *width,
            COMPACT_UTILITY_TAB_HEIGHT,
        );
        cursor_x += *width;
    }
    cursor_x
}

fn apply_compact_utility_locator_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    tabs_end: f32,
) {
    let row_right = x + width;
    let locator_x =
        (row_right - COMPACT_UTILITY_LOCATOR_WIDTH).max(tabs_end + COMPACT_UTILITY_LOCATOR_GAP);
    let locator_width = (row_right - locator_x)
        .max(0.0)
        .min(COMPACT_UTILITY_LOCATOR_WIDTH);
    set_node_frame(
        nodes,
        "AssetBrowserSelectionLocatorText",
        locator_x,
        y,
        locator_width,
        COMPACT_UTILITY_TAB_HEIGHT,
    );
}

fn compact_asset_browser_utility_height_for_viewport(viewport_height: f32) -> f32 {
    if viewport_height < COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD {
        COMPACT_COLLAPSED_UTILITY_HEIGHT
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
    let visual_width = 64.0_f32.min((width * 0.16).max(48.0));
    let text_x = x + visual_width + 22.0;
    let text_width = (width - visual_width - 34.0).max(64.0);
    set_node_frame(nodes, "AssetBrowserPreviewPanel", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetBrowserPreviewVisualPanel",
        x + 8.0,
        y + 8.0,
        visual_width,
        (height - 16.0).max(36.0),
    );
    for (control_id, offset_y, height) in [
        ("AssetBrowserPreviewNameText", 10.0, 14.0),
        ("AssetBrowserPreviewLocatorText", 27.0, 12.0),
        ("AssetBrowserPreviewKindText", 42.0, 12.0),
        ("AssetBrowserPreviewIdentityText", 56.0, 12.0),
        ("AssetBrowserPreviewToolkitText", 69.0, 10.0),
        ("AssetBrowserPreviewMetaPathText", 80.0, 10.0),
        ("AssetBrowserPreviewDiagnosticsText", 91.0, 10.0),
    ] {
        set_node_frame(nodes, control_id, text_x, y + offset_y, text_width, height);
    }
}

fn shift_asset_browser_utility_nodes(nodes: &mut [ViewTemplateNodeData], delta_y: f32) {
    if delta_y.abs() <= f32::EPSILON {
        return;
    }
    for node in nodes {
        let control_id = node.control_id.as_str();
        if is_asset_browser_utility_control(control_id) {
            node.frame.y += delta_y;
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

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}

fn set_node_height(nodes: &mut [ViewTemplateNodeData], control_id: &str, height: f32) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id == control_id)
    {
        node.frame.height = height.max(0.0);
    }
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
        node.frame.x = x;
        node.frame.y = y;
        node.frame.width = width.max(0.0);
        node.frame.height = height.max(0.0);
    }
}

fn collapse_duplicate_compact_container_nodes(
    nodes: &mut [ViewTemplateNodeData],
    control_ids: &[&str],
) {
    for control_id in control_ids {
        let mut first_visible_container = true;
        for node in nodes
            .iter_mut()
            .filter(|node| node.control_id == *control_id)
        {
            if first_visible_container {
                first_visible_container = false;
                continue;
            }
            node.frame.width = 0.0;
            node.frame.height = 0.0;
            node.border_width = 0.0;
            node.surface_variant = "frame_only".into();
        }
    }
}
