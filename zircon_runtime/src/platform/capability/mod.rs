//! 平台 capability 的公共边界：导出稳定的后端 token、静态矩阵/报告，以及
//! 依赖 PlatformHostSnapshot 的运行时状态投影；实现细节继续留在子模块内。
mod backends;
mod matrix;
mod report;
mod runtime;
mod status;

pub use backends::{
    CursorBoundaryBackend, CursorOptionsBackend, EventLoopPolicy, FileDragDropBackend,
    GamepadBackend, GamepadEventBackend, GamepadRumbleBackend, GestureEventBackend, ImeBackend,
    InputBackend, KeyboardEventBackend, LinuxWindowProtocol, MonitorBackend, MouseButtonBackend,
    MouseWheelBackend, PointerPositionBackend, RawMouseMotionBackend, TouchEventBackend,
    WindowBackend, WindowEventBackend, WindowLifecycleBackend, WindowMetricsBackend,
};
pub use matrix::PlatformCapabilityMatrix;
pub use report::PlatformCapabilityReport;
pub use runtime::{
    PlatformRuntimeCapabilityReport, PlatformRuntimeCapabilityStatus,
    PlatformRuntimeHostRequirement,
};
pub use status::CapabilityStatus;
