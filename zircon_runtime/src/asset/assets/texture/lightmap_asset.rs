use crate::asset::AssetUri;
use crate::core::framework::render::{
    LightmapBakeOutput, LightmapContractValidationError, RenderImageColorSpace,
    RenderImageDimension,
};

use super::{TextureAsset, TextureAssetDescriptor};

pub const LIGHTMAP_RGBA16F_FORMAT: &str = "zircon-lightmap-rgba16f-le-v1";
pub const LIGHTMAP_RGBA16F_GPU_FORMAT: &str = "rgba16float";

pub fn texture_asset_from_lightmap_bake_output(
    uri: AssetUri,
    output: &LightmapBakeOutput,
) -> Result<TextureAsset, LightmapContractValidationError> {
    output.validate()?;
    let payload = ordered_lightmap_payload(output)?;
    let mut descriptor =
        TextureAssetDescriptor::container(LIGHTMAP_RGBA16F_GPU_FORMAT, 1, output.atlas.page_count);
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.dimension = RenderImageDimension::D2;

    Ok(TextureAsset::new_container(
        uri,
        output.atlas.page_size,
        output.atlas.page_size,
        LIGHTMAP_RGBA16F_FORMAT,
        payload,
        1,
        output.atlas.page_count,
    )
    .with_descriptor(descriptor))
}

fn ordered_lightmap_payload(
    output: &LightmapBakeOutput,
) -> Result<Vec<u8>, LightmapContractValidationError> {
    let first_page = output
        .atlas_pages
        .first()
        .expect("validated lightmap output must contain at least one atlas page");
    let payload_capacity = first_page
        .texels_rgba16f_le
        .len()
        .checked_mul(output.atlas_pages.len())
        .ok_or(LightmapContractValidationError::AtlasPayloadSizeOverflow)?;
    let mut ordered_pages = vec![first_page; output.atlas_pages.len()];
    for page in &output.atlas_pages {
        ordered_pages[page.page_index as usize] = page;
    }

    let mut payload = Vec::with_capacity(payload_capacity);
    for page in ordered_pages {
        payload.extend_from_slice(&page.texels_rgba16f_le);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        LightmapAtlasDescriptor, LightmapAtlasFormat, LightmapAtlasPage, LightmapBakeOutput,
        LIGHTMAP_CONSUME_CONTRACT_VERSION,
    };

    use super::*;

    const PERFORMANCE_PAGE_COUNT: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;

    fn shuffled_page_indices(count: usize) -> Vec<u32> {
        let mut indices = (0..u32::try_from(count).unwrap()).collect::<Vec<_>>();
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for upper in (1..indices.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            indices.swap(upper, usize::try_from(state).unwrap() % (upper + 1));
        }
        indices
    }

    fn performance_output() -> LightmapBakeOutput {
        LightmapBakeOutput {
            contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
            request_id: 1,
            scene_revision: 1,
            light_set_generation: 1,
            atlas: LightmapAtlasDescriptor {
                page_size: 1,
                page_count: u32::try_from(PERFORMANCE_PAGE_COUNT).unwrap(),
                format: LightmapAtlasFormat::Rgba16Float,
            },
            atlas_pages: shuffled_page_indices(PERFORMANCE_PAGE_COUNT)
                .into_iter()
                .map(|page_index| LightmapAtlasPage {
                    page_index,
                    texels_rgba16f_le: vec![u8::try_from(page_index % 251).unwrap(); 8],
                })
                .collect(),
            slots: Vec::new(),
            probe_grid: None,
        }
    }

    fn legacy_sorted_payload(output: &LightmapBakeOutput) -> Vec<u8> {
        let mut pages = output.atlas_pages.iter().collect::<Vec<_>>();
        pages.sort_by_key(|page| page.page_index);
        pages
            .into_iter()
            .flat_map(|page| page.texels_rgba16f_le.iter().copied())
            .collect()
    }

    fn elapsed_micros(run: impl FnOnce()) -> u128 {
        let started = Instant::now();
        run();
        started.elapsed().as_micros()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * 95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn lightmap_bake_output_becomes_sorted_rgba16f_array_asset() {
        let output = LightmapBakeOutput {
            contract_version: LIGHTMAP_CONSUME_CONTRACT_VERSION,
            request_id: 1,
            scene_revision: 1,
            light_set_generation: 1,
            atlas: LightmapAtlasDescriptor {
                page_size: 1,
                page_count: 2,
                format: LightmapAtlasFormat::Rgba16Float,
            },
            atlas_pages: vec![
                LightmapAtlasPage {
                    page_index: 1,
                    texels_rgba16f_le: vec![2; 8],
                },
                LightmapAtlasPage {
                    page_index: 0,
                    texels_rgba16f_le: vec![1; 8],
                },
            ],
            slots: Vec::new(),
            probe_grid: None,
        };

        let asset = texture_asset_from_lightmap_bake_output(
            AssetUri::parse("res://lighting/test.lightmap-array").expect("valid test URI"),
            &output,
        )
        .expect("valid bake output should become a texture asset");

        assert_eq!(asset.width, 1);
        assert_eq!(asset.height, 1);
        assert_eq!(asset.rgba, Vec::<u8>::new());
        let super::super::TexturePayload::Container {
            format,
            bytes,
            mip_count,
            array_layers,
        } = &asset.payload
        else {
            panic!("lightmap atlas must use the raw container payload");
        };
        assert_eq!(format, LIGHTMAP_RGBA16F_FORMAT);
        assert_eq!(bytes, &[vec![1; 8], vec![2; 8]].concat());
        assert_eq!(*mip_count, 1);
        assert_eq!(*array_layers, 2);
        let descriptor = asset.render_image_descriptor();
        assert_eq!(descriptor.format, LIGHTMAP_RGBA16F_GPU_FORMAT);
        assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
        assert_eq!(descriptor.array_layer_count, 2);
    }

    #[test]
    fn optimization_batch_20260826d_runtime97_lightmap_pages_use_linear_index_assembly() {
        let source = include_str!("lightmap_asset.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;

        assert!(source.contains("fn ordered_lightmap_payload("));
        assert!(source.contains("ordered_pages[page.page_index as usize] = page"));
        assert!(source.contains("Vec::with_capacity(payload_capacity)"));
        assert!(!source.contains("pages.sort_by_key"));
    }

    #[test]
    fn optimization_batch_20260826d_runtime97_lightmap_pages_preserve_index_order() {
        let output = performance_output();
        output.validate().unwrap();

        let payload = ordered_lightmap_payload(&output).unwrap();
        assert_eq!(payload, legacy_sorted_payload(&output));
        assert_eq!(payload.capacity(), payload.len());
    }

    #[test]
    #[ignore = "release performance evidence for the managed validation coordinator"]
    fn optimization_batch_20260826d_runtime97_lightmap_page_assembly_performance_evidence() {
        let output = performance_output();
        output.validate().unwrap();
        let expected_bytes = PERFORMANCE_PAGE_COUNT * 8;

        for _ in 0..3 {
            assert_eq!(
                black_box(legacy_sorted_payload(black_box(&output))).len(),
                expected_bytes
            );
            assert_eq!(
                black_box(ordered_lightmap_payload(black_box(&output)).unwrap()).len(),
                expected_bytes
            );
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                elapsed_micros(|| {
                    black_box(legacy_sorted_payload(black_box(&output)));
                })
            };
            let measure_optimized = || {
                elapsed_micros(|| {
                    black_box(ordered_lightmap_payload(black_box(&output)).unwrap());
                })
            };
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
        println!(
            "RUNTIME97_LIGHTMAP_LINEAR_PAGE_ASSEMBLY_BENCH_V1 sample_pairs={} pages={} payload_bytes={} legacy_complexity=n_log_n optimized_complexity=n legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
            SAMPLE_PAIRS,
            PERFORMANCE_PAGE_COUNT,
            expected_bytes,
            legacy_p95,
            optimized_p95,
            legacy_samples,
            optimized_samples,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "linear page assembly p95 must be at least 30% below comparison sorting: legacy={legacy_p95}us optimized={optimized_p95}us"
        );
    }
}
