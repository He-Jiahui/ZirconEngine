use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile,
};
use crate::ui::workbench::snapshot::{AssetUtilityTab, AssetWorkspaceSnapshot};

const COMPACT_TOOLBAR_ROW_COUNT: f32 = 2.0;
const PREFERRED_UTILITY_ROW_COUNT: f32 = 4.0;
const MINIMUM_UTILITY_ROW_COUNT: f32 = 2.0;

pub(super) fn apply_assets_activity_responsive_layout(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) {
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let Some(root) = node_frame(nodes, "AssetsActivityRoot") else {
        return;
    };
    if root.width > density.compact_left_drawer_max_width {
        apply_wide_utility_tab_visibility(nodes, snapshot, &root);
        return;
    }

    let width = root.width.max(0.0).min(size.width.max(0.0));
    let height = root.height.max(0.0).min(size.height.max(0.0));
    let gap = density.gap_small;
    let row_height = density.row_height;
    let toolbar_height = row_height * COMPACT_TOOLBAR_ROW_COUNT + gap;
    let content_metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        snapshot.view_mode,
    );
    let minimum_main_height = content_metrics.list_height(0, 1);
    let maximum_utility_height =
        (height - toolbar_height - gap * 2.0 - minimum_main_height).max(0.0);
    let preferred_utility_height = row_height * PREFERRED_UTILITY_ROW_COUNT;
    let minimum_utility_height = row_height * MINIMUM_UTILITY_ROW_COUNT + gap;
    let utility_height = preferred_utility_height
        .min(maximum_utility_height)
        .max(minimum_utility_height.min(maximum_utility_height));
    let main_y = root.y + toolbar_height + gap;
    let main_height = (height - toolbar_height - utility_height - gap * 2.0).max(0.0);
    let utility_y = main_y + main_height + gap;

    layout_toolbar(
        nodes,
        snapshot,
        root.x,
        root.y,
        width,
        toolbar_height,
        row_height,
        density,
        controls,
    );
    layout_main_content(nodes, root.x, main_y, width, main_height);
    layout_utility(
        nodes,
        snapshot,
        root.x,
        utility_y,
        width,
        utility_height,
        row_height,
        density,
        controls,
    );
}

fn apply_wide_utility_tab_visibility(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    root: &ViewTemplateFrameData,
) {
    match snapshot.utility_tab {
        AssetUtilityTab::Preview => hide_nodes(nodes, REFERENCE_CONTROLS, root.x, root.y),
        AssetUtilityTab::References => hide_nodes(nodes, PREVIEW_CONTROLS, root.x, root.y),
        AssetUtilityTab::Metadata | AssetUtilityTab::Plugins => {
            hide_nodes(nodes, PREVIEW_CONTROLS, root.x, root.y);
            hide_nodes(nodes, REFERENCE_CONTROLS, root.x, root.y);
        }
    }
}

fn layout_toolbar(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    row_height: f32,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) {
    hide_nodes(
        nodes,
        &[
            "AssetsActivityToolbarTitleRow",
            "AssetsActivityTitleText",
            "AssetsActivityToolbarSubtitleRow",
            "AssetsActivitySubtitleText",
            "AssetsActivityToolbarKindSecondaryRow",
            "AssetsActivityKindPhysicsChip",
            "AssetsActivityKindSkeletonChip",
            "AssetsActivityKindClipChip",
            "AssetsActivityKindSequenceChip",
            "AssetsActivityKindGraphChip",
            "AssetsActivityKindStateChip",
        ],
        x,
        y,
    );
    set_node_frame(nodes, "AssetsActivityToolbarPanel", x, y, width, height);

    let padding = density.gap_medium.min(width * 0.1);
    let inner_width = (width - padding * 2.0).max(0.0);
    let browser_width =
        measured_button_width(nodes, "OpenAssetBrowser", density, controls).min(inner_width);
    let search_width = (inner_width - browser_width - density.gap_small).max(0.0);
    let browser_x = x + padding + search_width + density.gap_small;
    set_node_frame(
        nodes,
        "AssetsActivityToolbarSearchRow",
        x,
        y,
        width,
        row_height,
    );
    set_node_frame(
        nodes,
        "SearchEdited",
        x + padding,
        y,
        search_width,
        row_height,
    );
    set_node_frame(
        nodes,
        "OpenAssetBrowser",
        browser_x,
        y,
        browser_width,
        row_height,
    );

    let second_y = y + row_height + density.gap_small;
    set_node_frame(
        nodes,
        "AssetsActivityToolbarKindPrimaryRow",
        x,
        second_y,
        width,
        row_height,
    );
    layout_compact_toolbar_controls(
        nodes,
        snapshot,
        x + padding,
        second_y,
        inner_width,
        row_height,
        density,
        controls,
    );
}

fn layout_compact_toolbar_controls(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) {
    let selected_kind = selected_kind_control(snapshot.kind_filter);
    let mut visible = vec![
        "AssetsActivityViewModeListButton",
        "AssetsActivityViewModeThumbButton",
        "AssetsActivityKindAllChip",
    ];
    if let Some(control_id) = selected_kind {
        visible.push(control_id);
    }

    hide_nodes(nodes, PRIMARY_KIND_CONTROLS, x, y);
    let mut cursor_x = x;
    for control_id in visible {
        let control_width = measured_button_width(nodes, control_id, density, controls);
        let leading_gap = if cursor_x > x { density.gap_small } else { 0.0 };
        if cursor_x + leading_gap + control_width > x + width + f32::EPSILON {
            hide_nodes(nodes, &[control_id], x + width, y);
            continue;
        }
        cursor_x += leading_gap;
        set_node_frame(nodes, control_id, cursor_x, y, control_width, height);
        cursor_x += control_width;
    }
}

fn layout_main_content(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    hide_nodes(nodes, TREE_CONTROLS, x, y);
    set_node_frame(nodes, "AssetsActivityMainPanel", x, y, width, height);
    set_node_frame(nodes, "AssetsActivityContentPanel", x, y, width, height);
}

fn layout_utility(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    row_height: f32,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) {
    set_node_frame(nodes, "AssetsActivityUtilityPanel", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetsActivityUtilityTabsRow",
        x,
        y,
        width,
        row_height,
    );
    hide_nodes(nodes, &["AssetsActivitySelectionText"], x + width, y);

    let padding = density.gap_medium.min(width * 0.1);
    let preview_width =
        measured_button_width(nodes, "AssetsActivityPreviewTabButton", density, controls);
    let references_width = measured_button_width(
        nodes,
        "AssetsActivityReferencesTabButton",
        density,
        controls,
    );
    set_node_frame(
        nodes,
        "AssetsActivityPreviewTabButton",
        x + padding,
        y,
        preview_width,
        row_height,
    );
    set_node_frame(
        nodes,
        "AssetsActivityReferencesTabButton",
        x + padding + preview_width + density.gap_small,
        y,
        references_width,
        row_height,
    );
    let content_y = y + row_height + density.gap_small;
    let content_height = (height - row_height - density.gap_small).max(0.0);
    set_node_frame(
        nodes,
        "AssetsActivityUtilityDivider",
        x,
        y + row_height,
        width,
        1.0,
    );
    set_node_frame(
        nodes,
        "AssetsActivityUtilityContentPanel",
        x,
        content_y,
        width,
        content_height,
    );

    match snapshot.utility_tab {
        AssetUtilityTab::Preview => {
            hide_nodes(nodes, REFERENCE_CONTROLS, x, content_y);
            layout_preview(nodes, x, content_y, width, content_height, density);
        }
        AssetUtilityTab::References => {
            hide_nodes(nodes, PREVIEW_CONTROLS, x, content_y);
            layout_references(nodes, x, content_y, width, content_height, density);
        }
        AssetUtilityTab::Metadata | AssetUtilityTab::Plugins => {
            hide_nodes(nodes, PREVIEW_CONTROLS, x, content_y);
            hide_nodes(nodes, REFERENCE_CONTROLS, x, content_y);
        }
    }
}

fn layout_preview(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    density: EditorDensityTokens,
) {
    set_node_frame(nodes, "AssetsActivityPreviewPanel", x, y, width, height);
    let visual_extent = 56.0_f32
        .min((height - density.gap_medium * 2.0).max(0.0))
        .min(width * 0.3);
    set_node_frame(
        nodes,
        "AssetsActivityPreviewVisualPanel",
        x + density.gap_medium,
        y + density.gap_medium,
        visual_extent,
        visual_extent,
    );
    let text_x = x + density.gap_medium * 2.0 + visual_extent;
    let text_width = (x + width - density.gap_medium - text_x).max(0.0);
    for (control_id, offset_y) in [
        ("AssetsActivityPreviewNameText", density.gap_small),
        ("AssetsActivityPreviewLocatorText", density.gap_small + 18.0),
        ("AssetsActivityPreviewKindText", density.gap_small + 34.0),
    ] {
        set_node_frame(nodes, control_id, text_x, y + offset_y, text_width, 14.0);
    }
    hide_nodes(
        nodes,
        &[
            "AssetsActivityPreviewIdentityText",
            "AssetsActivityPreviewToolkitText",
            "AssetsActivityPreviewMetaPathText",
            "AssetsActivityPreviewDiagnosticsText",
        ],
        text_x,
        y + height,
    );
}

fn layout_references(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    density: EditorDensityTokens,
) {
    hide_nodes(nodes, REFERENCE_RIGHT_CONTROLS, x + width, y);
    hide_nodes(
        nodes,
        &[
            "AssetsActivityReferenceLeftRowPanel",
            "AssetsActivityReferenceLeftRowNameText",
            "AssetsActivityReferenceLeftRowLocatorText",
            "AssetsActivityReferenceLeftRowKindText",
        ],
        x,
        y + height,
    );
    set_node_frame(
        nodes,
        "AssetsActivityReferenceLeftPanel",
        x,
        y,
        width,
        height,
    );
    set_node_frame(
        nodes,
        "AssetsActivityReferenceLeftTitleText",
        x + density.gap_medium,
        y,
        (width - density.gap_medium * 2.0).max(0.0),
        14.0,
    );
    set_node_frame(
        nodes,
        "AssetsActivityReferenceLeftScrollBody",
        x + density.gap_medium,
        y + 18.0,
        (width - density.gap_medium * 2.0).max(0.0),
        (height - 18.0).max(0.0),
    );
    set_node_frame(
        nodes,
        "AssetsActivityReferenceLeftEmptyText",
        x + density.gap_medium,
        y + 22.0,
        (width - density.gap_medium * 2.0).max(0.0),
        14.0,
    );
}

fn selected_kind_control(kind: Option<ResourceKind>) -> Option<&'static str> {
    match kind {
        Some(ResourceKind::Texture) => Some("AssetsActivityKindTextureChip"),
        Some(ResourceKind::Material) => Some("AssetsActivityKindMaterialChip"),
        Some(ResourceKind::Scene) => Some("AssetsActivityKindSceneChip"),
        Some(ResourceKind::Model | ResourceKind::Mesh) => Some("AssetsActivityKindModelChip"),
        Some(ResourceKind::Shader) => Some("AssetsActivityKindShaderChip"),
        _ => None,
    }
}

fn measured_button_width(
    nodes: &[ViewTemplateNodeData],
    control_id: &str,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) -> f32 {
    let text = nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.text.as_str())
        .unwrap_or("");
    (measure_runtime_text_width(text, EditorTypographyTokens::WORKBENCH_BODY_SIZE)
        + density.gap_medium * 2.0)
        .max(controls.default_height)
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
}

fn hide_nodes(nodes: &mut [ViewTemplateNodeData], control_ids: &[&str], x: f32, y: f32) {
    for control_id in control_ids {
        set_node_frame(nodes, control_id, x, y, 0.0, 0.0);
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

const PRIMARY_KIND_CONTROLS: &[&str] = &[
    "AssetsActivityKindAllChip",
    "AssetsActivityKindTextureChip",
    "AssetsActivityKindMaterialChip",
    "AssetsActivityKindSceneChip",
    "AssetsActivityKindModelChip",
    "AssetsActivityKindShaderChip",
];

const TREE_CONTROLS: &[&str] = &[
    "AssetsActivityTreePanel",
    "AssetsActivityTreeHeaderPanel",
    "AssetsActivityTreeTitleText",
    "AssetsActivityTreeSubtitleText",
    "AssetsActivityTreeDivider",
    "AssetsActivityTreeScrollBody",
    "AssetsActivityTreeRowPanel",
    "AssetsActivityTreeRowIcon",
    "AssetsActivityTreeRowNameText",
    "AssetsActivityTreeRowCountText",
];

const PREVIEW_CONTROLS: &[&str] = &[
    "AssetsActivityPreviewPanel",
    "AssetsActivityPreviewVisualPanel",
    "AssetsActivityPreviewNameText",
    "AssetsActivityPreviewLocatorText",
    "AssetsActivityPreviewKindText",
    "AssetsActivityPreviewIdentityText",
    "AssetsActivityPreviewToolkitText",
    "AssetsActivityPreviewMetaPathText",
    "AssetsActivityPreviewDiagnosticsText",
];

const REFERENCE_RIGHT_CONTROLS: &[&str] = &[
    "AssetsActivityReferenceRightPanel",
    "AssetsActivityReferenceRightTitleText",
    "AssetsActivityReferenceRightScrollBody",
    "AssetsActivityReferenceRightEmptyText",
    "AssetsActivityReferenceRightRowPanel",
    "AssetsActivityReferenceRightRowNameText",
    "AssetsActivityReferenceRightRowLocatorText",
    "AssetsActivityReferenceRightRowKindText",
];

const REFERENCE_CONTROLS: &[&str] = &[
    "AssetsActivityReferenceLeftPanel",
    "AssetsActivityReferenceLeftTitleText",
    "AssetsActivityReferenceLeftScrollBody",
    "AssetsActivityReferenceLeftEmptyText",
    "AssetsActivityReferenceLeftRowPanel",
    "AssetsActivityReferenceLeftRowNameText",
    "AssetsActivityReferenceLeftRowLocatorText",
    "AssetsActivityReferenceLeftRowKindText",
    "AssetsActivityReferenceRightPanel",
    "AssetsActivityReferenceRightTitleText",
    "AssetsActivityReferenceRightScrollBody",
    "AssetsActivityReferenceRightEmptyText",
    "AssetsActivityReferenceRightRowPanel",
    "AssetsActivityReferenceRightRowNameText",
    "AssetsActivityReferenceRightRowLocatorText",
    "AssetsActivityReferenceRightRowKindText",
];
