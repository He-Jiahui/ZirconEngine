use std::collections::BTreeSet;
use std::path::Path;

use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::host::EditorError;

use super::parsed_document::ParsedUiAssetImportDocument;
use super::{UiAssetImportDocuments, UiAssetImportGeneration, UiAssetImportResolution};

#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportTraversal {
    documents: UiAssetImportDocuments,
    dependencies: BTreeSet<String>,
    expanded_physical_paths: BTreeSet<std::path::PathBuf>,
    generation: UiAssetImportGeneration,
}

impl UiAssetImportTraversal {
    pub(in crate::ui::host::asset_editor_sessions) fn into_documents(
        self,
    ) -> UiAssetImportDocuments {
        self.documents
    }

    pub(in crate::ui::host::asset_editor_sessions) fn finish_resolution(
        &mut self,
    ) -> UiAssetImportResolution {
        let resolution = UiAssetImportResolution {
            documents: std::mem::take(&mut self.documents),
            dependencies: std::mem::take(&mut self.dependencies),
        };
        self.expanded_physical_paths.clear();
        resolution
    }

    pub(super) fn generation_mut(&mut self) -> &mut UiAssetImportGeneration {
        &mut self.generation
    }

    pub(super) fn record_dependency(&mut self, reference: &str) {
        insert_dependency(
            &mut self.dependencies,
            normalize_ui_asset_asset_id(reference),
        );
    }

    pub(super) fn materialize_reference(
        &mut self,
        reference: &str,
        expected_kind: UiAssetKind,
        physical_path: &Path,
        parsed: &ParsedUiAssetImportDocument,
    ) -> Result<bool, EditorError> {
        if let Some(v2_document) = &parsed.v2_document {
            let actual_kind = super::super::legacy_asset_kind_for_v2(v2_document.asset.kind);
            if actual_kind != expected_kind {
                return Err(EditorError::UiAsset(format!(
                    "ui import {reference} expected {expected_kind:?} but parsed {:?}",
                    v2_document.asset.kind
                )));
            }
        }
        if parsed.document.asset.kind != expected_kind {
            return Err(EditorError::UiAsset(format!(
                "ui import {reference} expected {:?} but parsed {:?}",
                expected_kind, parsed.document.asset.kind
            )));
        }

        match expected_kind {
            UiAssetKind::Widget => {
                self.documents
                    .widgets
                    .insert(reference.to_string(), parsed.document.clone());
                if let Some(v2_document) = &parsed.v2_document {
                    self.documents
                        .v2_widgets
                        .insert(reference.to_string(), v2_document.clone());
                }
            }
            UiAssetKind::Style => {
                self.documents
                    .styles
                    .insert(reference.to_string(), parsed.document.clone());
                if let Some(v2_document) = &parsed.v2_document {
                    self.documents
                        .v2_styles
                        .insert(reference.to_string(), v2_document.clone());
                }
            }
            UiAssetKind::Layout => {}
        }

        Ok(self
            .expanded_physical_paths
            .insert(physical_path.to_path_buf()))
    }
}

fn insert_dependency(dependencies: &mut BTreeSet<String>, dependency: &str) {
    if !dependencies.contains(dependency) {
        dependencies.insert(dependency.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const DEPENDENCY_BYTES: usize = 2_048;
    const INSERTS_PER_SAMPLE: usize = 65_536;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_ft_editor406_reuses_duplicate_dependency_key() {
        let mut dependencies = BTreeSet::new();
        insert_dependency(&mut dependencies, "res://ui/shared.zui");
        insert_dependency(&mut dependencies, "res://ui/shared.zui");

        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains("res://ui/shared.zui"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ft_editor406_borrowed_duplicate_dependency_benchmark() {
        let dependency = format!("res://ui/{}.zui", "d".repeat(DEPENDENCY_BYTES - 13));
        for _ in 0..4 {
            black_box(measure_duplicate_inserts(&dependency, false));
            black_box(measure_duplicate_inserts(&dependency, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_duplicate_inserts(&dependency, false));
                optimized_samples.push(measure_duplicate_inserts(&dependency, true));
            } else {
                optimized_samples.push(measure_duplicate_inserts(&dependency, true));
                legacy_samples.push(measure_duplicate_inserts(&dependency, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR406_BORROWED_DUPLICATE_IMPORT_DEPENDENCY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} inserts_per_sample={INSERTS_PER_SAMPLE} dependency_bytes={} legacy_owned_keys_per_sample={INSERTS_PER_SAMPLE} optimized_owned_keys_per_sample=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
            dependency.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 65 / 100);
    }

    fn measure_duplicate_inserts(dependency: &str, optimized: bool) -> u128 {
        let mut dependencies = BTreeSet::new();
        let started = Instant::now();
        for _ in 0..INSERTS_PER_SAMPLE {
            if optimized {
                insert_dependency(black_box(&mut dependencies), black_box(dependency));
            } else {
                dependencies.insert(black_box(dependency).to_owned());
            }
        }
        black_box(dependencies);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
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
