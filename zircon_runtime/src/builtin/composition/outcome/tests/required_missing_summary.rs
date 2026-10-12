use std::hint::black_box;
use std::time::Instant;

use crate::builtin::RuntimePluginId;
use crate::plugin::{
    PluginMaturity, RuntimePluginAvailabilityEntry, RuntimePluginAvailabilityReport,
};

use super::RuntimeModuleCompositionRejection;

const ENTRIES_PER_SUMMARY: usize = 64;
const SUMMARIES_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn runtime869_required_missing_summary_single_buffer_preserves_exact_text() {
    let empty = rejection(Vec::new());
    assert_eq!(empty.required_missing_summary(), "");

    let entries = vec![
        missing_entry("third_party.weather", "weather backend is absent"),
        missing_entry("third_party.localization", "依赖不可用"),
        missing_entry("third_party.empty_reason", ""),
    ];
    let rejection = rejection(entries);
    assert_eq!(
        rejection.required_missing_summary(),
        legacy_required_missing_summary(&rejection)
    );
    assert_eq!(
        rejection.required_missing_summary(),
        "required runtime plugin third_party.weather is unavailable: weather backend is absent; required runtime plugin third_party.localization is unavailable: 依赖不可用; required runtime plugin third_party.empty_reason is unavailable: "
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime869_required_missing_summary_single_buffer_benchmark() {
    let entries = (0..ENTRIES_PER_SUMMARY)
        .map(|index| {
            missing_entry(
                &format!("third_party.plugin_{index:03}"),
                &format!("required capability {index:03} is unavailable"),
            )
        })
        .collect();
    let rejection = rejection(entries);
    assert_eq!(
        rejection.required_missing_summary(),
        legacy_required_missing_summary(&rejection)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&rejection, legacy_required_missing_summary));
            optimized.push(measure(
                &rejection,
                RuntimeModuleCompositionRejection::required_missing_summary,
            ));
        } else {
            optimized.push(measure(
                &rejection,
                RuntimeModuleCompositionRejection::required_missing_summary,
            ));
            legacy.push(measure(&rejection, legacy_required_missing_summary));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "RUNTIME869_REQUIRED_MISSING_SUMMARY_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} summaries_per_sample={SUMMARIES_PER_SAMPLE} entries_per_summary={ENTRIES_PER_SUMMARY} legacy_child_strings_per_sample=8192 legacy_temporary_vector_slots_per_sample=8192 optimized_child_strings_per_sample=0 optimized_temporary_vector_slots_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "single-buffer missing-plugin summary P95 {optimized_p95}ns must stay within 10% of collect/join P95 {legacy_p95}ns"
    );
}

fn missing_entry(id: &str, reason: &str) -> RuntimePluginAvailabilityEntry {
    RuntimePluginAvailabilityEntry {
        id: id.to_string(),
        runtime_id: RuntimePluginId::new(id),
        required: true,
        maturity: PluginMaturity::Externalized,
        reason: reason.to_string(),
    }
}

fn rejection(
    missing_required: Vec<RuntimePluginAvailabilityEntry>,
) -> RuntimeModuleCompositionRejection {
    RuntimeModuleCompositionRejection {
        runtime_plugin_availability: RuntimePluginAvailabilityReport {
            missing_required,
            ..RuntimePluginAvailabilityReport::default()
        },
        diagnostics: Vec::new(),
    }
}

fn legacy_required_missing_summary(rejection: &RuntimeModuleCompositionRejection) -> String {
    rejection
        .required_missing()
        .iter()
        .map(|entry| {
            format!(
                "required runtime plugin {} is unavailable: {}",
                entry.runtime_id.label(),
                entry.reason
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn measure(
    rejection: &RuntimeModuleCompositionRejection,
    render: fn(&RuntimeModuleCompositionRejection) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        checksum ^= black_box(render(black_box(rejection))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
