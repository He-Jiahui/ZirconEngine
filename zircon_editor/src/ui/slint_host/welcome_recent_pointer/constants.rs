use zircon_ui::UiNodeId;

pub(in crate::ui::slint_host::welcome_recent_pointer) const ROOT_NODE_ID: UiNodeId =
    UiNodeId::new(1);
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_NODE_ID: UiNodeId =
    UiNodeId::new(2);
pub(in crate::ui::slint_host::welcome_recent_pointer) const ITEM_NODE_ID_BASE: u64 = 100;
pub(in crate::ui::slint_host::welcome_recent_pointer) const OPEN_BUTTON_NODE_ID_BASE: u64 = 1000;
pub(in crate::ui::slint_host::welcome_recent_pointer) const REMOVE_BUTTON_NODE_ID_BASE: u64 =
    2000;

pub(in crate::ui::slint_host::welcome_recent_pointer) const OUTER_MARGIN: f32 = 18.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const OUTER_VERTICAL_MARGIN: f32 = 18.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const RECENT_PANEL_WIDTH: f32 = 320.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_X: f32 = OUTER_MARGIN + 16.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_Y: f32 =
    OUTER_VERTICAL_MARGIN + 94.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_HEIGHT_OFFSET: f32 =
    OUTER_VERTICAL_MARGIN + 110.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_MIN_HEIGHT: f32 = 24.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const VIEWPORT_INNER_WIDTH_INSET: f32 = 8.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const ITEM_HEIGHT: f32 = 112.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const ITEM_GAP: f32 = 10.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const OPEN_BUTTON_X: f32 = 14.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const REMOVE_BUTTON_X: f32 = 110.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const BUTTON_Y: f32 = 80.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const BUTTON_WIDTH: f32 = 88.0;
pub(in crate::ui::slint_host::welcome_recent_pointer) const BUTTON_HEIGHT: f32 = 24.0;
