//! 内建 Kernel 模块的声明集合；这些零大小类型交给启动构建器组装依赖图。
//! 实际任务、时间与诊断状态由 CoreRuntime 和对应服务持有。

mod diagnostics;
mod frame_count;
mod log;
mod tasks;
mod time;

pub use diagnostics::{DiagnosticsCoreModule, DIAGNOSTICS_CORE_MODULE_NAME};
pub use frame_count::{FrameCountModule, FRAME_COUNT_MODULE_NAME};
pub use log::{LogDiagnosticsModule, LogModule, LOG_DIAGNOSTICS_MODULE_NAME, LOG_MODULE_NAME};
pub use tasks::{TasksModule, TASKS_MODULE_NAME};
pub use time::{TimeModule, TIME_MODULE_NAME};
