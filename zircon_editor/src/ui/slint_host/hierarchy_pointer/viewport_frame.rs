use zircon_ui::UiFrame;

use super::hierarchy_pointer_layout::HierarchyPointerLayout;

pub(super) fn viewport_frame(layout: &HierarchyPointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        layout.pane_width.max(0.0),
        layout.pane_height.max(0.0),
    )
}
