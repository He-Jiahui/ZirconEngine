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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use zircon_runtime_interface::ui::design_tokens::{
        EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
    };

    const SINGLE_PASS_BENCHMARK_NODES: usize = 4_096;
    const SINGLE_PASS_BENCHMARK_ROWS: usize = 512;
    const SINGLE_PASS_BENCHMARK_ITERATIONS: usize = 256;
    const SINGLE_PASS_BENCHMARK_SAMPLES: usize = 11;

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

    #[test]
    fn editor57_compact_single_pass_sources_layout_preserves_duplicates_and_row_order() {
        let mut nodes = [
            "AssetBrowserSourcesPanel",
            "AssetBrowserSourcesHeaderPanel",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
            "AssetBrowserSourcesDivider",
            "AssetBrowserSourcesScrollBody",
            "AssetBrowserSourcesRowPanel",
            "AssetBrowserSourcesRowPanel",
            "UnrelatedNode",
        ]
        .into_iter()
        .map(node)
        .collect::<Vec<_>>();

        apply_compact_sources_panel_layout(&mut nodes, 20.0, 30.0, 300.0, 180.0);

        let titles = nodes
            .iter()
            .filter(|node| node.control_id == "AssetBrowserSourcesTitleText")
            .collect::<Vec<_>>();
        assert_eq!(titles.len(), 2);
        assert_eq!(titles[0].frame, titles[1].frame);
        let rows = nodes
            .iter()
            .filter(|node| node.control_id == "AssetBrowserSourcesRowPanel")
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert!(rows[1].frame.y > rows[0].frame.y);
        assert_eq!(
            node_by_id(&nodes, "UnrelatedNode").frame,
            ViewTemplateFrameData::default()
        );
    }

    #[test]
    #[ignore = "release performance gate; run through the managed Editor57 validator"]
    fn editor57_compact_single_pass_sources_layout_release_benchmark() {
        let source = source_layout_benchmark_nodes();
        let mut retired = source.clone();
        let mut optimized = source.clone();
        retired_apply_compact_sources_panel_layout(&mut retired, 20.0, 30.0, 1_024.0, 768.0);
        apply_compact_sources_panel_layout(&mut optimized, 20.0, 30.0, 1_024.0, 768.0);
        assert_layout_frames_eq(&optimized, &retired);

        let mut retired_samples = Vec::with_capacity(SINGLE_PASS_BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SINGLE_PASS_BENCHMARK_SAMPLES);
        for sample in 0..SINGLE_PASS_BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_retired_sources_layout(&source));
                optimized_samples.push(measure_single_pass_sources_layout(&source));
            } else {
                optimized_samples.push(measure_single_pass_sources_layout(&source));
                retired_samples.push(measure_retired_sources_layout(&source));
            }
        }

        let retired_p95 = nearest_rank(&retired_samples, 95);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95
                .saturating_mul(10_000)
                .checked_div(retired_p95)
                .unwrap_or(0),
        );
        println!(
            "EDITOR57_SINGLE_PASS_COMPACT_SOURCES_LAYOUT_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank node_count={} row_count={} iterations={} retired_full_node_passes=7 optimized_full_node_passes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
            SINGLE_PASS_BENCHMARK_SAMPLES,
            SINGLE_PASS_BENCHMARK_NODES,
            SINGLE_PASS_BENCHMARK_ROWS,
            SINGLE_PASS_BENCHMARK_ITERATIONS,
            retired_p95,
            optimized_p95,
            reduction_basis_points,
            join_samples(&retired_samples),
            join_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(2) <= retired_p95,
            "single-pass sources layout P95 must be at most 50% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn source_layout_benchmark_nodes() -> Vec<ViewTemplateNodeData> {
        let mut nodes = Vec::with_capacity(SINGLE_PASS_BENCHMARK_NODES);
        for control_id in [
            "AssetBrowserSourcesPanel",
            "AssetBrowserSourcesHeaderPanel",
            "AssetBrowserSourcesTitleText",
            "AssetBrowserSourcesSubtitleText",
            "AssetBrowserSourcesDivider",
            "AssetBrowserSourcesScrollBody",
        ] {
            nodes.push(node(control_id));
        }
        for _ in 0..SINGLE_PASS_BENCHMARK_ROWS {
            nodes.push(node("AssetBrowserSourcesRowPanel"));
        }
        while nodes.len() < SINGLE_PASS_BENCHMARK_NODES {
            nodes.push(node("UnrelatedNode"));
        }
        nodes
    }

    fn measure_retired_sources_layout(source: &[ViewTemplateNodeData]) -> u128 {
        let mut nodes = source.to_vec();
        let started = Instant::now();
        for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
            retired_apply_compact_sources_panel_layout(
                black_box(&mut nodes),
                20.0,
                30.0,
                1_024.0,
                768.0,
            );
        }
        started.elapsed().as_nanos()
    }

    fn measure_single_pass_sources_layout(source: &[ViewTemplateNodeData]) -> u128 {
        let mut nodes = source.to_vec();
        let started = Instant::now();
        for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
            apply_compact_sources_panel_layout(black_box(&mut nodes), 20.0, 30.0, 1_024.0, 768.0);
        }
        started.elapsed().as_nanos()
    }

    fn retired_apply_compact_sources_panel_layout(
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

        retired_set_node_frame(nodes, "AssetBrowserSourcesPanel", x, y, width, height);
        retired_set_node_frame(
            nodes,
            "AssetBrowserSourcesHeaderPanel",
            x,
            y,
            width,
            header_height,
        );
        retired_set_node_frame(
            nodes,
            "AssetBrowserSourcesTitleText",
            x + metrics.text_inset.min(width),
            y + title_offset_y,
            finite_non_negative(width - metrics.text_inset * 2.0),
            metrics
                .title_line_height
                .min(finite_non_negative(header_height - title_offset_y)),
        );
        retired_set_node_frame(
            nodes,
            "AssetBrowserSourcesSubtitleText",
            x + metrics.text_inset.min(width),
            y + subtitle_offset_y,
            finite_non_negative(width - metrics.text_inset * 2.0),
            metrics
                .subtitle_line_height
                .min(finite_non_negative(header_height - subtitle_offset_y)),
        );
        retired_set_node_frame(
            nodes,
            "AssetBrowserSourcesDivider",
            x,
            y + header_height,
            width,
            divider_height,
        );
        retired_set_node_frame(
            nodes,
            "AssetBrowserSourcesScrollBody",
            x,
            scroll_y,
            width,
            scroll_height,
        );
        retired_apply_source_tree_rows_layout(
            nodes,
            row_x,
            scroll_y,
            row_width,
            scroll_height,
            metrics,
        );
    }

    fn retired_apply_source_tree_rows_layout(
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
                y: finite_coordinate(
                    start_y + index as f32 * (metrics.row_height + metrics.row_gap),
                ),
                width: finite_non_negative(row_width),
                height: finite_non_negative(row_height),
            };
        }
    }

    fn retired_set_node_frame(
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

    fn assert_layout_frames_eq(actual: &[ViewTemplateNodeData], expected: &[ViewTemplateNodeData]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.control_id, expected.control_id);
            assert_eq!(actual.frame, expected.frame);
        }
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
        ordered[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
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
