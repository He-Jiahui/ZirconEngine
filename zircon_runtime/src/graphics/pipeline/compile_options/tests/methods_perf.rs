use std::hint::black_box;
use std::time::Instant;

use crate::graphics::feature::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};
use crate::graphics::pipeline::declarations::{RenderPipelineCompileOptions, RendererFeatureAsset};

const SAMPLE_PAIRS: usize = 17;
const CHECKS_PER_SAMPLE: usize = 512;
const DESCRIPTOR_SECTION_COUNT: usize = 128;

#[test]
fn optimization_batch_df_runtime413_borrowed_plugin_admission_matches_legacy_semantics() {
    let feature = plugin_feature();
    let enabled = RenderPipelineCompileOptions::default()
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    let disabled = enabled
        .clone()
        .with_plugin_feature_disabled("plugin.deep-feature");

    assert_eq!(
        enabled.permits_feature_asset(&feature),
        legacy_permits_feature_asset(&enabled, &feature)
    );
    assert_eq!(
        disabled.permits_feature_asset(&feature),
        legacy_permits_feature_asset(&disabled, &feature)
    );
    assert!(enabled.permits_feature_asset(&feature));
    assert!(!disabled.permits_feature_asset(&feature));
    assert!(!RenderPipelineCompileOptions::default().permits_feature_asset(&feature));

    let renamed = RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
        "plugin.source-name",
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
    .with_descriptor_override(RenderFeatureDescriptor::new(
        "plugin.override-name",
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ));
    let override_disabled = RenderPipelineCompileOptions::default()
        .with_plugin_feature_disabled("plugin.override-name");
    assert_eq!(
        override_disabled.permits_feature_asset(&renamed),
        legacy_permits_feature_asset(&override_disabled, &renamed)
    );
    assert!(!override_disabled.permits_feature_asset(&renamed));
}

#[test]
fn optimization_batch_df_runtime413_plugin_admission_borrows_name_and_descriptor() {
    let source = include_str!("../methods.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();
    let admission = production
        .split("pub(in crate::graphics::pipeline) fn permits_feature_asset")
        .nth(1)
        .unwrap()
        .split("pub(in crate::graphics::pipeline) fn resolve_queue")
        .next()
        .unwrap();
    let plugin_admission = admission
        .split("let RendererFeatureSource::Plugin(feature_name)")
        .nth(1)
        .unwrap();

    assert!(admission.contains("RendererFeatureSource::Plugin(feature_name)"));
    assert!(plugin_admission.contains("descriptor_override"));
    assert!(plugin_admission.contains(".as_ref()"));
    assert!(!plugin_admission.contains("feature.feature_name()"));
    assert!(!plugin_admission.contains("feature.descriptor()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_df_runtime413_borrowed_plugin_admission_p95() {
    let feature = plugin_feature();
    let options = RenderPipelineCompileOptions::default()
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&options, &feature, false));
            optimized.push(measure(&options, &feature, true));
        } else {
            optimized.push(measure(&options, &feature, true));
            legacy.push(measure(&options, &feature, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME413_PLUGIN_FEATURE_BORROWED_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} descriptor_sections={DESCRIPTOR_SECTION_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed plugin admission must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn plugin_feature() -> RendererFeatureAsset {
    let required_extract_sections = (0..DESCRIPTOR_SECTION_COUNT)
        .map(|index| format!("plugin.deep-feature.extract.section.{index:03}"))
        .collect();
    let descriptor = RenderFeatureDescriptor::new(
        "plugin.deep-feature",
        required_extract_sections,
        Vec::new(),
        Vec::new(),
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry);
    RendererFeatureAsset::plugin(descriptor)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::BufferReadback)
}

fn legacy_permits_feature_asset(
    options: &RenderPipelineCompileOptions,
    feature: &RendererFeatureAsset,
) -> bool {
    if options
        .disabled_plugin_features
        .contains(&feature.feature_name())
    {
        return false;
    }
    let descriptor = feature.descriptor();
    black_box(&descriptor.required_extract_sections);
    feature
        .capability_requirements
        .iter()
        .chain(descriptor.capability_requirements.iter())
        .all(|requirement| options.permits_capability_requirement(*requirement))
}

fn measure(
    options: &RenderPipelineCompileOptions,
    feature: &RendererFeatureAsset,
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        let permitted = if optimized {
            options.permits_feature_asset(black_box(feature))
        } else {
            legacy_permits_feature_asset(options, black_box(feature))
        };
        black_box(permitted);
    }
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() - 1) * percentile / 100]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
