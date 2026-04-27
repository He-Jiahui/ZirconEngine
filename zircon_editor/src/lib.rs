//! Editor host UI built on Slint, with viewport frames coming from core graphics.

pub mod core;
pub mod scene;
pub mod ui;

pub use core::editing::intent::EditorIntent;
pub use core::editor_plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor,
    EditorPluginRegistrationReport,
};
pub use ui::host::module::{
    module_descriptor, EditorHostDriver, EditorModule, EDITOR_ASSET_MANAGER_NAME,
    EDITOR_HOST_DRIVER_NAME, EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME,
};
pub use ui::host::{
    editor_host_minimal_contract, EditorCapabilitySnapshot, EditorExportBuildReport,
    EditorHostMinimalContract, EditorHostMinimalReport, EditorHostVmBridgeReport,
    EditorPluginEnableReport, EditorPluginStatus, EditorPluginStatusReport, EditorSubsystemReport,
    EditorVmExtensionLoadReport, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
pub use ui::slint_host::run_editor;
pub use ui::workbench::state::EditorState;

#[cfg(test)]
mod tests;
