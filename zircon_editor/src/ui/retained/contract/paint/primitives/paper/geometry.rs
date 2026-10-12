use super::super::super::super::data::FrameRect;
use super::super::bounded_extent;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn bounded_paper_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: bounded_extent(rect.width),
        height: bounded_extent(rect.height),
    }
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
