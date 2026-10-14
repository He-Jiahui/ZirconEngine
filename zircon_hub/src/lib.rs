#[cfg(any(feature = "account-broker", feature = "local-service"))]
#[path = "file_io/mod.rs"]
mod file_io;

#[cfg(feature = "account-broker")]
#[path = "account/mod.rs"]
pub mod account;
#[cfg(feature = "desktop")]
#[path = "assets/mod.rs"]
pub mod assets;
#[cfg(feature = "desktop")]
#[path = "build/mod.rs"]
pub mod build;
#[cfg(feature = "desktop")]
#[path = "engines/mod.rs"]
pub mod engines;
#[cfg(feature = "desktop")]
#[path = "error.rs"]
pub mod error;
#[cfg(feature = "desktop")]
#[path = "learn/mod.rs"]
pub mod learn;
#[cfg(feature = "desktop")]
#[path = "plugins/mod.rs"]
pub mod plugins;
#[cfg(feature = "desktop")]
#[path = "process/mod.rs"]
pub mod process;
#[cfg(feature = "desktop")]
#[path = "projects/mod.rs"]
pub mod projects;
#[cfg(feature = "local-service")]
#[path = "service/mod.rs"]
pub mod service;
#[cfg(any(feature = "desktop", feature = "hub-settings"))]
#[path = "settings/mod.rs"]
pub mod settings;
#[cfg(feature = "desktop")]
#[path = "state/mod.rs"]
pub mod state;
#[cfg(feature = "desktop")]
#[path = "tauri_app/mod.rs"]
pub mod tauri_app;
#[cfg(feature = "desktop")]
#[path = "team/mod.rs"]
pub mod team;

#[cfg(feature = "desktop")]
pub use error::HubError;
