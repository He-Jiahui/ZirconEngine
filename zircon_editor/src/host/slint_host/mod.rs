mod app;
pub(crate) mod callback_dispatch;
pub(crate) mod drawer_resize;
pub(crate) mod event_bridge;
pub(crate) mod tab_drag;
mod ui;
mod viewport;

slint::include_modules!();

pub use app::run_editor;
