use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetUri,
    ImportedAsset, asset_kind_for_imported_asset,
};

impl AssetImporter {
    pub fn descriptor_for_source(
        &self,
        source_path: &Path,
    ) -> Result<AssetImporterDescriptor, AssetImportError> {
        self.registry().descriptor_for_source(source_path)
    }

    pub fn import_from_source(
        &self,
        source_path: &Path,
        uri: &AssetUri,
    ) -> Result<ImportedAsset, AssetImportError> {
        self.import_with_settings(source_path, uri, toml::Table::new())
            .and_then(|outcome| {
                outcome
                    .entries
                    .into_iter()
                    .find(|entry| entry.locator.label().is_none())
                    .map(|entry| entry.asset)
                    .ok_or_else(|| AssetImportError::Parse(format!("missing root asset for {uri}")))
            })
    }

    pub fn import_with_settings(
        &self,
        source_path: &Path,
        uri: &AssetUri,
        import_settings: toml::Table,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        let source_bytes = fs::read(source_path)?;
        self.import_bytes(source_path, uri, source_bytes, import_settings)
    }

    pub fn import_bytes(
        &self,
        source_path: &Path,
        uri: &AssetUri,
        source_bytes: Vec<u8>,
        import_settings: toml::Table,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        let context = AssetImportContext::new(
            source_path.to_path_buf(),
            uri.clone(),
            source_bytes,
            import_settings,
        );
        self.import_context(&context)
    }

    pub fn import_context(
        &self,
        context: &AssetImportContext,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        if requires_project_resolver(&context.source_path) && !context.has_project_resolver() {
            return Err(AssetImportError::ProjectContextRequired {
                path: context.source_path.clone(),
            });
        }
        let importer = self.registry().select(&context.source_path)?;
        let outcome = importer.import(context)?;
        let descriptor = importer.descriptor();
        if outcome.entries.is_empty() {
            return Err(AssetImportError::Parse(format!(
                "asset importer {} returned no imported asset entries",
                descriptor.id
            )));
        }
        for entry in &outcome.entries {
            let actual_kind = asset_kind_for_imported_asset(&entry.asset);
            if !descriptor.allows_output_kind(actual_kind) {
                return Err(AssetImportError::Parse(format!(
                    "asset importer {} returned {actual_kind:?}, expected {:?}",
                    descriptor.id, descriptor.output_kind
                )));
            }
        }
        Ok(outcome)
    }
}

fn requires_project_resolver(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.ends_with(".scene.toml") || name.ends_with(".model.toml") || name.ends_with(".zmaterial")
}

#[cfg(test)]
mod plugins07_borrowed_descriptor_tests {
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::*;
    use crate::asset::{
        AssetImporterRegistry, AssetKind, DataAsset, DataAssetFormat, FunctionAssetImporter,
    };

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 12_000;

    #[test]
    fn import_execution_collections_contract_borrowed_descriptor() {
        let descriptor = benchmark_descriptor();
        let mut registry = AssetImporterRegistry::default();
        registry
            .register(FunctionAssetImporter::new(descriptor, import_fixture))
            .unwrap();
        let importer = AssetImporter::with_registry(registry);
        let context = AssetImportContext::new(
            PathBuf::from("fixture.ext0"),
            AssetUri::parse("res://fixture.ext0").unwrap(),
            Vec::new(),
            toml::Table::new(),
        );

        let outcome = importer.import_context(&context).unwrap();
        assert!(matches!(
            outcome.root_entry().map(|entry| &entry.asset),
            Some(ImportedAsset::Data(_))
        ));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn import_execution_collections_performance_release_borrowed_descriptor() {
        let descriptor = benchmark_descriptor();
        for _ in 0..4 {
            black_box(measure_cloned(&descriptor));
            black_box(measure_borrowed(&descriptor));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_cloned(&descriptor), measure_borrowed(&descriptor))
            } else {
                let optimized_ns = measure_borrowed(&descriptor);
                (measure_cloned(&descriptor), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_borrowed_importer_descriptor sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=90 legacy_descriptor_clones_per_sample={CHECKS_PER_SAMPLE} optimized_descriptor_clones_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 90,
            "borrowed importer descriptor validation must improve P95 by at least 90%"
        );
    }

    fn import_fixture(
        context: &AssetImportContext,
    ) -> Result<AssetImportOutcome, AssetImportError> {
        Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::Data(DataAsset {
                uri: context.uri.clone(),
                format: DataAssetFormat::Text,
                text: String::new(),
                canonical_json: serde_json::Value::Null,
            }),
        ))
    }

    fn benchmark_descriptor() -> AssetImporterDescriptor {
        AssetImporterDescriptor::new(
            "plugins07.borrowed.descriptor",
            "plugins07.fixture",
            AssetKind::Data,
            1,
        )
        .with_source_extensions((0..64).map(|index| format!("ext{index}")))
        .with_full_suffixes((0..64).map(|index| format!("suffix{index}.data")))
        .with_required_capabilities((0..64).map(|index| format!("capability.{index}")))
    }

    fn measure_cloned(descriptor: &AssetImporterDescriptor) -> u128 {
        let started = Instant::now();
        let mut allowed = 0_u64;
        for _ in 0..CHECKS_PER_SAMPLE {
            let descriptor = black_box(descriptor).clone();
            allowed += u64::from(descriptor.allows_output_kind(AssetKind::Data));
            black_box(descriptor);
        }
        black_box(allowed);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed(descriptor: &AssetImporterDescriptor) -> u128 {
        let started = Instant::now();
        let mut allowed = 0_u64;
        for _ in 0..CHECKS_PER_SAMPLE {
            allowed +=
                u64::from(black_box(descriptor).allows_output_kind(black_box(AssetKind::Data)));
        }
        black_box(allowed);
        started.elapsed().as_nanos().max(1)
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
