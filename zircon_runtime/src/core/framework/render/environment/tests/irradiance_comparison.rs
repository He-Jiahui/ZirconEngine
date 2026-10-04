use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const BENCH_FACE_SIZE: u32 = 256;
const PRIMARY_REPEATS: usize = 16;

#[test]
fn optimization_batch_20260830ew_runtime560_traverses_face_rows_without_texel_division() {
    let production = include_str!("../irradiance_comparison.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("chunks_exact(face_sample_count)"));
    assert!(!production.contains("index % face_sample_count"));
    assert!(!production.contains("texel_index"));
}

#[test]
fn optimization_batch_20260830ew_runtime561_hoists_row_edge_classification() {
    let production = include_str!("../irradiance_comparison.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("let row_is_edge"));
    assert!(production.contains("row_is_edge || x == 0 || x + 1 == face_size"));
}

#[test]
fn optimization_batch_20260830ew_runtime560_561_preserves_complete_statistics() {
    for face_size in [1, 2, 7, 32] {
        let (reference, candidate) = comparison_cubes(face_size);
        assert_eq!(
            compare_source_cubemap_irradiance(&reference, &candidate),
            Ok(legacy_statistics(&reference, &candidate)),
            "face_size={face_size}"
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_20260830ew_runtime560_face_row_traversal_benchmark() {
    let (reference, candidate) = comparison_cubes(BENCH_FACE_SIZE);
    assert_eq!(
        primary_error_legacy(&reference, &candidate),
        primary_error_sliced::<false>(&reference, &candidate)
    );
    let (legacy_p95, optimized_p95) = sample_performance(
        || measure_legacy_primary(&reference, &candidate),
        || measure_sliced_primary::<false>(&reference, &candidate),
    );
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME560_IRRADIANCE_FACE_ROW_TRAVERSAL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} face_size={BENCH_FACE_SIZE} repeats={PRIMARY_REPEATS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20"
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
        "face/row traversal must reduce P95 by at least 20%"
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_20260830ew_runtime561_row_edge_hoist_benchmark() {
    let (reference, candidate) = comparison_cubes(BENCH_FACE_SIZE);
    assert_eq!(
        primary_error_sliced::<false>(&reference, &candidate),
        primary_error_sliced::<true>(&reference, &candidate)
    );
    let (legacy_p95, optimized_p95) = sample_performance(
        || measure_sliced_primary::<false>(&reference, &candidate),
        || measure_sliced_primary::<true>(&reference, &candidate),
    );
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME561_IRRADIANCE_ROW_EDGE_HOIST_BENCH_V1 sample_pairs={SAMPLE_PAIRS} face_size={BENCH_FACE_SIZE} repeats={PRIMARY_REPEATS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20"
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
        "row-edge classification hoist must reduce P95 by at least 20%"
    );
}

#[test]
fn comparison_reports_per_channel_total_and_edge_error() {
    let reference = SourceCubemapIrradianceCube::new(3, vec![[0.0; 3]; 54]);
    let mut candidate_texels = vec![[0.0; 3]; 54];
    candidate_texels[0] = [0.2, 0.1, 0.0];
    candidate_texels[4] = [0.4, 0.0, 0.0];
    let candidate = SourceCubemapIrradianceCube::new(3, candidate_texels);

    let statistics = compare_source_cubemap_irradiance(&reference, &candidate)
        .expect("matching cubemap layouts should compare");

    assert_eq!(statistics.sample_count(), 54);
    assert_eq!(statistics.edge_sample_count(), 48);
    assert_eq!(statistics.seam_sample_count(), 72);
    assert_channels_close(
        statistics.mean_absolute_error(),
        [0.6 / 54.0, 0.1 / 54.0, 0.0],
    );
    assert_channels_close(statistics.max_absolute_error(), [0.4, 0.1, 0.0]);
    assert_channels_close(
        statistics.edge_mean_absolute_error(),
        [0.2 / 48.0, 0.1 / 48.0, 0.0],
    );
    assert_channels_close(statistics.edge_max_absolute_error(), [0.2, 0.1, 0.0]);
    assert!(statistics.seam_max_absolute_error()[0] > 0.0);
    assert!(statistics.seam_max_absolute_error()[1] > 0.0);

    let baseline = compare_source_cubemap_irradiance(&reference, &reference)
        .expect("matching cubemap layouts should compare");
    assert_channels_close(baseline.seam_mean_absolute_error(), [0.0; 3]);
    assert_channels_close(baseline.seam_max_absolute_error(), [0.0; 3]);
}

#[test]
fn comparison_rejects_different_face_sizes() {
    let reference = SourceCubemapIrradianceCube::new(2, vec![[0.0; 3]; 24]);
    let candidate = SourceCubemapIrradianceCube::new(3, vec![[0.0; 3]; 54]);

    assert_eq!(
        compare_source_cubemap_irradiance(&reference, &candidate),
        Err(SourceCubemapIrradianceComparisonError::FaceSizeMismatch {
            reference: 2,
            candidate: 3,
        })
    );
}

#[derive(Clone, Debug, PartialEq)]
struct PrimaryError {
    total: [Real; 3],
    maximum: [Real; 3],
    edge_total: [Real; 3],
    edge_maximum: [Real; 3],
    edge_count: usize,
}

impl Default for PrimaryError {
    fn default() -> Self {
        Self {
            total: [0.0; 3],
            maximum: [0.0; 3],
            edge_total: [0.0; 3],
            edge_maximum: [0.0; 3],
            edge_count: 0,
        }
    }
}

fn comparison_cubes(face_size: u32) -> (SourceCubemapIrradianceCube, SourceCubemapIrradianceCube) {
    let sample_count = face_size as usize * face_size as usize * CubemapFace::ALL.len();
    let mut reference = Vec::with_capacity(sample_count);
    let mut candidate = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let base = (index % 4_093) as Real / 4_093.0;
        reference.push([base, base * 0.5, 1.0 - base]);
        candidate.push([base + 0.01, base * 0.49, 1.0 - base * 0.99]);
    }
    (
        SourceCubemapIrradianceCube::new(face_size, reference),
        SourceCubemapIrradianceCube::new(face_size, candidate),
    )
}

fn legacy_statistics(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> SourceCubemapIrradianceErrorStatistics {
    let primary = primary_error_legacy(reference, candidate);
    let sample_count = reference.texels().len();
    let (seam_error, seam_max_error, seam_sample_count) =
        compare_cubemap_seam_error(reference, candidate);
    SourceCubemapIrradianceErrorStatistics {
        sample_count,
        edge_sample_count: primary.edge_count,
        seam_sample_count,
        mean_absolute_error: divide_channels(primary.total, sample_count.max(1) as Real),
        max_absolute_error: primary.maximum,
        edge_mean_absolute_error: divide_channels(
            primary.edge_total,
            primary.edge_count.max(1) as Real,
        ),
        edge_max_absolute_error: primary.edge_maximum,
        seam_mean_absolute_error: divide_channels(seam_error, seam_sample_count.max(1) as Real),
        seam_max_absolute_error: seam_max_error,
    }
}

fn primary_error_legacy(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> PrimaryError {
    let face_size = reference.face_size() as usize;
    let face_sample_count = face_size * face_size;
    let mut statistics = PrimaryError::default();
    for (index, (reference, candidate)) in reference
        .texels()
        .iter()
        .zip(candidate.texels())
        .enumerate()
    {
        let texel_index = index % face_sample_count;
        let x = texel_index % face_size;
        let y = texel_index / face_size;
        accumulate_primary_error(
            &mut statistics,
            reference,
            candidate,
            x == 0 || y == 0 || x + 1 == face_size || y + 1 == face_size,
        );
    }
    statistics
}

fn primary_error_sliced<const HOIST_ROW_EDGE: bool>(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> PrimaryError {
    let face_size = reference.face_size() as usize;
    let face_sample_count = face_size * face_size;
    let mut statistics = PrimaryError::default();
    for (reference_face, candidate_face) in reference
        .texels()
        .chunks_exact(face_sample_count)
        .zip(candidate.texels().chunks_exact(face_sample_count))
    {
        for (y, (reference_row, candidate_row)) in reference_face
            .chunks_exact(face_size)
            .zip(candidate_face.chunks_exact(face_size))
            .enumerate()
        {
            let row_is_edge = HOIST_ROW_EDGE && (y == 0 || y + 1 == face_size);
            for (x, (reference, candidate)) in reference_row.iter().zip(candidate_row).enumerate() {
                let is_edge = if HOIST_ROW_EDGE {
                    row_is_edge || x == 0 || x + 1 == face_size
                } else {
                    x == 0 || y == 0 || x + 1 == face_size || y + 1 == face_size
                };
                accumulate_primary_error(&mut statistics, reference, candidate, is_edge);
            }
        }
    }
    statistics
}

fn accumulate_primary_error(
    statistics: &mut PrimaryError,
    reference: &[Real; 3],
    candidate: &[Real; 3],
    is_edge: bool,
) {
    if is_edge {
        statistics.edge_count += 1;
    }
    for channel in 0..3 {
        let error = (candidate[channel] - reference[channel]).abs();
        statistics.total[channel] += error;
        statistics.maximum[channel] = statistics.maximum[channel].max(error);
        if is_edge {
            statistics.edge_total[channel] += error;
            statistics.edge_maximum[channel] = statistics.edge_maximum[channel].max(error);
        }
    }
}

fn measure_legacy_primary(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> u128 {
    let started = Instant::now();
    for _ in 0..PRIMARY_REPEATS {
        black_box(primary_error_legacy(
            black_box(reference),
            black_box(candidate),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_sliced_primary<const HOIST_ROW_EDGE: bool>(
    reference: &SourceCubemapIrradianceCube,
    candidate: &SourceCubemapIrradianceCube,
) -> u128 {
    let started = Instant::now();
    for _ in 0..PRIMARY_REPEATS {
        black_box(primary_error_sliced::<HOIST_ROW_EDGE>(
            black_box(reference),
            black_box(candidate),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn sample_performance(
    mut legacy: impl FnMut() -> u128,
    mut optimized: impl FnMut() -> u128,
) -> (u128, u128) {
    for _ in 0..3 {
        black_box(legacy());
        black_box(optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }
    (
        nearest_rank_p95(&legacy_samples),
        nearest_rank_p95(&optimized_samples),
    )
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn assert_channels_close(actual: [Real; 3], expected: [Real; 3]) {
    for channel in 0..3 {
        assert!(
            (actual[channel] - expected[channel]).abs() <= 0.000_001,
            "channel={channel} actual={} expected={}",
            actual[channel],
            expected[channel],
        );
    }
}
