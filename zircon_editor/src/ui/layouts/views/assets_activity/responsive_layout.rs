use std::collections::HashMap;

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
const PREVIEW_OVERLAY_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_OVERLAY_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;
const PREVIEW_CAPTION_LINE_HEIGHT: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE
    * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO;

struct ResponsiveNodeIndex<'a> {
    nodes: &'a mut [ViewTemplateNodeData],
    indices_by_control_id: HashMap<String, Vec<usize>>,
}

impl<'a> ResponsiveNodeIndex<'a> {
    fn new(nodes: &'a mut [ViewTemplateNodeData]) -> Self {
        let mut indices_by_control_id = HashMap::<String, Vec<usize>>::with_capacity(nodes.len());
        for (index, node) in nodes.iter().enumerate() {
            indices_by_control_id
                .entry(node.control_id.to_string())
                .or_default()
                .push(index);
        }
        Self {
            nodes,
            indices_by_control_id,
        }
    }

    fn node(&self, control_id: &str) -> Option<&ViewTemplateNodeData> {
        let index = *self.indices_by_control_id.get(control_id)?.first()?;
        self.nodes.get(index)
    }

    fn frame(&self, control_id: &str) -> Option<ViewTemplateFrameData> {
        self.node(control_id).map(|node| node.frame.clone())
    }

    fn set_frame(&mut self, control_id: &str, x: f32, y: f32, width: f32, height: f32) {
        let Some(indices) = self.indices_by_control_id.get(control_id) else {
            return;
        };
        for &index in indices {
            let node = &mut self.nodes[index];
            node.frame.x = x;
            node.frame.y = y;
            node.frame.width = width.max(0.0);
            node.frame.height = height.max(0.0);
        }
    }
}

pub(super) fn apply_assets_activity_responsive_layout(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
    size: UiSize,
) {
    let mut nodes = ResponsiveNodeIndex::new(nodes);
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let Some(root) = node_frame(&nodes, "AssetsActivityRoot") else {
        return;
    };
    if root.width > density.breakpoint_narrow_width {
        apply_wide_utility_tab_visibility(&mut nodes, snapshot, &root);
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
        &mut nodes,
        root.x,
        root.y,
        width,
        toolbar_height,
        row_height,
        density,
        controls,
    );
    layout_main_content(&mut nodes, root.x, main_y, width, main_height);
    layout_utility(
        &mut nodes,
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
    nodes: &mut ResponsiveNodeIndex<'_>,
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
    nodes: &mut ResponsiveNodeIndex<'_>,
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
        ],
        x,
        y,
    );
    set_node_frame(nodes, "AssetsActivityToolbarPanel", x, y, width, height);

    let padding = density.gap_medium.min(width * 0.1);
    let inner_width = (width - padding * 2.0).max(0.0);
    let (browser_width, search_width, search_gap) = fit_horizontal_pair(
        inner_width,
        measured_button_width(nodes, "OpenAssetBrowser", density, controls),
        inner_width,
        density.gap_small,
    );
    let browser_x = x + padding + search_width + search_gap;
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
        "AssetsActivityToolbarFilterRow",
        x,
        second_y,
        width,
        row_height,
    );
    layout_compact_toolbar_controls(
        nodes,
        x + padding,
        second_y,
        inner_width,
        row_height,
        density,
        controls,
    );
}

fn layout_compact_toolbar_controls(
    nodes: &mut ResponsiveNodeIndex<'_>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) {
    let icon_width = controls.dense_height.min(width.max(0.0));
    let first_gap = density.gap_small.min((width - icon_width).max(0.0));
    let second_icon_width = icon_width.min((width - icon_width - first_gap).max(0.0));
    let second_gap = density
        .gap_small
        .min((width - icon_width - first_gap - second_icon_width).max(0.0));
    let filter_width = (width - icon_width - first_gap - second_icon_width - second_gap).max(0.0);
    let list_x = x + filter_width + second_gap;
    let thumbnail_x = list_x + icon_width + first_gap;

    set_node_frame(
        nodes,
        "AssetsActivityKindFilterDropdown",
        x,
        y,
        filter_width,
        height,
    );
    set_node_frame(
        nodes,
        "AssetsActivityViewModeListButton",
        list_x,
        y,
        icon_width,
        height,
    );
    set_node_frame(
        nodes,
        "AssetsActivityViewModeThumbButton",
        thumbnail_x,
        y,
        second_icon_width,
        height,
    );
}

fn layout_main_content(
    nodes: &mut ResponsiveNodeIndex<'_>,
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
    nodes: &mut ResponsiveNodeIndex<'_>,
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
    let inner_width = (width - padding * 2.0).max(0.0);
    let (preview_width, references_width, utility_tab_gap) = fit_horizontal_pair(
        inner_width,
        measured_button_width(nodes, "AssetsActivityPreviewTabButton", density, controls),
        measured_button_width(
            nodes,
            "AssetsActivityReferencesTabButton",
            density,
            controls,
        ),
        density.gap_small,
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
        x + padding + preview_width + utility_tab_gap,
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
        }
        AssetUtilityTab::Metadata | AssetUtilityTab::Plugins => {
            hide_nodes(nodes, PREVIEW_CONTROLS, x, content_y);
            hide_nodes(nodes, REFERENCE_CONTROLS, x, content_y);
        }
    }
}

fn layout_preview(
    nodes: &mut ResponsiveNodeIndex<'_>,
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
    for (control_id, offset_y, line_height) in [
        (
            "AssetsActivityPreviewNameText",
            density.gap_small,
            PREVIEW_OVERLAY_LINE_HEIGHT,
        ),
        (
            "AssetsActivityPreviewLocatorText",
            density.gap_small + PREVIEW_OVERLAY_LINE_HEIGHT + density.gap_xsmall,
            PREVIEW_CAPTION_LINE_HEIGHT,
        ),
        (
            "AssetsActivityPreviewKindText",
            density.gap_small
                + PREVIEW_OVERLAY_LINE_HEIGHT
                + density.gap_xsmall
                + PREVIEW_CAPTION_LINE_HEIGHT
                + density.gap_xsmall,
            PREVIEW_CAPTION_LINE_HEIGHT,
        ),
    ] {
        set_node_frame(
            nodes,
            control_id,
            text_x,
            y + offset_y,
            text_width,
            line_height,
        );
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

fn measured_button_width(
    nodes: &ResponsiveNodeIndex<'_>,
    control_id: &str,
    density: EditorDensityTokens,
    controls: EditorControlTokens,
) -> f32 {
    let Some(node) = nodes.node(control_id) else {
        return controls.default_height;
    };
    if node.role.as_str() == "IconButton" {
        return controls.dense_height;
    }
    let text = node.text.as_str();
    (measure_runtime_text_width(text, EditorTypographyTokens::WORKBENCH_BODY_SIZE)
        + density.gap_medium * 2.0)
        .max(controls.default_height)
}

fn fit_horizontal_pair(
    available_width: f32,
    primary_preferred_width: f32,
    secondary_preferred_width: f32,
    preferred_gap: f32,
) -> (f32, f32, f32) {
    let available_width = if available_width.is_finite() {
        available_width.max(0.0)
    } else {
        0.0
    };
    let primary_width = if primary_preferred_width.is_finite() {
        primary_preferred_width.max(0.0).min(available_width)
    } else {
        0.0
    };
    let secondary_preferred_width = if secondary_preferred_width.is_finite() {
        secondary_preferred_width.max(0.0)
    } else {
        0.0
    };
    let remaining_width = (available_width - primary_width).max(0.0);
    let gap = if primary_width > 0.0 && secondary_preferred_width > 0.0 {
        preferred_gap.max(0.0).min(remaining_width)
    } else {
        0.0
    };
    let secondary_width = secondary_preferred_width.min((remaining_width - gap).max(0.0));

    (primary_width, secondary_width, gap)
}

fn node_frame(nodes: &ResponsiveNodeIndex<'_>, control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes.frame(control_id)
}

fn hide_nodes(nodes: &mut ResponsiveNodeIndex<'_>, control_ids: &[&str], x: f32, y: f32) {
    for control_id in control_ids {
        set_node_frame(nodes, control_id, x, y, 0.0, 0.0);
    }
}

fn set_node_frame(
    nodes: &mut ResponsiveNodeIndex<'_>,
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    nodes.set_frame(control_id, x, y, width, height);
}

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

#[cfg(test)]
mod tests {
    use super::fit_horizontal_pair;

    #[test]
    fn horizontal_pair_never_exceeds_an_ultra_narrow_budget() {
        let (primary, secondary, gap) = fit_horizontal_pair(16.0, 28.0, 64.0, 4.0);

        assert_eq!(primary, 16.0);
        assert_eq!(secondary, 0.0);
        assert_eq!(gap, 0.0);
        assert!(primary + gap + secondary <= 16.0);
    }

    #[test]
    fn horizontal_pair_keeps_preferred_controls_and_standard_gap_when_space_allows() {
        let (primary, secondary, gap) = fit_horizontal_pair(120.0, 32.0, 40.0, 4.0);

        assert_eq!((primary, secondary, gap), (32.0, 40.0, 4.0));
        assert!(primary + gap + secondary <= 120.0);
    }
}

#[cfg(test)]
mod control_index_tests;
