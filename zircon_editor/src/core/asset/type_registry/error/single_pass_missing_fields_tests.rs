use std::hint::black_box;
use std::time::Instant;

use super::{AssetTypeId, AssetTypeRegistryError};

const SAMPLE_PAIRS: usize = 31;
const MESSAGES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ae_editor250_missing_field_display_preserves_text() {
    let asset_type = AssetTypeId::parse("material.custom").expect("asset type");
    let error = AssetTypeRegistryError::IncompleteDefinition {
        asset_type: asset_type.clone(),
        missing_fields: vec!["presentation", "thumbnail_provider", "toolkit"],
    };
    assert_eq!(
        error.to_string(),
        "asset type `material.custom` is incomplete; missing presentation, thumbnail_provider, toolkit"
    );
    assert_eq!(
        error.to_string(),
        legacy_missing_field_message(
            &asset_type,
            &["presentation", "thumbnail_provider", "toolkit"]
        )
    );

    let empty = AssetTypeRegistryError::IncompleteDefinition {
        asset_type,
        missing_fields: Vec::new(),
    };
    assert_eq!(
        empty.to_string(),
        "asset type `material.custom` is incomplete; missing "
    );
}

#[test]
fn optimization_batch_20260829ae_editor250_missing_field_display_has_no_join_buffer() {
    let source = include_str!("../error.rs");
    let display = source
        .split("impl fmt::Display")
        .nth(1)
        .expect("display implementation")
        .split("impl std::error::Error")
        .next()
        .expect("display body");

    assert!(display.contains("let mut fields = missing_fields.iter()"));
    assert!(display.contains("formatter.write_str(\", \")"));
    assert!(display.contains("formatter.write_str(field)"));
    assert!(!display.contains("missing_fields.join"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ae_editor250_allocation_free_missing_field_display_bench() {
    let asset_type = AssetTypeId::parse("material.production_surface").expect("asset type");
    let missing_fields = vec![
        "presentation",
        "thumbnail_provider",
        "toolkit",
        "creation_template",
        "context_commands",
        "source_authority",
        "import_pipeline",
        "runtime_loader",
    ];
    let error = AssetTypeRegistryError::IncompleteDefinition {
        asset_type: asset_type.clone(),
        missing_fields: missing_fields.clone(),
    };
    assert_eq!(
        error.to_string(),
        legacy_missing_field_message(&asset_type, &missing_fields)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &error, &asset_type, &missing_fields));
            optimized_samples.push(measure(true, &error, &asset_type, &missing_fields));
        } else {
            optimized_samples.push(measure(true, &error, &asset_type, &missing_fields));
            legacy_samples.push(measure(false, &error, &asset_type, &missing_fields));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR250_ALLOCATION_FREE_MISSING_FIELD_DISPLAY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
messages_per_sample={MESSAGES_PER_SAMPLE} missing_field_count={} \
legacy_result_buffers_per_message=2 optimized_result_buffers_per_message=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        missing_fields.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_missing_field_message(asset_type: &AssetTypeId, missing_fields: &[&str]) -> String {
    format!(
        "asset type `{asset_type}` is incomplete; missing {}",
        missing_fields.join(", ")
    )
}

fn measure(
    optimized: bool,
    error: &AssetTypeRegistryError,
    asset_type: &AssetTypeId,
    missing_fields: &[&str],
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MESSAGES_PER_SAMPLE {
        let message = if optimized {
            black_box(error).to_string()
        } else {
            legacy_missing_field_message(black_box(asset_type), black_box(missing_fields))
        };
        checksum = checksum.wrapping_add(black_box(message).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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
