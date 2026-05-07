use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use super::constants::{
    POPUP_ANCHOR_GAP, POPUP_EDGE_MARGIN, POPUP_MIN_HEIGHT, POPUP_PADDING, POPUP_ROW_GAP,
    POPUP_ROW_HEIGHT, POPUP_WIDTHS, WINDOW_MENU_INDEX,
};
use super::host_menu_pointer_layout::HostMenuPointerLayout;
use super::menu_items_for_layout::menu_items_for_layout;
use crate::ui::workbench::window_registry::MenuOverflowMode;

/// Resolved popup geometry used by both the shared hit surface and scroll metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::slint_host::menu_pointer) struct PopupGridLayout {
    pub frame: UiFrame,
    pub content_frame: UiFrame,
    pub rows_per_column: usize,
    pub column_width: f32,
    pub row_step: f32,
    pub scroll_offset: f32,
    pub viewport_extent: f32,
    pub content_extent: f32,
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_content_frame(
    popup_frame: UiFrame,
) -> UiFrame {
    UiFrame::new(
        popup_frame.x + POPUP_PADDING,
        popup_frame.y + POPUP_PADDING,
        (popup_frame.width - POPUP_PADDING * 2.0).max(0.0),
        (popup_frame.height - POPUP_PADDING * 2.0).max(0.0),
    )
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        item_count as f32 * POPUP_ROW_HEIGHT + (item_count as f32 - 1.0) * POPUP_ROW_GAP
    }
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_viewport_extent(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
) -> f32 {
    popup_grid_layout(layout, menu_index, 0.0, 0.0).viewport_extent
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_scroll_metrics(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
) -> (f32, f32) {
    let metrics = popup_grid_layout(layout, menu_index, 0.0, 0.0);
    (metrics.viewport_extent, metrics.content_extent)
}

pub(in crate::ui::slint_host::menu_pointer) fn menu_button_frame(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
    menu_bar_scroll_offset: f32,
) -> UiFrame {
    layout
        .button_frames
        .get(menu_index)
        .copied()
        .map(|frame| {
            UiFrame::new(
                frame.x - menu_bar_scroll_offset,
                frame.y,
                frame.width,
                frame.height,
            )
        })
        .unwrap_or_default()
}

pub(in crate::ui::slint_host::menu_pointer) fn clipped_menu_button_frame(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
    menu_bar_scroll_offset: f32,
) -> Option<UiFrame> {
    intersect_frame(
        menu_button_frame(layout, menu_index, menu_bar_scroll_offset),
        menu_bar_viewport_frame(layout),
    )
}

pub(in crate::ui::slint_host::menu_pointer) fn menu_bar_viewport_frame(
    layout: &HostMenuPointerLayout,
) -> UiFrame {
    let button_bottom = layout
        .button_frames
        .iter()
        .map(|frame| frame.y + frame.height)
        .fold(layout.shell_frame.y, f32::max);
    let height = (button_bottom - layout.shell_frame.y)
        .max(28.0)
        .min(layout.shell_frame.height.max(0.0));
    UiFrame::new(
        layout.shell_frame.x,
        layout.shell_frame.y,
        layout.shell_frame.width.max(0.0),
        height,
    )
}

pub(in crate::ui::slint_host::menu_pointer) fn menu_bar_contains_point(
    layout: &HostMenuPointerLayout,
    point: UiPoint,
) -> bool {
    let viewport = menu_bar_viewport_frame(layout);
    point.x >= viewport.x
        && point.x <= viewport.x + viewport.width
        && point.y >= viewport.y
        && point.y <= viewport.y + viewport.height
}

pub(in crate::ui::slint_host::menu_pointer) fn menu_bar_max_scroll_offset(
    layout: &HostMenuPointerLayout,
) -> f32 {
    (layout.menu_bar_content_width - layout.shell_frame.width.max(0.0)).max(0.0)
}

pub(in crate::ui::slint_host::menu_pointer) fn clamped_menu_bar_scroll_offset(
    layout: &HostMenuPointerLayout,
    offset: f32,
) -> f32 {
    offset.clamp(0.0, menu_bar_max_scroll_offset(layout))
}

pub(in crate::ui::slint_host::menu_pointer) fn submenu_popup_grid_layout(
    layout: &HostMenuPointerLayout,
    anchor_frame: UiFrame,
    item_count: usize,
) -> PopupGridLayout {
    let shell_width = layout.shell_frame.width.max(0.0);
    let shell_height = layout.shell_frame.height.max(0.0);
    let shell_right = layout.shell_frame.x + layout.shell_frame.width;
    let shell_bottom = layout.shell_frame.y + layout.shell_frame.height;
    let base_width = POPUP_WIDTHS[0];
    let available_height = (shell_height - POPUP_EDGE_MARGIN * 2.0)
        .max(0.0)
        .min(shell_height);
    let column_count = popup_column_count(layout, item_count, base_width, available_height);
    let rows_per_column = rows_per_column(item_count, column_count);
    let requested_height = popup_content_height(rows_per_column) + POPUP_PADDING * 2.0;
    let width = (base_width * column_count as f32).min(shell_width).max(0.0);
    let height = requested_height.min(available_height).max(0.0);
    let min_x = layout.shell_frame.x + POPUP_EDGE_MARGIN;
    let max_x = (shell_right - width - POPUP_EDGE_MARGIN).max(min_x);
    let right_x = anchor_frame.x + anchor_frame.width + POPUP_ANCHOR_GAP;
    let left_x = anchor_frame.x - width - POPUP_ANCHOR_GAP;
    let x = if right_x + width <= shell_right - POPUP_EDGE_MARGIN {
        right_x.clamp(min_x, max_x)
    } else {
        left_x.clamp(min_x, max_x)
    };
    let min_y = layout.shell_frame.y + POPUP_EDGE_MARGIN;
    let max_y = (shell_bottom - height - POPUP_EDGE_MARGIN).max(min_y);
    let y = anchor_frame.y.clamp(min_y, max_y);
    let frame = UiFrame::new(x, y, width, height);
    let content_frame = popup_content_frame(frame);
    let row_step = POPUP_ROW_HEIGHT + POPUP_ROW_GAP;
    let viewport_extent = content_frame.height;
    let content_extent = popup_content_height(rows_per_column);

    PopupGridLayout {
        frame,
        content_frame,
        rows_per_column,
        column_width: if column_count > 0 {
            content_frame.width / column_count as f32
        } else {
            content_frame.width
        },
        row_step,
        scroll_offset: 0.0,
        viewport_extent,
        content_extent,
    }
}

pub(in crate::ui::slint_host::menu_pointer) fn popup_grid_layout(
    layout: &HostMenuPointerLayout,
    menu_index: usize,
    scroll_offset: f32,
    menu_bar_scroll_offset: f32,
) -> PopupGridLayout {
    let item_count = menu_items_for_layout(layout, menu_index).len();
    let button_frame = menu_button_frame(layout, menu_index, menu_bar_scroll_offset);
    let shell_width = layout.shell_frame.width.max(0.0);
    let shell_height = layout.shell_frame.height.max(0.0);
    let shell_right = layout.shell_frame.x + layout.shell_frame.width;
    let shell_bottom = layout.shell_frame.y + layout.shell_frame.height;
    let popup_y = button_frame.y + button_frame.height + POPUP_ANCHOR_GAP;
    let base_width = popup_width_for_menu(menu_index);
    let single_column_requested_height = if menu_index == WINDOW_MENU_INDEX {
        layout.window_popup_height.max(POPUP_MIN_HEIGHT)
    } else {
        popup_content_height(item_count) + POPUP_PADDING * 2.0
    };
    let available_below = (shell_bottom - popup_y - POPUP_EDGE_MARGIN).max(0.0);
    let available_above = (button_frame.y - layout.shell_frame.y - POPUP_EDGE_MARGIN).max(0.0);
    let available_height = available_below
        .max(available_above)
        .max(POPUP_MIN_HEIGHT)
        .min(shell_height);
    let column_count = popup_column_count(layout, item_count, base_width, available_height);
    let rows_per_column = rows_per_column(item_count, column_count);
    let requested_height = if column_count > 1 {
        popup_content_height(rows_per_column) + POPUP_PADDING * 2.0
    } else {
        single_column_requested_height
    };
    let width = (base_width * column_count as f32).min(shell_width).max(0.0);
    let height = requested_height.min(available_height).max(0.0);
    let x = button_frame.x.clamp(
        layout.shell_frame.x,
        (shell_right - width).max(layout.shell_frame.x),
    );
    let max_y = (shell_bottom - height).max(layout.shell_frame.y);
    let y = if popup_y + height <= shell_bottom {
        popup_y.clamp(layout.shell_frame.y, max_y)
    } else {
        (button_frame.y - height - POPUP_ANCHOR_GAP).clamp(layout.shell_frame.y, max_y)
    };
    let frame = UiFrame::new(x, y, width, height);
    let content_frame = popup_content_frame(frame);
    let row_step = POPUP_ROW_HEIGHT + POPUP_ROW_GAP;
    let viewport_extent = content_frame.height;
    let content_extent = popup_content_height(rows_per_column);

    PopupGridLayout {
        frame,
        content_frame,
        rows_per_column,
        column_width: if column_count > 0 {
            content_frame.width / column_count as f32
        } else {
            content_frame.width
        },
        row_step,
        scroll_offset,
        viewport_extent,
        content_extent,
    }
}

fn popup_column_count(
    layout: &HostMenuPointerLayout,
    item_count: usize,
    column_width: f32,
    available_height: f32,
) -> usize {
    if layout.menu_overflow_mode != MenuOverflowMode::MultiColumn || item_count == 0 {
        return 1;
    }

    let usable_height = (available_height - POPUP_PADDING * 2.0).max(POPUP_ROW_HEIGHT);
    let rows_per_column = ((usable_height + POPUP_ROW_GAP) / (POPUP_ROW_HEIGHT + POPUP_ROW_GAP))
        .floor()
        .max(1.0) as usize;
    let requested_columns = item_count.div_ceil(rows_per_column).max(1);
    let available_columns = (layout.shell_frame.width.max(0.0) / column_width.max(1.0))
        .floor()
        .max(1.0) as usize;
    requested_columns.min(available_columns).max(1)
}

fn rows_per_column(item_count: usize, column_count: usize) -> usize {
    if item_count == 0 {
        0
    } else {
        item_count.div_ceil(column_count.max(1))
    }
}

fn popup_width_for_menu(menu_index: usize) -> f32 {
    POPUP_WIDTHS.get(menu_index).copied().unwrap_or(224.0)
}

fn intersect_frame(frame: UiFrame, clip: UiFrame) -> Option<UiFrame> {
    let x0 = frame.x.max(clip.x);
    let y0 = frame.y.max(clip.y);
    let x1 = (frame.x + frame.width).min(clip.x + clip.width);
    let y1 = (frame.y + frame.height).min(clip.y + clip.height);
    (x1 > x0 && y1 > y0).then(|| UiFrame::new(x0, y0, x1 - x0, y1 - y0))
}
