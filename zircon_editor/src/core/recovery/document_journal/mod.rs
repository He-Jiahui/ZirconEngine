//! 文档日志协调入口；会话句柄只用于本次绑定，磁盘身份由项目相对源路径导出。

mod coordinator;
mod error;
mod model;

pub use coordinator::DocumentJournalCoordinator;
pub use error::DocumentJournalCoordinatorError;
pub use model::DocumentJournalAppend;
