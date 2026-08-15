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
mod tests {
    use super::{floating_pane_content_frame, floating_pane_content_size};
    use crate::ui::retained_host::host_contract::FrameRect;

    #[test]
    fn floating_content_geometry_prefers_each_window_header_and_reserves_the_border() {
        let window = FrameRect {
            x: 40.0,
            y: 60.0,
            width: 640.0,
            height: 480.0,
        };
        let header = FrameRect {
            x: 40.0,
            y: 60.0,
            width: 640.0,
            height: 46.0,
        };

        let content = floating_pane_content_frame(&window, &header, 28.0);
        let size = floating_pane_content_size(window.width, window.height, header.height, 28.0);

        assert_eq!(content.x, 40.0);
        assert_eq!(content.y, 106.0);
        assert_eq!(content.width, 640.0);
        assert_eq!(content.height, 433.0);
        assert_eq!(size.width, content.width);
        assert_eq!(size.height, content.height);
    }
}
