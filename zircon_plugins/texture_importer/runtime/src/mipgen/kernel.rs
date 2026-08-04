use zircon_runtime::core::framework::render::{
    RenderImageColorSpace, TextureMipFilter, TextureUsageHint,
};

use super::RGBA8_TEXEL_SIZE;

const KAISER_RADIUS: f32 = 2.0;
const KAISER_BETA: f32 = 4.0;

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
    // Normalize Kaiser weights once per generated level, outside the texel loop.
    let kaiser_normalizer = bessel_i0(KAISER_BETA);
    let mut target = vec![0; rgba8_level_len(target_width, target_height)?];
    for target_y in 0..target_height {
        for target_x in 0..target_width {
            let pixel = if usage_hint == TextureUsageHint::Normal {
                downsample_normal_pixel(source, source_width, source_height, target_x, target_y)
            } else {
                downsample_color_pixel(
                    source,
                    source_width,
                    source_height,
                    target_x,
                    target_y,
                    color_space,
                    mip_filter,
                    kaiser_normalizer,
                )
            };
            let offset = ((target_y * target_width + target_x) as usize) * RGBA8_TEXEL_SIZE;
            target[offset..offset + RGBA8_TEXEL_SIZE].copy_from_slice(&pixel);
        }
    }
    Some(target)
}

fn downsample_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
    mip_filter: TextureMipFilter,
    kaiser_normalizer: f32,
) -> [u8; RGBA8_TEXEL_SIZE] {
    match mip_filter {
        TextureMipFilter::Box => downsample_box_color_pixel(
            source,
            source_width,
            source_height,
            target_x,
            target_y,
            color_space,
            kaiser_normalizer,
        ),
        TextureMipFilter::Kaiser => downsample_kaiser_color_pixel(
            source,
            source_width,
            source_height,
            target_x,
            target_y,
            color_space,
        ),
    }
}

fn downsample_box_color_pixel(
    source: &[u8],
    source_width: u32,
    source_height: u32,
    target_x: u32,
    target_y: u32,
    color_space: RenderImageColorSpace,
    kaiser_normalizer: f32,
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
        let weight_y = kaiser_weight(source_y as f32 + 0.5 - center_y, kaiser_normalizer);
        for source_x in min_x..=max_x {
            let weight =
                weight_y * kaiser_weight(source_x as f32 + 0.5 - center_x, kaiser_normalizer);
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
    use super::*;

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
