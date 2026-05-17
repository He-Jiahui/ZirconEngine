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
