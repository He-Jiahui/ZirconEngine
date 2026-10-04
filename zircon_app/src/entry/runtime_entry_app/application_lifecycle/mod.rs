//! 运行时应用的表面生命周期状态与 Winit 回调策略边界。

mod action;
mod events;
mod machine;
mod state;
mod transitions;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use action::SurfaceReleaseAction;
pub(super) use machine::ApplicationLifecycleMachine;
