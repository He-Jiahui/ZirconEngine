use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::dispatch::UiPointerId;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostResizeStateData {
    pub capture_pointer_id: Option<UiPointerId>,
    pub resize_active: bool,
    pub resize_group: SharedString,
    pub resize_pointer_x: f32,
    pub resize_pointer_y: f32,
}
