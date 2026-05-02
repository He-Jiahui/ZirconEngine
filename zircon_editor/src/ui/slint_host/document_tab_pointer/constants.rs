use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(in crate::ui::slint_host::document_tab_pointer) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(in crate::ui::slint_host::document_tab_pointer) const SURFACE_NODE_ID_BASE: u64 = 10;
pub(in crate::ui::slint_host::document_tab_pointer) const TAB_NODE_ID_BASE: u64 = 100;
pub(in crate::ui::slint_host::document_tab_pointer) const CLOSE_NODE_ID_BASE: u64 = 10_000;

pub(in crate::ui::slint_host::document_tab_pointer) const STRIP_X: f32 = 8.0;
pub(in crate::ui::slint_host::document_tab_pointer) const STRIP_Y: f32 = 1.0;
pub(in crate::ui::slint_host::document_tab_pointer) const TAB_GAP: f32 = 2.0;
pub(in crate::ui::slint_host::document_tab_pointer) const TAB_HEIGHT: f32 = 30.0;
pub(in crate::ui::slint_host::document_tab_pointer) const TAB_MIN_WIDTH: f32 = 92.0;
pub(in crate::ui::slint_host::document_tab_pointer) const CLOSEABLE_TAB_MIN_WIDTH: f32 = 114.0;
pub(in crate::ui::slint_host::document_tab_pointer) const CLOSE_X_OFFSET: f32 = 24.0;
pub(in crate::ui::slint_host::document_tab_pointer) const CLOSE_Y_OFFSET: f32 = 7.0;
pub(in crate::ui::slint_host::document_tab_pointer) const CLOSE_EXTENT: f32 = 16.0;
