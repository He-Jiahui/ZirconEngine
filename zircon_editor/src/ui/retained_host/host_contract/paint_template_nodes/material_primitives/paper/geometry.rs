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
mod tests {
    use super::*;

    #[test]
    fn paper_frame_stays_inside_tight_parent_bounds() {
        let parent = FrameRect {
            x: 10.4,
            y: 20.8,
            width: 0.4,
            height: 0.6,
        };
        let frame = bounded_paper_rect(&parent);

        assert!(frame.x >= parent.x);
        assert!(frame.y >= parent.y);
        assert!(frame.right() <= parent.right());
        assert!(frame.bottom() <= parent.bottom());
    }
}
