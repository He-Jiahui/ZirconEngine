//! 平台事件到共享 UI 输入的桥接：上下文提供路由元数据，事件工厂组织拥有的数据，
//! [`UiWindowPlatformInputEvent::normalize`] 消费事件并生成分发 DTO。
//! 窗口度量和光标缓存由宿主维护，规范化本身不更新缓存或分发状态。
#[path = "input/context.rs"]
mod context;
#[path = "input/normalization.rs"]
mod normalization;
#[path = "input/platform_event.rs"]
mod platform_event;
#[path = "input/platform_event_kind.rs"]
mod platform_event_kind;
#[path = "input/touch.rs"]
mod touch;
#[path = "input/window_event.rs"]
mod window_event;

pub use context::UiWindowInputContext;
pub use platform_event::UiWindowPlatformInputEvent;
pub use platform_event_kind::UiWindowPlatformInputEventKind;
pub use touch::UiWindowTouchPhase;
