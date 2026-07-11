pub mod coordinator_client;
pub mod error;
pub mod lifecycle;
pub mod menu;
pub mod notifications;
pub mod process_identity;
pub mod recovery;
pub mod repository_identity;
pub mod runtime_descriptor;
pub mod startup;
pub mod tray_state;

pub use error::TrayError;

pub fn run() -> Result<(), TrayError> {
    app::run()
}
pub mod app;
