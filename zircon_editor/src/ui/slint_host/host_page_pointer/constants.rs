use zircon_runtime::ui::event_ui::UiNodeId;

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const STRIP_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(super) const TAB_NODE_ID_BASE: u64 = 100;
pub(super) const STRIP_X: f32 = 8.0;
pub(super) const STRIP_Y: f32 = 1.0;
pub(super) const TAB_MIN_WIDTH: f32 = 108.0;
pub(super) const TAB_HEIGHT: f32 = 30.0;
pub(super) const TAB_GAP: f32 = 4.0;
