//! 原生手柄宿主与动态 Runtime 的事件、震动适配边界。
//! Gilrs 与运行中的效果由 App 持有，窗口退出路径负责主动停止效果。

mod codes;
mod events;
mod host;
mod polling;
mod rumble;

pub(super) use host::create_gilrs;
pub(in crate::entry::runtime_entry_app) use rumble::{
    clear_finished_rumble_effects, clear_gamepad_rumble_effects, RunningRumbleEffects,
};
