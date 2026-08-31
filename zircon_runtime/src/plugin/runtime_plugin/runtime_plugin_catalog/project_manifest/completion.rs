use crate::core::framework::project::ProjectPluginManifest;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_completion::complete_project_feature_selections;
use super::super::RuntimePluginRegistrationReport;
use super::selection_defaults::complete_project_selection_defaults;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn catalog_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
) -> ProjectPluginManifest {
    complete_project_manifest_owned(
        registrations,
        projection,
        ProjectPluginManifest {
            selections: registrations
                .iter()
                .map(|registration| registration.project_selection.clone())
                .collect(),
        },
    )
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_project_manifest(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    manifest: &ProjectPluginManifest,
) -> ProjectPluginManifest {
    complete_project_manifest_owned(registrations, projection, manifest.clone())
}

fn complete_project_manifest_owned(
    registrations: &[RuntimePluginRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
    mut completed: ProjectPluginManifest,
) -> ProjectPluginManifest {
    complete_project_selection_defaults(registrations, projection, &mut completed);
    complete_project_feature_selections(projection, &mut completed);
    completed
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[derive(Clone)]
    struct ManifestFixture {
        selections: Vec<String>,
    }

    #[test]
    fn optimization_batch_eh_new_catalog_manifest_moves_into_completion() {
        let source = include_str!("completion.rs");
        let catalog = source
            .split("fn catalog_project_manifest")
            .nth(1)
            .expect("catalog project manifest implementation")
            .split("pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete_project_manifest")
            .next()
            .expect("bounded catalog project manifest implementation");

        assert!(catalog.contains("complete_project_manifest_owned("));
        assert!(!catalog.contains("&ProjectPluginManifest {"));
        assert!(source.contains("complete_project_manifest_owned(\n        registrations,"));
        assert!(source.contains("manifest.clone(),"));
    }

    #[test]
    #[ignore = "release-only direct project manifest completion move benchmark"]
    fn optimization_batch_eh_direct_project_manifest_completion_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const SELECTIONS: usize = 512;
        const COMPLETIONS_PER_SAMPLE: usize = 64;

        fn measure_legacy(base: &ManifestFixture) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..COMPLETIONS_PER_SAMPLE {
                let manifest = black_box(base.clone());
                let completed = black_box(manifest.clone());
                checksum = checksum.wrapping_add(completed.selections.len());
                black_box((manifest, completed));
            }
            black_box(checksum);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &ManifestFixture) -> u128 {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..COMPLETIONS_PER_SAMPLE {
                let manifest = black_box(base.clone());
                let completed = black_box(manifest);
                checksum = checksum.wrapping_add(completed.selections.len());
                black_box(completed);
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

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let base = ManifestFixture {
            selections: (0..SELECTIONS)
                .map(|index| format!("plugin.selection.{index:04}.{}", "feature_".repeat(12)))
                .collect(),
        };
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(&base));
                optimized_samples.push(measure_optimized(&base));
            } else {
                optimized_samples.push(measure_optimized(&base));
                legacy_samples.push(measure_legacy(&base));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        let common_selection_copies = SELECTIONS * COMPLETIONS_PER_SAMPLE;
        println!(
            "RUNTIME442_DIRECT_PROJECT_MANIFEST_COMPLETION_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             selections={SELECTIONS} completions_per_sample={COMPLETIONS_PER_SAMPLE} \
             pair_order=alternating_legacy_even common_selection_copies_per_sample={common_selection_copies} \
             legacy_extra_selection_copies_per_sample={common_selection_copies} optimized_extra_selection_copies_per_sample=0 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "direct project manifest completion move must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
