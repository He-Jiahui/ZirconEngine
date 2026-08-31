use std::collections::HashSet;

use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

mod manifest_metadata;

use self::manifest_metadata::register_package_manifest_metadata_contributions;

pub(in crate::plugin::runtime_plugin::registration_report) fn register_package_manifest_contributions(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    // Manifest rows may mirror direct runtime registrations; validate them before
    // ignoring duplicate ids so malformed package metadata cannot be shadowed.
    register_package_manifest_metadata_contributions(package_manifest, extensions, diagnostics);
    if package_manifest.asset_importers.is_empty() {
        return;
    }
    let mut registered_importer_ids = extensions
        .asset_importers()
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.id)
        .collect::<HashSet<_>>();
    for importer in package_manifest.asset_importers.iter().cloned() {
        if registered_importer_ids.contains(importer.id.as_str()) {
            validate_duplicate_package_asset_importer(importer, diagnostics);
            continue;
        }
        let importer_id = importer.id.clone();
        match extensions.register_asset_importer_descriptor(importer) {
            Ok(()) => {
                registered_importer_ids.insert(importer_id);
            }
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

fn validate_duplicate_package_asset_importer(
    importer: crate::asset::AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    let mut validation_registry = RuntimeExtensionRegistry::default();
    if let Err(error) = validation_registry.register_asset_importer_descriptor(importer) {
        diagnostics.push(error.to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::{AssetImporterDescriptor, AssetKind};

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const IMPORTER_COUNT: usize = 512;

    #[test]
    fn optimization_batch_eu_runtime453_preserves_duplicate_and_retry_admission() {
        let mut extensions = RuntimeExtensionRegistry::default();
        extensions
            .register_asset_importer_descriptor(importer("existing", "existing"))
            .unwrap();
        let mut manifest = PluginPackageManifest::new("package", "Package");
        manifest.asset_importers = vec![
            importer("existing", "existing"),
            AssetImporterDescriptor::new("retry", "package", AssetKind::Data, 1),
            importer("retry", "retry"),
            importer("added", "added"),
            importer("added", "added"),
        ];
        let mut diagnostics = Vec::new();

        register_package_manifest_contributions(&manifest, &mut extensions, &mut diagnostics);

        let importer_ids = extensions
            .asset_importers()
            .descriptors()
            .into_iter()
            .map(|descriptor| descriptor.id)
            .collect::<HashSet<_>>();
        assert_eq!(importer_ids.len(), 3);
        assert!(importer_ids.contains("existing"));
        assert!(importer_ids.contains("retry"));
        assert!(importer_ids.contains("added"));
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("must declare at least one source extension"));
    }

    #[test]
    fn optimization_batch_eu_runtime453_builds_one_importer_id_index() {
        let production = include_str!("package_contributions.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("collect::<HashSet<_>>()"));
        assert_eq!(production.matches(".descriptors()").count(), 1);
        assert!(production.contains("registered_importer_ids.contains"));
        assert!(production.contains("package_manifest.asset_importers.is_empty()"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_eu_runtime453_importer_id_index_benchmark() {
        let mut extensions = RuntimeExtensionRegistry::default();
        for index in 0..IMPORTER_COUNT {
            extensions
                .register_asset_importer_descriptor(importer(
                    format!("importer.{index:04}"),
                    format!("type_{index:04}"),
                ))
                .unwrap();
        }
        let probes = (0..IMPORTER_COUNT)
            .rev()
            .map(|index| importer(format!("importer.{index:04}"), format!("type_{index:04}")))
            .collect::<Vec<_>>();

        for _ in 0..3 {
            black_box(measure_legacy_membership(&extensions, &probes));
            black_box(measure_indexed_membership(&extensions, &probes));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy_membership(&extensions, &probes));
                optimized_samples.push(measure_indexed_membership(&extensions, &probes));
            } else {
                optimized_samples.push(measure_indexed_membership(&extensions, &probes));
                legacy_samples.push(measure_legacy_membership(&extensions, &probes));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn importer(id: impl Into<String>, extension: impl Into<String>) -> AssetImporterDescriptor {
        AssetImporterDescriptor::new(id, "package", AssetKind::Data, 1)
            .with_source_extensions([extension])
    }

    fn measure_legacy_membership(
        extensions: &RuntimeExtensionRegistry,
        probes: &[AssetImporterDescriptor],
    ) -> u128 {
        let started = Instant::now();
        let mut matches = 0_usize;
        for probe in probes {
            matches += usize::from(
                extensions
                    .asset_importers()
                    .descriptors()
                    .iter()
                    .any(|existing| existing.id == probe.id),
            );
        }
        assert_eq!(black_box(matches), probes.len());
        started.elapsed().as_nanos().max(1)
    }

    fn measure_indexed_membership(
        extensions: &RuntimeExtensionRegistry,
        probes: &[AssetImporterDescriptor],
    ) -> u128 {
        let started = Instant::now();
        let importer_ids = extensions
            .asset_importers()
            .descriptors()
            .into_iter()
            .map(|descriptor| descriptor.id)
            .collect::<HashSet<_>>();
        let matches = probes
            .iter()
            .filter(|probe| importer_ids.contains(probe.id.as_str()))
            .count();
        assert_eq!(black_box(matches), probes.len());
        started.elapsed().as_nanos().max(1)
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME453_IMPORTER_ID_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} importer_count={IMPORTER_COUNT} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=80",
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(20) / 100,
            "one importer id index must reduce P95 by at least 80%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
