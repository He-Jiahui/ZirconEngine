use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

const SOURCE_ROW_PANEL_CONTROL_ID: &str = "AssetBrowserSourcesRowPanel";
const SOURCE_TREE_ROW_PREFIX: &str = "AssetBrowserSourcesTreeRow";
const SOURCE_ROW_CHILD_CONTROL_IDS: [&str; 3] = [
    "AssetBrowserSourcesRowIcon",
    "AssetBrowserSourcesRowNameText",
    "AssetBrowserSourcesRowCountText",
];

pub(super) fn sync_asset_browser_source_tree_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    snapshot: &AssetWorkspaceSnapshot,
) {
    let Some(prototype) = nodes
        .iter()
        .find(|node| node.control_id == SOURCE_ROW_PANEL_CONTROL_ID)
        .cloned()
    else {
        return;
    };

    nodes.retain(|node| {
        !is_source_tree_row(node.control_id.as_str())
            && !SOURCE_ROW_CHILD_CONTROL_IDS.contains(&node.control_id.as_str())
    });
    nodes.reserve(source_tree_row_count(snapshot));

    if snapshot.folder_tree.is_empty() {
        nodes.push(source_tree_row_node(
            &prototype,
            0,
            source_tree_fallback_label(snapshot),
            snapshot.visible_assets.len(),
            snapshot.selected_folder_id.is_some(),
            0,
        ));
        return;
    }

    for (index, folder) in snapshot.folder_tree.iter().enumerate() {
        nodes.push(source_tree_row_node(
            &prototype,
            index,
            folder.display_name.clone(),
            folder.recursive_asset_count,
            folder.selected
                || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str()),
            folder.depth,
        ));
    }
}

fn source_tree_row_count(snapshot: &AssetWorkspaceSnapshot) -> usize {
    snapshot.folder_tree.len().max(1)
}

fn source_tree_row_node(
    prototype: &ViewTemplateNodeData,
    index: usize,
    text: String,
    recursive_asset_count: usize,
    selected: bool,
    depth: usize,
) -> ViewTemplateNodeData {
    let mut row = prototype.clone();
    row.node_id = format!("asset_browser.source_tree.row_{:02}", index + 1).into();
    row.control_id = source_tree_row_control_id(index).into();
    row.role = "TreeRow".into();
    row.component_role = "workbench-tree-row".into();
    row.component_variant = "asset-folder".into();
    row.surface_variant = "asset-tree-row".into();
    row.text = text.into();
    row.value_text = recursive_asset_count.to_string().into();
    row.selected = selected;
    row.focused = false;
    row.hovered = false;
    row.pressed = false;
    row.value_number = depth.min(i32::MAX as usize) as f32;
    row
}

fn source_tree_fallback_label(snapshot: &AssetWorkspaceSnapshot) -> String {
    snapshot
        .visible_folders
        .iter()
        .find(|folder| {
            folder.selected
                || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str())
        })
        .map(|folder| folder.display_name.clone())
        .filter(|label| !label.trim().is_empty())
        .or_else(|| {
            (!snapshot.project_root.trim().is_empty()).then(|| snapshot.project_root.clone())
        })
        .unwrap_or_else(|| "Content".to_string())
}

pub(super) fn is_source_tree_row(control_id: &str) -> bool {
    if control_id == SOURCE_ROW_PANEL_CONTROL_ID {
        return true;
    }
    let Some(row_number) = control_id
        .strip_prefix(SOURCE_TREE_ROW_PREFIX)
        .and_then(|suffix| suffix.strip_suffix("/AssetBrowserSourcesRowPanel"))
    else {
        return false;
    };
    !row_number.is_empty() && row_number.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) fn source_tree_row_control_id(index: usize) -> String {
    if index == 0 {
        SOURCE_ROW_PANEL_CONTROL_ID.to_string()
    } else {
        format!(
            "{SOURCE_TREE_ROW_PREFIX}{:02}/{SOURCE_ROW_PANEL_CONTROL_ID}",
            index + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::sync_asset_browser_source_tree_nodes;
    use crate::ui::layouts::views::ViewTemplateNodeData;
    use crate::ui::workbench::snapshot::{AssetFolderSnapshot, AssetWorkspaceSnapshot};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    #[test]
    fn source_tree_nodes_preserve_pointer_order_selection_and_depth() {
        let snapshot = AssetWorkspaceSnapshot {
            selected_folder_id: Some("materials".to_string()),
            folder_tree: vec![
                folder("content", None, "Content", 0, false),
                folder("materials", Some("content"), "Materials", 1, false),
                folder("textures", Some("content"), "Textures", 1, false),
            ],
            ..AssetWorkspaceSnapshot::default()
        };
        let mut nodes = vec![source_row_prototype()];

        sync_asset_browser_source_tree_nodes(&mut nodes, &snapshot);

        let rows = nodes
            .iter()
            .filter(|node| node.role == "TreeRow")
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].control_id, "AssetBrowserSourcesRowPanel");
        assert_eq!(
            rows[1].control_id,
            "AssetBrowserSourcesTreeRow02/AssetBrowserSourcesRowPanel"
        );
        assert_eq!(rows[0].text, "Content");
        assert_eq!(rows[1].text, "Materials");
        assert_eq!(rows[2].text, "Textures");
        assert_eq!(rows[1].value_number, 1.0);
        assert!(rows[1].selected);
        assert!(!rows[0].selected);
    }

    #[test]
    fn source_tree_capacity_preserves_legacy_rows() {
        let snapshot = source_tree_snapshot(128);
        let prototype = source_row_prototype();
        let mut legacy_nodes = vec![prototype.clone()];
        let mut optimized_nodes = vec![prototype];

        retired_sync_asset_browser_source_tree_nodes(&mut legacy_nodes, &snapshot);
        sync_asset_browser_source_tree_nodes(&mut optimized_nodes, &snapshot);

        assert_eq!(optimized_nodes.len(), legacy_nodes.len());
        for (optimized, legacy) in optimized_nodes.iter().zip(&legacy_nodes) {
            assert_eq!(optimized.node_id, legacy.node_id);
            assert_eq!(optimized.control_id, legacy.control_id);
            assert_eq!(optimized.text, legacy.text);
            assert_eq!(optimized.value_text, legacy.value_text);
            assert_eq!(optimized.selected, legacy.selected);
            assert_eq!(optimized.value_number, legacy.value_number);
        }
    }

    #[test]
    fn source_tree_capacity_reserves_before_row_pushes() {
        let source = include_str!("source_tree_nodes.rs");
        let implementation = source.split("#[cfg(test)]").next().unwrap_or(source);
        let reserve = implementation
            .find("nodes.reserve(source_tree_row_count(snapshot))")
            .expect("source-tree row reserve");
        let row_loop = implementation
            .find("for (index, folder) in snapshot.folder_tree.iter().enumerate()")
            .expect("source-tree row loop");

        assert!(reserve < row_loop);
    }

    #[test]
    #[ignore = "release-only asset source-tree capacity performance gate"]
    fn source_tree_capacity_release_benchmark() {
        const SAMPLE_COUNT: usize = 11;
        const ITERATIONS_PER_SAMPLE: usize = 32;
        const ROW_COUNT: usize = 2_048;
        const MAX_OPTIMIZED_TO_RETIRED_PERCENT: u128 = 90;

        let snapshot = source_tree_snapshot(ROW_COUNT);
        let prototype = source_row_prototype();
        black_box(measure_source_tree_batch(&snapshot, &prototype, 1, false));
        black_box(measure_source_tree_batch(&snapshot, &prototype, 1, true));

        let mut retired_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                retired_samples.push(measure_source_tree_batch(
                    &snapshot,
                    &prototype,
                    ITERATIONS_PER_SAMPLE,
                    false,
                ));
                optimized_samples.push(measure_source_tree_batch(
                    &snapshot,
                    &prototype,
                    ITERATIONS_PER_SAMPLE,
                    true,
                ));
            } else {
                optimized_samples.push(measure_source_tree_batch(
                    &snapshot,
                    &prototype,
                    ITERATIONS_PER_SAMPLE,
                    true,
                ));
                retired_samples.push(measure_source_tree_batch(
                    &snapshot,
                    &prototype,
                    ITERATIONS_PER_SAMPLE,
                    false,
                ));
            }
        }

        let retired_p95_ns = duration_p95_ns(retired_samples);
        let optimized_p95_ns = duration_p95_ns(optimized_samples);
        let retired_growths = measured_capacity_growths(&snapshot, &prototype, false);
        let optimized_growths = measured_capacity_growths(&snapshot, &prototype, true);
        let reduction_basis_points = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "EDITOR57_SOURCE_TREE_CAPACITY_BENCH_V1 retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} reduction_basis_points={reduction_basis_points} samples={SAMPLE_COUNT} iterations_per_sample={ITERATIONS_PER_SAMPLE} rows={ROW_COUNT} cold_capacity_growths={retired_growths}->{optimized_growths}"
        );
        assert!(retired_growths > 1);
        assert_eq!(optimized_growths, 1);
        assert!(
            optimized_p95_ns.saturating_mul(100)
                <= retired_p95_ns.saturating_mul(MAX_OPTIMIZED_TO_RETIRED_PERCENT),
            "optimized P95 {optimized_p95_ns}ns must be at most {MAX_OPTIMIZED_TO_RETIRED_PERCENT}% of retired P95 {retired_p95_ns}ns"
        );
    }

    fn measure_source_tree_batch(
        snapshot: &AssetWorkspaceSnapshot,
        prototype: &ViewTemplateNodeData,
        iterations: usize,
        optimized: bool,
    ) -> Duration {
        let started = Instant::now();
        for _ in 0..iterations {
            let mut nodes = vec![prototype.clone()];
            if optimized {
                sync_asset_browser_source_tree_nodes(&mut nodes, snapshot);
            } else {
                retired_sync_asset_browser_source_tree_nodes(&mut nodes, snapshot);
            }
            black_box(nodes);
        }
        started.elapsed()
    }

    fn measured_capacity_growths(
        snapshot: &AssetWorkspaceSnapshot,
        prototype: &ViewTemplateNodeData,
        optimized: bool,
    ) -> usize {
        let mut nodes = vec![prototype.clone()];
        nodes.clear();
        let mut growths = 0;
        if optimized {
            let previous_capacity = nodes.capacity();
            nodes.reserve(snapshot.folder_tree.len());
            growths += usize::from(nodes.capacity() != previous_capacity);
        }
        for (index, folder) in snapshot.folder_tree.iter().enumerate() {
            let previous_capacity = nodes.capacity();
            nodes.push(super::source_tree_row_node(
                prototype,
                index,
                folder.display_name.clone(),
                folder.recursive_asset_count,
                folder.selected,
                folder.depth,
            ));
            growths += usize::from(nodes.capacity() != previous_capacity);
        }
        growths
    }

    fn duration_p95_ns(mut samples: Vec<Duration>) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index].as_nanos()
    }

    fn retired_sync_asset_browser_source_tree_nodes(
        nodes: &mut Vec<ViewTemplateNodeData>,
        snapshot: &AssetWorkspaceSnapshot,
    ) {
        let Some(prototype) = nodes
            .iter()
            .find(|node| node.control_id == super::SOURCE_ROW_PANEL_CONTROL_ID)
            .cloned()
        else {
            return;
        };
        nodes.retain(|node| {
            !super::is_source_tree_row(node.control_id.as_str())
                && !super::SOURCE_ROW_CHILD_CONTROL_IDS.contains(&node.control_id.as_str())
        });
        if snapshot.folder_tree.is_empty() {
            nodes.push(super::source_tree_row_node(
                &prototype,
                0,
                super::source_tree_fallback_label(snapshot),
                snapshot.visible_assets.len(),
                snapshot.selected_folder_id.is_some(),
                0,
            ));
            return;
        }
        for (index, folder) in snapshot.folder_tree.iter().enumerate() {
            nodes.push(super::source_tree_row_node(
                &prototype,
                index,
                folder.display_name.clone(),
                folder.recursive_asset_count,
                folder.selected
                    || snapshot.selected_folder_id.as_deref() == Some(folder.folder_id.as_str()),
                folder.depth,
            ));
        }
    }

    fn source_tree_snapshot(row_count: usize) -> AssetWorkspaceSnapshot {
        AssetWorkspaceSnapshot {
            selected_folder_id: Some(format!("folder-{:04}", row_count / 2)),
            folder_tree: (0..row_count)
                .map(|index| {
                    folder(
                        format!("folder-{index:04}").as_str(),
                        (index > 0)
                            .then(|| format!("folder-{:04}", index - 1))
                            .as_deref(),
                        format!("Folder {index:04}").as_str(),
                        index,
                        false,
                    )
                })
                .collect(),
            ..AssetWorkspaceSnapshot::default()
        }
    }

    fn source_row_prototype() -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: "asset_browser.sources.row".into(),
            control_id: "AssetBrowserSourcesRowPanel".into(),
            role: "Panel".into(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn folder(
        folder_id: &str,
        parent_folder_id: Option<&str>,
        display_name: &str,
        depth: usize,
        selected: bool,
    ) -> AssetFolderSnapshot {
        AssetFolderSnapshot {
            folder_id: folder_id.to_string(),
            parent_folder_id: parent_folder_id.map(str::to_string),
            display_name: display_name.to_string(),
            recursive_asset_count: 0,
            depth,
            selected,
        }
    }
}
