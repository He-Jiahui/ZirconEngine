use zircon_runtime::core::framework::render::{
    RenderImageColorSpace, TextureMipFilter, TextureUsageHint,
};

use super::RGBA8_TEXEL_SIZE;

const KAISER_RADIUS: f32 = 2.0;
const KAISER_BETA: f32 = 4.0;
const MAX_KAISER_AXIS_SAMPLES: usize = 5;

#[derive(Clone, Copy, Debug)]
struct KaiserAxisWeights {
    samples: [(u32, f32); MAX_KAISER_AXIS_SAMPLES],
    len: usize,
}

impl KaiserAxisWeights {
    fn iter(&self) -> impl Iterator<Item = (u32, f32)> + '_ {
        self.samples[..self.len].iter().copied()
    }
}

pub(super) fn downsample_rgba8(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    color_space: RenderImageColorSpace,
    usage_hint: TextureUsageHint,
    mip_filter: TextureMipFilter,
) -> Option<Vec<u8>> {
    let target_width = (source_width / 2).max(1);
    let target_height = (source_height / 2).max(1);
    let kaiser_axis_weights =
        if usage_hint != TextureUsageHint::Normal && mip_filter == TextureMipFilter::Kaiser {
            // Normalize and cache separable Kaiser weights once per generated level.
            let kaiser_normalizer = bessel_i0(KAISER_BETA);
            Some((
                build_kaiser_axis_weights(target_width, source_width, kaiser_normalizer),
                build_kaiser_axis_weights(target_height, source_height, kaiser_normalizer),
            ))
        } else {
            None
        };
    let mut target = vec![0; rgba8_level_len(target_width, target_height)?];
    for target_y in 0..target_height {
        for target_x in 0..target_width {
            let pixel = if usage_hint == TextureUsageHint::Normal {
                downsample_normal_pixel(source, source_width, source_height, target_x, target_y)
            } else {
                match mip_filter {
                    TextureMipFilter::Box => downsample_box_color_pixel(
                        source,
                        source_width,
                        source_height,
                        target_x,
                        target_y,
                        color_space,
                    ),
                    TextureMipFilter::Kaiser => {
                        let (x_weights, y_weights) = kaiser_axis_weights
                            .as_ref()
                            .expect("Kaiser color mip weights are prepared once per level");
                        downsample_kaiser_color_pixel(
                            source,
                            source_width,
                            source_height,
                            target_x,
                            target_y,
                            color_space,
                            &x_weights[target_x as usize],
                            &y_weights[target_y as usize],
                        )
                    }
                }
            };
            let offset = ((target_y * target_width + target_x) as usize) * RGBA8_TEXEL_SIZE;
            target[offset..offset + RGBA8_TEXEL_SIZE].copy_from_slice(&pixel);
        }
    }
    Some(target)
}

fn build_kaiser_axis_weights(
    target_extent: u32,
    source_extent: u32,
    normalizer: f32,
) -> Vec<KaiserAxisWeights> {
    (0..target_extent)
        .map(|target| {
            let center = target as f32 * 2.0 + 1.0;
            let min = (center - KAISER_RADIUS).ceil().max(0.0) as u32;
            let max = (center + KAISER_RADIUS)
                .floor()
                .min((source_extent - 1) as f32) as u32;
            let mut weights = KaiserAxisWeights {
                samples: [(0, 0.0); MAX_KAISER_AXIS_SAMPLES],
                len: 0,
            };
            for source in min..=max {
                debug_assert!(weights.len < MAX_KAISER_AXIS_SAMPLES);
                weights.samples[weights.len] = (
                    source,
                    kaiser_weight(source as f32 + 0.5 - center, normalizer),
                );
                weights.len += 1;
            }
            weights
        })
        .collect()
}

fn encode_weighted_pixel(
    sums: [f32; RGBA8_TEXEL_SIZE],
    weight_sum: f32,
    color_space: RenderImageColorSpace,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut pixel = [0; RGBA8_TEXEL_SIZE];
    for channel in 0..3 {
        let average = sums[channel] / weight_sum;
        let encoded = if color_space == RenderImageColorSpace::Srgb {
            linear_to_srgb(average)
        } else {
            average
        };
        pixel[channel] = encode_unorm8(encoded);
    }
    pixel[3] = encode_unorm8(sums[3] / weight_sum);
    pixel
}

fn downsample_box_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let mut sample_count = 0.0;
    for source_y in target_y * 2..((target_y * 2 + 2).min(source_height)) {
        for source_x in target_x * 2..((target_x * 2 + 2).min(source_width)) {
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                let value = f32::from(source[offset + channel]) / 255.0;
                sums[channel] += if color_space == RenderImageColorSpace::Srgb {
                    srgb_to_linear(value)
                } else {
                    value
                };
            }
            sums[3] += f32::from(source[offset + 3]) / 255.0;
            sample_count += 1.0;
        }
    }

    let mut pixel = [0; RGBA8_TEXEL_SIZE];
    for channel in 0..3 {
        let average = sums[channel] / sample_count;
        let encoded = if color_space == RenderImageColorSpace::Srgb {
            linear_to_srgb(average)
        } else {
            average
        };
        pixel[channel] = encode_unorm8(encoded);
    }
    pixel[3] = encode_unorm8(sums[3] / sample_count);
    pixel
}

fn downsample_kaiser_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
    x_weights: &KaiserAxisWeights,
    y_weights: &KaiserAxisWeights,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let mut weight_sum = 0.0;

    for (source_y, weight_y) in y_weights.iter() {
        for (source_x, weight_x) in x_weights.iter() {
            let weight = weight_y * weight_x;
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                let value = f32::from(source[offset + channel]) / 255.0;
                sums[channel] += weight
                    * if color_space == RenderImageColorSpace::Srgb {
                        srgb_to_linear(value)
                    } else {
                        value
                    };
            }
            sums[3] += weight * f32::from(source[offset + 3]) / 255.0;
            weight_sum += weight;
        }
    }
    if weight_sum <= f32::EPSILON {
        return downsample_box_color_pixel(
            source,
            source_width,
            source_height,
            target_x,
            target_y,
            color_space,
        );
    }
    encode_weighted_pixel(sums, weight_sum, color_space)
}

fn kaiser_weight(distance: f32, normalizer: f32) -> f32 {
    let normalized = distance.abs() / KAISER_RADIUS;
    if normalized >= 1.0 {
        return 0.0;
    }
    let window = bessel_i0(KAISER_BETA * (1.0 - normalized * normalized).sqrt()) / normalizer;
    let phase = distance * 0.5;
    let sinc = if phase.abs() <= f32::EPSILON {
        1.0
    } else {
        (std::f32::consts::PI * phase).sin() / (std::f32::consts::PI * phase)
    };
    sinc * window
}

fn bessel_i0(value: f32) -> f32 {
    let mut term = 1.0;
    let mut sum = 1.0;
    for index in 1..=10 {
        let index = index as f32;
        term *= value * value / (4.0 * index * index);
        sum += term;
    }
    sum
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const KAISER_BENCH_SOURCE_EXTENT: u32 = 256;
    const KAISER_BENCH_SAMPLE_PAIRS: usize = 21;

    #[test]
    fn kaiser_filter_reaches_beyond_the_box_footprint() {
        let mut source = Vec::with_capacity(4 * 2 * RGBA8_TEXEL_SIZE);
        for red in [0_u8, 0, 255, 255, 0, 0, 255, 255] {
            source.extend([red, red, red, 255]);
        }

        let box_mip = downsample_rgba8(
            &source,
            4,
            2,
            RenderImageColorSpace::Linear,
            TextureUsageHint::Albedo,
            TextureMipFilter::Box,
        )
        .expect("box mip should fit in memory");
        let kaiser_mip = downsample_rgba8(
            &source,
            4,
            2,
            RenderImageColorSpace::Linear,
            TextureUsageHint::Albedo,
            TextureMipFilter::Kaiser,
        )
        .expect("kaiser mip should fit in memory");

        assert_eq!(box_mip.len(), 8);
        assert_eq!(kaiser_mip.len(), 8);
        assert!(kaiser_mip[0] > box_mip[0]);
        assert!(kaiser_mip[4] < box_mip[4]);
        assert_eq!(kaiser_mip[3], 255);
        assert_eq!(kaiser_mip[7], 255);
    }

    #[test]
    fn cached_kaiser_axis_weights_match_inline_reference_for_odd_extent() {
        let source = patterned_rgba8(7, 5);

        let inline = inline_kaiser_downsample(&source, 7, 5, RenderImageColorSpace::Srgb);
        let cached = downsample_rgba8(
            &source,
            7,
            5,
            RenderImageColorSpace::Srgb,
            TextureUsageHint::Albedo,
            TextureMipFilter::Kaiser,
        )
        .expect("odd cached Kaiser mip should fit in memory");

        assert_eq!(cached, inline);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn cached_kaiser_axis_weights_release_benchmark() {
        let source = patterned_rgba8(KAISER_BENCH_SOURCE_EXTENT, KAISER_BENCH_SOURCE_EXTENT);
        let inline = inline_kaiser_downsample(
            &source,
            KAISER_BENCH_SOURCE_EXTENT,
            KAISER_BENCH_SOURCE_EXTENT,
            RenderImageColorSpace::Linear,
        );
        let cached = downsample_rgba8(
            &source,
            KAISER_BENCH_SOURCE_EXTENT,
            KAISER_BENCH_SOURCE_EXTENT,
            RenderImageColorSpace::Linear,
            TextureUsageHint::Albedo,
            TextureMipFilter::Kaiser,
        )
        .expect("cached Kaiser benchmark mip should fit in memory");
        assert_eq!(cached, inline);

        let mut legacy_samples = Vec::with_capacity(KAISER_BENCH_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(KAISER_BENCH_SAMPLE_PAIRS);
        for pair_index in 0..KAISER_BENCH_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_inline_kaiser(&source));
                optimized_samples.push(measure_cached_kaiser(&source));
            } else {
                optimized_samples.push(measure_cached_kaiser(&source));
                legacy_samples.push(measure_inline_kaiser(&source));
            }
        }

        let target_extent = KAISER_BENCH_SOURCE_EXTENT / 2;
        let axis_weight_evaluations =
            axis_weight_evaluation_counts(KAISER_BENCH_SOURCE_EXTENT, KAISER_BENCH_SOURCE_EXTENT);
        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        println!(
            "TEXTURE_KAISER_AXIS_CACHE_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank source_width={} source_height={} target_pixels={} legacy_kaiser_weight_evaluations={} optimized_kaiser_weight_evaluations={} legacy_normalizer_evaluations=1 optimized_normalizer_evaluations=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
            KAISER_BENCH_SAMPLE_PAIRS,
            KAISER_BENCH_SOURCE_EXTENT,
            KAISER_BENCH_SOURCE_EXTENT,
            target_extent as usize * target_extent as usize,
            axis_weight_evaluations.0,
            axis_weight_evaluations.1,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
            join_samples(&legacy_samples),
            join_samples(&optimized_samples),
        );
        assert_eq!(axis_weight_evaluations, (487_305, 1_274));
        assert!(
            optimized_p95.saturating_mul(4) <= legacy_p95,
            "cached Kaiser P95 must be at most 25% of inline weighting: legacy={legacy_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn measure_inline_kaiser(source: &[u8]) -> u128 {
        let started = Instant::now();
        black_box(inline_kaiser_downsample(
            black_box(source),
            KAISER_BENCH_SOURCE_EXTENT,
            KAISER_BENCH_SOURCE_EXTENT,
            RenderImageColorSpace::Linear,
        ));
        started.elapsed().as_nanos()
    }

    fn measure_cached_kaiser(source: &[u8]) -> u128 {
        let started = Instant::now();
        black_box(
            downsample_rgba8(
                black_box(source),
                KAISER_BENCH_SOURCE_EXTENT,
                KAISER_BENCH_SOURCE_EXTENT,
                RenderImageColorSpace::Linear,
                TextureUsageHint::Albedo,
                TextureMipFilter::Kaiser,
            )
            .expect("cached Kaiser benchmark mip should fit in memory"),
        );
        started.elapsed().as_nanos()
    }

    fn inline_kaiser_downsample(
        source: &[u8],
        source_width: u32,
        source_height: u32,
        color_space: RenderImageColorSpace,
    ) -> Vec<u8> {
        let target_width = (source_width / 2).max(1);
        let target_height = (source_height / 2).max(1);
        let normalizer = bessel_i0(KAISER_BETA);
        let mut target = vec![0; rgba8_level_len(target_width, target_height).unwrap()];
        for target_y in 0..target_height {
            for target_x in 0..target_width {
                let pixel = inline_kaiser_color_pixel(
                    source,
                    source_width,
                    source_height,
                    target_x,
                    target_y,
                    color_space,
                    normalizer,
                );
                let offset = ((target_y * target_width + target_x) as usize) * RGBA8_TEXEL_SIZE;
                target[offset..offset + RGBA8_TEXEL_SIZE].copy_from_slice(&pixel);
            }
        }
        target
    }

    fn inline_kaiser_color_pixel(
        source: &[u8],
        source_width: u32,
        source_height: u32,
        target_x: u32,
        target_y: u32,
        color_space: RenderImageColorSpace,
        normalizer: f32,
    ) -> [u8; RGBA8_TEXEL_SIZE] {
        let center_x = target_x as f32 * 2.0 + 1.0;
        let center_y = target_y as f32 * 2.0 + 1.0;
        let min_x = (center_x - KAISER_RADIUS).ceil().max(0.0) as u32;
        let max_x = (center_x + KAISER_RADIUS)
            .floor()
            .min((source_width - 1) as f32) as u32;
        let min_y = (center_y - KAISER_RADIUS).ceil().max(0.0) as u32;
        let max_y = (center_y + KAISER_RADIUS)
            .floor()
            .min((source_height - 1) as f32) as u32;
        let mut sums = [0.0; RGBA8_TEXEL_SIZE];
        let mut weight_sum = 0.0;
        for source_y in min_y..=max_y {
            let weight_y = kaiser_weight(source_y as f32 + 0.5 - center_y, normalizer);
            for source_x in min_x..=max_x {
                let weight = weight_y * kaiser_weight(source_x as f32 + 0.5 - center_x, normalizer);
                let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
                for channel in 0..3 {
                    let value = f32::from(source[offset + channel]) / 255.0;
                    sums[channel] += weight
                        * if color_space == RenderImageColorSpace::Srgb {
                            srgb_to_linear(value)
                        } else {
                            value
                        };
                }
                sums[3] += weight * f32::from(source[offset + 3]) / 255.0;
                weight_sum += weight;
            }
        }
        if weight_sum <= f32::EPSILON {
            return downsample_box_color_pixel(
                source,
                source_width,
                source_height,
                target_x,
                target_y,
                color_space,
            );
        }
        encode_weighted_pixel(sums, weight_sum, color_space)
    }

    fn axis_weight_evaluation_counts(source_width: u32, source_height: u32) -> (usize, usize) {
        let target_width = (source_width / 2).max(1);
        let target_height = (source_height / 2).max(1);
        let x_samples = (0..target_width)
            .map(|target| axis_sample_count(target, source_width))
            .sum::<usize>();
        let y_samples = (0..target_height)
            .map(|target| axis_sample_count(target, source_height))
            .sum::<usize>();
        let inline = y_samples * target_width as usize + y_samples * x_samples;
        (inline, x_samples + y_samples)
    }

    fn axis_sample_count(target: u32, source_extent: u32) -> usize {
        let center = target as f32 * 2.0 + 1.0;
        let min = (center - KAISER_RADIUS).ceil().max(0.0) as u32;
        let max = (center + KAISER_RADIUS)
            .floor()
            .min((source_extent - 1) as f32) as u32;
        (max - min + 1) as usize
    }

    fn patterned_rgba8(width: u32, height: u32) -> Vec<u8> {
        (0..width * height)
            .flat_map(|index| {
                [
                    index as u8,
                    index.wrapping_mul(3) as u8,
                    index.wrapping_mul(7) as u8,
                    255,
                ]
            })
            .collect()
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
}

fn downsample_normal_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut normal = [0.0; 3];
    let mut alpha = 0.0;
    let mut sample_count = 0.0;
    for source_y in target_y * 2..((target_y * 2 + 2).min(source_height)) {
        for source_x in target_x * 2..((target_x * 2 + 2).min(source_width)) {
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                normal[channel] += f32::from(source[offset + channel]) / 127.5 - 1.0;
            }
            alpha += f32::from(source[offset + 3]) / 255.0;
            sample_count += 1.0;
        }
    }
    let length = normal
        .iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt();
    let normal = if length > f32::EPSILON {
        normal.map(|component| component / length)
    } else {
        [0.0, 0.0, 1.0]
    };

    [
        encode_unorm8(normal[0] * 0.5 + 0.5),
        encode_unorm8(normal[1] * 0.5 + 0.5),
        encode_unorm8(normal[2] * 0.5 + 0.5),
        encode_unorm8(alpha / sample_count),
    ]
}

fn rgba8_level_len(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(RGBA8_TEXEL_SIZE)
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn encode_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
