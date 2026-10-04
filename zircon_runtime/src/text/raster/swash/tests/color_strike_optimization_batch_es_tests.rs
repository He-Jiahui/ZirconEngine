use std::hint::black_box;
use std::time::Instant;

use super::*;

fn legacy_select_color_bitmap_strike(
    target_px: f32,
    strikes: &[ColorGlyphBitmapStrike],
) -> Option<ColorGlyphBitmapStrikeSelection> {
    let target_px = finite_positive_or_default(target_px, 1.0);
    let mut nearest_larger_or_equal = None;
    let mut largest_smaller = None;

    for strike in strikes.iter().copied().filter(valid_bitmap_strike) {
        if strike.ppem as f32 >= target_px {
            if nearest_larger_or_equal
                .map(|candidate: ColorGlyphBitmapStrike| strike.ppem < candidate.ppem)
                .unwrap_or(true)
            {
                nearest_larger_or_equal = Some(strike);
            }
        } else if largest_smaller
            .map(|candidate: ColorGlyphBitmapStrike| strike.ppem > candidate.ppem)
            .unwrap_or(true)
        {
            largest_smaller = Some(strike);
        }
    }

    let strike = nearest_larger_or_equal.or(largest_smaller)?;
    Some(ColorGlyphBitmapStrikeSelection {
        strike,
        target_px,
        scale: target_px / strike.ppem as f32,
        fit: strike_fit(strike.ppem, target_px),
    })
}

fn benchmark_strike(ppem: u16, bitmap_size: u32) -> ColorGlyphBitmapStrike {
    ColorGlyphBitmapStrike::new(
        ppem,
        UVec2::new(bitmap_size, bitmap_size),
        Vec2::new(1.0, bitmap_size as f32 - 1.0),
        bitmap_size as f32,
    )
}

#[test]
fn optimization_batch_es_exact_color_strike_preserves_first_match() {
    let strikes = [
        benchmark_strike(64, 64),
        benchmark_strike(96, 96),
        benchmark_strike(64, 63),
        benchmark_strike(32, 32),
    ];

    assert_eq!(
        select_color_bitmap_strike(64.0, &strikes),
        legacy_select_color_bitmap_strike(64.0, &strikes)
    );
    assert_eq!(
        select_color_bitmap_strike(f32::NAN, &[benchmark_strike(1, 1)]),
        legacy_select_color_bitmap_strike(f32::NAN, &[benchmark_strike(1, 1)])
    );
}

#[test]
#[ignore = "release-only exact color strike early-return benchmark"]
fn optimization_batch_es_exact_color_strike_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const SELECTIONS_PER_SAMPLE: usize = 2_048;
    const STRIKE_COUNT: usize = 1_024;

    fn measure(
        strikes: &[ColorGlyphBitmapStrike],
        select: fn(f32, &[ColorGlyphBitmapStrike]) -> Option<ColorGlyphBitmapStrikeSelection>,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..SELECTIONS_PER_SAMPLE {
            checksum = checksum.wrapping_add(
                select(black_box(64.0), black_box(strikes))
                    .expect("benchmark strike selection")
                    .strike
                    .ppem as u64,
            );
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let mut strikes = Vec::with_capacity(STRIKE_COUNT);
    strikes.push(benchmark_strike(64, 64));
    strikes.extend((1..STRIKE_COUNT).map(|index| {
        let ppem = 65 + (index % 1_024) as u16;
        benchmark_strike(ppem, ppem as u32)
    }));
    assert_eq!(
        select_color_bitmap_strike(64.0, &strikes),
        legacy_select_color_bitmap_strike(64.0, &strikes)
    );

    for _ in 0..4 {
        black_box(measure(&strikes, legacy_select_color_bitmap_strike));
        black_box(measure(&strikes, select_color_bitmap_strike));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(&strikes, legacy_select_color_bitmap_strike));
            optimized_samples.push(measure(&strikes, select_color_bitmap_strike));
        } else {
            optimized_samples.push(measure(&strikes, select_color_bitmap_strike));
            legacy_samples.push(measure(&strikes, legacy_select_color_bitmap_strike));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME451_EXACT_COLOR_STRIKE_EARLY_RETURN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             selections_per_sample={SELECTIONS_PER_SAMPLE} strike_count={STRIKE_COUNT} \
             exact_strike_index=0 pair_order=alternating_legacy_even \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(10),
        "exact color strike selection must reduce P95 by at least 90%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
