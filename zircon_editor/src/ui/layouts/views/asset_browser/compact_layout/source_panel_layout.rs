use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};

use super::super::source_tree_nodes::is_source_tree_row;

#[derive(Clone, Copy)]
struct SourcesPanelMetrics {
    header_height: f32,
    divider_height: f32,
    row_inset: f32,
    row_height: f32,
    row_gap: f32,
    text_inset: f32,
    title_line_height: f32,
    subtitle_line_height: f32,
    text_gap: f32,
}

fn sources_panel_metrics() -> SourcesPanelMetrics {
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let typography = EditorTypographyTokens::workbench_default();
    SourcesPanelMetrics {
        header_height: controls.large_height,
        divider_height: controls.border_width,
        row_inset: density.gap_medium,
        row_height: density.row_height,
        row_gap: density.gap_small,
        text_inset: density.gap_large,
        title_line_height: typography.body_size * typography.line_height,
        subtitle_line_height: typography.caption_size * typography.line_height,
        text_gap: density.gap_xsmall,
    }
}

pub(super) fn apply_compact_sources_panel_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let metrics = sources_panel_metrics();
    let width = finite_non_negative(width);
    let height = finite_non_negative(height);
    let header_height = metrics.header_height.min(height);
    let divider_height = metrics
        .divider_height
        .min(finite_non_negative(height - header_height));
    let combined_text_height =
        metrics.title_line_height + metrics.text_gap + metrics.subtitle_line_height;
    let title_offset_y = finite_non_negative((header_height - combined_text_height) / 2.0);
    let subtitle_offset_y =
        (title_offset_y + metrics.title_line_height + metrics.text_gap).min(header_height);
    let scroll_y = y + header_height + divider_height;
    let scroll_height = finite_non_negative(height - header_height - divider_height);
    let row_x = x + metrics.row_inset.min(width);
    let row_width = finite_non_negative(width - (row_x - x) - metrics.row_inset);

    set_node_frame(nodes, "AssetBrowserSourcesPanel", x, y, width, height);
    set_node_frame(
        nodes,
        "AssetBrowserSourcesHeaderPanel",
        x,
        y,
        width,
        header_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserSourcesTitleText",
        x + metrics.text_inset.min(width),
        y + title_offset_y,
        finite_non_negative(width - metrics.text_inset * 2.0),
        metrics
            .title_line_height
            .min(finite_non_negative(header_height - title_offset_y)),
    );
    set_node_frame(
        nodes,
        "AssetBrowserSourcesSubtitleText",
        x + metrics.text_inset.min(width),
        y + subtitle_offset_y,
        finite_non_negative(width - metrics.text_inset * 2.0),
        metrics
            .subtitle_line_height
            .min(finite_non_negative(header_height - subtitle_offset_y)),
    );
    set_node_frame(
        nodes,
        "AssetBrowserSourcesDivider",
        x,
        y + header_height,
        width,
        divider_height,
    );
    set_node_frame(
        nodes,
        "AssetBrowserSourcesScrollBody",
        x,
        scroll_y,
        width,
        scroll_height,
    );
    apply_source_tree_rows_layout(nodes, row_x, scroll_y, row_width, scroll_height, metrics);
}

pub(in crate::ui::layouts::views::asset_browser) fn apply_asset_browser_sources_layout(
    nodes: &mut [ViewTemplateNodeData],
) {
    let Some(panel) = node_frame(nodes, "AssetBrowserSourcesPanel") else {
        return;
    };
    apply_compact_sources_panel_layout(nodes, panel.x, panel.y, panel.width, panel.height);
}

fn apply_source_tree_rows_layout(
    nodes: &mut [ViewTemplateNodeData],
    row_x: f32,
    scroll_y: f32,
    row_width: f32,
    scroll_height: f32,
    metrics: SourcesPanelMetrics,
) {
    let row_height = metrics
        .row_height
        .min(finite_non_negative(scroll_height - metrics.row_inset * 2.0));
    let start_y = scroll_y + metrics.row_inset.min(scroll_height);
    for (index, node) in nodes
        .iter_mut()
        .filter(|node| is_source_tree_row(node.control_id.as_str()))
        .enumerate()
    {
        node.frame = ViewTemplateFrameData {
            x: finite_coordinate(row_x),
            y: finite_coordinate(start_y + index as f32 * (metrics.row_height + metrics.row_gap)),
            width: finite_non_negative(row_width),
            height: finite_non_negative(row_height),
        };
    }
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

#[cfg(test)]
mod tests {
    use super::{apply_compact_sources_panel_layout, sources_panel_metrics};
    use crate::ui::layouts::views::ViewTemplateNodeData;
    use zircon_runtime_interface::ui::design_tokens::{
        EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
    };

    #[test]
    fn sources_panel_metrics_follow_shared_component_tokens() {
        let metrics = sources_panel_metrics();
        let density = EditorDensityTokens::workbench_dense();
        let controls = EditorControlTokens::workbench_dense();
        let typography = EditorTypographyTokens::workbench_default();

        assert_eq!(metrics.header_height, controls.large_height);
        assert_eq!(metrics.divider_height, controls.border_width);
        assert_eq!(metrics.row_inset, density.gap_medium);
        assert_eq!(metrics.row_height, density.row_height);
        assert_eq!(metrics.row_gap, density.gap_small);
        assert_eq!(metrics.text_inset, density.gap_large);
        assert_eq!(
            metrics.title_line_height,
            typography.body_size * typography.line_height
        );
        assert_eq!(
            metrics.subtitle_line_height,
            typography.caption_size * typography.line_height
        );
    }

    #[test]
    fn sources_panel_layout_uses_header_then_scroll_body_and_relative_tree_rows() {
        let mut nodes = [
            "AssetBrowserSourcesPanel",
            "AssetBrowserSourcesHeaderPanel",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
            "AssetBrowserSourcesDivider",
            "AssetBrowserSourcesScrollBody",
            "AssetBrowserSourcesRowPanel",
        ]
        .into_iter()
        .map(node)
        .collect::<Vec<_>>();

        apply_compact_sources_panel_layout(&mut nodes, 20.0, 30.0, 300.0, 180.0);

        let metrics = sources_panel_metrics();
        let header = node_by_id(&nodes, "AssetBrowserSourcesHeaderPanel");
        let scroll = node_by_id(&nodes, "AssetBrowserSourcesScrollBody");
        let row = node_by_id(&nodes, "AssetBrowserSourcesRowPanel");
        assert_eq!(header.frame.height, metrics.header_height);
        assert_eq!(
            scroll.frame.y,
            30.0 + metrics.header_height + metrics.divider_height
        );
        assert_eq!(
            scroll.frame.height,
            180.0 - metrics.header_height - metrics.divider_height
        );
        assert_eq!(row.frame.x, 20.0 + metrics.row_inset);
        assert_eq!(row.frame.y, scroll.frame.y + metrics.row_inset);
        assert_eq!(row.frame.height, metrics.row_height);
    }

    #[test]
    fn sources_panel_layout_keeps_collapsed_text_inside_a_tiny_header() {
        let mut nodes = [
            "AssetBrowserSourcesPanel",
            "AssetBrowserSourcesHeaderPanel",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
            "AssetBrowserSourcesDivider",
            "AssetBrowserSourcesScrollBody",
        ]
        .into_iter()
        .map(node)
        .collect::<Vec<_>>();

        apply_compact_sources_panel_layout(&mut nodes, 20.0, 30.0, 300.0, 10.0);

        for control_id in [
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
        ] {
            let frame = &node_by_id(&nodes, control_id).frame;
            assert!(frame.y >= 30.0);
            assert!(frame.y <= 40.0);
            assert!(frame.height >= 0.0);
            assert!(frame.y + frame.height <= 40.0);
        }
    }

    #[test]
    fn sources_panel_layout_does_not_extend_the_first_row_beyond_a_shallow_scroll_body() {
        let mut nodes = [
            "AssetBrowserSourcesPanel",
            "AssetBrowserSourcesHeaderPanel",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
            "AssetBrowserSourcesDivider",
            "AssetBrowserSourcesScrollBody",
            "AssetBrowserSourcesRowPanel",
        ]
        .into_iter()
        .map(node)
        .collect::<Vec<_>>();

        apply_compact_sources_panel_layout(&mut nodes, 20.0, 30.0, 300.0, 54.0);

        let scroll = &node_by_id(&nodes, "AssetBrowserSourcesScrollBody").frame;
        let row = &node_by_id(&nodes, "AssetBrowserSourcesRowPanel").frame;
        assert_eq!(row.height, 0.0);
        assert_eq!(row.y, scroll.y + scroll.height);
        assert!(row.y + row.height <= scroll.y + scroll.height);
    }

    fn node(control_id: &str) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: control_id.into(),
            control_id: control_id.into(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn node_by_id<'a>(
        nodes: &'a [ViewTemplateNodeData],
        control_id: &str,
    ) -> &'a ViewTemplateNodeData {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .expect("source panel node")
    }
}
