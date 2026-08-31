use crate::core::framework::render::RenderImageDimension;

use super::super::TextureAsset;
use super::bytes::read_u24_le;
use super::layout::is_supported_astc_block;
use super::{TextureUploadCompressionFamily, TextureUploadPlan};

const ASTC_IDENTIFIER: &[u8] = b"\x13\xAB\xA1\x5C";
pub(super) fn astc_upload_plan(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
) -> Option<TextureUploadPlan> {
    if bytes.get(..ASTC_IDENTIFIER.len())? != ASTC_IDENTIFIER {
        return None;
    }
    let value = format.trim().to_ascii_lowercase();
    let dimensions = value.strip_prefix("astc/")?;
    let mut parts = dimensions.split('x');
    let block_width = parts.next()?.parse::<u32>().ok()?;
    let block_height = parts.next()?.parse::<u32>().ok()?;
    let block_depth = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some()
        || block_width == 0
        || block_height == 0
        || block_depth == 0
        || !is_supported_astc_block(block_width, block_height, block_depth)
    {
        return None;
    }
    if u32::from(bytes.get(4).copied()?) != block_width
        || u32::from(bytes.get(5).copied()?) != block_height
        || u32::from(bytes.get(6).copied()?) != block_depth
    {
        return None;
    }
    let header_width = read_u24_le(bytes, 7)?;
    let header_height = read_u24_le(bytes, 10)?;
    let header_depth = read_u24_le(bytes, 13)?;
    if block_depth == 1 && header_depth != 1 {
        return None;
    }
    let descriptor = texture.render_image_descriptor();
    let expected_depth = if descriptor.dimension == RenderImageDimension::D3 {
        descriptor.depth_or_array_layers.max(1)
    } else {
        1
    };
    if header_width != texture.width
        || header_height != texture.height
        || header_depth != expected_depth
    {
        return None;
    }
    Some(TextureUploadPlan {
        format: value,
        compression: TextureUploadCompressionFamily::Astc,
        data_offset: 16,
        data_length: None,
        block_width,
        block_height,
        block_depth,
        bytes_per_block: 16,
        subresources: Vec::new(),
    })
}

#[cfg(test)]
mod plugins07_astc_header_hotpath_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::asset::AssetUri;

    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 240_000;

    #[test]
    fn header_first_texture_probe_contract_astc() {
        let texture = fixture_texture();
        assert!(astc_upload_plan(&texture, " ASTC/4x4x1 ", &[0_u8; 16]).is_none());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn header_first_texture_probe_performance_release_astc() {
        let texture = fixture_texture();
        let bytes = [0_u8; 16];
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (
                    measure_legacy(&bytes),
                    measure_header_first(&texture, &bytes),
                )
            } else {
                let optimized_ns = measure_header_first(&texture, &bytes);
                (measure_legacy(&bytes), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_astc_header_first_rejection sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={LOOKUPS_PER_SAMPLE} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 50,
            "ASTC header-first rejection must improve P95 by at least 50%"
        );
    }

    fn measure_legacy(bytes: &[u8]) -> u128 {
        let started = Instant::now();
        let mut rejected = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            let value = black_box(" ASTC/4x4x1 ").trim().to_ascii_lowercase();
            let dimensions = value.strip_prefix("astc/").unwrap();
            let mut parts = dimensions.split('x');
            let parsed = parts
                .next()
                .and_then(|part| part.parse::<u32>().ok())
                .is_some()
                && parts
                    .next()
                    .and_then(|part| part.parse::<u32>().ok())
                    .is_some()
                && parts
                    .next()
                    .and_then(|part| part.parse::<u32>().ok())
                    .is_some()
                && parts.next().is_none();
            rejected += u64::from(
                parsed && black_box(bytes).get(..ASTC_IDENTIFIER.len()) != Some(ASTC_IDENTIFIER),
            );
            black_box(value);
        }
        black_box(rejected);
        started.elapsed().as_nanos()
    }

    fn measure_header_first(texture: &TextureAsset, bytes: &[u8]) -> u128 {
        let started = Instant::now();
        let mut rejected = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            rejected += u64::from(
                astc_upload_plan(
                    black_box(texture),
                    black_box(" ASTC/4x4x1 "),
                    black_box(bytes),
                )
                .is_none(),
            );
        }
        black_box(rejected);
        started.elapsed().as_nanos()
    }

    fn fixture_texture() -> TextureAsset {
        TextureAsset::new_container(
            AssetUri::parse("res://textures/header-probe.astc").unwrap(),
            4,
            4,
            "astc/4x4x1",
            Vec::new(),
            1,
            1,
        )
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
