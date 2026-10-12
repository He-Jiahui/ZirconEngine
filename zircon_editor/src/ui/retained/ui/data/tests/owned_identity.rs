use std::hint::black_box;
use std::time::Instant;

use super::{
    into_inspector_projection_identity, InspectorProjectionIdentity, InspectorVisualFields,
};

const FIELD_BYTES: usize = 96;
const ROW_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_20260829aw_editor268_inspector_identity_preserves_field_values() {
    let fields = fields(7);

    let identity = into_inspector_projection_identity(fields);

    assert!(identity.name.starts_with("name.00000007."));
    assert!(identity.parent.starts_with("parent.00000007."));
    assert!(identity.x.starts_with("x.00000007."));
    assert!(identity.y.starts_with("y.00000007."));
    assert!(identity.z.starts_with("z.00000007."));
    assert!(identity.delete_enabled);
}

#[test]
fn optimization_batch_20260829aw_editor268_inspector_identity_moves_string_buffers() {
    let fields = fields(11);
    let pointers = [
        fields.name.as_ptr(),
        fields.parent.as_ptr(),
        fields.x.as_ptr(),
        fields.y.as_ptr(),
        fields.z.as_ptr(),
    ];

    let identity = into_inspector_projection_identity(fields);

    assert_eq!(identity.name.as_ptr(), pointers[0]);
    assert_eq!(identity.parent.as_ptr(), pointers[1]);
    assert_eq!(identity.x.as_ptr(), pointers[2]);
    assert_eq!(identity.y.as_ptr(), pointers[3]);
    assert_eq!(identity.z.as_ptr(), pointers[4]);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aw_editor268_move_inspector_projection_identity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR268_MOVE_INSPECTOR_PROJECTION_IDENTITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
rows={ROW_COUNT} field_bytes={FIELD_BYTES} identity_fields_per_row=5 \
legacy_string_clones_per_row=5 optimized_string_clones_per_row=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let rows = (0..ROW_COUNT).map(fields).collect::<Vec<_>>();
    let started = Instant::now();
    let mut checksum = 0usize;
    for fields in rows {
        let identity = if optimized {
            into_inspector_projection_identity(black_box(fields))
        } else {
            legacy_into_inspector_projection_identity(black_box(fields))
        };
        checksum ^= identity.name.len()
            ^ identity.parent.len()
            ^ identity.x.len()
            ^ identity.y.len()
            ^ identity.z.len();
        black_box(identity);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_into_inspector_projection_identity(
    fields: InspectorVisualFields,
) -> InspectorProjectionIdentity {
    InspectorProjectionIdentity {
        name: fields.name.clone(),
        parent: fields.parent.clone(),
        x: fields.x.clone(),
        y: fields.y.clone(),
        z: fields.z.clone(),
        delete_enabled: fields.delete_enabled,
    }
}

fn fields(index: usize) -> InspectorVisualFields {
    InspectorVisualFields {
        info: long_field("info", index),
        name: long_field("name", index),
        parent: long_field("parent", index),
        x: long_field("x", index),
        y: long_field("y", index),
        z: long_field("z", index),
        delete_enabled: true,
        plugin_components: Vec::new(),
    }
}

fn long_field(label: &str, index: usize) -> String {
    let prefix = format!("{label}.{index:08}.");
    let value = format!("{prefix}{}", "x".repeat(FIELD_BYTES - prefix.len()));
    assert_eq!(value.len(), FIELD_BYTES);
    value
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
