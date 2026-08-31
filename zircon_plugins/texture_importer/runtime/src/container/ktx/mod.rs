use super::{
    support::{parse_error, parse_error_value, read_u32_le, KTX2_LEVEL_INDEX_ENTRY_SIZE},
    TextureContainerInfo,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

mod ktx1;
mod ktx2;

pub(super) fn parse_ktx1(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    ktx1::parse(context)
}

pub(super) fn parse_ktx2(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    ktx2::parse(context)
}

fn texture_dimension_from_header(height: u32, depth: u32) -> RenderImageDimension {
    if depth > 0 {
        RenderImageDimension::D3
    } else if height == 0 {
        RenderImageDimension::D1
    } else {
        RenderImageDimension::D2
    }
}

fn validate_3d_height(
    context: &AssetImportContext,
    label: &str,
    height: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && height == 0 {
        return parse_error(
            context,
            format_args!("{label} 3d texture height must be nonzero when depth is nonzero"),
        );
    }
    Ok(())
}

fn validate_cubemap_depth(
    context: &AssetImportContext,
    label: &str,
    face_count: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && face_count == KTX_CUBEMAP_FACE_COUNT {
        return parse_error(
            context,
            format_args!("{label} cubemap textures must not declare 3d depth"),
        );
    }
    Ok(())
}

fn validate_cubemap_2d_square_faces(
    context: &AssetImportContext,
    label: &str,
    face_count: u32,
    width: u32,
    height: u32,
) -> Result<(), AssetImportError> {
    if face_count == KTX_CUBEMAP_FACE_COUNT && (height == 0 || width != height) {
        return parse_error(
            context,
            format_args!("{label} cubemap faces must be 2d and square, got {width}x{height}"),
        );
    }
    Ok(())
}

fn validate_3d_array_layers(
    context: &AssetImportContext,
    label: &str,
    declared_layer_count: u32,
    depth: u32,
) -> Result<(), AssetImportError> {
    if depth > 0 && declared_layer_count > 0 {
        return parse_error(
            context,
            format_args!("{label} 3d textures must not declare array layers"),
        );
    }
    Ok(())
}

fn texture_array_layers(dimension: RenderImageDimension, array_layers: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        array_layers.max(1)
    }
}

fn validate_mip_count_fits_extent(
    context: &AssetImportContext,
    label: &str,
    width: u32,
    height: u32,
    depth: u32,
    mip_count: u32,
) -> Result<(), AssetImportError> {
    let max_extent = width.max(height).max(depth);
    let max_mip_count = u32::BITS - max_extent.leading_zeros();
    if mip_count > max_mip_count {
        return parse_error(
            context,
            format_args!(
                "{label} mip level count {mip_count} exceeds maximum {max_mip_count} for extent {width}x{height}x{depth}"
            ),
        );
    }
    Ok(())
}

const KTX_CUBEMAP_FACE_COUNT: u32 = 6;

fn read_face_count(
    context: &AssetImportContext,
    offset: usize,
    label: &str,
) -> Result<u32, AssetImportError> {
    match read_u32_le(context, offset)? {
        1 => Ok(1),
        KTX_CUBEMAP_FACE_COUNT => Ok(KTX_CUBEMAP_FACE_COUNT),
        value => parse_error(
            context,
            format_args!("{label} must be 1 for ordinary textures or 6 for cubemaps, got {value}"),
        ),
    }
}

fn ktx_four_byte_padding(byte_len: usize) -> usize {
    (4 - (byte_len % 4)) % 4
}

fn level_index_end(
    context: &AssetImportContext,
    level_count: u32,
) -> Result<u64, AssetImportError> {
    u64::try_from(super::support::KTX2_HEADER_SIZE)
        .ok()
        .and_then(|header_size| {
            u64::from(level_count)
                .checked_mul(u64::try_from(KTX2_LEVEL_INDEX_ENTRY_SIZE).ok()?)
                .and_then(|level_index_len| header_size.checked_add(level_index_len))
        })
        .ok_or_else(|| parse_error_value(context, "ktx2 level index length overflows u64"))
}

fn checked_u64_range_end(
    context: &AssetImportContext,
    label: &str,
    offset: u64,
    length: u64,
) -> Result<u64, AssetImportError> {
    offset
        .checked_add(length)
        .ok_or_else(|| parse_error_value(context, format_args!("{label} range overflows u64")))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::asset::AssetUri;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const ERRORS_PER_SAMPLE: usize = 8_192;

    #[test]
    fn borrowed_ktx_error_arguments_preserve_dynamic_diagnostics() {
        let context = test_context();

        let cubemap_error =
            validate_cubemap_2d_square_faces(&context, "ktx2", 6, 16, 8).unwrap_err();
        let range_error =
            checked_u64_range_end(&context, "ktx2 level payload", u64::MAX, 1).unwrap_err();

        assert_eq!(
            cubemap_error.to_string(),
            "parse texture container broken.ktx2: ktx2 cubemap faces must be 2d and square, got 16x8"
        );
        assert_eq!(
            range_error.to_string(),
            "parse texture container broken.ktx2: ktx2 level payload range overflows u64"
        );
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_borrowed_ktx_error_arguments() {
        let context = test_context();
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_errors(&context, legacy_error));
                optimized_raw.push(measure_errors(&context, optimized_error));
            } else {
                optimized_raw.push(measure_errors(&context, optimized_error));
                legacy_raw.push(measure_errors(&context, legacy_error));
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "borrowed KTX error arguments must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_borrowed_ktx_error_arguments sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank errors_per_sample={ERRORS_PER_SAMPLE} legacy_allocations_per_error=2 optimized_allocations_per_error=1 legacy_detail_allocations_per_sample={ERRORS_PER_SAMPLE} optimized_detail_allocations_per_sample=0 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn legacy_error(context: &AssetImportContext, index: usize) -> AssetImportError {
        let detail = format!(
            "ktx2 mip level count {} exceeds maximum {} for extent {}x{}x{}",
            18 + index % 5,
            12,
            2048,
            1024,
            1
        );
        parse_error_value(context, detail)
    }

    fn optimized_error(context: &AssetImportContext, index: usize) -> AssetImportError {
        parse_error_value(
            context,
            format_args!(
                "ktx2 mip level count {} exceeds maximum {} for extent {}x{}x{}",
                18 + index % 5,
                12,
                2048,
                1024,
                1
            ),
        )
    }

    fn measure_errors(
        context: &AssetImportContext,
        make_error: fn(&AssetImportContext, usize) -> AssetImportError,
    ) -> u64 {
        let started = Instant::now();
        for index in 0..ERRORS_PER_SAMPLE {
            black_box(make_error(black_box(context), black_box(index)));
        }
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn test_context() -> AssetImportContext {
        AssetImportContext::new(
            "broken.ktx2".into(),
            AssetUri::parse("res://textures/broken.ktx2").unwrap(),
            vec![0; 16],
            Default::default(),
        )
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u64]) -> String {
        let values = samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!("[{values}]")
    }
}
