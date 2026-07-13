//! Public editor plugin SDK facade.

pub mod examples;
pub mod lifecycle;

pub use crate::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution,
    AssetTypeDefinition, AssetTypeId, AssetTypeIdError, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailProviderDescriptor,
};
pub use crate::core::commands::{EditorCommandDescriptor, EditorCommandRegistryError};
pub use crate::core::editor_extension::{
    AssetImporterDescriptor, ComponentDrawerDescriptor, DrawerDescriptor,
    EditorExtensionRegistration, EditorExtensionRegistry, EditorExtensionRegistryError,
    EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
};
pub use crate::core::editor_operation::{EditorOperationPath, UndoableEditorOperation};
pub use crate::core::editor_plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginCatalog, EditorPluginDescriptor,
    EditorPluginRegistrationReport,
};
pub use lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleRecord,
    EditorPluginLifecycleReport, EditorPluginLifecycleStage,
};
