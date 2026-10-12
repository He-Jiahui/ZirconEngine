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
