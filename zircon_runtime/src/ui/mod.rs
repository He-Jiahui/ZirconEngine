//! Runtime UI subsystem: layout, template, surface, binding, and event data.

#[path = "editable_text_composition.rs"]
pub(in crate::ui) mod editable_text_composition;
#[path = "module.rs"]
mod module;
#[path = "prelude.rs"]
pub mod prelude;
#[path = "public_runtime_frame.rs"]
mod public_runtime_frame;
#[cfg(test)]
#[path = "tests/runtime_ui_support/mod.rs"]
mod runtime_ui_support;
#[path = "secure_text_policy.rs"]
pub(crate) mod secure_text_policy;

pub use crate::core::framework::ui::UI_MODULE_NAME;
pub use module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_CONFIG_KEY, UI_EVENT_MANAGER_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub(crate) use public_runtime_frame::PublicRuntimeFrame;
#[cfg(test)]
pub(crate) use runtime_ui_support::{RuntimeUiFixture, RuntimeUiManager};

#[path = "accessibility/mod.rs"]
pub mod accessibility;
#[path = "binding/mod.rs"]
pub mod binding;
#[path = "component/mod.rs"]
pub mod component;
#[path = "dispatch/mod.rs"]
pub mod dispatch;
#[path = "event_ui/mod.rs"]
pub mod event_ui;
#[path = "icon_atlas/mod.rs"]
pub mod icon_atlas;
#[path = "layout/mod.rs"]
pub mod layout;
#[cfg(feature = "platform-winit")]
#[path = "platform_input/mod.rs"]
pub mod platform_input;
#[path = "style.rs"]
pub mod style;
#[path = "surface/mod.rs"]
pub mod surface;
#[path = "template/mod.rs"]
pub mod template;
#[path = "text/mod.rs"]
pub(crate) mod text;
#[path = "theme/mod.rs"]
pub mod theme;
#[path = "tree/mod.rs"]
pub mod tree;
#[path = "v2/mod.rs"]
pub mod v2;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
