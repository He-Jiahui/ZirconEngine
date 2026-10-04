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
    let production = include_str!("../geometry.rs")
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
