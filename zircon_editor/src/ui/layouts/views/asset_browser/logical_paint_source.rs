use std::cell::RefCell;

use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter_batch, UiPerfCounter};
use crate::ui::workbench::asset_content_layout::{
    AssetBrowserLogicalPaintGeneration, AssetBrowserPaintItem,
};
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetViewMode, AssetWorkspaceItemGeneration, AssetWorkspaceSnapshot,
};

use super::table_nodes::asset_browser_list_paint_item;
use super::thumbnail_nodes::asset_browser_thumbnail_paint_item;

#[derive(Clone)]
struct AssetBrowserLogicalPaintCacheEntry {
    input: AssetBrowserLogicalPaintInput,
    source: AssetWorkspaceItemGeneration,
    items: AssetBrowserLogicalPaintGeneration,
}

#[derive(Clone, Copy)]
struct AssetBrowserLogicalPaintInput {
    view_mode: AssetViewMode,
    text_metrics_generation: [u64; 3],
}

thread_local! {
    static ASSET_BROWSER_LOGICAL_PAINT_CACHE: RefCell<Option<AssetBrowserLogicalPaintCacheEntry>> =
        const { RefCell::new(None) };
}

impl AssetBrowserLogicalPaintInput {
    fn new(snapshot: &AssetWorkspaceSnapshot, text_metrics_generation: [u64; 3]) -> Self {
        Self {
            view_mode: snapshot.view_mode,
            text_metrics_generation,
        }
    }

    fn matches_projection(
        &self,
        snapshot: &AssetWorkspaceSnapshot,
        text_metrics_generation: [u64; 3],
    ) -> bool {
        self.view_mode == snapshot.view_mode
            && self.text_metrics_generation == text_metrics_generation
    }
}

pub(super) fn asset_browser_logical_paint_items(
    snapshot: &AssetWorkspaceSnapshot,
    text_metrics_generation: [u64; 3],
) -> AssetBrowserLogicalPaintGeneration {
    let previous = ASSET_BROWSER_LOGICAL_PAINT_CACHE.with(|cache| cache.borrow().clone());
    if let Some(cached) = previous.as_ref().filter(|cached| {
        cached
            .input
            .matches_projection(snapshot, text_metrics_generation)
            && cached.source.shares_items_with(&snapshot.visible_assets)
    }) {
        return cached.items.clone();
    }

    let reusable = previous.as_ref().filter(|cached| {
        cached
            .input
            .matches_projection(snapshot, text_metrics_generation)
    });
    let mut first_item_index = 0;
    let source_chunks = snapshot.visible_assets.item_chunks();
    let mut chunks = Vec::with_capacity(source_chunks.len());
    let mut paint_chunk_build_count = 0;
    let mut paint_chunk_reuse_count = 0;
    let mut paint_item_projection_count = 0;
    for (chunk_index, source_chunk) in source_chunks.enumerate() {
        let reused = reusable
            .filter(|cached| {
                snapshot
                    .visible_assets
                    .shares_item_chunk_with(first_item_index, &cached.source)
            })
            .and_then(|cached| cached.items.cloned_chunk(chunk_index));
        chunks.push(if let Some(reused) = reused {
            paint_chunk_reuse_count += 1;
            reused
        } else {
            paint_chunk_build_count += 1;
            paint_item_projection_count += source_chunk.len();
            source_chunk
                .iter()
                .map(|asset| project_paint_item(snapshot.view_mode, asset))
                .collect::<Vec<_>>()
                .into()
        });
        first_item_index += source_chunk.len();
    }
    record_current_ui_perf_counter_batch(|counters| {
        counters.push((
            UiPerfCounter::AssetBrowserLogicalPaintChunkBuildCount,
            paint_chunk_build_count as f64,
        ));
        counters.push((
            UiPerfCounter::AssetBrowserLogicalPaintChunkReuseCount,
            paint_chunk_reuse_count as f64,
        ));
        counters.push((
            UiPerfCounter::AssetBrowserLogicalPaintItemProjectionCount,
            paint_item_projection_count as f64,
        ));
    });
    let items = AssetBrowserLogicalPaintGeneration::from_chunks(chunks);
    ASSET_BROWSER_LOGICAL_PAINT_CACHE.with(|cache| {
        *cache.borrow_mut() = Some(AssetBrowserLogicalPaintCacheEntry {
            input: AssetBrowserLogicalPaintInput::new(snapshot, text_metrics_generation),
            source: snapshot.visible_assets.clone(),
            items: items.clone(),
        });
    });
    items
}

fn project_paint_item(
    view_mode: AssetViewMode,
    asset: &AssetItemSnapshot,
) -> AssetBrowserPaintItem {
    match view_mode {
        AssetViewMode::List => asset_browser_list_paint_item(asset),
        AssetViewMode::Thumbnail => asset_browser_thumbnail_paint_item(asset),
    }
}

pub(super) fn selected_asset_item_indices(snapshot: &AssetWorkspaceSnapshot) -> Vec<usize> {
    if let Some(index) = snapshot
        .selected_asset_uuid
        .as_deref()
        .and_then(|uuid| snapshot.visible_assets.selected_index(uuid))
    {
        return vec![index];
    }

    snapshot.visible_assets.selected_indices().to_vec()
}

#[cfg(test)]
pub(super) fn clear_asset_browser_logical_paint_cache_for_tests() {
    ASSET_BROWSER_LOGICAL_PAINT_CACHE.with(|cache| *cache.borrow_mut() = None);
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::ui::workbench::asset_content_layout::AssetBrowserPaintItem;
    use crate::ui::workbench::snapshot::{
        AssetItemSnapshot, AssetTypeProjectionSnapshot, AssetViewMode, AssetWorkspaceSnapshot,
    };
    use zircon_runtime_interface::resource::ResourceKind;

    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 1_024;
    const BENCHMARK_ITEM_COUNT: usize = 16_384;

    #[test]
    fn local_asset_delta_rebuilds_only_the_affected_logical_paint_chunk() {
        clear_asset_browser_logical_paint_cache_for_tests();
        let mut snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::List,
            visible_assets: (0..130).map(test_asset_item).collect(),
            ..AssetWorkspaceSnapshot::default()
        };
        let first = asset_browser_logical_paint_items(&snapshot, [1, 2, 3]);

        let mut replacement = snapshot.visible_assets[65].clone();
        replacement.display_name = "changed-asset-065.mesh".to_string();
        snapshot.catalog_revision = snapshot.catalog_revision.wrapping_add(1);
        snapshot.visible_assets = snapshot
            .visible_assets
            .replace_existing_items([replacement])
            .expect("existing asset delta should preserve visible membership");
        let next = asset_browser_logical_paint_items(&snapshot, [1, 2, 3]);

        assert!(first.shares_item_chunk_with(0, &next));
        assert!(!first.shares_item_chunk_with(65, &next));
        assert!(first.shares_item_chunk_with(129, &next));
        let Some(AssetBrowserPaintItem::List(changed)) = next.get(65) else {
            panic!("changed list item should remain projected");
        };
        assert!(changed.text.contains("changed-asset-065"));
    }

    #[test]
    fn sparse_selection_index_tracks_local_item_replacements_without_a_source_scan() {
        let mut items = (0..130).map(test_asset_item).collect::<Vec<_>>();
        items[1].selected = true;
        items[129].selected = true;
        let mut snapshot = AssetWorkspaceSnapshot {
            visible_assets: items.into(),
            ..AssetWorkspaceSnapshot::default()
        };

        assert_eq!(selected_asset_item_indices(&snapshot), vec![1, 129]);

        let mut selected = snapshot.visible_assets[65].clone();
        selected.selected = true;
        snapshot.visible_assets = snapshot
            .visible_assets
            .replace_existing_items([selected])
            .expect("local selection replacement must preserve visible membership");
        assert_eq!(selected_asset_item_indices(&snapshot), vec![1, 65, 129]);

        let mut deselected = snapshot.visible_assets[1].clone();
        deselected.selected = false;
        snapshot.visible_assets = snapshot
            .visible_assets
            .replace_existing_items([deselected])
            .expect("local deselection replacement must preserve visible membership");
        assert_eq!(selected_asset_item_indices(&snapshot), vec![65, 129]);
    }

    #[test]
    fn sparse_selection_index_uses_the_last_duplicate_replacement() {
        let mut snapshot = AssetWorkspaceSnapshot {
            visible_assets: (0..130).map(test_asset_item).collect(),
            ..AssetWorkspaceSnapshot::default()
        };
        let mut selected = snapshot.visible_assets[65].clone();
        selected.selected = true;
        let mut deselected = selected.clone();
        deselected.selected = false;

        snapshot.visible_assets = snapshot
            .visible_assets
            .replace_existing_items([selected, deselected])
            .expect("duplicate local replacements must preserve visible membership");

        assert!(selected_asset_item_indices(&snapshot).is_empty());
    }

    #[test]
    fn sparse_selection_index_updates_only_reprojected_chunks() {
        let source = (0..130)
            .map(test_asset_item)
            .collect::<crate::ui::workbench::snapshot::AssetWorkspaceItemGeneration>();
        let previous_projected = source.project_items(|item| {
            item.selected = item.uuid == "asset-001";
        });
        let mut replacement = source[65].clone();
        replacement.display_name = "selected".to_string();
        let next_source = source
            .replace_existing_items([replacement])
            .expect("projection input delta must preserve visible membership");

        let next_projected =
            next_source.project_items_reusing(&source, &previous_projected, |item| {
                item.selected = item.display_name == "selected"
            });

        assert_eq!(next_projected.selected_indices(), &[1, 65]);
        assert!(next_projected.shares_item_chunk_with(1, &previous_projected));
        assert!(!next_projected.shares_item_chunk_with(65, &previous_projected));
    }

    #[test]
    fn uuid_selection_overrides_and_missing_uuid_falls_back_to_sparse_flags() {
        let mut items = (0..130).map(test_asset_item).collect::<Vec<_>>();
        items[1].selected = true;
        items[129].selected = true;
        let mut snapshot = AssetWorkspaceSnapshot {
            selected_asset_uuid: Some("asset-065".to_string()),
            visible_assets: items.into(),
            ..AssetWorkspaceSnapshot::default()
        };

        assert_eq!(selected_asset_item_indices(&snapshot), vec![65]);

        snapshot.selected_asset_uuid = Some("not-visible".to_string());
        assert_eq!(selected_asset_item_indices(&snapshot), vec![1, 129]);
    }

    #[test]
    fn exact_logical_paint_chunk_capacity_preserves_projected_item_count() {
        clear_asset_browser_logical_paint_cache_for_tests();
        let snapshot = AssetWorkspaceSnapshot {
            view_mode: AssetViewMode::List,
            visible_assets: (0..130).map(test_asset_item).collect(),
            ..AssetWorkspaceSnapshot::default()
        };

        let items = asset_browser_logical_paint_items(&snapshot, [1, 2, 3]);

        assert_eq!(items.len(), snapshot.visible_assets.len());
        assert!(items
            .iter()
            .all(|item| matches!(item, AssetBrowserPaintItem::List(_))));
    }

    #[test]
    fn exact_logical_paint_chunk_capacity_reserves_before_projection() {
        let source = include_str!("logical_paint_source.rs");
        let production = source.split("#[cfg(test)]").next().expect("implementation");
        let source_chunks = production
            .find("let source_chunks = snapshot.visible_assets.item_chunks()")
            .expect("exact source chunk iterator");
        let reserve = production
            .find("Vec::with_capacity(source_chunks.len())")
            .expect("exact logical-paint chunk reserve");
        let projection = production
            .find("for (chunk_index, source_chunk) in source_chunks.enumerate()")
            .expect("logical-paint chunk projection loop");

        assert!(source_chunks < reserve);
        assert!(reserve < projection);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn exact_logical_paint_chunk_capacity_release_benchmark() {
        let snapshot = AssetWorkspaceSnapshot {
            visible_assets: (0..BENCHMARK_ITEM_COUNT).map(test_asset_item).collect(),
            ..AssetWorkspaceSnapshot::default()
        };
        let chunk_count = snapshot.visible_assets.item_chunks().len();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_chunk_assembly(&snapshot, false));
                optimized_samples.push(measure_chunk_assembly(&snapshot, true));
            } else {
                optimized_samples.push(measure_chunk_assembly(&snapshot, true));
                retired_samples.push(measure_chunk_assembly(&snapshot, false));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let retired_growths = measured_chunk_capacity_growths(&snapshot, false);
        let optimized_growths = measured_chunk_capacity_growths(&snapshot, true);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "EDITOR57_EXACT_LOGICAL_PAINT_CHUNK_CAPACITY_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
items={BENCHMARK_ITEM_COUNT} chunks={chunk_count} \
retired_capacity_growths={retired_growths} optimized_capacity_growths={optimized_growths} \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(retired_growths > 1);
        assert_eq!(optimized_growths, 1);
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "exact chunk capacity must reduce chunk-vector assembly P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn measure_chunk_assembly(snapshot: &AssetWorkspaceSnapshot, optimized: bool) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let source_chunks = snapshot.visible_assets.item_chunks();
            let mut chunks = if optimized {
                Vec::with_capacity(source_chunks.len())
            } else {
                Vec::new()
            };
            for source_chunk in source_chunks {
                chunks.push(source_chunk.len());
            }
            black_box(chunks);
        }
        started.elapsed()
    }

    fn measured_chunk_capacity_growths(
        snapshot: &AssetWorkspaceSnapshot,
        optimized: bool,
    ) -> usize {
        let source_chunks = snapshot.visible_assets.item_chunks();
        let mut chunks = Vec::new();
        let mut growths = 0;
        if optimized {
            let previous_capacity = chunks.capacity();
            chunks.reserve(source_chunks.len());
            growths += usize::from(chunks.capacity() != previous_capacity);
        }
        for source_chunk in source_chunks {
            let previous_capacity = chunks.capacity();
            chunks.push(source_chunk.len());
            growths += usize::from(chunks.capacity() != previous_capacity);
        }
        growths
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    fn test_asset_item(index: usize) -> AssetItemSnapshot {
        let uuid = format!("asset-{index:03}");
        AssetItemSnapshot {
            uuid: uuid.clone(),
            locator: format!("res://{uuid}.mesh"),
            display_name: format!("{uuid}.mesh"),
            file_name: format!("{uuid}.mesh"),
            extension: "mesh".to_string(),
            kind: ResourceKind::Mesh,
            asset_type: AssetTypeProjectionSnapshot::from_resource_kind(ResourceKind::Mesh),
            preview_artifact_path: String::new(),
            dirty: false,
            diagnostics: Vec::new(),
            selected: false,
            resource_state: None,
            resource_revision: Some(1),
        }
    }
}
