use super::*;

fn field_id(index: usize) -> UiModelFieldId {
    UiModelFieldId::try_new(format!("field_{index:06}"))
        .expect("benchmark field IDs use the validated grammar")
}

fn schema(fields: Vec<UiModelFieldSchema>) -> UiModelSchema {
    UiModelSchema::new(
        UiModelSchemaId::try_new("benchmark.schema").unwrap(),
        UiModelSchemaVersion::try_new(1).unwrap(),
        fields,
    )
}

fn fields(count: usize) -> Vec<UiModelFieldSchema> {
    (0..count)
        .map(|index| {
            UiModelFieldSchema::new(
                field_id(index),
                UiValueKind::String,
                UiModelFieldAccess::ReadOnly,
            )
        })
        .collect()
}

#[test]
fn binary_model_field_lookup_preserves_linear_results() {
    let sorted = schema(fields(8));
    let mut reordered_fields = fields(8);
    reordered_fields.swap(1, 6);
    let reordered = schema(reordered_fields);

    for candidate in [field_id(0), field_id(4), field_id(7), field_id(99)] {
        assert_eq!(sorted.field(&candidate), sorted.field_linear(&candidate));
        assert_eq!(
            reordered.field(&candidate),
            reordered.field_linear(&candidate),
        );
    }
}

#[test]
#[ignore = "release-only binary model schema field lookup benchmark"]
fn runtime_interface03_batch36_binary_model_field_lookup_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const FIELD_COUNT: usize = 65_536;
    const LOOKUP_COUNT: usize = 32;
    const SAMPLE_COUNT: usize = 11;
    let schema = schema(fields(FIELD_COUNT));
    let target = field_id(FIELD_COUNT - 1);
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(schema.field_linear(black_box(&target)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(schema.field(black_box(&target)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            binary_samples.push(measure_binary());
        } else {
            binary_samples.push(measure_binary());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    binary_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_MODEL_FIELD_LOOKUP_BENCH_V1 fields={FIELD_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95], binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(5) <= linear_samples[p95],
        "binary model field lookup must improve P95 by at least 80%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}
