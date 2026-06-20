mod resize_capture;
mod tab_drag;

pub(in crate::ui::retained_host::host_contract) use resize_capture::{
    arm_native_resize, dispatch_native_resize_move, finish_native_resize,
};
pub(in crate::ui::retained_host::host_contract) use tab_drag::{
    arm_native_tab_drag, dispatch_native_tab_drag_move, finish_native_tab_drag,
};
