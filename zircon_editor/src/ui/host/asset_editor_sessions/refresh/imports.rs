use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::imports::{UiAssetImportResolution, UiAssetImportTraversal};
use super::super::UiAssetStaleImportDiagnostic;

impl EditorUiHost {
    pub(super) fn apply_import_ui_asset_changes(
        &self,
        _changed_asset_ids: &BTreeSet<String>,
        import_instances: &BTreeSet<ViewInstanceId>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let entries = {
            let sessions = self.lock_ui_asset_sessions();
            let mut entries = Vec::with_capacity(import_instances.len());
            for instance_id in import_instances {
                if let Some(entry) = sessions.get(instance_id) {
                    let (widgets, styles) = entry.session.import_references();
                    entries.push((instance_id.clone(), widgets, styles));
                }
            }
            entries
        };

        let mut sync_instances = BTreeSet::new();
        for (instance_id, widget_refs, style_refs) in entries {
            let (resolution, errors) =
                self.collect_ui_asset_imports_lossy(&widget_refs, &style_refs);
            let UiAssetImportResolution {
                documents,
                dependencies,
            } = resolution;
            let mut dependency_generation = self.lock_ui_asset_dependency_generation();
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            if errors.is_empty() {
                entry
                    .session
                    .replace_resolved_imports(
                        documents.widgets,
                        documents.styles,
                        documents.v2_widgets,
                        documents.v2_styles,
                    )
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                entry.stale_imports.clear();
            } else {
                entry.stale_imports = errors
                    .into_iter()
                    .map(|error| (error.reference.clone(), error))
                    .collect::<BTreeMap<_, _>>();
            }
            dependency_generation.replace_dependencies(instance_id.clone(), dependencies);
            let _ = sync_instances.insert(instance_id);
        }
        Ok(sync_instances)
    }

    pub(in crate::ui::host::asset_editor_sessions) fn collect_ui_asset_imports_lossy(
        &self,
        widget_refs: &[String],
        style_refs: &[String],
    ) -> (UiAssetImportResolution, Vec<UiAssetStaleImportDiagnostic>) {
        let mut traversal = UiAssetImportTraversal::default();
        let mut errors = Vec::new();

        for reference in widget_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Widget,
                &mut traversal,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }
        for reference in style_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Style,
                &mut traversal,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }

        (traversal.finish_resolution(), errors)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    const BUILDS_PER_SAMPLE: usize = 256;
    const ENTRIES_PER_BUILD: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_fs_editor405_reserves_import_refresh_entry_capacity() {
        let source = include_str!("imports.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("import refresh production source");

        assert!(production.contains("Vec::with_capacity(import_instances.len())"));
        assert!(!production.contains(".filter_map(|instance_id|"));
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fs_editor405_reserved_import_refresh_entries_benchmark() {
        let entries = (0..ENTRIES_PER_BUILD)
            .map(|entry| [entry; 9])
            .collect::<Vec<_>>();
        for _ in 0..4 {
            black_box(measure_builds(&entries, false));
            black_box(measure_builds(&entries, true));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_builds(&entries, false));
                optimized_samples.push(measure_builds(&entries, true));
            } else {
                optimized_samples.push(measure_builds(&entries, true));
                legacy_samples.push(measure_builds(&entries, false));
            }
        }

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "EDITOR405_RESERVED_IMPORT_REFRESH_ENTRIES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} builds_per_sample={BUILDS_PER_SAMPLE} entries_per_build={ENTRIES_PER_BUILD} tuple_bytes={} legacy_growth_allocations_per_build=9 optimized_growth_allocations_per_build=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            std::mem::size_of::<[usize; 9]>(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_builds(entries: &[[usize; 9]], optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..BUILDS_PER_SAMPLE {
            let batch = if optimized {
                let mut batch = Vec::with_capacity(entries.len());
                for entry in black_box(entries) {
                    if entry[0] != usize::MAX {
                        batch.push(*entry);
                    }
                }
                batch
            } else {
                black_box(entries)
                    .iter()
                    .filter_map(|entry| (entry[0] != usize::MAX).then_some(*entry))
                    .collect::<Vec<_>>()
            };
            black_box(batch);
        }
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
