use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostResizeStateData {
    pub resize_active: bool,
    pub resize_group: SharedString,
    pub resize_pointer_x: f32,
    pub resize_pointer_y: f32,
}
