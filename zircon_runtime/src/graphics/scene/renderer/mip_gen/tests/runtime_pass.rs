use super::*;

#[test]
fn runtime_mipgen_shader_writes_up_to_four_storage_mips() {
    assert_eq!(std::mem::size_of::<MipGenParams>(), 32);
    assert!(MIP_GEN_SHADER.contains("@compute @workgroup_size(8, 8, 1)"));
    assert!(MIP_GEN_SHADER.contains("var target_mip_four"));
    assert!(MIP_GEN_SHADER.contains("generated_mip_count >= 4u"));
    assert!(MIP_GEN_SHADER.contains("var<workgroup> level_one"));
    assert!(MIP_GEN_SHADER.contains("let level_one_extent = min("));
    assert!(MIP_GEN_SHADER.contains("let level_two_extent = min("));
    assert!(MIP_GEN_SHADER.contains("let level_three_extent = min("));
}

#[test]
fn runtime_mipgen_color_mode_tracks_texture_metadata() {
    let mut metadata = TextureMetadata::default();
    metadata.color_space = RenderImageColorSpace::Srgb;
    metadata.usage_hint = TextureUsageHint::Albedo;
    assert_eq!(
        MipGenColorMode::from_metadata(&metadata),
        MipGenColorMode {
            srgb: true,
            normal: false
        }
    );

    metadata.color_space = RenderImageColorSpace::Linear;
    metadata.usage_hint = TextureUsageHint::Normal;
    assert_eq!(
        MipGenColorMode::from_metadata(&metadata),
        MipGenColorMode {
            srgb: false,
            normal: true
        }
    );
}

#[test]
fn runtime_mipgen_target_views_use_fixed_dispatch_storage() {
    let source = include_str!("../runtime_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime mipgen pass implementation");

    assert!(
        implementation.contains("[Option<wgpu::TextureView>; MIP_GEN_MIPS_PER_DISPATCH as usize]")
    );
    assert!(implementation.contains("std::array::from_fn"));
    assert!(implementation.contains("fallback_storage_views[index]"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cr_runtime_mipgen_target_storage_p95() {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const VIEWS_PER_SAMPLE: usize = 4;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(VIEWS_PER_SAMPLE, false));
            optimized.push(measure(VIEWS_PER_SAMPLE, true));
        } else {
            optimized.push(measure(VIEWS_PER_SAMPLE, true));
            legacy.push(measure(VIEWS_PER_SAMPLE, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME393_MIPGEN_TARGET_STORAGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} views_per_sample={VIEWS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));

    fn measure(count: usize, use_fixed_storage: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..100_000 {
            if use_fixed_storage {
                let values =
                    std::array::from_fn::<_, 4, _>(|index| (index < count).then_some(index as u8));
                checksum ^= values.iter().flatten().count();
                std::hint::black_box(values);
            } else {
                let values = (0..count).map(|index| index as u8).collect::<Vec<_>>();
                checksum ^= values.len();
                std::hint::black_box(values);
            }
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
