//! 指针派发的共享契约：Runtime 按阶段收集处理器效果，UiSurface 负责提交状态并公布结果。

mod component_event;
mod context;
mod effect;
mod event;
mod invocation;
mod result;

pub use component_event::{
    UiPointerComponentEvent, UiPointerComponentEventReason, UiTemplateActionInvocation,
};
pub use context::UiPointerDispatchContext;
pub use effect::UiPointerDispatchEffect;
pub use event::UiPointerEvent;
pub use invocation::UiPointerDispatchInvocation;
pub use result::{UiPointerDispatchDiagnostics, UiPointerDispatchResult};
