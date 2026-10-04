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
            let mut projected_items = Vec::with_capacity(source_chunk.len());
            projected_items.extend(
                source_chunk
                    .iter()
                    .map(|asset| project_paint_item(snapshot.view_mode, asset)),
            );
            projected_items.into()
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
#[path = "logical_paint_source/tests/capacity_tests.rs"]
mod capacity_tests;

#[cfg(test)]
pub(super) fn clear_asset_browser_logical_paint_cache_for_tests() {
    ASSET_BROWSER_LOGICAL_PAINT_CACHE.with(|cache| *cache.borrow_mut() = None);
}

#[cfg(test)]
#[path = "tests/logical_paint_source.rs"]
mod tests;
