//! Editor host UI built on Slint, with viewport frames coming from core graphics.

pub mod core;
pub mod scene;
pub mod ui;

pub use core::editing::intent::EditorIntent;
pub use ui::host::module::{
    module_descriptor, EditorHostDriver, EditorModule, EDITOR_ASSET_MANAGER_NAME,
    EDITOR_HOST_DRIVER_NAME, EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME,
};
pub use ui::slint_host::run_editor;
pub use ui::workbench::state::EditorState;

#[cfg(test)]
mod tests;
