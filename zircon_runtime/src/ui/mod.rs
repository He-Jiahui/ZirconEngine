//! Runtime UI subsystem: layout, template, surface, binding, and event data.

mod module;
#[allow(dead_code)]
mod runtime_ui;

pub use module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_EVENT_MANAGER_NAME, UI_MODULE_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub(crate) use runtime_ui::PublicRuntimeFrame;
#[cfg(test)]
pub(crate) use runtime_ui::{RuntimeUiFixture, RuntimeUiManager};

pub mod binding;
pub mod component;
pub mod dispatch;
pub mod event_ui;
pub mod layout;
pub mod surface;
pub mod template;
pub(crate) mod text;
pub mod tree;

#[cfg(test)]
mod tests;
