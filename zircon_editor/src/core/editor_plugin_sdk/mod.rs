//! Public editor plugin SDK facade.

pub mod examples;
pub mod lifecycle;

pub use crate::core::editor_extension::{
    AssetEditorDescriptor, AssetImporterDescriptor, ComponentDrawerDescriptor, DrawerDescriptor,
    EditorExtensionRegistration, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
};
pub use crate::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationPath, EditorOperationRegistryError,
    UndoableEditorOperation,
};
pub use crate::core::editor_plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor,
    EditorPluginRegistrationReport,
};
pub use lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleRecord,
    EditorPluginLifecycleReport, EditorPluginLifecycleStage,
};
