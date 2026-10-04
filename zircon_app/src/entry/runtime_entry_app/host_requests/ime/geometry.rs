//! IME 光标区域的逻辑坐标与几何有效性边界。
//! 原始 ABI 值已为逻辑单位，宿主不得再次按窗口比例缩放。

use winit::dpi::{LogicalPosition, LogicalSize};
use zircon_runtime_interface::{ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCursorAreaV1};

const MAX_NATIVE_IME_COORDINATE: f64 = i32::MAX as f64;

/// Converts the actual layout caret rectangle to the public window-relative candidate ABI.
/// Windows IMM/TSF receives signed LONG coordinates and nonnegative LONG extents, so values that
/// cannot be represented are rejected before either the runtime contract or Winit is touched.
pub(in crate::entry::runtime_entry_app) fn ime_candidate_rect(
    area: ZrRuntimeImeCursorAreaV1,
) -> Option<ZrRuntimeImeCandidateRectV2> {
    if !(area.x.is_finite()
        && area.y.is_finite()
        && area.width.is_finite()
        && area.height.is_finite()
        && area.width >= 0.0
        && area.height > 0.0)
    {
        return None;
    }
    let origin_x = checked_coordinate(area.x)?;
    let origin_y = checked_coordinate(area.y)?;
    let extent_width = checked_extent(area.width)?;
    let extent_height = checked_extent(area.height)?;
    if i64::from(origin_x) + i64::from(extent_width) > i64::from(i32::MAX)
        || i64::from(origin_y) + i64::from(extent_height) > i64::from(i32::MAX)
    {
        return None;
    }
    Some(ZrRuntimeImeCandidateRectV2::window_relative(
        origin_x,
        origin_y,
        extent_width,
        extent_height,
    ))
}

pub(super) fn ime_candidate_rect_to_winit(
    rect: ZrRuntimeImeCandidateRectV2,
) -> (winit::dpi::Position, winit::dpi::Size) {
    (
        LogicalPosition::new(rect.origin_x as f64, rect.origin_y as f64).into(),
        LogicalSize::new(rect.extent_width as f64, rect.extent_height as f64).into(),
    )
}

fn checked_coordinate(value: f32) -> Option<i32> {
    let value = f64::from(value).round();
    (value.abs() <= MAX_NATIVE_IME_COORDINATE).then_some(value as i32)
}

fn checked_extent(value: f32) -> Option<u32> {
    let value = f64::from(value).ceil();
    // A caret is a valid zero-width topology.  The height remains checked by
    // `ime_candidate_rect`, while this conversion preserves the real layout
    // anchor instead of inventing a one-pixel width.
    (value >= 0.0 && value <= MAX_NATIVE_IME_COORDINATE).then_some(value as u32)
}

pub(super) fn ime_logical_cursor_area(
    area: ZrRuntimeImeCursorAreaV1,
) -> Option<(winit::dpi::Position, winit::dpi::Size)> {
    (area.x.is_finite()
        && area.y.is_finite()
        && area.width.is_finite()
        && area.height.is_finite()
        && area.width >= 0.0
        && area.height >= 0.0)
        .then(|| (ime_logical_position(area), ime_logical_size(area)))
}

pub(super) fn ime_logical_position(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Position {
    LogicalPosition::new(area.x as f64, area.y as f64).into()
}

pub(super) fn ime_logical_size(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Size {
    LogicalSize::new(area.width as f64, area.height as f64).into()
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
