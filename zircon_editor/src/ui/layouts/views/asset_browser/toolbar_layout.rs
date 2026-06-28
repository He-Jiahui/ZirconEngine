use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

const COMPACT_TOOLBAR_HEIGHT: f32 = 32.0;
const COMPACT_CONTROL_HEIGHT: f32 = 30.0;
const COMPACT_CONTROL_OFFSET_Y: f32 = 1.0;
const COMPACT_TOOLBAR_SIDE_PAD: f32 = 8.0;
const COMPACT_ROOT_GAP: f32 = 6.0;
const COMPACT_GROUP_GAP: f32 = 8.0;
const COMPACT_ROW_GAP: f32 = 4.0;
const COMPACT_SEARCH_MIN_WIDTH: f32 = 160.0;
const COMPACT_SEARCH_PREFERRED_RATIO: f32 = 0.38;
const COMPACT_SEARCH_PREFERRED_MIN_WIDTH: f32 = 240.0;
const COMPACT_IMPORT_PATH_MIN_WIDTH: f32 = 180.0;
const COMPACT_IMPORT_PATH_MAX_WIDTH: f32 = 260.0;
const COMPACT_IMPORT_PATH_VISIBLE_WIDTH: f32 = 860.0;
const COMPACT_VIEW_BUTTON_GAP: f32 = 4.0;
const COMPACT_VIEW_GROUP_VISIBLE_WIDTH: f32 = 560.0;

pub(super) struct AssetBrowserToolbarLayout {
    pub(super) main_y: f32,
}

pub(super) fn apply_asset_browser_toolbar_layout(
    nodes: &mut [ViewTemplateNodeData],
    viewport_width: f32,
) -> Option<AssetBrowserToolbarLayout> {
    let toolbar = node_frame(nodes, "AssetBrowserToolbarPanel")?;
    let import_panel = node_frame(nodes, "AssetBrowserImportPanel")?;
    let toolbar_width = viewport_width.max(toolbar.width).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserToolbarPanel",
        toolbar.x,
        toolbar.y,
        toolbar_width,
        COMPACT_TOOLBAR_HEIGHT,
    );
    collapse_redundant_header_nodes(nodes, toolbar.x, toolbar.y);
    layout_single_toolbar_row(nodes, toolbar.x, toolbar.y, toolbar_width);

    set_node_frame(
        nodes,
        "AssetBrowserImportPanel",
        import_panel.x,
        toolbar.y,
        toolbar_width,
        COMPACT_TOOLBAR_HEIGHT,
    );

    Some(AssetBrowserToolbarLayout {
        main_y: toolbar.y + COMPACT_TOOLBAR_HEIGHT + COMPACT_ROOT_GAP,
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

fn layout_single_toolbar_row(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32, width: f32) {
    let row_x = x + COMPACT_TOOLBAR_SIDE_PAD.min(width * 0.04);
    let row_y = y + COMPACT_CONTROL_OFFSET_Y;
    let row_width = (width - (row_x - x) * 2.0).max(0.0);
    let import = compact_import_group(nodes, row_width);
    let view = compact_view_group(nodes, row_width);
    let view_x = row_x + row_width - import.width - import.leading_gap - view.width;
    let leading_span_width = (view_x - COMPACT_GROUP_GAP - row_x).max(0.0);
    let all_chip_width = control_width(nodes, "AssetBrowserKindAllChip", 44.0);
    let preferred_search_width = (row_width * COMPACT_SEARCH_PREFERRED_RATIO)
        .max(COMPACT_SEARCH_PREFERRED_MIN_WIDTH)
        .min((leading_span_width - all_chip_width - COMPACT_GROUP_GAP).max(0.0));
    let search_width = preferred_search_width.max(COMPACT_SEARCH_MIN_WIDTH.min(leading_span_width));
    let chip_x = row_x + search_width + COMPACT_GROUP_GAP;
    let chip_width_limit = (view_x - COMPACT_GROUP_GAP - chip_x).max(0.0);
    let import_x = row_x + row_width - import.width;

    set_node_frame(
        nodes,
        "AssetBrowserToolbarSearchRow",
        x,
        y,
        width,
        COMPACT_TOOLBAR_HEIGHT,
    );
    set_node_frame(
        nodes,
        "SearchEdited",
        row_x,
        row_y,
        search_width,
        COMPACT_CONTROL_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserToolbarKindPrimaryRow",
        x,
        y,
        width,
        COMPACT_TOOLBAR_HEIGHT,
    );
    layout_kind_chips(nodes, chip_x, row_y, chip_width_limit);
    if view.visible {
        set_node_frame(
            nodes,
            "AssetBrowserViewModeListButton",
            view_x,
            row_y,
            view.list_width,
            COMPACT_CONTROL_HEIGHT,
        );
        set_node_frame(
            nodes,
            "AssetBrowserViewModeThumbButton",
            view_x + view.list_width + COMPACT_VIEW_BUTTON_GAP,
            row_y,
            view.thumb_width,
            COMPACT_CONTROL_HEIGHT,
        );
    } else {
        hide_node(nodes, "AssetBrowserViewModeListButton", view_x, row_y);
        hide_node(nodes, "AssetBrowserViewModeThumbButton", view_x, row_y);
    }

    hide_node(nodes, "LocateSelectedAsset", row_x + row_width, row_y);
    hide_node(nodes, "AssetBrowserImportLabel", import_x, row_y);
    layout_import_group(nodes, import_x, row_y, import);
}

fn layout_kind_chips(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32, width_limit: f32) {
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
        if chip_stack_width(nodes, &candidate) <= width_limit {
            visible.push(control_id);
        }
    }

    let mut cursor_x = x;
    let mut has_visible_chip = false;
    for &(control_id, fallback_width) in KIND_CHIPS {
        if !visible.contains(&control_id) {
            hide_node(nodes, control_id, x + width_limit, y);
            continue;
        }
        if has_visible_chip {
            cursor_x += COMPACT_ROW_GAP;
        }
        let chip_width = control_width(nodes, control_id, fallback_width);
        set_node_frame(
            nodes,
            control_id,
            cursor_x,
            y,
            chip_width,
            COMPACT_CONTROL_HEIGHT,
        );
        cursor_x += chip_width;
        has_visible_chip = true;
    }
}

fn chip_stack_width(nodes: &[ViewTemplateNodeData], control_ids: &[&str]) -> f32 {
    let mut width = 0.0;
    let mut visible_count = 0;
    for &(control_id, fallback_width) in KIND_CHIPS {
        if control_ids.contains(&control_id) {
            if visible_count > 0 {
                width += COMPACT_ROW_GAP;
            }
            width += control_width(nodes, control_id, fallback_width);
            visible_count += 1;
        }
    }
    width
}

fn compact_view_group(nodes: &[ViewTemplateNodeData], row_width: f32) -> CompactViewGroup {
    if row_width < COMPACT_VIEW_GROUP_VISIBLE_WIDTH {
        return CompactViewGroup::hidden();
    }
    let list_width = control_width(nodes, "AssetBrowserViewModeListButton", 64.0);
    let thumb_width = control_width(nodes, "AssetBrowserViewModeThumbButton", 78.0);
    CompactViewGroup {
        visible: true,
        list_width,
        thumb_width,
        width: list_width + COMPACT_VIEW_BUTTON_GAP + thumb_width,
    }
}

fn compact_import_group(nodes: &[ViewTemplateNodeData], row_width: f32) -> CompactImportGroup {
    let button_width = control_width(nodes, "ImportModel", 96.0)
        .min((row_width * 0.14).max(72.0))
        .min(row_width);
    let show_path = row_width >= COMPACT_IMPORT_PATH_VISIBLE_WIDTH;
    let path_width = if show_path {
        (row_width * 0.26)
            .max(COMPACT_IMPORT_PATH_MIN_WIDTH)
            .min(COMPACT_IMPORT_PATH_MAX_WIDTH)
            .min((row_width - button_width - COMPACT_GROUP_GAP).max(0.0))
    } else {
        0.0
    };
    let width = if path_width > 0.0 {
        path_width + COMPACT_GROUP_GAP + button_width
    } else {
        button_width
    };
    CompactImportGroup {
        path_visible: path_width > 0.0,
        path_width,
        button_width,
        width,
        leading_gap: COMPACT_GROUP_GAP,
    }
}

fn layout_import_group(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    group: CompactImportGroup,
) {
    if group.path_visible {
        set_node_frame(
            nodes,
            "AssetBrowserImportPathField",
            x,
            y,
            group.path_width,
            COMPACT_CONTROL_HEIGHT,
        );
        set_node_frame(
            nodes,
            "ImportModel",
            x + group.path_width + COMPACT_GROUP_GAP,
            y,
            group.button_width,
            COMPACT_CONTROL_HEIGHT,
        );
    } else {
        hide_node(nodes, "AssetBrowserImportPathField", x, y);
        set_node_frame(
            nodes,
            "ImportModel",
            x,
            y,
            group.button_width,
            COMPACT_CONTROL_HEIGHT,
        );
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
