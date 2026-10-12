#[path = "../../tab_drag/lifecycle/arm.rs"]
mod arm;
#[path = "../../tab_drag/lifecycle/finish.rs"]
mod finish;
#[path = "../../tab_drag/lifecycle/move_event.rs"]
mod move_event;

pub(in crate::ui::retained_host::host_contract) use self::arm::arm_native_tab_drag;
pub(in crate::ui::retained_host::host_contract) use self::finish::finish_native_tab_drag;
pub(in crate::ui::retained_host::host_contract) use self::move_event::dispatch_native_tab_drag_move;
