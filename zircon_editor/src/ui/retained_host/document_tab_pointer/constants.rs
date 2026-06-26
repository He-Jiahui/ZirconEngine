use zircon_runtime_interface::ui::event_ui::UiNodeId;

use crate::ui::workbench::document_tabs::{
    DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH, DOCUMENT_TAB_CLOSE_EXTENT, DOCUMENT_TAB_CLOSE_RIGHT_INSET,
    DOCUMENT_TAB_CLOSE_TOP_INSET, DOCUMENT_TAB_GAP, DOCUMENT_TAB_HEIGHT, DOCUMENT_TAB_MIN_WIDTH,
    DOCUMENT_TAB_STRIP_X, DOCUMENT_TAB_STRIP_Y,
};

pub(in crate::ui::retained_host::document_tab_pointer) const ROOT_NODE_ID: UiNodeId =
    UiNodeId::new(1);
pub(in crate::ui::retained_host::document_tab_pointer) const SURFACE_NODE_ID_BASE: u64 = 10;
pub(in crate::ui::retained_host::document_tab_pointer) const TAB_NODE_ID_BASE: u64 = 100;
pub(in crate::ui::retained_host::document_tab_pointer) const CLOSE_NODE_ID_BASE: u64 = 10_000;
pub(in crate::ui::retained_host::document_tab_pointer) const DOCUMENT_TAB_ROUTE_ID_BASE: u64 =
    52_000;

pub(in crate::ui::retained_host::document_tab_pointer) const STRIP_X: f32 = DOCUMENT_TAB_STRIP_X;
pub(in crate::ui::retained_host::document_tab_pointer) const STRIP_Y: f32 = DOCUMENT_TAB_STRIP_Y;
pub(in crate::ui::retained_host::document_tab_pointer) const TAB_GAP: f32 = DOCUMENT_TAB_GAP;
pub(in crate::ui::retained_host::document_tab_pointer) const TAB_HEIGHT: f32 = DOCUMENT_TAB_HEIGHT;
pub(in crate::ui::retained_host::document_tab_pointer) const TAB_MIN_WIDTH: f32 =
    DOCUMENT_TAB_MIN_WIDTH;
pub(in crate::ui::retained_host::document_tab_pointer) const CLOSEABLE_TAB_MIN_WIDTH: f32 =
    DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH;
pub(in crate::ui::retained_host::document_tab_pointer) const CLOSE_X_OFFSET: f32 =
    DOCUMENT_TAB_CLOSE_RIGHT_INSET + DOCUMENT_TAB_CLOSE_EXTENT;
pub(in crate::ui::retained_host::document_tab_pointer) const CLOSE_Y_OFFSET: f32 =
    DOCUMENT_TAB_CLOSE_TOP_INSET;
pub(in crate::ui::retained_host::document_tab_pointer) const CLOSE_EXTENT: f32 =
    DOCUMENT_TAB_CLOSE_EXTENT;
