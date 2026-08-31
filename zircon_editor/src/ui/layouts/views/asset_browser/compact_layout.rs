mod column_budget;
#[cfg(test)]
mod optimization_tests;
mod source_panel_layout;
mod utility_layout;

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::workbench::snapshot::AssetViewMode;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::layout::UiSize;

use self::column_budget::{resolve_compact_column_budget, CompactColumnBudget};
pub(super) use self::source_panel_layout::apply_asset_browser_sources_layout;
use self::source_panel_layout::apply_compact_sources_panel_layout;
use self::utility_layout::{
    apply_compact_utility_panel_layout, compact_asset_browser_utility_height_for_viewport,
    compact_asset_browser_vertical_budget, compact_line_height, shift_asset_browser_utility_nodes,
    COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD,
};
use super::compact_table_layout::{
    apply_compact_table_layout, asset_table_row_count, collapse_compact_table_nodes,
    compact_table_stack_height,
};
use super::summary_layout::apply_compact_content_preview_summary_layout;
use super::thumbnail_layout::{apply_compact_thumbnail_grid_layout, has_thumbnail_grid};

const COMPACT_LAYOUT_HEIGHT_THRESHOLD: f32 = 760.0;
const COMPACT_LAYOUT_WIDTH_THRESHOLD: f32 = 1200.0;
const COMPACT_PANEL_GAP: f32 = 6.0;
const COMPACT_CONTENT_GAP: f32 = 8.0;
const COMPACT_CONTENT_HEADER_HEIGHT: f32 = 20.0;
const COMPACT_HEADER_TABLE_GAP: f32 = 4.0;
const COMPACT_HEADER_HORIZONTAL_PADDING: f32 = 8.0;
const COMPACT_CONTENT_TITLE_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const COMPACT_CONTENT_PATH_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const COMPACT_COLLAPSED_DETAILS_MAIN_HEIGHT_THRESHOLD: f32 = 300.0;
const COMPACT_PREVIEW_CARD_HEIGHT: f32 = 50.0;
const COMPACT_DETAILS_HEADER_HEIGHT: f32 = 42.0;
const COMPACT_DETAILS_DIVIDER_HEIGHT: f32 = 1.0;
const COMPACT_DETAILS_PREVIEW_HEIGHT: f32 = 96.0;
const DETAILS_CAPTION_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const DETAILS_BODY_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const DETAILS_TEXT_TOP: f32 = 6.0;
const DETAILS_TEXT_GAP: f32 = 2.0;
const DETAILS_TEXT_BOTTOM: f32 = 6.0;
const DETAILS_VALUE_OFFSET: f32 = DETAILS_TEXT_TOP + DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_GAP;
const DETAILS_TERTIARY_OFFSET: f32 =
    DETAILS_VALUE_OFFSET + DETAILS_BODY_LINE_HEIGHT + DETAILS_TEXT_GAP;
const COMPACT_DETAILS_FIELD_HEIGHT: f32 =
    DETAILS_VALUE_OFFSET + DETAILS_BODY_LINE_HEIGHT + DETAILS_TEXT_BOTTOM;
const COMPACT_DETAILS_IDENTITY_HEIGHT: f32 =
    DETAILS_TERTIARY_OFFSET + DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_BOTTOM;
const COMPACT_DETAILS_METADATA_HEIGHT: f32 = COMPACT_DETAILS_IDENTITY_HEIGHT;
const COMPACT_DETAILS_DIAGNOSTICS_HEIGHT: f32 =
    DETAILS_VALUE_OFFSET + DETAILS_BODY_LINE_HEIGHT * 2.0 + DETAILS_TEXT_BOTTOM;
const COMPACT_MINIMUM_DRAWABLE_EXTENT: f32 = f32::EPSILON;

#[derive(Default)]
struct CompactLayoutAnchors {
    root: Option<ViewTemplateFrameData>,
    main: Option<ViewTemplateFrameData>,
    utility: Option<ViewTemplateFrameData>,
    sources: Option<ViewTemplateFrameData>,
    content: Option<ViewTemplateFrameData>,
    details: Option<ViewTemplateFrameData>,
}

#[derive(Clone, Copy)]
struct CompactPanelPresence {
    sources: bool,
    content: bool,
    details: bool,
}

pub(super) fn apply_asset_browser_compact_layout(
    nodes: &mut [ViewTemplateNodeData],
    size: UiSize,
    view_mode: AssetViewMode,
    toolbar_main_y: Option<f32>,
) {
    let compact_width = !size.width.is_finite() || size.width < COMPACT_LAYOUT_WIDTH_THRESHOLD;
    let compact_height = !size.height.is_finite() || size.height < COMPACT_LAYOUT_HEIGHT_THRESHOLD;
    if !compact_width && !compact_height {
        return;
    }

    let anchors = compact_layout_anchors(nodes);
    let panel_presence = CompactPanelPresence {
        sources: anchors.sources.is_some(),
        content: anchors.content.is_some(),
        details: anchors.details.is_some(),
    };
    let sources_width = anchors
        .sources
        .as_ref()
        .map(|frame| frame.width)
        .unwrap_or(0.0);
    let details_width = anchors
        .details
        .as_ref()
        .map(|frame| frame.width)
        .unwrap_or(0.0);
    let Some(root) = anchors.root else {
        return;
    };
    let Some(main) = anchors.main else {
        return;
    };
    let Some(utility) = anchors.utility else {
        return;
    };

    let viewport_height = finite_non_negative(size.height);
    let viewport_width = finite_non_negative(size.width);
    let root_x = finite_coordinate(root.x);
    let root_y = finite_coordinate(root.y);
    if viewport_width <= COMPACT_MINIMUM_DRAWABLE_EXTENT
        || viewport_height <= COMPACT_MINIMUM_DRAWABLE_EXTENT
    {
        collapse_asset_browser_nodes(nodes, root_x, root_y);
        return;
    }
    set_node_frame(
        nodes,
        "AssetBrowserRoot",
        root_x,
        root_y,
        viewport_width,
        viewport_height,
    );

    let main_y = finite_coordinate(toolbar_main_y.unwrap_or(main.y)).min(viewport_height);
    let (main_height, utility_y, compact_utility_height) =
        compact_asset_browser_vertical_budget(viewport_height, main_y, COMPACT_PANEL_GAP);
    let utility_delta_y = utility_y - finite_coordinate(utility.y);

    shift_asset_browser_utility_nodes(nodes, utility_delta_y);
    set_node_frame(
        nodes,
        "AssetBrowserMainPanel",
        finite_coordinate(main.x),
        main_y,
        finite_non_negative(main.width).min(viewport_width),
        main_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserUtilityPanel",
        finite_coordinate(utility.x),
        utility_y,
        finite_non_negative(utility.width).min(viewport_width),
        compact_utility_height,
    );

    let details_allowed_by_height = main_height >= COMPACT_COLLAPSED_DETAILS_MAIN_HEIGHT_THRESHOLD
        && viewport_height >= COMPACT_COLLAPSED_UTILITY_HEIGHT_THRESHOLD;
    let column_budget = resolve_compact_column_budget(
        viewport_width,
        sources_width,
        details_width,
        COMPACT_PANEL_GAP,
        details_allowed_by_height,
    );
    apply_compact_main_panel_layout(
        nodes,
        main,
        main_y,
        main_height,
        column_budget,
        panel_presence,
        view_mode,
    );
    apply_compact_utility_panel_layout(
        nodes,
        finite_coordinate(utility.x),
        utility_y,
        finite_non_negative(utility.width).min(viewport_width),
        compact_utility_height,
    );
}

fn apply_compact_main_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    main: ViewTemplateFrameData,
    main_y: f32,
    main_height: f32,
    column_budget: CompactColumnBudget,
    panel_presence: CompactPanelPresence,
    view_mode: AssetViewMode,
) {
    if main_height <= COMPACT_MINIMUM_DRAWABLE_EXTENT {
        collapse_compact_main_nodes(nodes, 0.0, main_y);
        return;
    }
    let main_x = finite_coordinate(main.x);
    let main_right = main_x + finite_non_negative(main.width);
    let sources_visible = !column_budget.collapse_sources
        && column_budget.sources_width > COMPACT_MINIMUM_DRAWABLE_EXTENT;
    let details_visible = !column_budget.collapse_details
        && column_budget.details_width > COMPACT_MINIMUM_DRAWABLE_EXTENT;
    let sources_width = if sources_visible {
        column_budget.sources_width
    } else {
        0.0
    };
    let details_width = if details_visible {
        column_budget.details_width
    } else {
        0.0
    };
    let source_x = main_x;
    let content_x = source_x
        + sources_width
        + if sources_visible {
            COMPACT_PANEL_GAP
        } else {
            0.0
        };
    let details_x = main_right - details_width;
    let content_right = details_x
        - if details_visible {
            COMPACT_PANEL_GAP
        } else {
            0.0
        };
    let content_width = finite_non_negative(content_right - content_x);

    if panel_presence.sources {
        if sources_visible {
            apply_compact_sources_panel_layout(nodes, source_x, main_y, sources_width, main_height);
        } else {
            collapse_compact_sources_nodes(nodes, source_x, main_y);
        }
    }

    if panel_presence.content {
        set_node_frame(
            nodes,
            "AssetBrowserContentPanel",
            content_x,
            main_y,
            content_width,
            main_height,
        );
        apply_compact_content_panel_layout(
            nodes,
            content_x,
            main_y,
            content_width,
            main_height,
            view_mode,
        );
    }

    if panel_presence.details {
        if details_visible {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsPanel",
                details_x,
                main_y,
                details_width,
                main_height,
            );
            apply_compact_details_panel_layout(
                nodes,
                details_x,
                main_y,
                details_width,
                main_height,
            );
        } else {
            collapse_compact_details_nodes(nodes, main_right, main_y);
        }
    }
}

fn collapse_asset_browser_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        node.frame.x = x;
        node.frame.y = y;
        node.frame.width = 0.0;
        node.frame.height = 0.0;
    }
}

fn collapse_compact_main_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        let control_id = node.control_id.as_str();
        if control_id.starts_with("AssetBrowserSources")
            || control_id.starts_with("AssetBrowserContent")
            || control_id.starts_with("AssetBrowserDetails")
            || control_id.starts_with("AssetBrowserTable")
            || control_id.starts_with("AssetBrowserThumb")
            || control_id.starts_with("WorkbenchAssetBrowser")
        {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = 0.0;
            node.frame.height = 0.0;
        }
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
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let header_height = COMPACT_CONTENT_HEADER_HEIGHT.min(height);
    let has_thumbnail_view = view_mode == AssetViewMode::Thumbnail && has_thumbnail_grid(nodes);
    let table_available_height = finite_non_negative(
        height - header_height - COMPACT_HEADER_TABLE_GAP - COMPACT_CONTENT_GAP,
    );
    let preview_height = if has_thumbnail_view {
        0.0
    } else {
        COMPACT_PREVIEW_CARD_HEIGHT.min(table_available_height)
    };
    let row_count = asset_table_row_count(nodes);
    let table_height =
        compact_table_stack_height(table_available_height - preview_height, row_count);
    let table_y = y + header_height + COMPACT_HEADER_TABLE_GAP;
    let preview_y = y + height - preview_height;

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
        let grid_height = finite_non_negative(height - header_height - COMPACT_HEADER_TABLE_GAP);
        apply_compact_thumbnail_grid_layout(nodes, x, table_y, width, grid_height);
    } else if table_height <= COMPACT_MINIMUM_DRAWABLE_EXTENT {
        collapse_compact_table_nodes(nodes, x, table_y);
    } else {
        apply_compact_table_layout(nodes, x, table_y, width, table_height, row_count);
        if preview_height > COMPACT_MINIMUM_DRAWABLE_EXTENT {
            apply_compact_content_preview_summary_layout(
                nodes,
                x,
                preview_y,
                width,
                preview_height,
            );
        } else {
            collapse_compact_content_preview_nodes(nodes, x, preview_y);
        }
    }
    collapse_duplicate_compact_container_nodes(
        nodes,
        &["AssetBrowserContentPanel", "AssetBrowserAssetTablePanel"],
    );
}

fn collapse_compact_content_preview_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for node in nodes {
        if node.control_id.starts_with("AssetBrowserContentPreview") {
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = 0.0;
            node.frame.height = 0.0;
        }
    }
}

fn apply_compact_content_header_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let inner_x = x + COMPACT_HEADER_HORIZONTAL_PADDING;
    let inner_width = finite_non_negative(width - COMPACT_HEADER_HORIZONTAL_PADDING * 2.0);
    let path_width = (inner_width * 0.32).min(220.0);
    let title_width =
        finite_non_negative(inner_width - path_width - COMPACT_HEADER_HORIZONTAL_PADDING);
    let title_height = complete_text_line_height(height, COMPACT_CONTENT_TITLE_LINE_HEIGHT);
    let path_height = complete_text_line_height(height, COMPACT_CONTENT_PATH_LINE_HEIGHT);
    let text_y = y + finite_non_negative((height - title_height) * 0.5);

    set_node_frame(
        nodes,
        "AssetBrowserContentHeaderTitleText",
        inner_x,
        text_y,
        title_width,
        title_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserContentHeaderPathText",
        x + width - COMPACT_HEADER_HORIZONTAL_PADDING - path_width,
        y + finite_non_negative((height - path_height) * 0.5),
        path_width,
        path_height,
    );
}

fn apply_compact_details_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let header_height = COMPACT_DETAILS_HEADER_HEIGHT.min(height);
    let divider_height =
        COMPACT_DETAILS_DIVIDER_HEIGHT.min(finite_non_negative(height - header_height));
    let scroll_y = y + header_height + divider_height;
    let scroll_height = finite_non_negative(height - header_height - divider_height);
    let content_x = x + 8.0;
    let content_width = finite_non_negative(width - 16.0);

    set_node_frame(
        nodes,
        "AssetBrowserDetailsHeaderPanel",
        x,
        y,
        width,
        header_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserDetailsDivider",
        x,
        y + header_height,
        width,
        divider_height,
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
    let content_end = scroll_y + scroll_height;
    let preview_height =
        COMPACT_DETAILS_PREVIEW_HEIGHT.min(finite_non_negative(content_end - field_y));
    apply_compact_details_preview_layout(nodes, content_x, field_y, content_width, preview_height);
    field_y += preview_height + COMPACT_CONTENT_GAP;
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
        let field_height = field_height.min(finite_non_negative(content_end - field_y));
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
    let text_width = finite_non_negative(width - 20.0);
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
                height,
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
                height,
            );
        }
        "AssetBrowserDetailsIdentityPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityLabel",
                label_x,
                y + DETAILS_TEXT_TOP,
                text_width,
                details_line_height(height, DETAILS_TEXT_TOP, DETAILS_CAPTION_LINE_HEIGHT),
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityUuidValue",
                value_x,
                y + DETAILS_VALUE_OFFSET,
                text_width,
                details_line_height(height, DETAILS_VALUE_OFFSET, DETAILS_BODY_LINE_HEIGHT),
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsIdentityRevisionValue",
                value_x,
                y + DETAILS_TERTIARY_OFFSET,
                text_width,
                details_line_height(height, DETAILS_TERTIARY_OFFSET, DETAILS_CAPTION_LINE_HEIGHT),
            );
        }
        "AssetBrowserDetailsMetadataPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataLabel",
                label_x,
                y + DETAILS_TEXT_TOP,
                text_width,
                details_line_height(height, DETAILS_TEXT_TOP, DETAILS_CAPTION_LINE_HEIGHT),
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataMetaPathValue",
                value_x,
                y + DETAILS_VALUE_OFFSET,
                text_width,
                details_line_height(height, DETAILS_VALUE_OFFSET, DETAILS_BODY_LINE_HEIGHT),
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsMetadataToolkitValue",
                value_x,
                y + DETAILS_TERTIARY_OFFSET,
                text_width,
                details_line_height(height, DETAILS_TERTIARY_OFFSET, DETAILS_CAPTION_LINE_HEIGHT),
            );
        }
        "AssetBrowserDetailsDiagnosticsPanel" => {
            set_node_frame(
                nodes,
                "AssetBrowserDetailsDiagnosticsLabel",
                label_x,
                y + DETAILS_TEXT_TOP,
                text_width,
                details_line_height(height, DETAILS_TEXT_TOP, DETAILS_CAPTION_LINE_HEIGHT),
            );
            set_node_frame(
                nodes,
                "AssetBrowserDetailsDiagnosticsText",
                value_x,
                y + DETAILS_VALUE_OFFSET,
                text_width,
                details_remaining_text_height(
                    height,
                    DETAILS_VALUE_OFFSET,
                    DETAILS_BODY_LINE_HEIGHT,
                ),
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
    height: f32,
) {
    set_node_frame(
        nodes,
        label_control_id,
        label_x,
        y + DETAILS_TEXT_TOP,
        text_width,
        details_line_height(height, DETAILS_TEXT_TOP, DETAILS_CAPTION_LINE_HEIGHT),
    );
    set_node_frame(
        nodes,
        value_control_id,
        value_x,
        y + DETAILS_VALUE_OFFSET,
        text_width,
        details_line_height(height, DETAILS_VALUE_OFFSET, DETAILS_BODY_LINE_HEIGHT),
    );
}

fn apply_compact_details_preview_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    preview_height: f32,
) {
    let width = finite_non_negative(width);
    let preview_height = finite_non_negative(preview_height);
    let visual_width = 48.0_f32.min(width * 0.34);
    let text_x = x + visual_width + 14.0;
    let text_width = finite_non_negative(x + width - text_x - 8.0);
    for node in nodes {
        let frame = match node.control_id.as_str() {
            "AssetBrowserDetailsPreviewPanel" => ViewTemplateFrameData {
                x: finite_coordinate(x),
                y: finite_coordinate(y),
                width,
                height: preview_height,
            },
            "AssetBrowserDetailsPreviewVisualPanel" => ViewTemplateFrameData {
                x: finite_coordinate(x + 8.0),
                y: finite_coordinate(y + 8.0),
                width: visual_width,
                height: finite_non_negative(preview_height - 16.0),
            },
            control_id => {
                let (offset_y, preferred_height) = match control_id {
                    "AssetBrowserDetailsPreviewNameText" => (10.0, DETAILS_BODY_LINE_HEIGHT),
                    "AssetBrowserDetailsPreviewLocatorText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT + DETAILS_TEXT_GAP,
                        DETAILS_CAPTION_LINE_HEIGHT,
                    ),
                    "AssetBrowserDetailsPreviewKindText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT
                            + DETAILS_TEXT_GAP
                            + DETAILS_CAPTION_LINE_HEIGHT
                            + DETAILS_TEXT_GAP,
                        DETAILS_CAPTION_LINE_HEIGHT,
                    ),
                    "AssetBrowserDetailsPreviewIdentityText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT
                            + DETAILS_TEXT_GAP
                            + (DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_GAP) * 2.0,
                        DETAILS_CAPTION_LINE_HEIGHT,
                    ),
                    "AssetBrowserDetailsPreviewToolkitText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT
                            + DETAILS_TEXT_GAP
                            + (DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_GAP) * 3.0,
                        DETAILS_CAPTION_LINE_HEIGHT,
                    ),
                    "AssetBrowserDetailsPreviewMetaPathText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT
                            + DETAILS_TEXT_GAP
                            + (DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_GAP) * 4.0,
                        DETAILS_CAPTION_LINE_HEIGHT,
                    ),
                    "AssetBrowserDetailsPreviewDiagnosticsText" => (
                        10.0 + DETAILS_BODY_LINE_HEIGHT
                            + DETAILS_TEXT_GAP
                            + (DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_GAP) * 5.0,
                        DETAILS_BODY_LINE_HEIGHT,
                    ),
                    _ => continue,
                };
                ViewTemplateFrameData {
                    x: finite_coordinate(text_x),
                    y: finite_coordinate(y + offset_y),
                    width: text_width,
                    height: details_line_height(preview_height, offset_y, preferred_height),
                }
            }
        };
        node.frame = frame;
    }
}

fn details_line_height(container_height: f32, offset: f32, line_height: f32) -> f32 {
    let available = finite_non_negative(container_height - offset);
    complete_text_line_height(available, line_height)
}

fn complete_text_line_height(container_height: f32, line_height: f32) -> f32 {
    if finite_non_negative(container_height) + f32::EPSILON >= line_height {
        line_height
    } else {
        0.0
    }
}

fn details_remaining_text_height(
    container_height: f32,
    offset: f32,
    minimum_line_height: f32,
) -> f32 {
    let available = finite_non_negative(container_height - offset - DETAILS_TEXT_BOTTOM);
    if available + f32::EPSILON >= minimum_line_height {
        available
    } else {
        0.0
    }
}

fn compact_layout_anchors(nodes: &[ViewTemplateNodeData]) -> CompactLayoutAnchors {
    let mut anchors = CompactLayoutAnchors::default();
    for node in nodes {
        let target = match node.control_id.as_str() {
            "AssetBrowserRoot" if anchors.root.is_none() => &mut anchors.root,
            "AssetBrowserMainPanel" if anchors.main.is_none() => &mut anchors.main,
            "AssetBrowserUtilityPanel" if anchors.utility.is_none() => &mut anchors.utility,
            "AssetBrowserSourcesPanel" if anchors.sources.is_none() => &mut anchors.sources,
            "AssetBrowserContentPanel" if anchors.content.is_none() => &mut anchors.content,
            "AssetBrowserDetailsPanel" if anchors.details.is_none() => &mut anchors.details,
            _ => continue,
        };
        *target = Some(node.frame.clone());
        if anchors.root.is_some()
            && anchors.main.is_some()
            && anchors.utility.is_some()
            && anchors.sources.is_some()
            && anchors.content.is_some()
            && anchors.details.is_some()
        {
            break;
        }
    }
    anchors
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
        node.frame.x = finite_coordinate(x);
        node.frame.y = finite_coordinate(y);
        node.frame.width = finite_non_negative(width);
        node.frame.height = finite_non_negative(height);
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
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_vertical_budget_never_expands_a_short_viewport() {
        let (main_height, utility_y, utility_height) =
            compact_asset_browser_vertical_budget(120.0, 96.0, 8.0);

        assert_eq!(main_height, 0.0);
        assert_eq!(utility_y, 96.0);
        assert_eq!(utility_height, 24.0);
    }

    #[test]
    fn invalid_viewport_values_collapse_to_zero_geometry() {
        assert_eq!(
            compact_asset_browser_utility_height_for_viewport(f32::NAN),
            0.0
        );
        assert_eq!(finite_non_negative(f32::INFINITY), 0.0);
        assert_eq!(finite_coordinate(f32::NEG_INFINITY), 0.0);
    }

    #[test]
    fn compact_text_line_is_clipped_to_the_remaining_parent_height() {
        assert_eq!(compact_line_height(18.0, 12.0, 10.0), 6.0);
        assert_eq!(compact_line_height(8.0, 12.0, 10.0), 0.0);
    }

    #[test]
    fn compact_details_keep_complete_typography_lines_or_hide_them() {
        assert_eq!(
            details_line_height(
                DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_TOP,
                DETAILS_TEXT_TOP,
                DETAILS_CAPTION_LINE_HEIGHT,
            ),
            DETAILS_CAPTION_LINE_HEIGHT
        );
        assert_eq!(
            details_line_height(
                DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_TOP - 0.5,
                DETAILS_TEXT_TOP,
                DETAILS_CAPTION_LINE_HEIGHT,
            ),
            0.0
        );
    }

    #[test]
    fn compact_content_header_keeps_complete_typography_lines_or_hides_them() {
        assert_eq!(
            complete_text_line_height(
                COMPACT_CONTENT_TITLE_LINE_HEIGHT,
                COMPACT_CONTENT_TITLE_LINE_HEIGHT,
            ),
            COMPACT_CONTENT_TITLE_LINE_HEIGHT
        );
        assert_eq!(
            complete_text_line_height(
                COMPACT_CONTENT_PATH_LINE_HEIGHT - 0.5,
                COMPACT_CONTENT_PATH_LINE_HEIGHT,
            ),
            0.0
        );
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
