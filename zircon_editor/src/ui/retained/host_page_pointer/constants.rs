use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const STRIP_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(super) const OVERFLOW_NODE_ID: UiNodeId = UiNodeId::new(3);
pub(super) const TAB_NODE_ID_BASE: u64 = 100;
pub(super) const CLOSE_NODE_ID_BASE: u64 = 10_000;
pub(super) const HOST_PAGE_ROUTE_ID_BASE: u64 = 58_000;
pub(crate) const HOST_PAGE_OVERFLOW_POINTER_INDEX: i32 = -2;
