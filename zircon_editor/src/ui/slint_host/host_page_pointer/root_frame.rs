use zircon_runtime::ui::layout::UiFrame;

use super::workbench_host_page_pointer_layout::WorkbenchHostPagePointerLayout;

pub(super) fn root_frame(layout: &WorkbenchHostPagePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        (layout.strip_frame.x + layout.strip_frame.width).max(1.0),
        (layout.strip_frame.y + layout.strip_frame.height).max(1.0),
    )
}
