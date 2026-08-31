use zircon_runtime_interface::ui::layout::UiSize;

use super::entry::AssetReferenceListPointerEntry;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetReferenceListPointerLayout {
    pub pane_size: UiSize,
    pub entries: Vec<AssetReferenceListPointerEntry>,
}

impl Default for AssetReferenceListPointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            entries: Vec::new(),
        }
    }
}

impl AssetReferenceListPointerLayout {
    pub(crate) fn from_references(
        references: &[crate::ui::workbench::snapshot::AssetReferenceSnapshot],
        pane_size: UiSize,
    ) -> Self {
        let mut entries = Vec::with_capacity(references.len());
        entries.extend(
            references
                .iter()
                .map(|reference| AssetReferenceListPointerEntry {
                    asset_uuid: reference.uuid.clone(),
                    known_project_asset: reference.known_project_asset,
                }),
        );
        Self { pane_size, entries }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::ui::workbench::snapshot::AssetReferenceSnapshot;

    const SAMPLE_PAIRS: usize = 17;
    const REFERENCES_PER_SAMPLE: usize = 512;

    #[test]
    fn reference_projection_reserves_snapshot_capacity() {
        let source = include_str!("layout.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("asset reference layout implementation");

        assert!(implementation.contains("Vec::with_capacity(references.len())"));
        assert!(implementation.contains("entries.extend("));
    }

    #[test]
    fn reference_projection_preserves_order_and_project_flags() {
        let references = vec![
            AssetReferenceSnapshot {
                uuid: "asset-a".to_string(),
                known_project_asset: true,
                ..AssetReferenceSnapshot::default()
            },
            AssetReferenceSnapshot {
                uuid: "asset-b".to_string(),
                known_project_asset: false,
                ..AssetReferenceSnapshot::default()
            },
        ];

        let layout = AssetReferenceListPointerLayout::from_references(
            &references,
            UiSize::new(320.0, 240.0),
        );

        assert_eq!(layout.entries.len(), 2);
        assert_eq!(layout.entries[0].asset_uuid, "asset-a");
        assert!(layout.entries[0].known_project_asset);
        assert_eq!(layout.entries[1].asset_uuid, "asset-b");
        assert!(!layout.entries[1].known_project_asset);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cp_editor_reference_layout_capacity_p95() {
        let references = (0..REFERENCES_PER_SAMPLE)
            .map(|index| AssetReferenceSnapshot {
                uuid: format!("asset-{index:04}"),
                known_project_asset: index % 2 == 0,
                ..AssetReferenceSnapshot::default()
            })
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&references, false));
                optimized.push(measure(&references, true));
            } else {
                optimized.push(measure(&references, true));
                legacy.push(measure(&references, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR335_REFERENCE_LAYOUT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} references_per_sample={REFERENCES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(references: &[AssetReferenceSnapshot], use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..512 {
            let entries = if use_capacity {
                let mut entries = Vec::with_capacity(references.len());
                entries.extend(
                    references
                        .iter()
                        .map(|reference| (reference.uuid.as_str(), reference.known_project_asset)),
                );
                entries
            } else {
                black_box(references)
                    .iter()
                    .map(|reference| (reference.uuid.as_str(), reference.known_project_asset))
                    .collect()
            };
            checksum ^= entries.len();
            black_box(entries);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
