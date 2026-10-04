use super::super::super::super::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::bounded_extent;

pub(super) fn centered_square(rect: &FrameRect) -> FrameRect {
    let width = bounded_extent(rect.width);
    let height = bounded_extent(rect.height);
    let size = width.min(height);
    FrameRect {
        x: rect.x + (width - size) * 0.5,
        y: rect.y + (height - size) * 0.5,
        width: size,
        height: size,
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
