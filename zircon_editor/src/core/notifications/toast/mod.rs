//! 挂载无动作回执的短时提示模型与过期中心；展示消费快照，决策和任务进度沿各自生命周期入口处理。
mod center;
mod error;
mod model;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use center::{ToastCenterConfig, ToastNotificationCenter, ToastNotificationSnapshot};
pub use error::ToastNotificationError;
pub use model::{ToastNotification, ToastSeverity};
