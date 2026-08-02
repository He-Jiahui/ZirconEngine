use zircon_runtime_interface::ui::layout::UiFrame;

const GRID_PADDING: f32 = 8.0;
const GRID_GAP: f32 = 8.0;
const CARD_MIN_WIDTH: f32 = 104.0;
const CARD_MAX_WIDTH: f32 = 132.0;
const CARD_HEIGHT_RATIO: f32 = 1.14;
const CARD_MIN_HEIGHT: f32 = 146.0;
const CARD_MAX_HEIGHT: f32 = 150.0;
const MAX_COLUMNS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AssetThumbnailGridMetrics {
    columns: usize,
    card_width: f32,
    card_height: f32,
    item_count: usize,
}

impl AssetThumbnailGridMetrics {
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
mod tests {
    use super::*;

    #[test]
    fn responsive_grid_uses_columns_and_preserves_scrollable_content_extent() {
        let metrics = AssetThumbnailGridMetrics::new(420.0, 12);

        assert_eq!(metrics.columns(), 3);
        assert!(metrics.content_extent() > 600.0);
        let second = metrics.item_frame(1).expect("second thumbnail");
        let fourth = metrics.item_frame(3).expect("fourth thumbnail");
        assert!(second.x > 140.0);
        assert_eq!(fourth.x, 8.0);
        assert!(fourth.y > 150.0);
    }

    #[test]
    fn grid_omits_cards_when_the_viewport_cannot_fit_the_minimum_card_width() {
        let minimum_viewport_width = GRID_PADDING * 2.0 + CARD_MIN_WIDTH;
        let too_narrow = AssetThumbnailGridMetrics::new(minimum_viewport_width - 0.1, 12);
        let exact_fit = AssetThumbnailGridMetrics::new(minimum_viewport_width, 12);

        assert_eq!(too_narrow.columns(), 0);
        assert!(too_narrow.item_frame(0).is_none());
        assert_eq!(too_narrow.content_extent(), 0.0);

        assert_eq!(exact_fit.columns(), 1);
        assert_eq!(
            exact_fit.item_frame(0).expect("minimum-width card").width,
            CARD_MIN_WIDTH
        );
    }

    #[test]
    fn grid_rejects_non_finite_or_collapsed_viewports() {
        for viewport_width in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let metrics = AssetThumbnailGridMetrics::new(viewport_width, 1);

            assert_eq!(metrics.columns(), 0);
            assert!(metrics.item_frame(0).is_none());
            assert_eq!(metrics.content_extent(), 0.0);
        }
    }
}
