use std::ops::Range;

use super::super::super::data::FrameRect;
use super::super::super::settings_window_geometry::SettingsWindowLayout;

const SETTINGS_WINDOW_PAINT_OVERSCAN_ROWS: usize = 1;

pub(super) fn settings_window_visible_rows(
    list: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
    layout: &SettingsWindowLayout,
) -> Range<usize> {
    visible_rows(
        list,
        clip,
        row_count,
        layout.setting_row_height,
        layout.setting_scroll_offset(),
    )
}

pub(super) fn category_visible_rows(
    list: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
    layout: &SettingsWindowLayout,
) -> Range<usize> {
    visible_rows(
        list,
        clip,
        row_count,
        layout.category_row_height,
        layout.category_scroll_offset(),
    )
}

fn visible_rows(
    list: &FrameRect,
    clip: &FrameRect,
    row_count: usize,
    row_height: f32,
    scroll_offset: f32,
) -> Range<usize> {
    if row_count == 0 || !valid_frame(list) || !valid_frame(clip) || row_height <= 0.0 {
        return 0..0;
    }
    if clip.x >= list.x + list.width || clip.x + clip.width <= list.x {
        return 0..0;
    }
    let visible_top = clip.y.max(list.y);
    let visible_bottom = (clip.y + clip.height).min(list.y + list.height);
    if visible_bottom <= visible_top {
        return 0..0;
    }
    let first = ((visible_top - list.y + scroll_offset) / row_height)
        .floor()
        .max(0.0) as usize;
    let end = ((visible_bottom - list.y + scroll_offset) / row_height)
        .ceil()
        .max(0.0) as usize;
    let end = end.min(row_count);
    if first >= end {
        return 0..0;
    }
    first.saturating_sub(SETTINGS_WINDOW_PAINT_OVERSCAN_ROWS)
        ..end
            .saturating_add(SETTINGS_WINDOW_PAINT_OVERSCAN_ROWS)
            .min(row_count)
}

pub(super) fn valid_frame(frame: &FrameRect) -> bool {
    [frame.x, frame.y, frame.width, frame.height]
        .into_iter()
        .all(f32::is_finite)
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x = left.x.max(right.x);
    let y = left.y.max(right.y);
    let right_edge = (left.x + left.width).min(right.x + right.width);
    let bottom_edge = (left.y + left.height).min(right.y + right.height);
    (right_edge > x && bottom_edge > y).then_some(FrameRect {
        x,
        y,
        width: right_edge - x,
        height: bottom_edge - y,
    })
}

#[cfg(test)]
#[path = "tests/visible_rows.rs"]
mod tests;
