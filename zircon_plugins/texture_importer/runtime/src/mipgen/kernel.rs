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
    let srgb_decode_lut =
        if usage_hint != TextureUsageHint::Normal && color_space == RenderImageColorSpace::Srgb {
            Some(build_srgb_decode_lut())
        } else {
            None
        };
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
                        srgb_decode_lut.as_ref(),
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
                            srgb_decode_lut.as_ref(),
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

fn build_srgb_decode_lut() -> [f32; 256] {
    std::array::from_fn(|value| srgb_to_linear(value as f32 / 255.0))
}

fn decode_color_byte(value: u8, srgb_decode_lut: Option<&[f32; 256]>) -> f32 {
    if let Some(lut) = srgb_decode_lut {
        lut[value as usize]
    } else {
        f32::from(value) / 255.0
    }
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
    srgb_decode_lut: Option<&[f32; 256]>,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let source_x = target_x * 2;
    let source_y = target_y * 2;
    if source_x + 1 < source_width && source_y + 1 < source_height {
        let row_stride = source_width as usize * RGBA8_TEXEL_SIZE;
        let top_left =
            (source_y as usize * source_width as usize + source_x as usize) * RGBA8_TEXEL_SIZE;
        for offset in [
            top_left,
            top_left + RGBA8_TEXEL_SIZE,
            top_left + row_stride,
            top_left + row_stride + RGBA8_TEXEL_SIZE,
        ] {
            for channel in 0..3 {
                sums[channel] += decode_color_byte(source[offset + channel], srgb_decode_lut);
            }
            sums[3] += f32::from(source[offset + 3]) / 255.0;
        }
        return encode_weighted_pixel(sums, 4.0, color_space);
    }

    let mut sample_count = 0.0;
    for source_y in source_y..((source_y + 2).min(source_height)) {
        for source_x in source_x..((source_x + 2).min(source_width)) {
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                sums[channel] += decode_color_byte(source[offset + channel], srgb_decode_lut);
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
    srgb_decode_lut: Option<&[f32; 256]>,
) -> [u8; RGBA8_TEXEL_SIZE] {
    let mut sums = [0.0; RGBA8_TEXEL_SIZE];
    let mut weight_sum = 0.0;

    for (source_y, weight_y) in y_weights.iter() {
        for (source_x, weight_x) in x_weights.iter() {
            let weight = weight_y * weight_x;
            let offset = ((source_y * source_width + source_x) as usize) * RGBA8_TEXEL_SIZE;
            for channel in 0..3 {
                sums[channel] +=
                    weight * decode_color_byte(source[offset + channel], srgb_decode_lut);
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
            srgb_decode_lut,
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
#[path = "tests/kernel.rs"]
mod tests;

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
