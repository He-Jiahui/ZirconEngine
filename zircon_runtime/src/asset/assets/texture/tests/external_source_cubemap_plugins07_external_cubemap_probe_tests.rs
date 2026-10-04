use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 240_000;

#[test]
fn borrowed_texture_metadata_contract_external_cubemap_probe() {
    assert_eq!(
        external_cubemap_format_kind(" DDS/DXT1 "),
        Some(ExternalSourceCubemapContainerKind::Dds)
    );
    assert_eq!(
        external_cubemap_format_kind(" KtX2/VK-133 "),
        Some(ExternalSourceCubemapContainerKind::Ktx2)
    );
    assert_eq!(external_cubemap_format_kind("png"), None);
    assert_eq!(
        external_source_cubemap_container_info(&fixture_texture()),
        Ok(None)
    );
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_metadata_performance_release_external_cubemap_probe() {
    let texture = fixture_texture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(&texture), measure_borrowed(&texture))
        } else {
            let optimized_ns = measure_borrowed(&texture);
            (measure_legacy(&texture), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_external_cubemap_borrowed_probe sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50 legacy_allocations_per_sample={LOOKUPS_PER_SAMPLE} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= 50,
        "borrowed external cubemap probe must improve P95 by at least 50%"
    );
}

fn measure_legacy(texture: &TextureAsset) -> u128 {
    let started = Instant::now();
    let mut rejected = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let TexturePayload::Container { format, .. } = &black_box(texture).payload else {
            unreachable!();
        };
        let normalized = format.trim().to_ascii_lowercase();
        rejected += u64::from(
            !normalized.starts_with("dds/")
                && !normalized.starts_with("ktx2/")
                && !normalized.starts_with("ktx/"),
        );
        black_box(normalized);
    }
    black_box(rejected);
    started.elapsed().as_nanos()
}

fn measure_borrowed(texture: &TextureAsset) -> u128 {
    let started = Instant::now();
    let mut rejected = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        rejected += u64::from(
            external_source_cubemap_container_info(black_box(texture))
                .unwrap()
                .is_none(),
        );
    }
    black_box(rejected);
    started.elapsed().as_nanos()
}

fn fixture_texture() -> TextureAsset {
    TextureAsset::new_container(
        AssetUri::parse("res://textures/albedo.png").unwrap(),
        4,
        4,
        " PNG ",
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
