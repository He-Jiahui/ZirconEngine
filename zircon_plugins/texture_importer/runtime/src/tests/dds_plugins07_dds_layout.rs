use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 40_000;
const TOKENS: [&str; 10] = [
    " DDS/DXT1 ",
    "dds/ATI1",
    "dds/bc4u",
    "DDS/BC4S",
    "dds/dxt3",
    "DDS/DXT5",
    "dds/ati2",
    "dds/BC5U",
    "DDS/BC5S",
    "dds/uncompressed",
];

#[test]
fn borrowed_texture_format_contract_dds_layout() {
    for token in &TOKENS[..4] {
        assert_eq!(
            dds_legacy_payload_layout(token).map(|layout| layout.bytes_per_block),
            Some(DDS_BC1_BYTES_PER_BLOCK)
        );
    }
    for token in &TOKENS[4..9] {
        assert_eq!(
            dds_legacy_payload_layout(token).map(|layout| layout.bytes_per_block),
            Some(DDS_BC_FULL_BYTES_PER_BLOCK)
        );
    }
    assert!(dds_legacy_payload_layout(TOKENS[9]).is_none());
}

#[test]
fn borrowed_texture_format_contract_dds_fourcc_header() {
    let context = import_context();
    let ascii = fourcc_string(&context, b"DxT1").unwrap().unwrap();
    assert!(matches!(ascii, std::borrow::Cow::Borrowed("DxT1")));

    let numeric_bytes = 36_u32.to_le_bytes();
    let numeric = fourcc_string(&context, &numeric_bytes).unwrap().unwrap();
    assert!(matches!(numeric, std::borrow::Cow::Owned(ref value) if value == "D3DFMT-36"));
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_format_performance_release_dds_layout() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(), measure_borrowed())
        } else {
            let optimized_ns = measure_borrowed();
            (measure_legacy(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_dds_layout_token_dispatch sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * TOKENS.len(),
    );
    assert!(
        improvement_percent >= 25,
        "borrowed DDS layout matching must improve P95 by at least 25%"
    );
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_texture_format_performance_release_dds_fourcc_header() {
    let context = import_context();
    let tokens: [&[u8; 4]; 3] = [b"DXT1", b"DxT5", b"ATI2"];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (
                measure_legacy_fourcc(&tokens),
                measure_borrowed_fourcc(&context, &tokens),
            )
        } else {
            let optimized_ns = measure_borrowed_fourcc(&context, &tokens);
            (measure_legacy_fourcc(&tokens), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_dds_fourcc_header_borrow sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * tokens.len(),
    );
    assert!(
        improvement_percent >= 25,
        "borrowed DDS FourCC header decoding must improve P95 by at least 25%"
    );
}

fn measure_legacy() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            let bytes_per_block = match black_box(token).trim().to_ascii_lowercase().as_str() {
                "dds/dxt1" | "dds/ati1" | "dds/bc4u" | "dds/bc4s" => {
                    Some(DDS_BC1_BYTES_PER_BLOCK)
                }
                "dds/dxt3" | "dds/dxt5" | "dds/ati2" | "dds/bc5u" | "dds/bc5s" => {
                    Some(DDS_BC_FULL_BYTES_PER_BLOCK)
                }
                _ => None,
            };
            matched += bytes_per_block.unwrap_or_default() as u64;
        }
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn measure_borrowed() -> u128 {
    let started = Instant::now();
    let mut matched = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for token in TOKENS {
            matched += dds_legacy_payload_layout(black_box(token))
                .map(|layout| layout.bytes_per_block)
                .unwrap_or_default() as u64;
        }
    }
    black_box(matched);
    started.elapsed().as_nanos()
}

fn measure_legacy_fourcc(tokens: &[&[u8; 4]]) -> u128 {
    let started = Instant::now();
    let mut decoded = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for bytes in tokens {
            let value = black_box(*bytes)
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>();
            decoded += value.len() as u64;
            black_box(value);
        }
    }
    black_box(decoded);
    started.elapsed().as_nanos()
}

fn measure_borrowed_fourcc(context: &AssetImportContext, tokens: &[&[u8; 4]]) -> u128 {
    let started = Instant::now();
    let mut decoded = 0_u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        for bytes in tokens {
            let value = fourcc_string(black_box(context), black_box(*bytes))
                .unwrap()
                .unwrap();
            decoded += value.len() as u64;
            black_box(value);
        }
    }
    black_box(decoded);
    started.elapsed().as_nanos()
}

fn import_context() -> AssetImportContext {
    AssetImportContext::new(
        "fixture.dds".into(),
        AssetUri::parse("res://textures/fixture.dds").unwrap(),
        Vec::new(),
        "".parse().unwrap(),
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
