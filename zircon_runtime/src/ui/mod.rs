//! Runtime UI subsystem: layout, template, surface, binding, and event data.

mod module;
pub mod prelude;
mod public_runtime_frame;
#[cfg(test)]
#[path = "tests/runtime_ui_support/mod.rs"]
mod runtime_ui_support;

pub use module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_EVENT_MANAGER_NAME, UI_MODULE_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub(crate) use public_runtime_frame::PublicRuntimeFrame;
#[cfg(test)]
pub(crate) use runtime_ui_support::{RuntimeUiFixture, RuntimeUiManager};

pub mod accessibility;
pub mod binding;
pub mod component;
pub mod dispatch;
pub mod event_ui;
pub mod icon_atlas;
pub mod layout;
#[cfg(feature = "platform-winit")]
pub mod platform_input;
pub mod style;
pub mod surface;
pub mod template;
pub(crate) mod text;
pub mod theme;
pub mod tree;
pub mod v2;

#[cfg(test)]
mod tests;
