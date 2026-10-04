#[cfg(any(feature = "account-broker", feature = "local-service"))]
mod file_io;

#[cfg(feature = "account-broker")]
pub mod account;
#[cfg(feature = "desktop")]
pub mod assets;
#[cfg(feature = "desktop")]
pub mod build;
#[cfg(feature = "desktop")]
pub mod engines;
#[cfg(feature = "desktop")]
pub mod error;
#[cfg(feature = "desktop")]
pub mod learn;
#[cfg(feature = "desktop")]
pub mod plugins;
#[cfg(feature = "desktop")]
pub mod process;
#[cfg(feature = "desktop")]
pub mod projects;
#[cfg(feature = "local-service")]
pub mod service;
#[cfg(feature = "desktop")]
pub mod settings;
#[cfg(feature = "desktop")]
pub mod state;
#[cfg(feature = "desktop")]
pub mod tauri_app;
#[cfg(feature = "desktop")]
pub mod team;

#[cfg(feature = "desktop")]
pub use error::HubError;
