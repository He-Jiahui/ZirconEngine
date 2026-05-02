use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_page_pointer_layout::HostPagePointerLayout;

pub(super) fn root_frame(layout: &HostPagePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        (layout.strip_frame.x + layout.strip_frame.width).max(1.0),
        (layout.strip_frame.y + layout.strip_frame.height).max(1.0),
    )
}
