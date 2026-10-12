//! 平台能力矩阵按 feature 与目标拓扑选择静态后端枚举；诊断将它们导出为稳定 token。
//! 枚举声明不证明宿主已安装，运行时就绪还需宿主生命周期与观测证据。
#[path = "backends/cursor.rs"]
mod cursor;
#[path = "backends/drag_drop.rs"]
mod drag_drop;
#[path = "backends/event_loop.rs"]
mod event_loop;
#[path = "backends/gamepad.rs"]
mod gamepad;
#[path = "backends/input.rs"]
mod input;
#[path = "backends/linux.rs"]
mod linux;
#[path = "backends/window.rs"]
mod window;

pub use cursor::{
    CursorBoundaryBackend, CursorOptionsBackend, PointerPositionBackend, RawMouseMotionBackend,
};
pub use drag_drop::FileDragDropBackend;
pub use event_loop::EventLoopPolicy;
pub use gamepad::{GamepadBackend, GamepadEventBackend, GamepadRumbleBackend};
pub use input::{
    GestureEventBackend, InputBackend, KeyboardEventBackend, MouseButtonBackend, MouseWheelBackend,
    TouchEventBackend,
};
pub use linux::LinuxWindowProtocol;
pub use window::{
    ImeBackend, MonitorBackend, WindowBackend, WindowEventBackend, WindowLifecycleBackend,
    WindowMetricsBackend,
};
