//! Runtime UI subsystem: layout, template, events (absorbed from legacy `zircon_ui`).

mod module;
mod runtime_ui;

pub use module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_EVENT_MANAGER_NAME, UI_MODULE_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub use runtime_ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};
pub use runtime_ui::PublicRuntimeFrame;

pub mod binding;
pub mod dispatch;
pub mod event_ui;
pub mod layout;
pub mod surface;
pub mod template;
pub mod tree;

#[cfg(test)]
mod tests;
