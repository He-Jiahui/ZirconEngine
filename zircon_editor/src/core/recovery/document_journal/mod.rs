//! 文档日志协调入口；会话句柄只用于本次绑定，磁盘身份由项目相对源路径导出。

#[path = "coordinator.rs"]
mod coordinator;
#[path = "error.rs"]
mod error;
#[path = "model.rs"]
mod model;

pub use coordinator::DocumentJournalCoordinator;
pub use error::DocumentJournalCoordinatorError;
pub use model::DocumentJournalAppend;
