use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime91_material_override_index_preserves_first_field_and_diagnostics() {
    let payload = RenderMaterialPropertyUniformPayload {
        layout: vec![
            field("gain", "float", 0),
            field("flag", "float", 4),
            field("gain", "float", 16),
        ],
        bytes: vec![0; 32],
        unsupported: Vec::new(),
    };
    let overrides = MaterialPropertyOverrideBlock::new()
        .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.5 })
        .with_value("flag", RenderMaterialPropertyValue::Bool { value: true })
        .with_value("missing", RenderMaterialPropertyValue::Float { value: 1.0 });

    let overridden = payload.with_override_block(&overrides);

    assert_eq!(f32_at(&overridden.bytes, 0), 2.5);
    assert_eq!(f32_at(&overridden.bytes, 16), 0.0);
    assert_eq!(overridden.unsupported.len(), 2);
    assert_eq!(
        overridden.unsupported[0].reason,
        RenderMaterialPropertyUniformUnsupportedReason::TypeMismatch
    );
    assert_eq!(
        overridden.unsupported[1].reason,
        RenderMaterialPropertyUniformUnsupportedReason::UnknownProperty
    );
}

#[test]
fn runtime91_material_override_uses_borrowed_field_index() {
    let source = include_str!("../property_uniform.rs");
    let overrides = bounded_source(source, "pub fn with_override_block(", "pub fn is_empty(");

    assert!(overrides.contains("HashMap::with_capacity"));
    assert!(overrides.contains("field_indices.entry"));
    assert!(overrides.contains(".or_insert(index)"));
    assert!(overrides.contains("field_indices.get(name.as_str())"));
    assert!(!overrides.contains("payload.layout.iter().find"));
    assert!(!overrides.contains(".find(|field|"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime91_material_override_field_index_p95() {
    const FIELD_COUNT: usize = 2_048;
    const APPLICATIONS: usize = 4;
    let values = (0..FIELD_COUNT)
        .map(|index| {
            (
                format!("property-{index:04}"),
                RenderMaterialPropertyValue::Float {
                    value: index as f32,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let payload = RenderMaterialPropertyUniformPayload::from_values(&values);
    let overrides = MaterialPropertyOverrideBlock::from_values(values);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(APPLICATIONS, || {
                legacy_with_override_block(black_box(&payload), black_box(&overrides))
            }));
            optimized_ns.push(measure_ns(APPLICATIONS, || {
                payload
                    .with_override_block(black_box(&overrides))
                    .bytes
                    .len()
            }));
        } else {
            optimized_ns.push(measure_ns(APPLICATIONS, || {
                payload
                    .with_override_block(black_box(&overrides))
                    .bytes
                    .len()
            }));
            legacy_ns.push(measure_ns(APPLICATIONS, || {
                legacy_with_override_block(black_box(&payload), black_box(&overrides))
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "material override field index P95 must be at least 90% below repeated layout scans: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    let legacy_field_comparisons = FIELD_COUNT * (FIELD_COUNT + 1) / 2 * APPLICATIONS;
    println!(
        "RUNTIME91_MATERIAL_OVERRIDE_FIELD_INDEX_BENCH_V1 fields={FIELD_COUNT} overrides={FIELD_COUNT} applications_per_sample={APPLICATIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_field_comparisons_per_sample={legacy_field_comparisons} optimized_field_index_visits_per_sample={} optimized_hash_lookups_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        FIELD_COUNT * APPLICATIONS,
        FIELD_COUNT * APPLICATIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn field(name: &str, kind: &str, offset: u32) -> RenderMaterialPropertyUniformField {
    RenderMaterialPropertyUniformField {
        name: name.to_string(),
        kind: kind.to_string(),
        offset,
        size: 4,
        alignment: 4,
    }
}

fn legacy_with_override_block(
    payload: &RenderMaterialPropertyUniformPayload,
    overrides: &MaterialPropertyOverrideBlock,
) -> usize {
    let mut payload = payload.clone();
    for (name, value) in overrides.values() {
        let Some(field) = payload.layout.iter().find(|field| field.name == *name) else {
            payload
                .unsupported
                .push(RenderMaterialPropertyUniformUnsupported {
                    name: name.clone(),
                    reason: RenderMaterialPropertyUniformUnsupportedReason::UnknownProperty,
                });
            continue;
        };
        let Some(kind) = MaterialPropertyKind::parse_token(&field.kind) else {
            continue;
        };
        if let Some(reason) = write_field_override_value(&mut payload.bytes, field, kind, value) {
            payload
                .unsupported
                .push(RenderMaterialPropertyUniformUnsupported {
                    name: name.clone(),
                    reason,
                });
        }
    }
    black_box(payload).bytes.len()
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
