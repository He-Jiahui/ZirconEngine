use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{MaterialOptionKind, MaterialOptionRef};

const SAMPLE_PAIRS: usize = 101;
const DESCRIPTIONS_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime876_material_enum_expected_direct_preserves_exact_text() {
    for values in [
        vec![],
        vec!["only".to_string()],
        vec!["".to_string(), "another".to_string(), "".to_string()],
        vec!["中文".to_string(), "emoji🙂".to_string()],
    ] {
        let option = option(MaterialOptionKind::Enum, values);
        assert_eq!(
            option.expected_value_description(),
            legacy_description(&option)
        );
    }
    let boolean = option(MaterialOptionKind::Bool, vec!["ignored".to_string()]);
    assert_eq!(
        boolean.expected_value_description(),
        legacy_description(&boolean)
    );
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime876_material_enum_expected_direct_release_percentiles() {
    let option = option(
        MaterialOptionKind::Enum,
        (0..64).map(|index| format!("value_{index:02}")).collect(),
    );
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&option, legacy_description));
            optimized.push(measure(
                &option,
                MaterialOptionRef::expected_value_description,
            ));
        } else {
            optimized.push(measure(
                &option,
                MaterialOptionRef::expected_value_description,
            ));
            legacy.push(measure(&option, legacy_description));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "RUNTIME876_MATERIAL_ENUM_EXPECTED_DIRECT_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn option(kind: MaterialOptionKind, enum_values: Vec<String>) -> MaterialOptionRef {
    MaterialOptionRef {
        name: "surface".to_string(),
        kind,
        bit_offset: 0,
        bit_width: 3,
        enum_values,
        default_bits: 0,
    }
}

fn legacy_description(option: &MaterialOptionRef) -> String {
    match option.kind {
        MaterialOptionKind::Bool => "bool".to_string(),
        MaterialOptionKind::Enum if option.enum_values.is_empty() => "enum value".to_string(),
        MaterialOptionKind::Enum => format!("one of {}", option.enum_values.join(", ")),
    }
}

fn measure(option: &MaterialOptionRef, description: fn(&MaterialOptionRef) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..DESCRIPTIONS_PER_SAMPLE)
        .map(|_| black_box(description(black_box(option))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
