use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

pub(crate) const WELCOME_RECENT_ROW_HEIGHT: f32 = 54.0;
pub(crate) const WELCOME_RECENT_ROW_GAP: f32 = 8.0;

const OUTER_INSET: f32 = 18.0;
const RECENT_PANEL_TOP_INSET: f32 = 18.0;
const RECENT_HEADER_HEIGHT: f32 = 46.0;
const RECENT_HEADER_LIST_GAP: f32 = 8.0;
const RECENT_PANEL_MIN_WIDTH: f32 = 220.0;
const RECENT_PANEL_MAX_WIDTH: f32 = 320.0;
const MAIN_PANEL_MIN_WIDTH: f32 = 280.0;
const ROW_INSET: f32 = 8.0;
const ROW_TEXT_INSET: f32 = 12.0;
const ROW_ACTION_INSET: f32 = 8.0;
const ROW_ACTION_GAP: f32 = 4.0;
const ROW_ACTION_HEIGHT: f32 = 24.0;
const OPEN_ACTION_WIDTH: f32 = 52.0;
const REMOVE_ACTION_WIDTH: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WelcomeRecentRowGeometry {
    pub row: UiFrame,
    pub text: UiFrame,
    pub open: UiFrame,
    pub remove: UiFrame,
}

pub(crate) fn welcome_recent_viewport(pane_size: UiSize) -> UiFrame {
    let outer_width = (pane_size.width - OUTER_INSET * 2.0).max(0.0);
    let width_available_after_main = (outer_width - MAIN_PANEL_MIN_WIDTH).max(0.0);
    let recent_min = RECENT_PANEL_MIN_WIDTH.min(outer_width);
    let recent_max = RECENT_PANEL_MAX_WIDTH.min(outer_width);
    let recent_width = if recent_max >= recent_min {
        width_available_after_main.clamp(recent_min, recent_max)
    } else {
        recent_max
    };
    let y = OUTER_INSET + RECENT_PANEL_TOP_INSET + RECENT_HEADER_HEIGHT + RECENT_HEADER_LIST_GAP;

    UiFrame::new(
        OUTER_INSET,
        y,
        recent_width,
        (pane_size.height - y - OUTER_INSET).max(0.0),
    )
}

pub(crate) fn welcome_recent_row_geometry(
    viewport: UiFrame,
    index: usize,
    scroll_offset: f32,
) -> WelcomeRecentRowGeometry {
    let row = UiFrame::new(
        viewport.x + ROW_INSET,
        viewport.y
            + ROW_INSET
            + index as f32 * (WELCOME_RECENT_ROW_HEIGHT + WELCOME_RECENT_ROW_GAP)
            - scroll_offset,
        (viewport.width - ROW_INSET * 2.0).max(0.0),
        WELCOME_RECENT_ROW_HEIGHT,
    );
    let action_y = row.y + (row.height - ROW_ACTION_HEIGHT) * 0.5;
    let remove_width = REMOVE_ACTION_WIDTH.min((row.width - ROW_ACTION_INSET * 2.0).max(0.0));
    let remove = UiFrame::new(
        (row.right() - ROW_ACTION_INSET - remove_width).max(row.x),
        action_y,
        remove_width,
        ROW_ACTION_HEIGHT.min(row.height.max(0.0)),
    );
    let open_width = OPEN_ACTION_WIDTH.min((remove.x - ROW_ACTION_GAP - row.x).max(0.0));
    let open = UiFrame::new(
        remove.x - ROW_ACTION_GAP - open_width,
        action_y,
        open_width,
        ROW_ACTION_HEIGHT.min(row.height.max(0.0)),
    );
    let text_x = row.x + ROW_TEXT_INSET;
    let text = UiFrame::new(
        text_x,
        row.y,
        (open.x - ROW_ACTION_GAP - text_x).max(0.0),
        row.height,
    );

    WelcomeRecentRowGeometry {
        row,
        text,
        open,
        remove,
    }
}

pub(crate) fn welcome_recent_content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        return 0.0;
    }
    ROW_INSET * 2.0
        + item_count as f32 * WELCOME_RECENT_ROW_HEIGHT
        + (item_count.saturating_sub(1)) as f32 * WELCOME_RECENT_ROW_GAP
}

pub(crate) fn welcome_recent_visible_row_count(viewport_height: f32, item_count: usize) -> usize {
    if item_count == 0 {
        return 0;
    }
    let inner_height = (viewport_height - ROW_INSET * 2.0).max(0.0);
    if inner_height <= 0.0 {
        return 0;
    }
    ((inner_height / (WELCOME_RECENT_ROW_HEIGHT + WELCOME_RECENT_ROW_GAP)).ceil() as usize)
        .min(item_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.01;

    #[test]
    fn welcome_recent_geometry_keeps_compact_rows_and_actions_inside_responsive_columns() {
        for (pane_width, expected_recent_width) in [(560.0, 244.0), (640.0, 320.0), (900.0, 320.0)]
        {
            let viewport = welcome_recent_viewport(UiSize::new(pane_width, 520.0));
            assert_close(viewport.width, expected_recent_width);

            let first = welcome_recent_row_geometry(viewport, 0, 0.0);
            let second = welcome_recent_row_geometry(viewport, 1, 0.0);
            assert_close(first.row.height, WELCOME_RECENT_ROW_HEIGHT);
            assert_close(
                second.row.y - first.row.y,
                WELCOME_RECENT_ROW_HEIGHT + WELCOME_RECENT_ROW_GAP,
            );
            assert!(first.text.x >= first.row.x);
            assert!(first.text.right() <= first.open.x);
            assert!(first.open.right() <= first.remove.x);
            assert!(first.remove.right() <= first.row.right());
            for action in [first.open, first.remove] {
                assert!(action.y >= first.row.y);
                assert!(action.bottom() <= first.row.bottom());
            }
        }
    }

    #[test]
    fn welcome_recent_geometry_derives_content_and_visible_rows_from_one_metric_owner() {
        assert_close(welcome_recent_content_height(0), 0.0);
        assert_close(
            welcome_recent_content_height(1),
            WELCOME_RECENT_ROW_HEIGHT + 16.0,
        );
        assert_close(
            welcome_recent_content_height(3),
            WELCOME_RECENT_ROW_HEIGHT * 3.0 + WELCOME_RECENT_ROW_GAP * 2.0 + 16.0,
        );
        assert_eq!(welcome_recent_visible_row_count(0.0, 8), 0);
        assert_eq!(welcome_recent_visible_row_count(54.0 + 16.0, 8), 1);
        assert_eq!(welcome_recent_visible_row_count(116.0 + 16.0, 8), 2);
        assert_eq!(welcome_recent_visible_row_count(520.0, 2), 2);
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}
