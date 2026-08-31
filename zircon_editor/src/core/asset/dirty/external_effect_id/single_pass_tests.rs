use std::hint::black_box;
use std::time::Instant;

use super::{
    validate_external_effect_id, DirtyExternalEffectId, DirtyExternalEffectIdError,
    ExternalEffectIdValidationFailure,
};

const MARKER: &str = "EDITOR191_EXTERNAL_EFFECT_ID_SINGLE_PASS_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 50_000;
const VALID_ID: &str = "a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a";

#[test]
fn optimization_batch_20260826gy_editor191_external_effect_id_validation_preserves_errors() {
    assert_eq!(validate_external_effect_id(VALID_ID), Ok(()));
    assert_eq!(
        validate_external_effect_id("editor..@invalid"),
        Err(ExternalEffectIdValidationFailure::EmptySegment)
    );
    assert_eq!(
        validate_external_effect_id("editor.@invalid"),
        Err(ExternalEffectIdValidationFailure::InvalidCharacter {
            index: 7,
            character: '@',
        })
    );
    assert_eq!(
        validate_external_effect_id("editor.trailing."),
        Err(ExternalEffectIdValidationFailure::EmptySegment)
    );
    assert!(matches!(
        DirtyExternalEffectId::parse("editor..@invalid"),
        Err(DirtyExternalEffectIdError::EmptySegment { .. })
    ));
    assert!(matches!(
        DirtyExternalEffectId::parse("editor.@invalid"),
        Err(DirtyExternalEffectIdError::InvalidCharacter {
            index: 7,
            character: '@',
            ..
        })
    ));
}

#[test]
fn optimization_batch_20260826gy_editor191_external_effect_id_validation_is_single_pass() {
    let source = include_str!("../external_effect_id.rs");
    let implementation = source
        .split("fn validate_external_effect_id")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("external effect id validation implementation");
    assert!(implementation.contains("char_indices()"));
    assert!(implementation.contains("previous_was_separator"));
    assert!(!implementation.contains("split('.')"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gy_editor191_external_effect_id_single_pass_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_validate_external_effect_id));
            optimized_samples.push(measure(optimized_validate_external_effect_id));
        } else {
            optimized_samples.push(measure(optimized_validate_external_effect_id));
            legacy_samples.push(measure(legacy_validate_external_effect_id));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-pass validation must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_validate_external_effect_id(value: &str) -> bool {
    if value.split('.').any(str::is_empty) {
        return false;
    }
    value.char_indices().all(|(_, character)| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '_' | '-' | '.')
    })
}

fn optimized_validate_external_effect_id(value: &str) -> bool {
    validate_external_effect_id(value).is_ok()
}

fn measure(implementation: fn(&str) -> bool) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        checksum = checksum.wrapping_add(usize::from(implementation(black_box(VALID_ID))));
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
