mod arm;
mod finish;
mod move_event;

pub(in crate::ui::retained_host::host_contract) use self::arm::arm_native_tab_drag;
pub(in crate::ui::retained_host::host_contract) use self::finish::finish_native_tab_drag;
pub(in crate::ui::retained_host::host_contract) use self::move_event::dispatch_native_tab_drag_move;
