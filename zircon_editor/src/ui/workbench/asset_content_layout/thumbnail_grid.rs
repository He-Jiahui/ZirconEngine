use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

const GRID_PADDING: f32 = 8.0;
const GRID_GAP: f32 = 8.0;
const CARD_MIN_WIDTH: f32 = 104.0;
const CARD_MAX_WIDTH: f32 = 132.0;
const CARD_HEIGHT_RATIO: f32 = 1.14;
const CARD_MIN_HEIGHT: f32 = 146.0;
const CARD_MAX_HEIGHT: f32 = 150.0;
const MAX_COLUMNS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 同时驱动缩略图排布、滚动长度和命中的网格度量；item数指逻辑总量。
pub(crate) struct AssetThumbnailGridMetrics {
    columns: usize,
    card_width: f32,
    card_height: f32,
    item_count: usize,
}

impl AssetThumbnailGridMetrics {
    /// 按视口宽度决定列数；不能容纳最小卡片时形成空网格。
    pub(crate) fn new(viewport_width: f32, item_count: usize) -> Self {
        if item_count == 0 || !viewport_width.is_finite() {
            return Self::empty(item_count);
        }

        let inner_width = (viewport_width - GRID_PADDING * 2.0).max(0.0);
        if inner_width < CARD_MIN_WIDTH {
            return Self::empty(item_count);
        }
        let columns = (((inner_width + GRID_GAP) / (CARD_MIN_WIDTH + GRID_GAP))
            .floor()
            .max(1.0) as usize)
            .min(item_count)
            .min(MAX_COLUMNS);
        let available_card_width =
            (inner_width - GRID_GAP * columns.saturating_sub(1) as f32).max(0.0) / columns as f32;
        let card_width = available_card_width.min(CARD_MAX_WIDTH);
        let card_height = (card_width * CARD_HEIGHT_RATIO).clamp(CARD_MIN_HEIGHT, CARD_MAX_HEIGHT);

        Self {
            columns,
            card_width,
            card_height,
            item_count,
        }
    }

    fn empty(item_count: usize) -> Self {
        Self {
            columns: 0,
            card_width: 0.0,
            card_height: 0.0,
            item_count,
        }
    }

    pub(crate) fn columns(self) -> usize {
        self.columns
    }

    /// 排布前保守预算，使用最大列数；已知几何后应改用实际列数预算。
    pub(crate) fn conservative_materialized_item_budget(
        viewport_height: f32,
        item_count: usize,
        overscan_rows: usize,
    ) -> usize {
        if item_count == 0 || !viewport_height.is_finite() || viewport_height <= 0.0 {
            return 0;
        }
        let visible_rows = (viewport_height / CARD_MIN_HEIGHT).ceil().max(1.0) as usize;
        let retained_rows = visible_rows.saturating_add(overscan_rows.saturating_mul(2));
        item_count.min(retained_rows.saturating_mul(MAX_COLUMNS))
    }

    /// 按实际列数和overscan控制物化slot数量，不随逻辑资产总量增长。
    pub(crate) fn materialized_item_budget(
        self,
        viewport_height: f32,
        overscan_rows: usize,
    ) -> usize {
        if self.item_count == 0
            || self.columns == 0
            || !viewport_height.is_finite()
            || viewport_height <= 0.0
        {
            return 0;
        }
        let row_stride = self.card_height + GRID_GAP;
        let visible_rows = (viewport_height / row_stride).ceil().max(1.0) as usize;
        let retained_rows = visible_rows.saturating_add(overscan_rows.saturating_mul(2));
        self.item_count
            .min(retained_rows.saturating_mul(self.columns))
    }

    pub(crate) fn item_frame(self, index: usize) -> Option<UiFrame> {
        if index >= self.item_count || self.columns == 0 {
            return None;
        }
        let column = index % self.columns;
        let row = index / self.columns;
        Some(UiFrame::new(
            GRID_PADDING + column as f32 * (self.card_width + GRID_GAP),
            GRID_PADDING + row as f32 * (self.card_height + GRID_GAP),
            self.card_width,
            self.card_height,
        ))
    }

    /// 输入未滚动的网格局部点；宿主先还原滚动位移，卡片间隙不命中。
    pub(crate) fn item_index_at_point(self, point: UiPoint) -> Option<usize> {
        if self.columns == 0 || point.x < GRID_PADDING || point.y < GRID_PADDING {
            return None;
        }

        let column = ((point.x - GRID_PADDING) / (self.card_width + GRID_GAP)).floor() as usize;
        let row = ((point.y - GRID_PADDING) / (self.card_height + GRID_GAP)).floor() as usize;
        if column >= self.columns {
            return None;
        }

        let index = row.checked_mul(self.columns)?.checked_add(column)?;
        self.item_frame(index)
            .filter(|frame| frame.contains_point(point))
            .map(|_| index)
    }

    pub(crate) fn content_extent(self) -> f32 {
        if self.columns == 0 {
            return 0.0;
        }
        let rows = self.item_count.div_ceil(self.columns);
        GRID_PADDING * 2.0
            + rows as f32 * self.card_height
            + rows.saturating_sub(1) as f32 * GRID_GAP
    }
}

#[cfg(test)]
#[path = "tests/thumbnail_grid.rs"]
mod tests;
