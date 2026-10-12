//! Neutral contracts shared by foundation services and their runtime consumers.

//! 此处只导出框架级配置服务契约；具体服务由 Runtime foundation 模块实现并注册到服务目录。
#[path = "config_manager.rs"]
mod config_manager;
#[path = "config_manager_error.rs"]
mod config_manager_error;
#[path = "config_persistence_report.rs"]
mod config_persistence_report;
#[path = "module_identity.rs"]
mod module_identity;

pub use config_manager::ConfigManager;
pub use config_manager_error::ConfigManagerError;
pub use config_persistence_report::ConfigPersistenceReport;
pub use module_identity::FOUNDATION_MODULE_NAME;
