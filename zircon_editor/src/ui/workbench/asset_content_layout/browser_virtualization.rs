use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::paint_metadata::{AssetContentRect, AssetContentRowGroup};

#[derive(Clone, Debug)]
pub(crate) struct AssetBrowserListPaintItem {
    pub(crate) text: String,
    pub(crate) cells: ModelRc<SharedString>,
}

#[derive(Clone, Debug)]
pub(crate) struct AssetBrowserThumbnailPaintItem {
    pub(crate) name: String,
    pub(crate) source_file_name: String,
    pub(crate) file_extension: String,
    pub(crate) name_continuation: String,
    pub(crate) type_label: String,
    pub(crate) type_label_width: f32,
    pub(crate) state_label: String,
    pub(crate) visual_variant: String,
    pub(crate) preview_artifact_path: String,
}

#[derive(Clone, Debug)]
pub(crate) enum AssetBrowserPaintItem {
    List(AssetBrowserListPaintItem),
    Thumbnail(AssetBrowserThumbnailPaintItem),
}

/// Immutable formatted rows shared by virtual slots across pane publications.
#[derive(Clone, Debug)]
pub(crate) struct AssetBrowserLogicalPaintGeneration {
    chunks: Rc<[Rc<[AssetBrowserPaintItem]>]>,
    len: usize,
    chunk_size: usize,
}

impl Default for AssetBrowserLogicalPaintGeneration {
    fn default() -> Self {
        Self {
            chunks: Rc::from([]),
            len: 0,
            chunk_size: 1,
        }
    }
}

impl AssetBrowserLogicalPaintGeneration {
    pub(crate) fn from_chunks(chunks: Vec<Rc<[AssetBrowserPaintItem]>>) -> Self {
        let len = chunks.iter().map(|chunk| chunk.len()).sum();
        let chunk_size = chunks.first().map_or(1, |chunk| chunk.len().max(1));
        debug_assert!(
            chunks
                .iter()
                .take(chunks.len().saturating_sub(1))
                .all(|chunk| chunk.len() == chunk_size),
            "logical paint chunks must stay aligned with the source generation"
        );
        Self {
            chunks: chunks.into(),
            len,
            chunk_size,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn get(&self, index: usize) -> Option<&AssetBrowserPaintItem> {
        let chunk = self.chunks.get(index / self.chunk_size)?;
        chunk.get(index % self.chunk_size)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &AssetBrowserPaintItem> {
        self.chunks.iter().flat_map(|chunk| chunk.iter())
    }

    pub(crate) fn cloned_chunk(&self, chunk_index: usize) -> Option<Rc<[AssetBrowserPaintItem]>> {
        self.chunks.get(chunk_index).map(Rc::clone)
    }

    fn shares_items_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.chunks, &other.chunks)
    }

    #[cfg(test)]
    pub(crate) fn shares_item_chunk_with(&self, index: usize, other: &Self) -> bool {
        let chunk_index = index / self.chunk_size;
        self.chunks
            .get(chunk_index)
            .zip(other.chunks.get(chunk_index))
            .is_some_and(|(left, right)| Rc::ptr_eq(left, right))
    }
}

#[derive(Clone, Debug)]
pub(super) struct AssetBrowserVirtualization {
    items: AssetBrowserLogicalPaintGeneration,
    selected_item_indices: Vec<usize>,
    materialized_item_count: usize,
    columns: usize,
    base_top: f32,
    row_stride: f32,
    viewport_top: f32,
    overscan_rows: usize,
}

impl AssetBrowserVirtualization {
    pub(super) fn new(
        items: AssetBrowserLogicalPaintGeneration,
        selected_item_indices: Vec<usize>,
        materialized_item_count: usize,
        columns: usize,
        base_top: f32,
        row_stride: f32,
        viewport_top: f32,
        overscan_rows: usize,
    ) -> Self {
        Self {
            items,
            selected_item_indices,
            materialized_item_count,
            columns: columns.max(1),
            base_top,
            row_stride: finite_positive(row_stride),
            viewport_top,
            overscan_rows,
        }
    }

    pub(super) fn logical_item_count(&self) -> usize {
        self.items.len()
    }

    pub(super) fn binding(
        &self,
        scroll_px: f32,
        slot_index: usize,
    ) -> Option<AssetBrowserSlotBinding<'_>> {
        let start_row = self.window_start_row(scroll_px);
        let (logical_index, physical_row, logical_row) =
            self.logical_index_for_slot(start_row, slot_index)?;
        let y_offset_rows = logical_row.checked_sub(physical_row)?;
        let item = self.items.get(logical_index)?;
        Some(AssetBrowserSlotBinding {
            logical_index,
            y_offset: y_offset_rows as f32 * self.row_stride,
            selected: self
                .selected_item_indices
                .binary_search(&logical_index)
                .is_ok(),
            item,
        })
    }

    fn materialized_row_count(&self) -> usize {
        self.materialized_item_count.div_ceil(self.columns)
    }

    fn logical_index_for_slot(
        &self,
        start_row: usize,
        slot_index: usize,
    ) -> Option<(usize, usize, usize)> {
        if slot_index >= self.materialized_item_count {
            return None;
        }
        let materialized_row_count = self.materialized_row_count();
        if materialized_row_count == 0 {
            return None;
        }

        let physical_row = slot_index / self.columns;
        let column = slot_index % self.columns;
        if self.items.len() > self.materialized_item_count
            && self.materialized_item_count % self.columns != 0
        {
            let logical_row = start_row.checked_add(physical_row)?;
            let logical_index = start_row
                .checked_mul(self.columns)?
                .checked_add(slot_index)?;
            return (logical_index < self.items.len()).then_some((
                logical_index,
                physical_row,
                logical_row,
            ));
        }
        let first_physical_row = start_row % materialized_row_count;
        let row_offset = physical_row
            .checked_add(materialized_row_count)?
            .checked_sub(first_physical_row)?
            % materialized_row_count;
        let logical_row = start_row.checked_add(row_offset)?;
        let logical_index = logical_row.checked_mul(self.columns)?.checked_add(column)?;
        (logical_index < self.items.len()).then_some((logical_index, physical_row, logical_row))
    }

    #[cfg(test)]
    pub(super) fn shares_items_with(&self, other: &Self) -> bool {
        self.items.shares_items_with(&other.items)
    }

    #[cfg(test)]
    pub(super) fn shares_item_chunk_with(&self, index: usize, other: &Self) -> bool {
        self.items.shares_item_chunk_with(index, &other.items)
    }

    fn window_start_row(&self, scroll_px: f32) -> usize {
        if self.row_stride <= 0.0 || self.items.is_empty() {
            return 0;
        }
        let visible_top = self.viewport_top + finite_non_negative(scroll_px);
        let first_visible_row = ((visible_top - self.base_top) / self.row_stride)
            .floor()
            .max(0.0) as usize;
        let requested_start_row = first_visible_row.saturating_sub(self.overscan_rows);
        let logical_row_count = self.items.len().div_ceil(self.columns);
        let max_start_row = logical_row_count.saturating_sub(self.materialized_row_count());
        requested_start_row.min(max_start_row)
    }
}

pub(super) fn append_visible_virtual_group_rows(
    rows: &mut Vec<usize>,
    groups: &[AssetContentRowGroup],
    virtualization: &AssetBrowserVirtualization,
    viewport: Option<AssetContentRect>,
    scroll_px: f32,
    origin_x: f32,
    origin_y: f32,
    damage_clip: AssetContentRect,
) -> usize {
    let Some(viewport) = viewport.map(|viewport| viewport.translated(origin_x, origin_y)) else {
        return 0;
    };
    let Some(visible) = viewport.intersect(damage_clip) else {
        return 0;
    };
    let visible_top = visible.y - origin_y + scroll_px.max(0.0);
    let visible_bottom = visible.bottom() - origin_y + scroll_px.max(0.0);
    let mut visible_item_count = 0;
    for (slot_index, group) in groups.iter().enumerate() {
        let Some(binding) = virtualization.binding(scroll_px, slot_index) else {
            continue;
        };
        let top = group.top + binding.y_offset;
        let bottom = group.bottom + binding.y_offset;
        if bottom <= visible_top || top >= visible_bottom {
            continue;
        }
        rows.extend_from_slice(&group.node_rows);
        visible_item_count += 1;
    }
    visible_item_count
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AssetBrowserSlotBinding<'a> {
    pub(crate) logical_index: usize,
    pub(crate) y_offset: f32,
    pub(crate) selected: bool,
    pub(crate) item: &'a AssetBrowserPaintItem,
}

fn finite_positive(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssetBrowserLogicalPaintGeneration, AssetBrowserPaintItem, AssetBrowserThumbnailPaintItem,
        AssetBrowserVirtualization,
    };

    #[test]
    fn empty_logical_paint_generation_has_a_safe_lookup_stride() {
        let generation = AssetBrowserLogicalPaintGeneration::default();

        assert!(generation.is_empty());
        assert_eq!(generation.len(), 0);
        assert!(generation.get(0).is_none());
    }

    #[test]
    fn one_row_scroll_rebinds_only_the_entering_physical_row() {
        let virtualization = virtualization(20, 6, 2);
        let initial = (0..6)
            .map(|slot| virtualization.binding(0.0, slot).unwrap().logical_index)
            .collect::<Vec<_>>();
        let scrolled = (0..6)
            .map(|slot| virtualization.binding(10.0, slot).unwrap().logical_index)
            .collect::<Vec<_>>();

        assert_eq!(initial, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(scrolled, vec![6, 7, 2, 3, 4, 5]);
        assert_eq!(
            initial
                .iter()
                .zip(&scrolled)
                .filter(|(left, right)| left != right)
                .count(),
            2
        );
        for (slot, logical_index) in scrolled.into_iter().enumerate() {
            assert_eq!(logical_index % 2, slot % 2);
        }
        assert_eq!(virtualization.binding(10.0, 0).unwrap().y_offset, 30.0);
        assert_eq!(virtualization.binding(10.0, 2).unwrap().y_offset, 0.0);
    }

    #[test]
    fn bottom_window_backfills_the_materialized_rows() {
        let virtualization = virtualization(10, 6, 2);
        let mut logical_indices = (0..6)
            .map(|slot| {
                virtualization
                    .binding(10_000.0, slot)
                    .unwrap()
                    .logical_index
            })
            .collect::<Vec<_>>();

        logical_indices.sort_unstable();
        assert_eq!(logical_indices, vec![4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn partial_row_pool_falls_back_without_omitting_logical_items() {
        let virtualization = virtualization(20, 5, 3);
        let logical_indices = (0..5)
            .map(|slot| virtualization.binding(10.0, slot).unwrap().logical_index)
            .collect::<Vec<_>>();

        assert_eq!(logical_indices, vec![3, 4, 5, 6, 7]);
    }

    fn virtualization(
        item_count: usize,
        materialized_item_count: usize,
        columns: usize,
    ) -> AssetBrowserVirtualization {
        let items = (0..item_count)
            .map(|index| {
                AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
                    name: format!("Asset {index}"),
                    source_file_name: String::new(),
                    file_extension: String::new(),
                    name_continuation: String::new(),
                    type_label: String::new(),
                    type_label_width: 0.0,
                    state_label: String::new(),
                    visual_variant: String::new(),
                    preview_artifact_path: String::new(),
                })
            })
            .collect::<Vec<_>>();
        AssetBrowserVirtualization::new(
            AssetBrowserLogicalPaintGeneration::from_chunks(vec![items.into()]),
            Vec::new(),
            materialized_item_count,
            columns,
            0.0,
            10.0,
            0.0,
            0,
        )
    }
}
use super::controls::BROWSER_CONTENT_LIST_ROW_HEIGHT;
use super::thumbnail_grid::AssetThumbnailGridMetrics;
use crate::ui::workbench::snapshot::AssetViewMode;

pub(crate) fn asset_browser_materialized_item_budget(
    view_mode: AssetViewMode,
    viewport_height: f32,
    item_count: usize,
    overscan_rows: usize,
) -> usize {
    if item_count == 0 || !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 0;
    }
    match view_mode {
        AssetViewMode::List => {
            let visible_rows = (viewport_height / BROWSER_CONTENT_LIST_ROW_HEIGHT)
                .ceil()
                .max(1.0) as usize;
            item_count.min(visible_rows.saturating_add(overscan_rows.saturating_mul(2)))
        }
        AssetViewMode::Thumbnail => {
            AssetThumbnailGridMetrics::conservative_materialized_item_budget(
                viewport_height,
                item_count,
                overscan_rows,
            )
        }
    }
}
