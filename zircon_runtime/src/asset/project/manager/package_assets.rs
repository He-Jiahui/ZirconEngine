use std::path::Path;

use crate::asset::AssetImportError;

use super::ProjectManager;

impl ProjectManager {
    pub fn register_package_asset_root(
        &mut self,
        package_id: impl Into<String>,
        assets_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError> {
        let mut package_assets = std::mem::take(&mut self.package_assets);
        if let Err(error) = package_assets.register_root(package_id, assets_root) {
            self.package_assets = package_assets;
            return Err(error);
        }
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }

    pub fn register_package_asset_roots<Root>(
        &mut self,
        package_id: impl Into<String>,
        asset_roots: impl IntoIterator<Item = Root>,
        package_root: impl AsRef<Path>,
    ) -> Result<(), AssetImportError>
    where
        Root: AsRef<str>,
    {
        let mut package_assets = std::mem::take(&mut self.package_assets);
        if let Err(error) =
            package_assets.register_package_roots(package_id, asset_roots, package_root)
        {
            self.package_assets = package_assets;
            return Err(error);
        }
        let catalog_input_generation =
            super::super::ProjectCatalogInputGeneration::publish_metadata(
                &self.catalog_input_generation,
                self.paths.root(),
                &self.manifest,
                &package_assets,
            );
        self.package_assets = package_assets;
        self.catalog_input_generation = catalog_input_generation;
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    #[derive(Clone, Default)]
    struct RegistryFixture {
        roots: BTreeMap<String, PathBuf>,
    }

    #[test]
    fn optimization_batch_ei_package_asset_registration_stages_by_move() {
        let source = include_str!("package_assets.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("package asset registration implementation");

        assert_eq!(
            implementation
                .matches("std::mem::take(&mut self.package_assets)")
                .count(),
            2
        );
        assert!(!implementation.contains("self.package_assets.clone()"));
        assert_eq!(
            implementation
                .matches("self.package_assets = package_assets;")
                .count(),
            4
        );
    }

    #[test]
    #[ignore = "release-only package asset registry staging move benchmark"]
    fn optimization_batch_ei_package_asset_registry_staging_move_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const ROOTS: usize = 256;
        const REGISTRATIONS_PER_SAMPLE: usize = 256;

        fn measure_legacy(base: &RegistryFixture) -> u128 {
            let mut registry = base.clone();
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..REGISTRATIONS_PER_SAMPLE {
                let staged = black_box(registry.clone());
                checksum = checksum.wrapping_add(staged.roots.len());
                registry = black_box(staged);
            }
            black_box((registry, checksum));
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(base: &RegistryFixture) -> u128 {
            let mut registry = base.clone();
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..REGISTRATIONS_PER_SAMPLE {
                let staged = black_box(std::mem::take(&mut registry));
                checksum = checksum.wrapping_add(staged.roots.len());
                registry = black_box(staged);
            }
            black_box((registry, checksum));
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

        let base = RegistryFixture {
            roots: (0..ROOTS)
                .map(|index| {
                    (
                        format!("com.zircon.package.{index:04}.{}", "feature.".repeat(8)),
                        PathBuf::from(format!(
                            "E:/ZirconPackages/{index:04}/{}assets",
                            "nested/".repeat(16)
                        )),
                    )
                })
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
        let legacy_entry_copies = ROOTS * REGISTRATIONS_PER_SAMPLE;
        println!(
            "RUNTIME443_PACKAGE_ASSET_REGISTRY_STAGING_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             roots={ROOTS} registrations_per_sample={REGISTRATIONS_PER_SAMPLE} \
             pair_order=alternating_legacy_even legacy_registry_entry_copies_per_sample={legacy_entry_copies} \
             optimized_registry_entry_copies_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20),
            "moving package asset registry staging must reduce P95 by at least 80%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
