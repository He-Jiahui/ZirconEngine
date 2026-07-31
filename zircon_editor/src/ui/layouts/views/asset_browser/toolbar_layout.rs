use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

const COMPACT_SEARCH_MIN_WIDTH: f32 = 160.0;
const COMPACT_SEARCH_PREFERRED_RATIO: f32 = 0.38;
const COMPACT_SEARCH_PREFERRED_MIN_WIDTH: f32 = 240.0;
const COMPACT_IMPORT_PATH_MIN_WIDTH: f32 = 180.0;
const COMPACT_IMPORT_PATH_MAX_WIDTH: f32 = 260.0;
const COMPACT_IMPORT_PATH_VISIBLE_WIDTH: f32 = 1040.0;
const COMPACT_VIEW_GROUP_VISIBLE_WIDTH: f32 = 560.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct AssetBrowserToolbarMetrics {
    toolbar_height: f32,
    control_height: f32,
    control_offset_y: f32,
    side_pad: f32,
    root_gap: f32,
    group_gap: f32,
    group_frame_pad: f32,
    row_gap: f32,
    view_button_gap: f32,
}

pub(super) struct AssetBrowserToolbarLayout {
    pub(super) main_y: f32,
}

pub(super) fn apply_asset_browser_toolbar_layout(
    nodes: &mut [ViewTemplateNodeData],
    viewport_width: f32,
) -> Option<AssetBrowserToolbarLayout> {
    let metrics = asset_browser_toolbar_metrics();
    let toolbar = node_frame(nodes, "AssetBrowserToolbarPanel")?;
    let import_panel = node_frame(nodes, "AssetBrowserImportPanel")?;
    let toolbar_width = viewport_width.max(toolbar.width).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserToolbarPanel",
        toolbar.x,
        toolbar.y,
        toolbar_width,
        metrics.toolbar_height,
    );
    collapse_redundant_header_nodes(nodes, toolbar.x, toolbar.y);
    layout_single_toolbar_row(nodes, toolbar.x, toolbar.y, toolbar_width, metrics);

    set_node_frame(
        nodes,
        "AssetBrowserImportPanel",
        import_panel.x,
        toolbar.y,
        toolbar_width,
        metrics.toolbar_height,
    );

    Some(AssetBrowserToolbarLayout {
        main_y: toolbar.y + metrics.toolbar_height + metrics.root_gap,
    })
}

fn collapse_redundant_header_nodes(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32) {
    for control_id in [
        "AssetBrowserToolbarTitleRow",
        "AssetBrowserTitleText",
        "AssetBrowserToolbarSubtitleRow",
        "AssetBrowserSubtitleText",
        "AssetBrowserToolbarKindSecondaryRow",
        "AssetBrowserKindPhysicsChip",
        "AssetBrowserKindSkeletonChip",
        "AssetBrowserKindClipChip",
        "AssetBrowserKindSequenceChip",
        "AssetBrowserKindGraphChip",
        "AssetBrowserKindStateChip",
    ] {
        hide_node(nodes, control_id, x, y);
    }
}

fn layout_single_toolbar_row(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    metrics: AssetBrowserToolbarMetrics,
) {
    let row_x = x + metrics.side_pad.min(width * 0.04);
    let row_y = y + metrics.control_offset_y;
    let row_width = (width - (row_x - x) * 2.0).max(0.0);
    let import = compact_import_group(nodes, row_width, metrics);
    let view = compact_view_group(nodes, row_width, metrics);
    let view_x = row_x + row_width - import.width - import.leading_gap - view.width;
    let leading_span_width = (view_x - metrics.group_gap - row_x).max(0.0);
    let all_chip_width = control_width(nodes, "AssetBrowserKindAllChip", 44.0);
    let preferred_search_width = (row_width * COMPACT_SEARCH_PREFERRED_RATIO)
        .max(COMPACT_SEARCH_PREFERRED_MIN_WIDTH)
        .min((leading_span_width - all_chip_width - metrics.group_gap).max(0.0));
    let search_width = preferred_search_width.max(COMPACT_SEARCH_MIN_WIDTH.min(leading_span_width));
    let chip_x = row_x + search_width + metrics.group_gap;
    let chip_width_limit = (view_x - metrics.group_gap - chip_x).max(0.0);
    let import_x = row_x + row_width - import.width;

    set_node_frame(
        nodes,
        "AssetBrowserToolbarSearchRow",
        x,
        y,
        width,
        metrics.toolbar_height,
    );
    set_node_frame(
        nodes,
        "SearchEdited",
        row_x,
        row_y,
        search_width,
        metrics.control_height,
    );
    let chips_width = layout_kind_chips(nodes, chip_x, row_y, chip_width_limit, metrics);
    if view.visible {
        set_node_frame(
            nodes,
            "AssetBrowserViewModeListButton",
            view_x,
            row_y,
            view.list_width,
            metrics.control_height,
        );
        set_node_frame(
            nodes,
            "AssetBrowserViewModeThumbButton",
            view_x + view.list_width + metrics.view_button_gap,
            row_y,
            view.thumb_width,
            metrics.control_height,
        );
    } else {
        hide_node(nodes, "AssetBrowserViewModeListButton", view_x, row_y);
        hide_node(nodes, "AssetBrowserViewModeThumbButton", view_x, row_y);
    }
    layout_filter_group_frame(nodes, chip_x, row_y, chips_width, view_x, view, metrics);

    hide_node(nodes, "LocateSelectedAsset", row_x + row_width, row_y);
    hide_node(nodes, "AssetBrowserImportLabel", import_x, row_y);
    layout_import_group(nodes, import_x, row_y, import, metrics);
}

fn layout_kind_chips(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> f32 {
    let selected_chip = KIND_CHIPS
        .iter()
        .find(|(control_id, _)| is_selected(nodes, control_id))
        .map(|(control_id, _)| *control_id);
    let mut visible = Vec::new();
    for &(control_id, _) in KIND_CHIPS {
        if control_id == "AssetBrowserKindAllChip" || Some(control_id) == selected_chip {
            visible.push(control_id);
        }
    }
    for &(control_id, _) in KIND_CHIPS {
        if visible.contains(&control_id) {
            continue;
        }
        let mut candidate = visible.clone();
        candidate.push(control_id);
        if chip_stack_width(nodes, &candidate, metrics) <= width_limit {
            visible.push(control_id);
        }
    }

    let mut cursor_x = x;
    let mut has_visible_chip = false;
    let mut used_width = 0.0;
    for &(control_id, fallback_width) in KIND_CHIPS {
        if !visible.contains(&control_id) {
            hide_node(nodes, control_id, x + width_limit, y);
            continue;
        }
        let chip_width = control_width(nodes, control_id, fallback_width);
        if !chip_fits_in_width(
            used_width,
            has_visible_chip,
            chip_width,
            width_limit,
            metrics,
        ) {
            hide_node(nodes, control_id, x + width_limit, y);
            continue;
        }
        if has_visible_chip {
            cursor_x += metrics.row_gap;
            used_width += metrics.row_gap;
        }
        set_node_frame(
            nodes,
            control_id,
            cursor_x,
            y,
            chip_width,
            metrics.control_height,
        );
        cursor_x += chip_width;
        used_width += chip_width;
        has_visible_chip = true;
    }
    used_width
}

fn chip_fits_in_width(
    used_width: f32,
    has_visible_chip: bool,
    chip_width: f32,
    width_limit: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> bool {
    let leading_gap = if has_visible_chip {
        metrics.row_gap
    } else {
        0.0
    };
    used_width + leading_gap + chip_width <= width_limit.max(0.0)
}

fn chip_stack_width(
    nodes: &[ViewTemplateNodeData],
    control_ids: &[&str],
    metrics: AssetBrowserToolbarMetrics,
) -> f32 {
    let mut width = 0.0;
    let mut visible_count = 0;
    for &(control_id, fallback_width) in KIND_CHIPS {
        if control_ids.contains(&control_id) {
            if visible_count > 0 {
                width += metrics.row_gap;
            }
            width += control_width(nodes, control_id, fallback_width);
            visible_count += 1;
        }
    }
    width
}

fn compact_view_group(
    nodes: &[ViewTemplateNodeData],
    row_width: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> CompactViewGroup {
    if row_width < COMPACT_VIEW_GROUP_VISIBLE_WIDTH {
        return CompactViewGroup::hidden();
    }
    let list_width = control_width(nodes, "AssetBrowserViewModeListButton", 64.0);
    let thumb_width = control_width(nodes, "AssetBrowserViewModeThumbButton", 78.0);
    CompactViewGroup {
        visible: true,
        list_width,
        thumb_width,
        width: list_width + metrics.view_button_gap + thumb_width,
    }
}

fn compact_import_group(
    nodes: &[ViewTemplateNodeData],
    row_width: f32,
    metrics: AssetBrowserToolbarMetrics,
) -> CompactImportGroup {
    let button_width = control_width(nodes, "ImportModel", 96.0)
        .min((row_width * 0.14).max(72.0))
        .min(row_width);
    let show_path = row_width >= COMPACT_IMPORT_PATH_VISIBLE_WIDTH;
    let path_width = if show_path {
        (row_width * 0.26)
            .max(COMPACT_IMPORT_PATH_MIN_WIDTH)
            .min(COMPACT_IMPORT_PATH_MAX_WIDTH)
            .min((row_width - button_width - metrics.group_gap).max(0.0))
    } else {
        0.0
    };
    let width = if path_width > 0.0 {
        path_width + metrics.group_gap + button_width
    } else {
        button_width
    };
    CompactImportGroup {
        path_visible: path_width > 0.0,
        path_width,
        button_width,
        width,
        leading_gap: metrics.group_gap,
    }
}

fn layout_import_group(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    group: CompactImportGroup,
    metrics: AssetBrowserToolbarMetrics,
) {
    if group.path_visible {
        set_node_frame(
            nodes,
            "AssetBrowserImportPathField",
            x,
            y,
            group.path_width,
            metrics.control_height,
        );
        set_node_frame(
            nodes,
            "ImportModel",
            x + group.path_width + metrics.group_gap,
            y,
            group.button_width,
            metrics.control_height,
        );
    } else {
        hide_node(nodes, "AssetBrowserImportPathField", x, y);
        set_node_frame(
            nodes,
            "ImportModel",
            x,
            y,
            group.button_width,
            metrics.control_height,
        );
    }
}

fn layout_filter_group_frame(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    chips_width: f32,
    view_x: f32,
    view: CompactViewGroup,
    metrics: AssetBrowserToolbarMetrics,
) {
    let right_edge = if view.visible {
        view_x + view.width
    } else {
        x + chips_width
    };
    let group_x = x - metrics.group_frame_pad;
    let group_y = y - metrics.control_offset_y;
    let group_width = (right_edge - x + metrics.group_frame_pad * 2.0).max(0.0);
    set_node_frame(
        nodes,
        "AssetBrowserToolbarKindPrimaryRow",
        group_x,
        group_y,
        group_width,
        metrics.toolbar_height,
    );
}

fn asset_browser_toolbar_metrics() -> AssetBrowserToolbarMetrics {
    asset_browser_toolbar_metrics_from_tokens(
        EditorDensityTokens::workbench_dense(),
        EditorControlTokens::workbench_dense(),
    )
}

fn asset_browser_toolbar_metrics_from_tokens(
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) -> AssetBrowserToolbarMetrics {
    let toolbar_height = density.row_height + density.gap_medium;
    let control_offset_y = controls.border_width;
    AssetBrowserToolbarMetrics {
        toolbar_height,
        control_height: (toolbar_height - controls.border_width * 2.0).max(controls.border_width),
        control_offset_y,
        side_pad: density.gap_medium,
        root_gap: (density.gap_medium - controls.border_width * 2.0).max(0.0),
        group_gap: density.gap_medium,
        group_frame_pad: (density.gap_small - controls.border_width).max(0.0),
        row_gap: density.gap_small,
        view_button_gap: density.gap_small,
    }
}

struct CompactViewGroup {
    visible: bool,
    list_width: f32,
    thumb_width: f32,
    width: f32,
}

impl CompactViewGroup {
    fn hidden() -> Self {
        Self {
            visible: false,
            list_width: 0.0,
            thumb_width: 0.0,
            width: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct CompactImportGroup {
    path_visible: bool,
    path_width: f32,
    button_width: f32,
    width: f32,
    leading_gap: f32,
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}

fn control_width(nodes: &[ViewTemplateNodeData], control_id: &str, fallback: f32) -> f32 {
    node_frame(nodes, control_id)
        .map(|frame| frame.width)
        .filter(|width| *width > 0.0)
        .unwrap_or(fallback)
}

fn is_selected(nodes: &[ViewTemplateNodeData], control_id: &str) -> bool {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.selected)
        .unwrap_or(false)
}

fn hide_node(nodes: &mut [ViewTemplateNodeData], control_id: &str, x: f32, y: f32) {
    set_node_frame(nodes, control_id, x, y, 0.0, 0.0);
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

const KIND_CHIPS: &[(&str, f32)] = &[
    ("AssetBrowserKindAllChip", 44.0),
    ("AssetBrowserKindTextureChip", 78.0),
    ("AssetBrowserKindMaterialChip", 84.0),
    ("AssetBrowserKindSceneChip", 64.0),
    ("AssetBrowserKindModelChip", 64.0),
    ("AssetBrowserKindShaderChip", 72.0),
];

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};

    #[test]
    fn asset_browser_toolbar_metrics_project_from_dense_design_tokens() {
        let mut density = EditorDensityTokens::workbench_dense();
        density.row_height = 28.0;
        density.gap_small = 5.0;
        density.gap_medium = 10.0;
        let mut controls = EditorControlTokens::workbench_dense();
        controls.border_width = 2.0;

        let metrics = asset_browser_toolbar_metrics_from_tokens(density, controls);

        assert_eq!(metrics.toolbar_height, 38.0);
        assert_eq!(metrics.control_height, 34.0);
        assert_eq!(metrics.control_offset_y, 2.0);
        assert_eq!(metrics.side_pad, 10.0);
        assert_eq!(metrics.root_gap, 6.0);
        assert_eq!(metrics.group_gap, 10.0);
        assert_eq!(metrics.group_frame_pad, 3.0);
        assert_eq!(metrics.row_gap, 5.0);
        assert_eq!(metrics.view_button_gap, 5.0);

        assert!(!chip_fits_in_width(0.0, false, 44.0, 0.0, metrics));
        assert!(chip_fits_in_width(0.0, false, 44.0, 44.0, metrics));
        assert!(!chip_fits_in_width(44.0, true, 78.0, 126.0, metrics));
        assert!(chip_fits_in_width(44.0, true, 78.0, 127.0, metrics));
    }
}
