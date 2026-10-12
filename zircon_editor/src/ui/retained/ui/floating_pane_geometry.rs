use crate::ui::retained_host::host_contract::FrameRect;

const FLOATING_PANE_BOTTOM_BORDER_PX: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FloatingPaneContentSize {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

pub(crate) fn floating_pane_content_size(
    window_width: f32,
    window_height: f32,
    header_frame_height: f32,
    fallback_header_height: f32,
) -> FloatingPaneContentSize {
    FloatingPaneContentSize {
        width: window_width,
        height: floating_pane_content_height(
            window_height,
            header_frame_height,
            fallback_header_height,
        ),
    }
}

pub(crate) fn floating_pane_content_frame(
    window_frame: &FrameRect,
    header_frame: &FrameRect,
    fallback_header_height: f32,
) -> FrameRect {
    let header_height =
        resolved_floating_header_height(header_frame.height, fallback_header_height);
    FrameRect {
        x: window_frame.x,
        y: window_frame.y + header_height,
        width: window_frame.width,
        height: floating_pane_content_height(
            window_frame.height,
            header_frame.height,
            fallback_header_height,
        ),
    }
}

fn floating_pane_content_height(
    window_height: f32,
    header_frame_height: f32,
    fallback_header_height: f32,
) -> f32 {
    (window_height
        - resolved_floating_header_height(header_frame_height, fallback_header_height)
        - FLOATING_PANE_BOTTOM_BORDER_PX)
        .max(0.0)
}

fn resolved_floating_header_height(header_frame_height: f32, fallback_header_height: f32) -> f32 {
    if header_frame_height > 0.0 {
        header_frame_height
    } else {
        fallback_header_height
    }
}

#[cfg(test)]
#[path = "tests/floating_pane_geometry.rs"]
mod tests;
