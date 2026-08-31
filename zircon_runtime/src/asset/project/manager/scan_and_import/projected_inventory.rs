use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::asset::project::{AssetMetaDocument, ProjectGenerationObservation};
use crate::asset::registry::{AssetRegistryDiagnostic, AssetRegistryIndex};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::{AssetImportError, AssetKind};
use crate::core::resource::ResourceScheme;

use super::sources::AssetImportSource;
use super::ProjectManager;

pub(super) struct ProjectedMetaInventory {
    originals: BTreeMap<PathBuf, Option<AssetMetaDocument>>,
    documents: BTreeMap<PathBuf, AssetMetaDocument>,
    identity_changes: Vec<AssetChange>,
}

impl ProjectedMetaInventory {
    pub(super) fn load(
        manager: &ProjectManager,
        sources: &[AssetImportSource],
        observation: &mut ProjectGenerationObservation,
    ) -> Result<Self, AssetImportError> {
        let mut originals = BTreeMap::new();
        let mut documents = BTreeMap::new();
        let mut identity_changes = Vec::new();

        for source in sources {
            let fallback_kind = manager
                .importer
                .descriptor_for_source(&source.path)
                .map(|descriptor| descriptor.output_kind)
                .unwrap_or(AssetKind::Data);
            let original = if source.meta_path.exists() {
                Some(observation.load_metadata_document(&source.meta_path)?)
            } else {
                None
            };
            let mut projected = original.clone().unwrap_or_else(|| {
                AssetMetaDocument::new(
                    crate::asset::AssetUuid::new(),
                    source.uri.clone(),
                    fallback_kind,
                )
            });
            if let Some(previous) = original
                .as_ref()
                .filter(|previous| previous.url != source.uri)
            {
                identity_changes.push(AssetChange::new(
                    AssetChangeKind::Renamed,
                    source.uri.clone(),
                    Some(previous.url.clone()),
                ));
            }
            projected.url = source.uri.clone();
            projected.asset_kind = fallback_kind;
            originals.insert(source.meta_path.clone(), original);
            documents.insert(source.meta_path.clone(), projected);
        }

        observation.record_metadata_inventory(
            documents.len(),
            originals
                .values()
                .filter(|document| document.is_some())
                .count(),
        );
        Ok(Self {
            originals,
            documents,
            identity_changes,
        })
    }

    pub(super) fn normalize_duplicate_guids(
        &mut self,
        registry: &AssetRegistryIndex,
        watch_changes: Option<&[AssetChange]>,
    ) -> Vec<AssetRegistryDiagnostic> {
        let identity_changes = merged_identity_changes(&self.identity_changes, watch_changes);
        let changes = (!identity_changes.is_empty()).then_some(identity_changes.as_ref());
        registry.prepare_duplicate_guids_from_loaded(&mut self.documents, changes)
    }

    pub(super) fn document(&self, meta_path: &Path) -> &AssetMetaDocument {
        self.documents
            .get(meta_path)
            .expect("every collected source owns one projected metadata document")
    }

    pub(super) fn document_mut(&mut self, meta_path: &Path) -> &mut AssetMetaDocument {
        self.documents
            .get_mut(meta_path)
            .expect("every collected source owns one projected metadata document")
    }

    pub(super) fn preconditions(
        &self,
    ) -> impl Iterator<Item = (&PathBuf, Option<&AssetMetaDocument>)> {
        self.originals
            .iter()
            .map(|(path, original)| (path, original.as_ref()))
    }

    pub(super) fn project_documents(&self) -> impl Iterator<Item = &AssetMetaDocument> {
        self.documents
            .values()
            .filter(|document| document.url.scheme() == ResourceScheme::Res)
    }

    pub(super) fn documents(&self) -> impl Iterator<Item = &AssetMetaDocument> {
        self.documents.values()
    }

    pub(super) fn changed_documents(&self) -> impl Iterator<Item = (&PathBuf, &AssetMetaDocument)> {
        self.documents.iter().filter(|(path, document)| {
            self.originals.get(*path).and_then(Option::as_ref) != Some(*document)
        })
    }
}

fn merged_identity_changes<'a>(
    identity_changes: &'a [AssetChange],
    watch_changes: Option<&[AssetChange]>,
) -> Cow<'a, [AssetChange]> {
    match watch_changes {
        Some(watch_changes) if !watch_changes.is_empty() => {
            let mut merged = Vec::with_capacity(identity_changes.len() + watch_changes.len());
            merged.extend_from_slice(identity_changes);
            merged.extend_from_slice(watch_changes);
            Cow::Owned(merged)
        }
        _ => Cow::Borrowed(identity_changes),
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::watch::{AssetChange, AssetChangeKind};
    use crate::asset::AssetUri;

    use super::merged_identity_changes;

    const BENCHMARK_CHANGE_COUNT: usize = 16_384;
    const BENCHMARK_MERGES_PER_SAMPLE: usize = 4;
    const BENCHMARK_WARMUP_PAIRS: usize = 4;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;

    fn change(kind: AssetChangeKind, uri: &str) -> AssetChange {
        AssetChange::new(kind, AssetUri::parse(uri).unwrap(), None)
    }

    #[test]
    fn identity_change_merge_borrows_without_a_watch_delta() {
        let identity_changes = vec![change(AssetChangeKind::Renamed, "res://renamed.data")];

        for watch_changes in [None, Some(&[][..])] {
            let merged = merged_identity_changes(&identity_changes, watch_changes);
            let Cow::Borrowed(borrowed) = merged else {
                panic!("the no-watch fast path must borrow the loaded identity changes");
            };
            assert!(std::ptr::eq(borrowed, identity_changes.as_slice()));
        }
    }

    #[test]
    fn identity_change_merge_preserves_base_then_watch_order() {
        let identity_changes = vec![change(AssetChangeKind::Renamed, "res://renamed.data")];
        let watch_changes = vec![change(AssetChangeKind::Modified, "res://modified.data")];

        let merged = merged_identity_changes(&identity_changes, Some(&watch_changes));

        assert!(matches!(merged, Cow::Owned(_)));
        assert_eq!(merged[0], identity_changes[0]);
        assert_eq!(merged[1], watch_changes[0]);
    }

    #[test]
    #[ignore = "performance acceptance benchmark"]
    fn identity_change_merge_performance_acceptance() {
        let identity_changes = benchmark_changes(BENCHMARK_CHANGE_COUNT);

        for _ in 0..BENCHMARK_WARMUP_PAIRS {
            black_box(time_legacy_merge(&identity_changes));
            black_box(time_borrowed_merge(&identity_changes));
        }

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut legacy_checksum = 0_usize;
        let mut optimized_checksum = 0_usize;
        for pair in 0..BENCHMARK_SAMPLE_PAIRS {
            let ((legacy_ns, legacy_result), (optimized_ns, optimized_result)) = if pair % 2 == 0 {
                (
                    time_legacy_merge(&identity_changes),
                    time_borrowed_merge(&identity_changes),
                )
            } else {
                let optimized = time_borrowed_merge(&identity_changes);
                let legacy = time_legacy_merge(&identity_changes);
                (legacy, optimized)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
            legacy_checksum = legacy_checksum.wrapping_add(legacy_result);
            optimized_checksum = optimized_checksum.wrapping_add(optimized_result);
        }

        let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
        let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
        let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
        let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
        let legacy_change_clones = BENCHMARK_CHANGE_COUNT * BENCHMARK_MERGES_PER_SAMPLE;

        println!(
            "RUNTIME04_IDENTITY_CHANGE_MERGE_PERF changes={} merges_per_sample={} warmup_pairs={} sample_pairs={} order=alternating percentile=nearest-rank legacy_change_clones={} optimized_change_clones=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_checksum={} optimized_checksum={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
            BENCHMARK_CHANGE_COUNT,
            BENCHMARK_MERGES_PER_SAMPLE,
            BENCHMARK_WARMUP_PAIRS,
            BENCHMARK_SAMPLE_PAIRS,
            legacy_change_clones,
            legacy_p50_ns,
            legacy_p95_ns,
            optimized_p50_ns,
            optimized_p95_ns,
            legacy_checksum,
            optimized_checksum,
            legacy_samples,
            optimized_samples,
        );

        assert_eq!(legacy_change_clones, 65_536);
        assert_eq!(legacy_checksum, optimized_checksum);
        assert_ne!(optimized_checksum, 0);
        assert!(
            optimized_p50_ns.saturating_mul(20) <= legacy_p50_ns,
            "borrowed merge must reduce P50 by at least 95%: legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns",
        );
        assert!(
            optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns,
            "borrowed merge must reduce P95 by at least 95%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns",
        );
    }

    fn benchmark_changes(count: usize) -> Vec<AssetChange> {
        (0..count)
            .map(|index| {
                AssetChange::new(
                    AssetChangeKind::Renamed,
                    AssetUri::parse(&format!("res://runtime04/current-{index:05}.data")).unwrap(),
                    Some(
                        AssetUri::parse(&format!("res://runtime04/previous-{index:05}.data"))
                            .unwrap(),
                    ),
                )
            })
            .collect()
    }

    fn time_legacy_merge(identity_changes: &[AssetChange]) -> (u128, usize) {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..BENCHMARK_MERGES_PER_SAMPLE {
            let merged = black_box(identity_changes).to_vec();
            checksum = checksum.wrapping_add(identity_change_checksum(
                black_box(merged.as_slice()),
                identity_changes,
            ));
        }
        (started.elapsed().as_nanos(), checksum)
    }

    fn time_borrowed_merge(identity_changes: &[AssetChange]) -> (u128, usize) {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..BENCHMARK_MERGES_PER_SAMPLE {
            let merged = merged_identity_changes(black_box(identity_changes), None);
            checksum = checksum.wrapping_add(identity_change_checksum(
                black_box(merged.as_ref()),
                identity_changes,
            ));
        }
        (started.elapsed().as_nanos(), checksum)
    }

    fn identity_change_checksum(changes: &[AssetChange], expected: &[AssetChange]) -> usize {
        changes.len()
            + usize::from(changes.first() == expected.first())
            + usize::from(changes.last() == expected.last())
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100).max(1);
        sorted[rank - 1]
    }
}
