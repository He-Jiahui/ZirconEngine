use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetFolderTreePointerLayout {
    pub pane_size: UiSize,
    pub folder_ids: Vec<String>,
}

impl Default for AssetFolderTreePointerLayout {
    fn default() -> Self {
        Self {
            pane_size: UiSize::new(0.0, 0.0),
            folder_ids: Vec::new(),
        }
    }
}

impl AssetFolderTreePointerLayout {
    pub(crate) fn from_snapshot(
        snapshot: &crate::ui::workbench::snapshot::AssetWorkspaceSnapshot,
        pane_size: UiSize,
    ) -> Self {
        let mut folder_ids = Vec::with_capacity(snapshot.folder_tree.len());
        folder_ids.extend(
            snapshot
                .folder_tree
                .iter()
                .map(|folder| folder.folder_id.clone()),
        );
        Self {
            pane_size,
            folder_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::ui::workbench::snapshot::{AssetFolderSnapshot, AssetWorkspaceSnapshot};

    const SAMPLE_PAIRS: usize = 17;
    const FOLDERS_PER_SAMPLE: usize = 512;

    #[test]
    fn folder_tree_projection_reserves_snapshot_capacity() {
        let source = include_str!("layout.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("asset folder tree layout implementation");

        assert!(implementation.contains("Vec::with_capacity(snapshot.folder_tree.len())"));
        assert!(implementation.contains("folder_ids.extend("));
    }

    #[test]
    fn folder_tree_projection_preserves_folder_order() {
        let mut snapshot = AssetWorkspaceSnapshot::default();
        snapshot.folder_tree = vec![
            AssetFolderSnapshot {
                folder_id: "folder-a".to_string(),
                ..AssetFolderSnapshot::default()
            },
            AssetFolderSnapshot {
                folder_id: "folder-b".to_string(),
                ..AssetFolderSnapshot::default()
            },
        ];

        let layout =
            AssetFolderTreePointerLayout::from_snapshot(&snapshot, UiSize::new(320.0, 240.0));
        assert_eq!(layout.folder_ids, vec!["folder-a", "folder-b"]);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cq_editor_folder_tree_capacity_p95() {
        let folder_ids = (0..FOLDERS_PER_SAMPLE)
            .map(|index| format!("folder-{index:04}"))
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&folder_ids, false));
                optimized.push(measure(&folder_ids, true));
            } else {
                optimized.push(measure(&folder_ids, true));
                legacy.push(measure(&folder_ids, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR336_FOLDER_TREE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} folders_per_sample={FOLDERS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(folder_ids: &[String], use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..512 {
            let values = if use_capacity {
                let mut values = Vec::with_capacity(folder_ids.len());
                values.extend(folder_ids.iter().cloned());
                values
            } else {
                black_box(folder_ids).iter().cloned().collect()
            };
            checksum ^= values.len();
            black_box(values);
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
