use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::{avatar_bounded_extent, AVATAR_DEFAULT_EDGE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_frame(
    rect: &FrameRect,
) -> FrameRect {
    let available_height = avatar_bounded_extent(rect.height);
    let size = avatar_bounded_extent(rect.width)
        .min(available_height)
        .min(AVATAR_DEFAULT_EDGE);
    FrameRect {
        x: rect.x,
        y: rect.y + (available_height - size) * 0.5,
        width: size,
        height: size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_frame_stays_inside_tight_parent_bounds() {
        let parent = FrameRect {
            x: 10.4,
            y: 20.8,
            width: 0.4,
            height: 0.6,
        };
        let frame = avatar_frame(&parent);

        assert!(frame.x >= parent.x);
        assert!(frame.y >= parent.y);
        assert!(frame.right() <= parent.right());
        assert!(frame.bottom() <= parent.bottom());
    }
}
