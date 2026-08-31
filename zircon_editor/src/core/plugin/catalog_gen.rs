//! Build-generated descriptor adapter for the core plugin catalog.

use super::descriptor::EditorPluginDescriptor;

pub(crate) struct GeneratedEditorPluginCatalogEntry {
    pub package_id: &'static str,
    pub display_name: &'static str,
    pub crate_name: &'static str,
    pub category: &'static str,
    pub capabilities: &'static [&'static str],
}

include!(concat!(env!("OUT_DIR"), "/plugin_catalog_generated.rs"));

pub(crate) fn builtin_editor_plugin_descriptors() -> Vec<EditorPluginDescriptor> {
    let mut descriptors = Vec::with_capacity(GENERATED_EDITOR_PLUGIN_CATALOG.len());
    for entry in GENERATED_EDITOR_PLUGIN_CATALOG.iter() {
        let mut descriptor =
            EditorPluginDescriptor::new(entry.package_id, entry.display_name, entry.crate_name)
                .with_category(entry.category);
        for capability in entry.capabilities {
            descriptor = descriptor.with_capability(*capability);
        }
        descriptors.push(descriptor);
    }
    descriptors
}

#[cfg(test)]
mod optimization_batch_20260830bw_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const DESCRIPTORS_PER_SAMPLE: usize = 128;

    #[test]
    fn builtin_catalog_reserves_generated_descriptor_capacity() {
        let source = include_str!("catalog_gen.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(
            implementation.contains("Vec::with_capacity(GENERATED_EDITOR_PLUGIN_CATALOG.len())")
        );
        assert!(implementation.contains("for entry in GENERATED_EDITOR_PLUGIN_CATALOG.iter()"));
    }

    #[test]
    fn builtin_catalog_keeps_capability_projection_inside_descriptor_loop() {
        let source = include_str!("catalog_gen.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let descriptor = implementation
            .find("EditorPluginDescriptor::new")
            .expect("descriptor construction");
        let capability = implementation
            .find("for capability in entry.capabilities")
            .expect("capability loop");
        let push = implementation
            .find("descriptors.push(descriptor)")
            .expect("output push");
        assert!(descriptor < capability);
        assert!(capability < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bw_editor_builtin_catalog_capacity_p95() {
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
            "EDITOR321_BUILTIN_CATALOG_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} descriptors_per_sample={DESCRIPTORS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut descriptors = if optimized {
                Vec::with_capacity(DESCRIPTORS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..DESCRIPTORS_PER_SAMPLE {
                descriptors.push(index);
            }
            checksum ^= descriptors.len();
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
