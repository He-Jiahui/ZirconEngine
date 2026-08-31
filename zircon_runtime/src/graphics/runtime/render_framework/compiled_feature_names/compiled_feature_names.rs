use crate::graphics::CompiledRenderPipeline;

pub(in crate::graphics::runtime::render_framework) fn compiled_feature_names(
    pipeline: &CompiledRenderPipeline,
) -> Vec<String> {
    let features = pipeline.enabled_features();
    let mut names = Vec::with_capacity(features.len());
    for feature in features {
        names.push(feature.feature_name());
    }
    names
}

#[cfg(test)]
mod optimization_batch_20260830bz_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const FEATURES_PER_SAMPLE: usize = 256;

    #[test]
    fn compiled_feature_names_reserve_enabled_feature_capacity() {
        let source = include_str!("compiled_feature_names.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(features.len())"));
        assert!(implementation.contains("for feature in features"));
        assert!(!implementation.contains(".map(|feature| feature.feature_name())"));
    }

    #[test]
    fn compiled_feature_names_keep_enabled_feature_order() {
        let source = include_str!("compiled_feature_names.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let loop_start = implementation
            .find("for feature in features")
            .expect("feature loop");
        let push = implementation
            .find("names.push(feature.feature_name())")
            .expect("name push");
        assert!(loop_start < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bz_runtime_compiled_feature_names_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME378_COMPILED_FEATURE_NAMES_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} features_per_sample={FEATURES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut names = if optimized {
                Vec::with_capacity(FEATURES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..FEATURES_PER_SAMPLE {
                names.push(index.to_string());
            }
            checksum ^= names.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
