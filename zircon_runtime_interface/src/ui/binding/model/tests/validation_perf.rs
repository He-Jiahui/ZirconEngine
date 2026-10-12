use super::*;

fn validate_conversion_id_two_pass(value: &str) -> Result<(), UiBindingConversionIdentityError> {
    if value.is_empty() {
        return Err(UiBindingConversionIdentityError::Empty);
    }
    if value.len() > UI_BINDING_CONVERSION_ID_MAX_BYTES {
        return Err(UiBindingConversionIdentityError::TooLong {
            actual_bytes: value.len(),
            maximum_bytes: UI_BINDING_CONVERSION_ID_MAX_BYTES,
        });
    }
    for (segment_index, segment) in value.split('.').enumerate() {
        if segment.is_empty() {
            return Err(UiBindingConversionIdentityError::EmptySegment { segment_index });
        }
    }
    for (byte_index, character) in value.char_indices() {
        if !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')) {
            return Err(UiBindingConversionIdentityError::InvalidCharacter {
                character,
                byte_index,
            });
        }
    }
    Ok(())
}

#[test]
fn runtime_interface03_batch55_61_single_pass_conversion_id_validation_preserves_two_pass_results()
{
    let long = "a".repeat(UI_BINDING_CONVERSION_ID_MAX_BYTES + 1);
    for value in [
        "",
        "font.primary",
        "a_b-c.9",
        ".leading",
        "trailing.",
        "double..separator",
        "a b",
        "a/ b",
        "界面",
        ".界面.",
        long.as_str(),
    ] {
        assert_eq!(
            validate_conversion_id(value),
            validate_conversion_id_two_pass(value),
            "validation result changed for {value:?}",
        );
        assert_eq!(
            UiBindingConversionId::try_new(value.to_string()).map(|_| ()),
            validate_conversion_id_two_pass(value),
            "public constructor result changed for {value:?}",
        );
    }
}

#[test]
#[ignore = "release-only single-pass conversion identity validation benchmark"]
fn runtime_interface03_batch55_61_single_pass_conversion_id_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const VALIDATE_COUNT: usize = 250_000;
    const SAMPLE_COUNT: usize = 11;
    let input = "provider.primary_conversion_v2";
    let mut two_pass_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut single_pass_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_two_pass = || {
            let started = Instant::now();
            for _ in 0..VALIDATE_COUNT {
                black_box(validate_conversion_id_two_pass(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        let measure_single_pass = || {
            let started = Instant::now();
            for _ in 0..VALIDATE_COUNT {
                black_box(validate_conversion_id(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            two_pass_samples.push(measure_two_pass());
            single_pass_samples.push(measure_single_pass());
        } else {
            single_pass_samples.push(measure_single_pass());
            two_pass_samples.push(measure_two_pass());
        }
    }

    two_pass_samples.sort_unstable();
    single_pass_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SINGLE_PASS_CONVERSION_ID_BENCH_V1 validations={VALIDATE_COUNT} bytes={} samples={SAMPLE_COUNT} two_pass_p95_ns={} single_pass_p95_ns={}",
        input.len(),
        two_pass_samples[p95],
        single_pass_samples[p95],
    );
    assert!(
        single_pass_samples[p95].saturating_mul(5) <= two_pass_samples[p95].saturating_mul(4),
        "single-pass conversion identity validation must improve P95 by at least 20%: two_pass={}ns single_pass={}ns",
        two_pass_samples[p95],
        single_pass_samples[p95],
    );
}
