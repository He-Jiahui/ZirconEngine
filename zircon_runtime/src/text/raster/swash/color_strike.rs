use crate::core::math::{UVec2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColorGlyphBitmapStrike {
    pub(crate) ppem: u16,
    pub(crate) bitmap_size: UVec2,
    pub(crate) bearing: Vec2,
    pub(crate) advance_px: f32,
}

impl ColorGlyphBitmapStrike {
    pub(crate) fn new(ppem: u16, bitmap_size: UVec2, bearing: Vec2, advance_px: f32) -> Self {
        Self {
            ppem,
            bitmap_size,
            bearing,
            advance_px,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorGlyphBitmapStrikeFit {
    Exact,
    Downsample,
    UpscaleFallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColorGlyphBitmapStrikeSelection {
    pub(crate) strike: ColorGlyphBitmapStrike,
    pub(crate) target_px: f32,
    pub(crate) scale: f32,
    pub(crate) fit: ColorGlyphBitmapStrikeFit,
}

impl ColorGlyphBitmapStrikeSelection {
    pub(crate) fn scaled_size(self) -> UVec2 {
        UVec2::new(
            scale_dimension(self.strike.bitmap_size.x, self.scale),
            scale_dimension(self.strike.bitmap_size.y, self.scale),
        )
    }

    pub(crate) fn scaled_bearing(self) -> Vec2 {
        Vec2::new(
            self.strike.bearing.x * self.scale,
            self.strike.bearing.y * self.scale,
        )
    }

    pub(crate) fn scaled_advance_px(self) -> f32 {
        self.strike.advance_px * self.scale
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ColorGlyphRasterPlan {
    ColrCpalVector,
    BitmapStrike(ColorGlyphBitmapStrikeSelection),
    Missing,
}

pub(crate) fn color_glyph_raster_plan(
    has_colr_cpal: bool,
    target_px: f32,
    bitmap_strikes: &[ColorGlyphBitmapStrike],
) -> ColorGlyphRasterPlan {
    if has_colr_cpal {
        return ColorGlyphRasterPlan::ColrCpalVector;
    }

    select_color_bitmap_strike(target_px, bitmap_strikes)
        .map(ColorGlyphRasterPlan::BitmapStrike)
        .unwrap_or(ColorGlyphRasterPlan::Missing)
}

pub(crate) fn select_color_bitmap_strike(
    target_px: f32,
    strikes: &[ColorGlyphBitmapStrike],
) -> Option<ColorGlyphBitmapStrikeSelection> {
    let target_px = finite_positive_or_default(target_px, 1.0);
    let mut nearest_larger_or_equal = None;
    let mut largest_smaller = None;

    for strike in strikes.iter().copied().filter(valid_bitmap_strike) {
        let fit = strike_fit(strike.ppem, target_px);
        if fit == ColorGlyphBitmapStrikeFit::Exact {
            return Some(ColorGlyphBitmapStrikeSelection {
                strike,
                target_px,
                scale: target_px / strike.ppem as f32,
                fit,
            });
        }

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
    let scale = target_px / strike.ppem as f32;
    Some(ColorGlyphBitmapStrikeSelection {
        strike,
        target_px,
        scale,
        fit: strike_fit(strike.ppem, target_px),
    })
}

fn valid_bitmap_strike(strike: &ColorGlyphBitmapStrike) -> bool {
    strike.ppem > 0
        && strike.bitmap_size.x > 0
        && strike.bitmap_size.y > 0
        && strike.bearing.x.is_finite()
        && strike.bearing.y.is_finite()
        && strike.advance_px.is_finite()
        && strike.advance_px >= 0.0
}

fn strike_fit(ppem: u16, target_px: f32) -> ColorGlyphBitmapStrikeFit {
    let ppem = ppem as f32;
    if (ppem - target_px).abs() <= f32::EPSILON {
        ColorGlyphBitmapStrikeFit::Exact
    } else if ppem > target_px {
        ColorGlyphBitmapStrikeFit::Downsample
    } else {
        ColorGlyphBitmapStrikeFit::UpscaleFallback
    }
}

fn scale_dimension(value: u32, scale: f32) -> u32 {
    ((value as f32 * scale).round()).max(1.0) as u32
}

fn finite_positive_or_default(value: f32, default_value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default_value
    }
}

#[cfg(test)]
mod optimization_batch_es_tests {
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
}
