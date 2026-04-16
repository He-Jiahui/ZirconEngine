use zircon_ui::UiNodeId;

pub(in crate::editing::viewport::pointer) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(in crate::editing::viewport::pointer) const VIEWPORT_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(in crate::editing::viewport::pointer) const FIRST_CANDIDATE_NODE_ID: u64 = 128;
pub(in crate::editing::viewport::pointer) const HANDLE_PRIORITY: u8 = 0;
pub(in crate::editing::viewport::pointer) const GIZMO_PRIORITY: u8 = 1;
pub(in crate::editing::viewport::pointer) const RENDERABLE_PRIORITY: u8 = 2;
pub(in crate::editing::viewport::pointer) const HANDLE_PICK_THRESHOLD_PX: f32 = 12.0;
pub(in crate::editing::viewport::pointer) const GIZMO_PICK_THRESHOLD_PX: f32 = 10.0;
pub(in crate::editing::viewport::pointer) const RENDERABLE_PICK_MIN_RADIUS_PX: f32 = 14.0;
