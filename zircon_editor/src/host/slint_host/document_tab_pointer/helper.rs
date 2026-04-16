use zircon_ui::{UiFrame, UiNodeId, UiStateFlags};

use super::constants::{
    CLOSEABLE_TAB_MIN_WIDTH, CLOSE_NODE_ID_BASE, TAB_MIN_WIDTH, TAB_NODE_ID_BASE,
};
use super::{
    workbench_document_tab_pointer_layout::WorkbenchDocumentTabPointerLayout,
    workbench_document_tab_pointer_surface::WorkbenchDocumentTabPointerSurface,
};

pub(in crate::host::slint_host::document_tab_pointer) fn tab_node_id(
    surface_index: usize,
    item_index: usize,
) -> UiNodeId {
    UiNodeId::new(TAB_NODE_ID_BASE + surface_index as u64 * 100 + item_index as u64)
}

pub(in crate::host::slint_host::document_tab_pointer) fn close_node_id(
    surface_index: usize,
    item_index: usize,
) -> UiNodeId {
    UiNodeId::new(CLOSE_NODE_ID_BASE + surface_index as u64 * 100 + item_index as u64)
}

pub(in crate::host::slint_host::document_tab_pointer) fn tab_min_width(
    surface: &WorkbenchDocumentTabPointerSurface,
    item_index: usize,
) -> f32 {
    surface
        .items
        .get(item_index)
        .map(|item| {
            if item.closeable {
                CLOSEABLE_TAB_MIN_WIDTH
            } else {
                TAB_MIN_WIDTH
            }
        })
        .unwrap_or(TAB_MIN_WIDTH)
}

pub(in crate::host::slint_host::document_tab_pointer) fn root_frame(
    layout: &WorkbenchDocumentTabPointerLayout,
) -> UiFrame {
    let max_x = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.x + surface.strip_frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.y + surface.strip_frame.height)
        .fold(1.0_f32, f32::max);
    UiFrame::new(0.0, 0.0, max_x.max(1.0), max_y.max(1.0))
}

pub(in crate::host::slint_host::document_tab_pointer) fn base_state(
    interactive: bool,
) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
