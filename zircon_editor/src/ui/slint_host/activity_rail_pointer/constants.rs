use zircon_ui::event_ui::UiNodeId;

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const LEFT_STRIP_NODE_ID: UiNodeId = UiNodeId::new(10);
pub(super) const RIGHT_STRIP_NODE_ID: UiNodeId = UiNodeId::new(20);
pub(super) const LEFT_BUTTON_NODE_ID_BASE: u64 = 100;
pub(super) const RIGHT_BUTTON_NODE_ID_BASE: u64 = 200;

pub(super) const STRIP_X_INSET: f32 = 3.0;
pub(super) const STRIP_Y_INSET: f32 = 6.0;
pub(super) const BUTTON_EXTENT: f32 = 30.0;
pub(super) const BUTTON_GAP: f32 = 2.0;
