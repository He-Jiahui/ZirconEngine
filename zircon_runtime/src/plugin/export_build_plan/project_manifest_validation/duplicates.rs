use crate::core::framework::project::ProjectPluginManifest;

use super::ProjectPluginManifestValidationProjection;

pub(in crate::plugin::export_build_plan) fn project_duplicate_selection_diagnostics(
    manifest: &ProjectPluginManifest,
    projection: &ProjectPluginManifestValidationProjection,
) -> (Vec<String>, Vec<String>) {
    let mut diagnostics = Vec::with_capacity(manifest.selections.len());
    let mut fatal_diagnostics = Vec::with_capacity(manifest.selections.len());
    for (selection_index, selection) in manifest.selections.iter().enumerate() {
        if let Some(first_required) = projection.duplicate_selection_first_required(selection_index)
        {
            let diagnostic = format!(
                "project plugin selection id `{}` is declared more than once",
                selection.id
            );
            if selection.required || first_required {
                fatal_diagnostics.push(diagnostic.clone());
            }
            diagnostics.push(diagnostic);
        }

        for (feature_index, feature) in selection.features.iter().enumerate() {
            if let Some(first_required) =
                projection.duplicate_feature_first_required(selection_index, feature_index)
            {
                let diagnostic = format!(
                    "project plugin feature id `{}` is declared more than once under project plugin `{}`",
                    feature.id, selection.id
                );
                if feature.required || first_required {
                    fatal_diagnostics.push(diagnostic.clone());
                }
                diagnostics.push(diagnostic);
            }
        }
    }
    (diagnostics, fatal_diagnostics)
}

#[cfg(test)]
mod optimization_batch_20260830bt_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const SELECTIONS_PER_SAMPLE: usize = 1_024;

    #[test]
    fn duplicate_diagnostics_reserve_selection_capacity() {
        let source = include_str!("duplicates.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert_eq!(
            implementation
                .matches("Vec::with_capacity(manifest.selections.len())")
                .count(),
            2
        );
        assert!(!implementation.contains("let mut diagnostics = Vec::new()"));
        assert!(!implementation.contains("let mut fatal_diagnostics = Vec::new()"));
    }

    #[test]
    fn duplicate_diagnostics_keep_selection_then_feature_scan_order() {
        let source = include_str!("duplicates.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let reserve = implementation
            .find("Vec::with_capacity(manifest.selections.len())")
            .expect("diagnostic capacity reservation");
        let selection_loop = implementation
            .find("for (selection_index, selection) in manifest.selections.iter().enumerate()")
            .expect("selection loop");
        let feature_loop = implementation
            .find("for (feature_index, feature) in selection.features.iter().enumerate()")
            .expect("feature loop");
        assert!(reserve < selection_loop);
        assert!(selection_loop < feature_loop);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bt_runtime_duplicate_diagnostics_capacity_p95() {
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
            "RUNTIME372_DUPLICATE_DIAGNOSTICS_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} selections_per_sample={SELECTIONS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut diagnostics = if optimized {
                Vec::with_capacity(SELECTIONS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..SELECTIONS_PER_SAMPLE {
                diagnostics.push(index);
            }
            checksum ^= diagnostics.len();
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
