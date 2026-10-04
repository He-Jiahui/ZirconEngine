mod config_manager;
#[cfg(test)]
#[path = "tests/config_manager_tests.rs"]
mod config_manager_tests;
mod config_path;

pub use config_manager::DefaultConfigManager;
#[cfg(test)]
pub(super) use config_path::override_config_file_path_for_test;
