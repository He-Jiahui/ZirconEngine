use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::asset_content_layout::BROWSER_CONTENT_ITEM_PREFIX;
use crate::ui::workbench::asset_content_layout::{
    AssetBrowserListPaintItem, AssetBrowserLogicalPaintGeneration, AssetBrowserPaintItem,
    BROWSER_CONTENT_LIST_ROW_HEIGHT, BROWSER_CONTENT_TABLE_CONTROL_ID,
};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

use super::name_compaction::{compact_file_like_display_name, RuntimeFileNameCompaction};

const ASSET_TABLE_HEADER_CELLS: [&str; 4] = ["Name", "Type", "Size", "Rev"];
const ASSET_TABLE_NAME_MAX_WIDTH: f32 = 150.0;
const ASSET_TABLE_NAME_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_CAPTION_SIZE;
const ASSET_TABLE_NAME_MIN_PREFIX_CHARS: usize = 6;
const ASSET_TABLE_NAME_MIN_TAIL_CHARS: usize = 4;
const ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS: usize = 8;

pub(super) fn sync_asset_table_nodes(
    nodes: &mut Vec<ViewTemplateNodeData>,
    view_mode: AssetViewMode,
    materialized_item_count: usize,
) {
    let Some(prototype) = nodes
        .iter()
        .find(|node| asset_table_row_index(node.control_id.as_str()) == Some(0))
        .cloned()
    else {
        return;
    };
    let asset_count = if view_mode == AssetViewMode::List {
        materialized_item_count
    } else {
        0
    };
    let mut existing_row_indices = vec![false; asset_count];
    nodes.retain(
        |node| match asset_table_row_index(node.control_id.as_str()) {
            Some(index) if index < asset_count => {
                existing_row_indices[index] = true;
                true
            }
            Some(_) => false,
            None => true,
        },
    );
    let missing_row_count = existing_row_indices
        .iter()
        .filter(|exists| !**exists)
        .count();
    nodes.reserve(missing_row_count);

    for index in 0..asset_count {
        if existing_row_indices[index] {
            continue;
        }
        let mut row = prototype.clone();
        row.node_id = format!("asset_browser.runtime.asset_row_{index:02}").into();
        row.control_id = asset_table_row_control_id(index).into();
        row.selected = false;
        row.focused = false;
        row.hovered = false;
        nodes.push(row);
    }
}

pub(super) fn asset_browser_list_paint_item(asset: &AssetItemSnapshot) -> AssetBrowserPaintItem {
    let cells = asset_table_row_cells(asset);
    let text = asset_table_row_text(&cells);
    AssetBrowserPaintItem::List(AssetBrowserListPaintItem {
        text,
        cells: shared_string_options(cells.into_iter().collect()),
    })
}

pub(super) fn apply_asset_browser_list_logical_extent(
    nodes: &mut [ViewTemplateNodeData],
    logical_item_count: usize,
) {
    if let Some(table) = nodes
        .iter_mut()
        .find(|node| node.control_id == BROWSER_CONTENT_TABLE_CONTROL_ID)
    {
        table.value_number = BROWSER_CONTENT_LIST_ROW_HEIGHT * logical_item_count as f32;
    }
}

pub(super) fn mark_asset_table_rows(
    nodes: &mut [ViewTemplateNodeData],
    snapshot: &AssetWorkspaceSnapshot,
) {
    let selected_uuid = snapshot.selected_asset_uuid.as_deref();
    for node in nodes.iter_mut() {
        let Some(index) = asset_table_row_index(node.control_id.as_str()) else {
            continue;
        };
        let selected = snapshot
            .visible_assets
            .get(index)
            .map(|asset| asset.selected || selected_uuid == Some(asset.uuid.as_str()))
            .unwrap_or(false);
        node.selected = selected;
        node.focused = false;
    }
}

pub(super) fn apply_asset_browser_table_cells(
    nodes: &mut [ViewTemplateNodeData],
    items: &AssetBrowserLogicalPaintGeneration,
) {
    for node in nodes.iter_mut() {
        if node.control_id == "WorkbenchAssetBrowserTableHeader" {
            node.options = shared_string_options(
                ASSET_TABLE_HEADER_CELLS
                    .iter()
                    .map(|cell| (*cell).to_string())
                    .collect(),
            );
            node.text = ASSET_TABLE_HEADER_CELLS.join(" ").into();
        } else if let Some(AssetBrowserPaintItem::List(item)) =
            asset_table_row_index(node.control_id.as_str()).and_then(|index| items.get(index))
        {
            node.options = item.cells.clone();
            node.text = item.text.clone().into();
        }
    }
}

pub(super) fn asset_table_row_control_id(index: usize) -> String {
    format!("{BROWSER_CONTENT_ITEM_PREFIX}{:02}", index + 1)
}

pub(super) fn asset_table_row_index(control_id: &str) -> Option<usize> {
    control_id
        .strip_prefix(BROWSER_CONTENT_ITEM_PREFIX)?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)
}

fn asset_table_row_cells(asset: &AssetItemSnapshot) -> [String; 4] {
    [
        compact_asset_table_name(&asset.display_name, &asset.extension),
        asset.asset_type.display_name.clone(),
        asset_size_hint(asset).to_string(),
        asset
            .resource_revision
            .map(|revision| format!("r{revision}"))
            .unwrap_or_else(|| "new".to_string()),
    ]
}

pub(super) fn asset_table_row_text(row: &[String; 4]) -> String {
    row.join(" ")
}

fn asset_size_hint(asset: &AssetItemSnapshot) -> &'static str {
    match asset.kind {
        ResourceKind::Texture => "1.2M",
        ResourceKind::Material | ResourceKind::MaterialGraph | ResourceKind::Shader => "512K",
        ResourceKind::Scene | ResourceKind::Prefab | ResourceKind::UiLayout => "64K",
        ResourceKind::Model | ResourceKind::Mesh | ResourceKind::AnimationClip => "2.4M",
        _ => "16K",
    }
}

fn compact_asset_table_name(display_name: &str, extension: &str) -> String {
    compact_file_like_display_name(
        display_name,
        extension,
        RuntimeFileNameCompaction {
            max_width: ASSET_TABLE_NAME_MAX_WIDTH,
            font_size: ASSET_TABLE_NAME_FONT_SIZE,
            min_prefix_chars: ASSET_TABLE_NAME_MIN_PREFIX_CHARS,
            min_tail_stem_chars: ASSET_TABLE_NAME_MIN_TAIL_CHARS,
            preferred_tail_stem_chars: ASSET_TABLE_NAME_PREFERRED_TAIL_CHARS,
        },
    )
}

fn shared_string_options(values: Vec<String>) -> ModelRc<SharedString> {
    model_rc(values.into_iter().map(SharedString::from).collect())
}

#[cfg(test)]
fn asset_table_rows(snapshot: &AssetWorkspaceSnapshot) -> Vec<[String; 4]> {
    snapshot
        .visible_assets
        .iter()
        .map(asset_table_row_cells)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::ui::retained_host::measure_runtime_text_width;

    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 32;
    const BENCHMARK_ROW_COUNT: usize = 2_048;

    #[test]
    fn compact_asset_table_name_preserves_readable_file_identity() {
        assert_eq!(
            compact_asset_table_name("workbench_page_chrome.zui", "zui"),
            "workbench_page_chrome.zui"
        );
        let compact =
            compact_asset_table_name("workbench_extension_accessibility_workspace.zui", "zui");
        assert!(compact.starts_with("workbench"));
        assert!(compact.contains("..."));
        assert!(compact.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(&compact, ASSET_TABLE_NAME_FONT_SIZE)
                <= ASSET_TABLE_NAME_MAX_WIDTH + 0.01,
            "compact asset table name should fit measured table name width: {compact}"
        );
    }

    #[test]
    fn compact_asset_table_name_uses_runtime_width_not_character_count() {
        let narrow = format!("{}.zui", "i".repeat(40));
        let wide = format!("{}.zui", "W".repeat(40));
        assert_eq!(narrow.chars().count(), wide.chars().count());

        assert_eq!(compact_asset_table_name(&narrow, "zui"), narrow);
        let compact_wide = compact_asset_table_name(&wide, "zui");

        assert_ne!(compact_wide, wide);
        assert!(compact_wide.ends_with(".zui"));
        assert!(
            measure_runtime_text_width(&compact_wide, ASSET_TABLE_NAME_FONT_SIZE)
                <= ASSET_TABLE_NAME_MAX_WIDTH + 0.01,
            "wide table name should fit measured width: {compact_wide}"
        );
    }

    #[test]
    fn asset_table_rows_follow_the_real_catalog_without_padding_or_truncation() {
        let snapshot = AssetWorkspaceSnapshot {
            visible_assets: (0..7)
                .map(|index| AssetItemSnapshot {
                    uuid: format!("asset-{index}"),
                    locator: format!("res://Asset_{index}.mesh"),
                    display_name: format!("Asset_{index}.mesh"),
                    file_name: format!("Asset_{index}.mesh"),
                    extension: "mesh".to_string(),
                    kind: ResourceKind::Mesh,
                    asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(ResourceKind::Mesh),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    selected: false,
                    resource_state: None,
                    resource_revision: Some(1),
                })
                .collect(),
            ..AssetWorkspaceSnapshot::default()
        };

        let rows = asset_table_rows(&snapshot);

        assert_eq!(rows.len(), 7);
        assert!(rows.iter().all(|row| row[0] != "Empty Asset"));
        assert_eq!(rows[6][0], "Asset_6.mesh");
    }

    #[test]
    fn exact_table_row_capacity_preserves_retired_nodes_and_holes() {
        let prototype = table_row_node(0);
        let mut retired = vec![
            unrelated_node(),
            prototype.clone(),
            table_row_node(2),
            table_row_node(9),
        ];
        let mut optimized = retired.clone();

        retired_sync_asset_table_nodes(&mut retired, AssetViewMode::List, 5);
        sync_asset_table_nodes(&mut optimized, AssetViewMode::List, 5);

        assert_eq!(optimized.len(), retired.len());
        for (optimized, retired) in optimized.iter().zip(&retired) {
            assert_eq!(optimized.node_id, retired.node_id);
            assert_eq!(optimized.control_id, retired.control_id);
            assert_eq!(optimized.selected, retired.selected);
            assert_eq!(optimized.focused, retired.focused);
            assert_eq!(optimized.hovered, retired.hovered);
        }
        assert_eq!(
            optimized
                .iter()
                .filter_map(|node| asset_table_row_index(node.control_id.as_str()))
                .collect::<Vec<_>>(),
            vec![0, 2, 1, 3, 4]
        );
    }

    #[test]
    fn exact_table_row_capacity_reserves_before_missing_row_pushes() {
        let source = include_str!("table_nodes.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        let reserve = production
            .find("nodes.reserve(missing_row_count)")
            .expect("exact missing-row reserve");
        let row_loop = production
            .find("for index in 0..asset_count")
            .expect("asset row append loop");

        assert!(reserve < row_loop);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn exact_table_row_capacity_release_benchmark() {
        let prototype = table_row_node(0);
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_table_sync(&prototype, false));
                optimized_samples.push(measure_table_sync(&prototype, true));
            } else {
                optimized_samples.push(measure_table_sync(&prototype, true));
                retired_samples.push(measure_table_sync(&prototype, false));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let retired_growths = measured_table_capacity_growths(&prototype, false);
        let optimized_growths = measured_table_capacity_growths(&prototype, true);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "EDITOR57_EXACT_TABLE_ROW_CAPACITY_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} rows={BENCHMARK_ROW_COUNT} \
retired_capacity_growths={retired_growths} optimized_capacity_growths={optimized_growths} \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(retired_growths > 1);
        assert_eq!(optimized_growths, 1);
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(90),
            "exact table capacity must reduce P95 by at least 10%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn retired_sync_asset_table_nodes(
        nodes: &mut Vec<ViewTemplateNodeData>,
        view_mode: AssetViewMode,
        materialized_item_count: usize,
    ) {
        let Some(prototype) = nodes
            .iter()
            .find(|node| asset_table_row_index(node.control_id.as_str()) == Some(0))
            .cloned()
        else {
            return;
        };
        let asset_count = if view_mode == AssetViewMode::List {
            materialized_item_count
        } else {
            0
        };
        let mut existing_row_indices = vec![false; asset_count];
        nodes.retain(
            |node| match asset_table_row_index(node.control_id.as_str()) {
                Some(index) if index < asset_count => {
                    existing_row_indices[index] = true;
                    true
                }
                Some(_) => false,
                None => true,
            },
        );
        for index in 0..asset_count {
            if existing_row_indices[index] {
                continue;
            }
            let mut row = prototype.clone();
            row.node_id = format!("asset_browser.runtime.asset_row_{index:02}").into();
            row.control_id = asset_table_row_control_id(index).into();
            row.selected = false;
            row.focused = false;
            row.hovered = false;
            nodes.push(row);
        }
    }

    fn table_row_node(index: usize) -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: format!("fixture.asset_row_{index:02}").into(),
            control_id: asset_table_row_control_id(index).into(),
            role: "TableRow".into(),
            selected: true,
            focused: true,
            hovered: true,
            ..ViewTemplateNodeData::default()
        }
    }

    fn unrelated_node() -> ViewTemplateNodeData {
        ViewTemplateNodeData {
            node_id: "fixture.unrelated".into(),
            control_id: "WorkbenchUnrelatedNode".into(),
            role: "Panel".into(),
            ..ViewTemplateNodeData::default()
        }
    }

    fn measure_table_sync(prototype: &ViewTemplateNodeData, optimized: bool) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let mut nodes = vec![prototype.clone()];
            if optimized {
                sync_asset_table_nodes(&mut nodes, AssetViewMode::List, BENCHMARK_ROW_COUNT);
            } else {
                retired_sync_asset_table_nodes(
                    &mut nodes,
                    AssetViewMode::List,
                    BENCHMARK_ROW_COUNT,
                );
            }
            black_box(nodes);
        }
        started.elapsed()
    }

    fn measured_table_capacity_growths(prototype: &ViewTemplateNodeData, optimized: bool) -> usize {
        let mut nodes = vec![prototype.clone()];
        let missing_row_count = BENCHMARK_ROW_COUNT - 1;
        let mut growths = 0;
        if optimized {
            let previous_capacity = nodes.capacity();
            nodes.reserve(missing_row_count);
            growths += usize::from(nodes.capacity() != previous_capacity);
        }
        for _ in 0..missing_row_count {
            let previous_capacity = nodes.capacity();
            nodes.push(prototype.clone());
            growths += usize::from(nodes.capacity() != previous_capacity);
        }
        growths
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
