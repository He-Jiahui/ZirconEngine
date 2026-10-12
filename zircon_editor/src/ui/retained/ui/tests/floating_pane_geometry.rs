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
