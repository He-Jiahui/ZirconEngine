#[path = "tab_drag/lifecycle.rs"]
mod lifecycle;
#[path = "tab_drag/payload.rs"]
mod payload;

pub(in crate::ui::retained_host::host_contract) use self::lifecycle::{
    arm_native_tab_drag, dispatch_native_tab_drag_move, finish_native_tab_drag,
};
