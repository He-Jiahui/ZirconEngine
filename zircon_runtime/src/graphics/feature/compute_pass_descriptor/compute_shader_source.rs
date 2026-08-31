use zircon_runtime_interface::resource::{AssetReference, ResourceScheme};

use crate::render_graph::RenderGraphComputeShaderSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComputeShaderSource {
    BuiltinWgsl {
        label: &'static str,
        source: &'static str,
    },
    Asset {
        asset: AssetReference,
    },
    InlineWgsl {
        label: String,
        source: String,
    },
}

impl ComputeShaderSource {
    pub const fn builtin_wgsl(label: &'static str, source: &'static str) -> Self {
        Self::BuiltinWgsl { label, source }
    }

    pub fn asset(asset: AssetReference) -> Self {
        Self::Asset { asset }
    }

    pub fn inline_wgsl(label: impl Into<String>, source: impl Into<String>) -> Self {
        Self::InlineWgsl {
            label: label.into(),
            source: source.into(),
        }
    }

    pub(crate) fn pipeline_label(&self) -> String {
        match self {
            Self::BuiltinWgsl { label, .. } => (*label).to_string(),
            Self::Asset { asset } => asset_pipeline_label(asset),
            Self::InlineWgsl { label, .. } => label.clone(),
        }
    }

    pub(crate) fn graph_source(&self) -> RenderGraphComputeShaderSource {
        match self {
            Self::BuiltinWgsl { label, source } => {
                RenderGraphComputeShaderSource::wgsl(*label, *source)
            }
            Self::Asset { asset } => RenderGraphComputeShaderSource::asset(asset.clone()),
            Self::InlineWgsl { label, source } => {
                RenderGraphComputeShaderSource::wgsl(label.clone(), source.clone())
            }
        }
    }
}

fn asset_pipeline_label(asset: &AssetReference) -> String {
    const PREFIX: &str = "compute.asset:";
    let locator = &asset.locator;
    let scheme = match locator.scheme() {
        ResourceScheme::Res => "res",
        ResourceScheme::Library => "lib",
        ResourceScheme::Package => "package",
        ResourceScheme::Builtin => "builtin",
        ResourceScheme::Memory => "mem",
    };
    let label_capacity = locator.label().map_or(0, |label| 1 + label.len());
    let mut output = String::with_capacity(
        PREFIX.len() + scheme.len() + "://".len() + locator.path().len() + label_capacity,
    );
    output.push_str(PREFIX);
    output.push_str(scheme);
    output.push_str("://");
    output.push_str(locator.path());
    if let Some(label) = locator.label() {
        output.push('#');
        output.push_str(label);
    }
    output
}

#[cfg(test)]
mod optimization_batch_fi_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use zircon_runtime_interface::resource::ResourceLocator;

    const SAMPLE_PAIRS: usize = 17;
    const LABELS_PER_SAMPLE: usize = 262_144;

    #[test]
    fn optimization_batch_fi_runtime465_compute_asset_label_preserves_locator_bytes() {
        for locator in [
            "res://shaders/clustered.wgsl",
            "lib://cache/shaders/reduce.wgsl#main",
            "package://zircon.render/shaders/hzb.wgsl#build",
            "builtin://shaders/ui.wgsl",
            "mem://generated/post_process.wgsl#entry",
        ] {
            let asset = AssetReference::from_locator(ResourceLocator::parse(locator).unwrap());
            let source = ComputeShaderSource::asset(asset.clone());
            assert_eq!(source.pipeline_label(), legacy_asset_pipeline_label(&asset));
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fi_runtime465_direct_compute_asset_label_benchmark() {
        let asset = AssetReference::from_locator(
            ResourceLocator::parse(
                "package://zircon.render/shaders/post_process/clustered_reduce.wgsl#main",
            )
            .unwrap(),
        );
        for _ in 0..4 {
            black_box(measure(legacy_asset_pipeline_label, &asset));
            black_box(measure(asset_pipeline_label, &asset));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure(legacy_asset_pipeline_label, &asset));
                optimized_samples.push(measure(asset_pipeline_label, &asset));
            } else {
                optimized_samples.push(measure(asset_pipeline_label, &asset));
                legacy_samples.push(measure(legacy_asset_pipeline_label, &asset));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn legacy_asset_pipeline_label(asset: &AssetReference) -> String {
        format!("compute.asset:{asset}")
    }

    fn measure(mut build: impl FnMut(&AssetReference) -> String, asset: &AssetReference) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..LABELS_PER_SAMPLE {
            checksum = checksum.wrapping_add(black_box(build(black_box(asset))).len());
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME465_DIRECT_COMPUTE_ASSET_LABEL_BENCH_V1 sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized p95 {optimized_p95}ns must be at most 75% of legacy p95 {legacy_p95}ns"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
