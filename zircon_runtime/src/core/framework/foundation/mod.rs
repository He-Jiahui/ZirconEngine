//! Neutral contracts shared by foundation services and their runtime consumers.

mod config_manager;
mod config_persistence_report;
mod event_manager;
mod module_identity;

pub use config_manager::ConfigManager;
pub use config_persistence_report::ConfigPersistenceReport;
pub use event_manager::EventManager;
pub use module_identity::FOUNDATION_MODULE_NAME;
