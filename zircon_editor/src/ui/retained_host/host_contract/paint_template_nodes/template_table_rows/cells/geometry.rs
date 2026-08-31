use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::actions::table_action_column_width;
use super::super::identity::{is_table_header, is_table_tail};
use super::allocation::{allocate_table_columns_for_node, TableColumnLayout};
use super::metrics::{table_cell_metrics, WorkbenchTableCellMetrics, TABLE_COLUMN_COUNT};

#[derive(Clone, Copy)]
struct TableCellLayout {
    content_offset_x: f32,
    content_offset_y: f32,
    metrics: WorkbenchTableCellMetrics,
    columns: TableColumnLayout,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cell_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
) -> FrameRect {
    let layout = table_cell_layout(node, rect);
    table_cell_rect_from_layout(node, rect, index, layout, layout.columns.x_offset(index))
}

pub(super) fn table_cell_rects(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> [FrameRect; TABLE_COLUMN_COUNT] {
    // Column minimums use Runtime Text measurement, so a row shares one allocation snapshot.
    let layout = table_cell_layout(node, rect);
    let mut x_offset = 0.0;
    std::array::from_fn(|index| {
        let cell = table_cell_rect_from_layout(node, rect, index, layout, x_offset);
        x_offset += layout.columns.width(index);
        cell
    })
}

fn table_cell_layout(node: &TemplatePaneNodeData, rect: &FrameRect) -> TableCellLayout {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let metrics = table_cell_metrics();
    let available_width =
        (rect.width - metrics.inset_x * 2.0 - table_action_column_width()).max(0.0);
    TableCellLayout {
        content_offset_x,
        content_offset_y,
        metrics,
        columns: allocate_table_columns_for_node(node, available_width),
    }
}

fn table_cell_rect_from_layout(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    index: usize,
    layout: TableCellLayout,
    x_offset: f32,
) -> FrameRect {
    let x = rect.x + layout.metrics.inset_x + layout.content_offset_x + x_offset;
    let width = layout.columns.width(index);
    FrameRect {
        x: x + table_cell_offset_x(node, index),
        y: rect.y + layout.metrics.inset_y + layout.content_offset_y,
        width: width.max(0.0),
        height: (rect.height - layout.metrics.inset_y * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_content_offset(
    node: &TemplatePaneNodeData,
) -> (f32, f32) {
    if is_table_header(node) || is_table_tail(node) {
        (node.layout_content_offset_x, node.layout_content_offset_y)
    } else {
        (0.0, 0.0)
    }
}

fn table_cell_offset_x(node: &TemplatePaneNodeData, index: usize) -> f32 {
    match index {
        0 => node.layout_first_cell_offset_x,
        1 => node.layout_second_cell_offset_x,
        2 => node.layout_third_cell_offset_x,
        3 => node.layout_fourth_cell_offset_x,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const ROWS_PER_SAMPLE: usize = 524_288;

    #[test]
    fn row_cell_snapshot_matches_individual_cell_geometry() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 11.0,
            y: 17.0,
            width: 360.0,
            height: 28.0,
        };
        let snapshot = table_cell_rects(&node, &rect);

        for (index, cell_rect) in snapshot.iter().enumerate() {
            assert_eq!(cell_rect, &table_cell_rect(&node, &rect, index));
        }
    }

    #[test]
    fn optimization_batch_eu_editor383_uses_one_running_table_offset() {
        let production = include_str!("geometry.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("let mut x_offset = 0.0;"));
        assert!(production.contains("x_offset += layout.columns.width(index);"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_eu_editor383_single_pass_table_offsets_benchmark() {
        let widths = [137.5_f32, 103.25, 72.75, 68.5];
        for _ in 0..4 {
            black_box(measure_legacy_offsets(widths));
            black_box(measure_single_pass_offsets(widths));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_offsets(widths));
                optimized_samples.push(measure_single_pass_offsets(widths));
            } else {
                optimized_samples.push(measure_single_pass_offsets(widths));
                legacy_samples.push(measure_legacy_offsets(widths));
            }
        }

        report_offset_performance(&legacy_samples, &optimized_samples);
    }

    fn measure_legacy_offsets(widths: [f32; TABLE_COLUMN_COUNT]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0.0_f32;
        for row in 0..ROWS_PER_SAMPLE {
            let row_widths = black_box(widths);
            let offsets = std::array::from_fn::<_, TABLE_COLUMN_COUNT, _>(|index| {
                row_widths.iter().take(index).sum::<f32>()
            });
            checksum += black_box(offsets[row % TABLE_COLUMN_COUNT]);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_single_pass_offsets(widths: [f32; TABLE_COLUMN_COUNT]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0.0_f32;
        for row in 0..ROWS_PER_SAMPLE {
            let row_widths = black_box(widths);
            let mut x_offset = 0.0;
            let offsets = std::array::from_fn::<_, TABLE_COLUMN_COUNT, _>(|index| {
                let current = x_offset;
                x_offset += row_widths[index];
                current
            });
            checksum += black_box(offsets[row % TABLE_COLUMN_COUNT]);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_offset_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR383_SINGLE_PASS_TABLE_OFFSETS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} rows_per_sample={ROWS_PER_SAMPLE} columns={TABLE_COLUMN_COUNT} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=15",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(85) / 100,
            "single-pass table offsets must reduce P95 by at least 15%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
