use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};

const COMPACT_TOOLBAR_HEIGHT: f32 = 56.0;
const COMPACT_SEARCH_ROW_HEIGHT: f32 = 28.0;
const COMPACT_KIND_ROW_HEIGHT: f32 = 24.0;
const COMPACT_ROW_GAP: f32 = 4.0;
const COMPACT_ROOT_GAP: f32 = 6.0;
const COMPACT_IMPORT_HEIGHT: f32 = 32.0;
const COMPACT_IMPORT_FIELD_HEIGHT: f32 = 28.0;
const COMPACT_IMPORT_BUTTON_HEIGHT: f32 = 26.0;
const COMPACT_IMPORT_SIDE_PAD: f32 = 8.0;
const COMPACT_IMPORT_LABEL_WIDTH: f32 = 86.0;
const COMPACT_SEARCH_LOCATE_GAP: f32 = 6.0;
const COMPACT_SEARCH_MIN_WIDTH: f32 = 120.0;
const COMPACT_LOCATE_MIN_WIDTH: f32 = 120.0;
const COMPACT_VIEW_BUTTON_GAP: f32 = 4.0;

pub(super) struct CompactToolbarLayout {
    pub(super) main_y: f32,
}

pub(super) fn apply_compact_toolbar_layout(
    nodes: &mut [ViewTemplateNodeData],
    viewport_width: f32,
) -> Option<CompactToolbarLayout> {
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
    layout_search_row(nodes, toolbar.x, toolbar.y, toolbar_width);
    layout_kind_row(
        nodes,
        toolbar.x,
        toolbar.y + COMPACT_SEARCH_ROW_HEIGHT + COMPACT_ROW_GAP,
        toolbar_width,
    );

    let import_y = toolbar.y + COMPACT_TOOLBAR_HEIGHT + COMPACT_ROOT_GAP;
    layout_import_row(nodes, import_panel.x, import_y, toolbar_width);

    Some(CompactToolbarLayout {
        main_y: import_y + COMPACT_IMPORT_HEIGHT + COMPACT_ROOT_GAP,
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

fn layout_search_row(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32, width: f32) {
    let locate_authored_width = control_width(nodes, "LocateSelectedAsset", 156.0);
    let locate_available = width - COMPACT_SEARCH_MIN_WIDTH - COMPACT_SEARCH_LOCATE_GAP;
    let show_locate = locate_available >= COMPACT_LOCATE_MIN_WIDTH;
    let locate_width = if show_locate {
        locate_authored_width
            .min((width * 0.34).max(COMPACT_LOCATE_MIN_WIDTH))
            .min(locate_available)
    } else {
        0.0
    };
    let search_width = if show_locate {
        (width - locate_width - COMPACT_SEARCH_LOCATE_GAP).max(COMPACT_SEARCH_MIN_WIDTH)
    } else {
        width
    };

    set_node_frame(
        nodes,
        "AssetBrowserToolbarSearchRow",
        x,
        y,
        width,
        COMPACT_SEARCH_ROW_HEIGHT,
    );
    set_node_frame(
        nodes,
        "SearchEdited",
        x,
        y,
        search_width,
        COMPACT_SEARCH_ROW_HEIGHT,
    );
    if show_locate {
        set_node_frame(
            nodes,
            "LocateSelectedAsset",
            x + width - locate_width,
            y,
            locate_width,
            COMPACT_SEARCH_ROW_HEIGHT,
        );
    } else {
        hide_node(nodes, "LocateSelectedAsset", x + width, y);
    }
}

fn layout_kind_row(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32, width: f32) {
    let list_width = control_width(nodes, "AssetBrowserViewModeListButton", 64.0);
    let thumb_width = control_width(nodes, "AssetBrowserViewModeThumbButton", 78.0);
    let view_group_width = list_width + COMPACT_VIEW_BUTTON_GAP + thumb_width;
    let view_group_x = (x + width - view_group_width).max(x);
    let chip_width_limit = (view_group_x - COMPACT_ROW_GAP - x).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserToolbarKindPrimaryRow",
        x,
        y,
        width,
        COMPACT_KIND_ROW_HEIGHT,
    );
    layout_kind_chips(nodes, x, y, chip_width_limit);
    if view_group_width <= width {
        set_node_frame(
            nodes,
            "AssetBrowserViewModeListButton",
            view_group_x,
            y,
            list_width,
            COMPACT_KIND_ROW_HEIGHT,
        );
        set_node_frame(
            nodes,
            "AssetBrowserViewModeThumbButton",
            view_group_x + list_width + COMPACT_VIEW_BUTTON_GAP,
            y,
            thumb_width,
            COMPACT_KIND_ROW_HEIGHT,
        );
    } else {
        hide_node(nodes, "AssetBrowserViewModeListButton", x + width, y);
        hide_node(nodes, "AssetBrowserViewModeThumbButton", x + width, y);
    }
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
            COMPACT_KIND_ROW_HEIGHT,
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

fn layout_import_row(nodes: &mut [ViewTemplateNodeData], x: f32, y: f32, width: f32) {
    let button_width = control_width(nodes, "ImportModel", 96.0)
        .min((width * 0.24).max(72.0))
        .min((width - COMPACT_IMPORT_SIDE_PAD * 2.0).max(0.0));
    let label_width = COMPACT_IMPORT_LABEL_WIDTH.min((width * 0.22).max(0.0));
    let button_x = x + width - COMPACT_IMPORT_SIDE_PAD - button_width;
    let label_x = x + COMPACT_IMPORT_SIDE_PAD;
    let field_x = label_x + label_width + COMPACT_IMPORT_SIDE_PAD;
    let field_right = button_x - COMPACT_IMPORT_SIDE_PAD;
    let field_width = (field_right - field_x).max(0.0);

    set_node_frame(
        nodes,
        "AssetBrowserImportPanel",
        x,
        y,
        width,
        COMPACT_IMPORT_HEIGHT,
    );
    set_node_frame(
        nodes,
        "AssetBrowserImportLabel",
        label_x,
        y + 9.0,
        label_width,
        14.0,
    );
    set_node_frame(
        nodes,
        "AssetBrowserImportPathField",
        field_x,
        y + (COMPACT_IMPORT_HEIGHT - COMPACT_IMPORT_FIELD_HEIGHT) * 0.5,
        field_width,
        COMPACT_IMPORT_FIELD_HEIGHT,
    );
    set_node_frame(
        nodes,
        "ImportModel",
        button_x,
        y + (COMPACT_IMPORT_HEIGHT - COMPACT_IMPORT_BUTTON_HEIGHT) * 0.5,
        button_width,
        COMPACT_IMPORT_BUTTON_HEIGHT,
    );
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
