use zircon_runtime::asset::{
    normalize_texture_normal_map_convention, AssetImportError, TextureAsset,
};

/// Converts decoded normal maps into the engine-wide right-handed tangent-space GL convention.
pub(crate) fn normalize_normal_map_convention(
    texture: TextureAsset,
) -> Result<TextureAsset, AssetImportError> {
    normalize_texture_normal_map_convention(texture)
        .map_err(|error| AssetImportError::Parse(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;
    use zircon_runtime::asset::{AssetUri, TexturePayload};
    use zircon_runtime::core::framework::render::{
        TextureMetadata, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
    };

    const BENCHMARK_TEXTURES: usize = 8_192;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_THRESHOLD_PERCENT: u128 = 30;

    fn texture_fixture(
        usage_hint: TextureUsageHint,
        normal_convention: TextureNormalConvention,
    ) -> TextureAsset {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/normal-convention.png").unwrap(),
            1,
            1,
            vec![128, 64, 255, 255],
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint,
            normal_convention,
            mip_policy: TextureMipPolicy::GenerateOffline,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);
        texture
    }

    fn legacy_normalize_normal_map_convention(
        mut texture: TextureAsset,
    ) -> Result<TextureAsset, AssetImportError> {
        let mut descriptor = texture.texture_descriptor();
        if descriptor.metadata.usage_hint != TextureUsageHint::Normal {
            return Ok(texture);
        }
        match descriptor.metadata.normal_convention {
            TextureNormalConvention::None | TextureNormalConvention::TangentSpaceGl => {
                descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
            }
            TextureNormalConvention::TangentSpaceDx => {
                if !matches!(&texture.payload, TexturePayload::Rgba8) {
                    return Err(AssetImportError::Parse(format!(
                        "normal convention conversion requires a decoded rgba8 payload for {}",
                        texture.uri
                    )));
                }
                for texel in texture.rgba.chunks_exact_mut(4) {
                    texel[1] = u8::MAX - texel[1];
                }
                descriptor.metadata.normal_convention = TextureNormalConvention::TangentSpaceGl;
            }
        }
        texture.descriptor = Some(descriptor);
        Ok(texture)
    }

    fn measure_normalization(
        fixture: &TextureAsset,
        mut normalize: impl FnMut(TextureAsset) -> Result<TextureAsset, AssetImportError>,
    ) -> u128 {
        let inputs = vec![fixture.clone(); BENCHMARK_TEXTURES];
        let timer = Instant::now();
        let mut checksum = 0_u64;
        for texture in inputs {
            checksum += u64::from(black_box(normalize(texture).unwrap()).rgba[1]);
        }
        black_box(checksum);
        timer.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 - 1) / 100]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn run_descriptor_ownership_benchmark(marker: &str, fixture: &TextureAsset) {
        assert_eq!(
            legacy_normalize_normal_map_convention(fixture.clone()).unwrap(),
            normalize_normal_map_convention(fixture.clone()).unwrap()
        );
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_normalization(
                    black_box(fixture),
                    legacy_normalize_normal_map_convention,
                ));
                optimized_samples.push(measure_normalization(
                    black_box(fixture),
                    normalize_normal_map_convention,
                ));
            } else {
                optimized_samples.push(measure_normalization(
                    black_box(fixture),
                    normalize_normal_map_convention,
                ));
                legacy_samples.push(measure_normalization(
                    black_box(fixture),
                    legacy_normalize_normal_map_convention,
                ));
            }
        }

        let legacy_raw = legacy_samples.clone();
        let optimized_raw = optimized_samples.clone();
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);

        println!(
            "PERF_RESULT {marker} textures_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_descriptor_clones_per_sample={} optimized_descriptor_clones_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
            BENCHMARK_TEXTURES,
            BENCHMARK_SAMPLE_PAIRS,
            BENCHMARK_TEXTURES,
            legacy_p95_ns,
            optimized_p95_ns,
            improvement_percent,
            BENCHMARK_THRESHOLD_PERCENT,
            sample_csv(&legacy_raw),
            sample_csv(&optimized_raw),
        );

        assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
        assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
        assert!(
            improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
            "{marker} P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
        );
    }

    #[test]
    fn non_normal_payload_returns_without_descriptor_mutation() {
        let texture = texture_fixture(TextureUsageHint::Albedo, TextureNormalConvention::None);

        let normalized = normalize_normal_map_convention(texture.clone()).unwrap();

        assert_eq!(normalized, texture);
    }

    #[test]
    fn normal_dx_payload_is_converted_to_gl_before_mip_generation() {
        let mut texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/gl-normal.png").expect("valid texture uri"),
            1,
            1,
            vec![128, 64, 255, 255],
        );
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata = TextureMetadata {
            usage_hint: TextureUsageHint::Normal,
            normal_convention: TextureNormalConvention::TangentSpaceDx,
            mip_policy: TextureMipPolicy::GenerateOffline,
            ..TextureMetadata::default()
        };
        texture.descriptor = Some(descriptor);

        let texture = normalize_normal_map_convention(texture).expect("normal conversion succeeds");

        assert_eq!(texture.rgba, vec![128, 191, 255, 255]);
        assert_eq!(
            texture.texture_descriptor().metadata.normal_convention,
            TextureNormalConvention::TangentSpaceGl
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_descriptor_ownership_non_normal_fast_return() {
        run_descriptor_ownership_benchmark(
            "plugins07_non_normal_descriptor_fast_return",
            &texture_fixture(TextureUsageHint::Albedo, TextureNormalConvention::None),
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_descriptor_ownership_normal_conversion() {
        run_descriptor_ownership_benchmark(
            "plugins07_normal_descriptor_ownership",
            &texture_fixture(
                TextureUsageHint::Normal,
                TextureNormalConvention::TangentSpaceDx,
            ),
        );
    }
}
