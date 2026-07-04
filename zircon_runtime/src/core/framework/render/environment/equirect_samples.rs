use crate::core::math::Real;

pub const SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH: usize = 128;
pub const SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT: usize = 64;
pub const SAMPLED_EQUIRECT_ENVIRONMENT_WIDTH: usize = SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH;
pub const SAMPLED_EQUIRECT_ENVIRONMENT_HEIGHT: usize = SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT;
pub const SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT: usize = 8;
pub const SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT: usize = 10_923;

pub type SampledEquirectangularSamples = [[Real; 4]; SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT];

pub static EMPTY_SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLES: SampledEquirectangularSamples =
    [[0.0; 4]; SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT];

pub fn sampled_equirect_mip_dimensions(mip_level: usize) -> (usize, usize) {
    let clamped_mip = mip_level.min(SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT.saturating_sub(1));
    let mut width = SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH;
    let mut height = SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT;
    for _ in 0..clamped_mip {
        width = (width / 2).max(1);
        height = (height / 2).max(1);
    }
    (width, height)
}

pub fn sampled_equirect_mip_offset(mip_level: usize) -> usize {
    let clamped_mip = mip_level.min(SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT.saturating_sub(1));
    let mut offset = 0;
    for mip in 0..clamped_mip {
        let (width, height) = sampled_equirect_mip_dimensions(mip);
        offset += width * height;
    }
    offset
}

pub fn build_sampled_equirect_mip_chain<F>(mut sample_base: F) -> SampledEquirectangularSamples
where
    F: FnMut(usize, usize) -> [Real; 4],
{
    let mut samples = [[0.0; 4]; SAMPLED_EQUIRECT_ENVIRONMENT_SAMPLE_COUNT];
    for y in 0..SAMPLED_EQUIRECT_ENVIRONMENT_BASE_HEIGHT {
        for x in 0..SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH {
            samples[y * SAMPLED_EQUIRECT_ENVIRONMENT_BASE_WIDTH + x] = sample_base(x, y);
        }
    }

    for mip in 1..SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT {
        let source_mip = mip - 1;
        let source_offset = sampled_equirect_mip_offset(source_mip);
        let dest_offset = sampled_equirect_mip_offset(mip);
        let (source_width, source_height) = sampled_equirect_mip_dimensions(source_mip);
        let (dest_width, dest_height) = sampled_equirect_mip_dimensions(mip);

        for y in 0..dest_height {
            for x in 0..dest_width {
                let source_x0 = (x * 2).min(source_width - 1);
                let source_x1 = (source_x0 + 1) % source_width;
                let source_y0 = (y * 2).min(source_height - 1);
                let source_y1 = (source_y0 + 1).min(source_height - 1);
                let taps = [
                    samples[source_offset + source_y0 * source_width + source_x0],
                    samples[source_offset + source_y0 * source_width + source_x1],
                    samples[source_offset + source_y1 * source_width + source_x0],
                    samples[source_offset + source_y1 * source_width + source_x1],
                ];
                let mut averaged = [0.0; 4];
                for tap in taps {
                    averaged[0] += tap[0] * 0.25;
                    averaged[1] += tap[1] * 0.25;
                    averaged[2] += tap[2] * 0.25;
                    averaged[3] += tap[3] * 0.25;
                }
                samples[dest_offset + y * dest_width + x] = averaged;
            }
        }
    }

    samples
}

pub fn reflection_capture_mip_from_roughness(roughness: Real) -> Real {
    let max_mip = SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT as Real - 1.0;
    let level_from_one_by_one = 1.0 - 1.2 * roughness.max(0.001).log2();
    (max_mip - 1.0 - level_from_one_by_one).clamp(0.0, max_mip)
}

pub fn reflection_capture_roughness_from_mip(mip: Real) -> Real {
    let max_mip = SAMPLED_EQUIRECT_ENVIRONMENT_MIP_COUNT as Real - 1.0;
    let level_from_one_by_one = max_mip - 1.0 - mip;
    2.0_f32.powf((1.0 - level_from_one_by_one) / 1.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_equirect_mip_chain_has_expected_layout() {
        assert_eq!(sampled_equirect_mip_dimensions(0), (128, 64));
        assert_eq!(sampled_equirect_mip_dimensions(1), (64, 32));
        assert_eq!(sampled_equirect_mip_dimensions(6), (2, 1));
        assert_eq!(sampled_equirect_mip_dimensions(7), (1, 1));
        assert_eq!(sampled_equirect_mip_offset(0), 0);
        assert_eq!(sampled_equirect_mip_offset(1), 8_192);
        assert_eq!(sampled_equirect_mip_offset(7), 10_922);
    }

    #[test]
    fn sampled_equirect_mip_chain_downsamples_with_wrapped_x_and_clamped_y() {
        let samples = build_sampled_equirect_mip_chain(|x, y| [x as Real, y as Real, 0.0, 1.0]);
        let mip1 = sampled_equirect_mip_offset(1);

        assert_eq!(samples[mip1], [0.5, 0.5, 0.0, 1.0]);
        assert_eq!(samples[mip1 + 63], [126.5, 0.5, 0.0, 1.0]);
    }

    #[test]
    fn render_env_mip_from_roughness_roundtrip() {
        for roughness in [0.04, 0.1, 0.25, 0.5, 0.75, 1.0] {
            let mip = reflection_capture_mip_from_roughness(roughness);
            let roundtrip = reflection_capture_roughness_from_mip(mip);
            assert!(
                (roughness - roundtrip).abs() <= 0.0001 || mip <= 0.0001,
                "roughness {roughness} mip {mip} roundtrip {roundtrip}"
            );
        }
    }
}
