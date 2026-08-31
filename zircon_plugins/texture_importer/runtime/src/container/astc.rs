use super::{
    support::{
        parse_error, read_nonzero_u24_le, read_nonzero_u8, require_len,
        texture_depth_or_array_layers, ASTC_MAGIC,
    },
    TextureContainerInfo,
};
use zircon_runtime::asset::{AssetImportContext, AssetImportError};
use zircon_runtime::core::framework::render::RenderImageDimension;

const ASTC_HEADER_SIZE: usize = 16;
const ASTC_BYTES_PER_BLOCK: usize = 16;

pub(super) fn parse(
    context: &AssetImportContext,
) -> Result<TextureContainerInfo, AssetImportError> {
    let bytes = &context.source_bytes;
    require_len(context, ASTC_HEADER_SIZE, "astc header")?;
    if &bytes[..4] != ASTC_MAGIC {
        return parse_error(context, "astc header missing ASTC magic");
    }

    let block_x = read_nonzero_u8(context, 4, "astc block x")?;
    let block_y = read_nonzero_u8(context, 5, "astc block y")?;
    let block_z = read_nonzero_u8(context, 6, "astc block z")?;
    let format = validate_block_footprint(context, block_x, block_y, block_z)?;
    let width = read_nonzero_u24_le(context, 7, "astc width")?;
    let height = read_nonzero_u24_le(context, 10, "astc height")?;
    let depth = read_nonzero_u24_le(context, 13, "astc depth")?;
    validate_block_depth_pair(context, block_z, depth)?;
    let payload_len = astc_payload_len(context, block_x, block_y, block_z, width, height, depth)?;
    let required_len = ASTC_HEADER_SIZE
        .checked_add(payload_len)
        .ok_or_else(|| astc_payload_range_error(context))?;
    require_len(context, required_len, "astc block payload")?;
    let dimension = if depth > 1 || block_z > 1 {
        RenderImageDimension::D3
    } else {
        RenderImageDimension::D2
    };

    Ok(TextureContainerInfo {
        format: format.to_owned(),
        upload_bytes: None,
        width,
        height,
        dimension,
        depth_or_array_layers: texture_depth_or_array_layers(dimension, depth, depth),
        mip_count: 1,
        array_layers: array_layers(dimension, depth),
    })
}

fn validate_block_footprint(
    context: &AssetImportContext,
    block_x: u8,
    block_y: u8,
    block_z: u8,
) -> Result<&'static str, AssetImportError> {
    supported_block_format(block_x, block_y, block_z).ok_or_else(|| {
        super::support::parse_error_value(
            context,
            format!("astc block footprint {block_x}x{block_y}x{block_z} is not supported"),
        )
    })
}

fn supported_block_format(block_x: u8, block_y: u8, block_z: u8) -> Option<&'static str> {
    match (block_x, block_y, block_z) {
        (4, 4, 1) => Some("astc/4x4x1"),
        (5, 4, 1) => Some("astc/5x4x1"),
        (5, 5, 1) => Some("astc/5x5x1"),
        (6, 5, 1) => Some("astc/6x5x1"),
        (6, 6, 1) => Some("astc/6x6x1"),
        (8, 5, 1) => Some("astc/8x5x1"),
        (8, 6, 1) => Some("astc/8x6x1"),
        (8, 8, 1) => Some("astc/8x8x1"),
        (10, 5, 1) => Some("astc/10x5x1"),
        (10, 6, 1) => Some("astc/10x6x1"),
        (10, 8, 1) => Some("astc/10x8x1"),
        (10, 10, 1) => Some("astc/10x10x1"),
        (12, 10, 1) => Some("astc/12x10x1"),
        (12, 12, 1) => Some("astc/12x12x1"),
        (3, 3, 3) => Some("astc/3x3x3"),
        (4, 3, 3) => Some("astc/4x3x3"),
        (4, 4, 3) => Some("astc/4x4x3"),
        (4, 4, 4) => Some("astc/4x4x4"),
        (5, 4, 4) => Some("astc/5x4x4"),
        (5, 5, 4) => Some("astc/5x5x4"),
        (5, 5, 5) => Some("astc/5x5x5"),
        (6, 5, 5) => Some("astc/6x5x5"),
        (6, 6, 5) => Some("astc/6x6x5"),
        (6, 6, 6) => Some("astc/6x6x6"),
        _ => None,
    }
}

fn validate_block_depth_pair(
    context: &AssetImportContext,
    block_z: u8,
    depth: u32,
) -> Result<(), AssetImportError> {
    if block_z == 1 && depth > 1 {
        return parse_error(
            context,
            format!("astc 2d block footprint requires depth 1, got {depth}"),
        );
    }
    Ok(())
}

fn astc_payload_len(
    context: &AssetImportContext,
    block_x: u8,
    block_y: u8,
    block_z: u8,
    width: u32,
    height: u32,
    depth: u32,
) -> Result<usize, AssetImportError> {
    let blocks_x = div_ceil_checked(
        usize::try_from(width).expect("u32 width fits usize"),
        block_x,
    );
    let blocks_y = div_ceil_checked(
        usize::try_from(height).expect("u32 height fits usize"),
        block_y,
    );
    let blocks_z = div_ceil_checked(
        usize::try_from(depth).expect("u32 depth fits usize"),
        block_z,
    );
    blocks_x
        .checked_mul(blocks_y)
        .and_then(|blocks| blocks.checked_mul(blocks_z))
        .and_then(|blocks| blocks.checked_mul(ASTC_BYTES_PER_BLOCK))
        .ok_or_else(|| astc_payload_range_error(context))
}

fn div_ceil_checked(value: usize, divisor: u8) -> usize {
    let divisor = usize::from(divisor);
    value.div_ceil(divisor)
}

fn astc_payload_range_error(context: &AssetImportContext) -> AssetImportError {
    super::support::parse_error_value(context, "astc block payload range overflows usize")
}

fn array_layers(dimension: RenderImageDimension, depth: u32) -> u32 {
    if dimension == RenderImageDimension::D3 {
        1
    } else {
        depth.max(1)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const BENCHMARK_FORMATS: usize = 65_536;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_THRESHOLD_PERCENT: u128 = 25;

    fn legacy_canonical_format(block_x: u8, block_y: u8, block_z: u8) -> String {
        assert!(supported_block_format(block_x, block_y, block_z).is_some());
        format!("astc/{block_x}x{block_y}x{block_z}")
    }

    fn optimized_canonical_format(block_x: u8, block_y: u8, block_z: u8) -> String {
        supported_block_format(block_x, block_y, block_z)
            .unwrap()
            .to_owned()
    }

    fn measure_format_creation(mut create: impl FnMut() -> String) -> u128 {
        let timer = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..BENCHMARK_FORMATS {
            checksum += black_box(create()).len();
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

    fn run_format_benchmark(marker: &str, block_x: u8, block_y: u8, block_z: u8) {
        assert_eq!(
            legacy_canonical_format(block_x, block_y, block_z),
            optimized_canonical_format(block_x, block_y, block_z)
        );
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_format_creation(|| {
                    legacy_canonical_format(
                        black_box(block_x),
                        black_box(block_y),
                        black_box(block_z),
                    )
                }));
                optimized_samples.push(measure_format_creation(|| {
                    optimized_canonical_format(
                        black_box(block_x),
                        black_box(block_y),
                        black_box(block_z),
                    )
                }));
            } else {
                optimized_samples.push(measure_format_creation(|| {
                    optimized_canonical_format(
                        black_box(block_x),
                        black_box(block_y),
                        black_box(block_z),
                    )
                }));
                legacy_samples.push(measure_format_creation(|| {
                    legacy_canonical_format(
                        black_box(block_x),
                        black_box(block_y),
                        black_box(block_z),
                    )
                }));
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
            "PERF_RESULT {marker} formats_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_integer_format_calls_per_sample={} optimized_integer_format_calls_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
            BENCHMARK_FORMATS,
            BENCHMARK_SAMPLE_PAIRS,
            BENCHMARK_FORMATS,
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
    fn canonical_astc_format_maps_supported_2d_footprint() {
        assert_eq!(supported_block_format(12, 10, 1), Some("astc/12x10x1"));
    }

    #[test]
    fn canonical_astc_format_maps_supported_3d_footprint() {
        assert_eq!(supported_block_format(6, 6, 6), Some("astc/6x6x6"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_canonical_astc_format_2d() {
        run_format_benchmark("plugins07_canonical_astc_format_2d", 12, 10, 1);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_canonical_astc_format_3d() {
        run_format_benchmark("plugins07_canonical_astc_format_3d", 6, 6, 6);
    }
}
