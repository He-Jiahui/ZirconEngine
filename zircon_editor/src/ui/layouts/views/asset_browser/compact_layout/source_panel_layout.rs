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
    let text_x = x + metrics.text_inset.min(width);
    let text_width = finite_non_negative(width - metrics.text_inset * 2.0);
    let title_height = metrics
        .title_line_height
        .min(finite_non_negative(header_height - title_offset_y));
    let subtitle_height = metrics
        .subtitle_line_height
        .min(finite_non_negative(header_height - subtitle_offset_y));
    let row_height = metrics
        .row_height
        .min(finite_non_negative(scroll_height - metrics.row_inset * 2.0));
    let row_start_y = scroll_y + metrics.row_inset.min(scroll_height);
    let mut row_index = 0;

    for node in nodes.iter_mut() {
        let frame = match node.control_id.as_str() {
            "AssetBrowserSourcesPanel" => ViewTemplateFrameData {
                x: finite_coordinate(x),
                y: finite_coordinate(y),
                width,
                height,
            },
            "AssetBrowserSourcesHeaderPanel" => ViewTemplateFrameData {
                x: finite_coordinate(x),
                y: finite_coordinate(y),
                width,
                height: header_height,
            },
            "AssetBrowserSourcesTitleText" => ViewTemplateFrameData {
                x: finite_coordinate(text_x),
                y: finite_coordinate(y + title_offset_y),
                width: text_width,
                height: title_height,
            },
            "AssetBrowserSourcesSubtitleText" => ViewTemplateFrameData {
                x: finite_coordinate(text_x),
                y: finite_coordinate(y + subtitle_offset_y),
                width: text_width,
                height: subtitle_height,
            },
            "AssetBrowserSourcesDivider" => ViewTemplateFrameData {
                x: finite_coordinate(x),
                y: finite_coordinate(y + header_height),
                width,
                height: divider_height,
            },
            "AssetBrowserSourcesScrollBody" => ViewTemplateFrameData {
                x: finite_coordinate(x),
                y: finite_coordinate(scroll_y),
                width,
                height: scroll_height,
            },
            control_id if is_source_tree_row(control_id) => {
                let frame = ViewTemplateFrameData {
                    x: finite_coordinate(row_x),
                    y: finite_coordinate(
                        row_start_y + row_index as f32 * (metrics.row_height + metrics.row_gap),
                    ),
                    width: finite_non_negative(row_width),
                    height: finite_non_negative(row_height),
                };
                row_index += 1;
                frame
            }
            _ => continue,
        };
        node.frame = frame;
    }
}

pub(in crate::ui::layouts::views::asset_browser) fn apply_asset_browser_sources_layout(
    nodes: &mut [ViewTemplateNodeData],
) {
    let Some(panel) = node_frame(nodes, "AssetBrowserSourcesPanel") else {
        return;
    };
    apply_compact_sources_panel_layout(nodes, panel.x, panel.y, panel.width, panel.height);
}

fn node_frame(nodes: &[ViewTemplateNodeData], control_id: &str) -> Option<ViewTemplateFrameData> {
    nodes
        .iter()
        .find(|node| node.control_id == control_id)
        .map(|node| node.frame.clone())
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
#[path = "tests/source_panel_layout.rs"]
mod tests;
