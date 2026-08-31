use std::hint::black_box;
use std::time::Instant;

use super::{
    default_feature_runtime_crate_name, local_feature_runtime_crate_path,
    sanitize_crate_path_character, ProjectPluginFeatureSelection,
};

const SAMPLE_PAIRS: usize = 31;
const NAMES_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ab_runtime301_feature_paths_preserve_bytes() {
    let feature = ProjectPluginFeatureSelection::new("Rendering.Virtual-Geometry");
    assert_eq!(
        feature.runtime_crate_name(),
        "zircon_plugin_rendering_virtual_geometry_runtime"
    );
    assert_eq!(
        feature.runtime_crate_path("rendering.owner-plugin"),
        "rendering.owner-plugin/features/virtual_geometry/runtime"
    );
    assert_eq!(
        default_feature_runtime_crate_name("editor.\u{96ea}-Bridge"),
        "zircon_plugin_editor___bridge_runtime"
    );
    assert_eq!(
        local_feature_runtime_crate_path("owner", "editor.\u{96ea}-Bridge"),
        "owner/features/__bridge/runtime"
    );

    let overridden =
        ProjectPluginFeatureSelection::new("ignored").with_runtime_crate("custom_feature_runtime");
    assert_eq!(overridden.runtime_crate_name(), "custom_feature_runtime");
    let external = ProjectPluginFeatureSelection::new("rendering.virtual-geometry")
        .with_provider_package_id("vendor.rendering");
    assert_eq!(
        external.runtime_crate_path("owner"),
        "vendor.rendering/runtime"
    );
}

#[test]
fn optimization_batch_20260829ab_runtime301_feature_paths_use_one_buffer() {
    let source = include_str!("../project_plugin_feature_selection.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let crate_builder = implementation
        .split("fn default_feature_runtime_crate_name")
        .nth(1)
        .and_then(|body| body.split("fn local_feature_runtime_crate_path").next())
        .expect("default feature crate builder");
    let path_builder = implementation
        .split("fn local_feature_runtime_crate_path")
        .nth(1)
        .and_then(|body| body.split("impl ProjectPluginFeatureSelection").next())
        .expect("local feature path builder");

    assert!(crate_builder.contains("String::with_capacity"));
    assert!(path_builder.contains("String::with_capacity"));
    assert!(crate_builder.contains("sanitize_crate_path_character"));
    assert!(path_builder.contains("sanitize_crate_path_character"));
    assert!(!crate_builder.contains("format!("));
    assert!(!path_builder.contains("format!("));
    assert!(!implementation.contains("fn feature_crate_stem"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ab_runtime301_single_buffer_feature_crate_name_bench() {
    let feature_id = "Rendering.Virtual-Geometry.Streaming-Production-Bridge";
    assert_eq!(
        default_feature_runtime_crate_name(feature_id),
        legacy_feature_runtime_crate_name(feature_id)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, feature_id));
            optimized_samples.push(measure(true, feature_id));
        } else {
            optimized_samples.push(measure(true, feature_id));
            legacy_samples.push(measure(false, feature_id));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME301_SINGLE_BUFFER_PLUGIN_FEATURE_PATHS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
names_per_sample={NAMES_PER_SAMPLE} feature_id_bytes={} \
legacy_result_allocations_per_name=2 optimized_result_allocations_per_name=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        feature_id.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_feature_runtime_crate_name(feature_id: &str) -> String {
    let stem = feature_id
        .chars()
        .map(sanitize_crate_path_character)
        .collect::<String>();
    format!("zircon_plugin_{stem}_runtime")
}

fn measure(optimized: bool, feature_id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..NAMES_PER_SAMPLE {
        let name = if optimized {
            default_feature_runtime_crate_name(black_box(feature_id))
        } else {
            legacy_feature_runtime_crate_name(black_box(feature_id))
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
