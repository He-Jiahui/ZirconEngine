//! 运行时应用的表面生命周期状态与 Winit 回调策略边界。

#[path = "action.rs"]
mod action;
#[path = "events.rs"]
mod events;
#[path = "machine.rs"]
mod machine;
#[path = "state.rs"]
mod state;
#[path = "transitions.rs"]
mod transitions;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use action::SurfaceReleaseAction;
pub(super) use machine::ApplicationLifecycleMachine;
