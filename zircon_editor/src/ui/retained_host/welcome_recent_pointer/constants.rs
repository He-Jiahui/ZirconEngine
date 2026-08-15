use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(in crate::ui::retained_host::welcome_recent_pointer) const ROOT_NODE_ID: UiNodeId =
    UiNodeId::new(1);
pub(in crate::ui::retained_host::welcome_recent_pointer) const VIEWPORT_NODE_ID: UiNodeId =
    UiNodeId::new(2);
pub(in crate::ui::retained_host::welcome_recent_pointer) const WELCOME_RECENT_ROUTE_ID_BASE: u64 =
    60_000;
