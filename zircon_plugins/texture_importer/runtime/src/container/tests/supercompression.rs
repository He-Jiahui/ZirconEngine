use std::hint::black_box;
use std::io::Cursor;
use std::time::Instant;

use super::*;
use zircon_runtime::asset::AssetUri;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn import_pipeline_hotpath_direct_decode_append_matches_temporary_reference() {
    let context = test_context();
    let payload = patterned_bytes(65_537);
    let mut optimized = vec![7, 11, 13];
    let mut legacy = optimized.clone();
    let mut budget = DecodedLevelBudget::new(MAX_STANDARD_SUPERCOMPRESSED_DECODED_BYTES);

    let appended = append_decoded_level(
        &context,
        0,
        Cursor::new(payload.as_slice()),
        payload.len(),
        &mut budget,
        &mut optimized,
    )
    .expect("direct decode append succeeds");
    legacy_append_decoded_level(&payload, payload.len(), &mut legacy);

    assert_eq!(appended, payload.len());
    assert_eq!(optimized, legacy);

    let before_failure = optimized.clone();
    let error = append_decoded_level(
        &context,
        1,
        Cursor::new(payload.as_slice()),
        payload.len() - 1,
        &mut budget,
        &mut optimized,
    )
    .expect_err("oversized decode must be rejected");
    assert!(error
        .to_string()
        .contains("expected 65536 bytes, got 65537"));
    assert_eq!(optimized, before_failure, "failed append must roll back");
}

#[test]
fn decoded_level_budget_rejects_declared_and_cumulative_expansion_before_append() {
    let context = test_context();
    let mut output = vec![7, 11, 13];
    let original_output = output.clone();
    let mut budget = DecodedLevelBudget::new(8);

    let declared_over_limit = append_decoded_level(
        &context,
        0,
        Cursor::new([1_u8; 9]),
        9,
        &mut budget,
        &mut output,
    )
    .expect_err("a declared decode larger than the budget must be rejected");
    assert!(declared_over_limit
        .to_string()
        .contains("decoded payload budget of 8 bytes"));
    assert_eq!(
        output, original_output,
        "budget rejection must not append bytes"
    );

    let first = append_decoded_level(
        &context,
        1,
        Cursor::new([2_u8; 5]),
        5,
        &mut budget,
        &mut output,
    )
    .expect("first decoded level fits the budget");
    assert_eq!(first, 5);
    let after_first = output.clone();

    let cumulative_over_limit = append_decoded_level(
        &context,
        2,
        Cursor::new([3_u8; 4]),
        4,
        &mut budget,
        &mut output,
    )
    .expect_err("cumulative decoded levels must not exceed the budget");
    assert!(cumulative_over_limit
        .to_string()
        .contains("decoded payload budget of 8 bytes"));
    assert_eq!(output, after_first, "cumulative rejection must roll back");
}

#[test]
#[ignore = "release performance gate"]
fn import_pipeline_hotpath_direct_decode_append_release_benchmark() {
    const PAYLOAD_BYTES: usize = 1_048_576;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

    let context = test_context();
    let payload = patterned_bytes(PAYLOAD_BYTES);
    let mut legacy_output = Vec::with_capacity(PAYLOAD_BYTES);
    let mut optimized_output = Vec::with_capacity(PAYLOAD_BYTES);
    let mut budget = DecodedLevelBudget::new(MAX_STANDARD_SUPERCOMPRESSED_DECODED_BYTES);
    legacy_append_decoded_level(&payload, PAYLOAD_BYTES, &mut legacy_output);
    append_decoded_level(
        &context,
        0,
        Cursor::new(payload.as_slice()),
        PAYLOAD_BYTES,
        &mut budget,
        &mut optimized_output,
    )
    .expect("optimized warmup succeeds");
    assert_eq!(optimized_output, legacy_output);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy_append(
                &payload,
                PAYLOAD_BYTES,
                &mut legacy_output,
            ));
            optimized_samples.push(measure_optimized_append(
                &context,
                &payload,
                PAYLOAD_BYTES,
                &mut optimized_output,
            ));
        } else {
            optimized_samples.push(measure_optimized_append(
                &context,
                &payload,
                PAYLOAD_BYTES,
                &mut optimized_output,
            ));
            legacy_samples.push(measure_legacy_append(
                &payload,
                PAYLOAD_BYTES,
                &mut legacy_output,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement = improvement_percent(legacy_p95, optimized_p95);
    println!(
        "PERF_RESULT plugins07_direct_decode_append sample_pairs={} order=alternating_legacy_first_even payload_bytes={} legacy_temporary_buffers_per_sample=1 optimized_temporary_buffers_per_sample=0 legacy_payload_copies_per_sample=2 optimized_payload_copies_per_sample=1 legacy_ns={} optimized_ns={} legacy_p95_ns={} optimized_p95_ns={} threshold_percent={} improvement_percent={}",
        SAMPLE_PAIRS,
        PAYLOAD_BYTES,
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
        legacy_p95,
        optimized_p95,
        REQUIRED_IMPROVEMENT_PERCENT,
        improvement
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "direct decode append improved {improvement}%, below {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn test_context() -> AssetImportContext {
    AssetImportContext::new(
        "fixture.ktx2".into(),
        AssetUri::parse("res://textures/fixture.ktx2").expect("valid fixture URI"),
        Vec::new(),
        Default::default(),
    )
}

fn patterned_bytes(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(37).wrapping_add(11) as u8)
        .collect()
}

fn legacy_append_decoded_level(payload: &[u8], expected_length: usize, output: &mut Vec<u8>) {
    let mut decoded = Vec::new();
    Cursor::new(payload)
        .take((expected_length + 1) as u64)
        .read_to_end(&mut decoded)
        .unwrap();
    assert_eq!(decoded.len(), expected_length);
    output.extend_from_slice(&decoded);
}

fn measure_legacy_append(payload: &[u8], expected_length: usize, output: &mut Vec<u8>) -> u128 {
    output.clear();
    let started = Instant::now();
    legacy_append_decoded_level(black_box(payload), expected_length, output);
    let elapsed = started.elapsed().as_nanos();
    black_box(output.as_slice());
    elapsed
}

fn measure_optimized_append(
    context: &AssetImportContext,
    payload: &[u8],
    expected_length: usize,
    output: &mut Vec<u8>,
) -> u128 {
    output.clear();
    let started = Instant::now();
    let mut budget = DecodedLevelBudget::new(MAX_STANDARD_SUPERCOMPRESSED_DECODED_BYTES);
    append_decoded_level(
        context,
        0,
        Cursor::new(black_box(payload)),
        expected_length,
        &mut budget,
        output,
    )
    .expect("optimized benchmark append succeeds");
    let elapsed = started.elapsed().as_nanos();
    black_box(output.as_slice());
    elapsed
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    assert_eq!(samples.len(), SAMPLE_PAIRS);
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100) - 1]
}

fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
    assert!(legacy > 0);
    legacy.saturating_sub(optimized) * 100 / legacy
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
