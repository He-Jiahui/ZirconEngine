//! Public editor-plugin SDK facade.

pub mod examples;
pub mod lifecycle;

pub use crate::core::asset::{
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution,
    AssetTypeDefinition, AssetTypeId, AssetTypeIdError, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailProviderDescriptor,
};
pub use crate::core::commands::{EditorCommandDescriptor, EditorCommandRegistryError};
pub use crate::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration,
};
pub use crate::core::editor_extension::{
    AssetImporterDescriptor, DrawerDescriptor, EditorExtensionRegistration,
    EditorExtensionRegistry, EditorExtensionRegistryError, EditorMenuItemDescriptor,
    EditorUiTemplateDescriptor, ViewDescriptor,
};
pub use crate::core::editor_operation::EditorOperationPath;
pub use crate::core::extension::{InspectorCustomizationDescriptor, InspectorCustomizationSurface};
pub use crate::core::plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginDescriptor, EditorPluginRegistrationReport,
};
pub(crate) use crate::core::plugin::EditorPluginCatalog;
pub use lifecycle::{
    EditorPluginLifecycleError, EditorPluginLifecycleEvent, EditorPluginLifecycleRecord,
    EditorPluginLifecycleReport, EditorPluginLifecycleStage,
};
