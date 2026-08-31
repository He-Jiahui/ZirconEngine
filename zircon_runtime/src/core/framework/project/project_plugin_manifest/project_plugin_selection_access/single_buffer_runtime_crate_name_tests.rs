use std::hint::black_box;
use std::time::Instant;

use super::{default_runtime_crate_name, ProjectPluginSelection};

const SAMPLE_PAIRS: usize = 31;
const NAMES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829aa_runtime300_plugin_crate_name_preserves_bytes() {
    for (plugin_id, expected) in [
        (
            "rendering-virtual-geometry",
            "zircon_plugin_rendering_virtual_geometry_runtime",
        ),
        ("physics", "zircon_plugin_physics_runtime"),
        (
            "editor.\u{96ea}-bridge",
            "zircon_plugin_editor.\u{96ea}_bridge_runtime",
        ),
    ] {
        let selection = ProjectPluginSelection::runtime_plugin(plugin_id, true, false);
        assert_eq!(selection.runtime_crate_name(), expected);
        assert_eq!(default_runtime_crate_name(plugin_id), expected);
    }

    let overridden = ProjectPluginSelection::runtime_plugin("ignored-id", true, false)
        .with_runtime_crate("custom_runtime_crate");
    assert_eq!(overridden.runtime_crate_name(), "custom_runtime_crate");
}

#[test]
fn optimization_batch_20260829aa_runtime300_plugin_crate_name_uses_one_buffer() {
    let source = include_str!("../project_plugin_selection_access.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn default_runtime_crate_name")
        .nth(1)
        .and_then(|body| body.split("impl ProjectPluginSelection").next())
        .expect("default runtime crate name builder");

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("name.push"));
    assert!(!body.contains(".replace("));
    assert!(!body.contains("format!("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aa_runtime300_single_buffer_plugin_crate_name_bench() {
    let selection = ProjectPluginSelection::runtime_plugin(
        "rendering-virtual-geometry-streaming-production-bridge",
        true,
        false,
    );
    assert_eq!(
        selection.runtime_crate_name(),
        legacy_runtime_crate_name(&selection)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &selection));
            optimized_samples.push(measure(true, &selection));
        } else {
            optimized_samples.push(measure(true, &selection));
            legacy_samples.push(measure(false, &selection));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME300_SINGLE_BUFFER_PLUGIN_CRATE_NAME_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
names_per_sample={NAMES_PER_SAMPLE} plugin_id_bytes={} \
legacy_result_allocations_per_name=2 optimized_result_allocations_per_name=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        selection.id.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_runtime_crate_name(selection: &ProjectPluginSelection) -> String {
    selection
        .runtime_crate
        .clone()
        .unwrap_or_else(|| format!("zircon_plugin_{}_runtime", selection.id.replace('-', "_")))
}

fn measure(optimized: bool, selection: &ProjectPluginSelection) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..NAMES_PER_SAMPLE {
        let name = if optimized {
            black_box(selection).runtime_crate_name()
        } else {
            legacy_runtime_crate_name(black_box(selection))
        };
        checksum = checksum.wrapping_add(black_box(name).len());
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
