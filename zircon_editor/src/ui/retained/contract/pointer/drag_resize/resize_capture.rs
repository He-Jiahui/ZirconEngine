#[path = "resize_capture/arm.rs"]
mod arm;
#[path = "../resize_capture/finish.rs"]
mod finish;
#[path = "../resize_capture/move_event.rs"]
mod move_event;

pub(in crate::ui::retained_host::host_contract) use self::arm::arm_native_resize;
pub(in crate::ui::retained_host::host_contract) use self::finish::finish_native_resize;
pub(in crate::ui::retained_host::host_contract) use self::move_event::dispatch_native_resize_move;
