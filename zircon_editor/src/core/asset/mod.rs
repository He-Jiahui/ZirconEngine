//! Canonical editor-side asset type contracts.

mod dirty;
mod import_flow;
mod index;
mod source_authority;
mod toolkit_route;
mod type_registry;

pub use dirty::{
    DirtyDocumentSnapshot, DirtyExternalEffectId, DirtyExternalEffectIdError,
    DirtyExternalEffectRevision, DirtyRegistry, DirtyRegistryCursor, DirtyRegistryDelta,
    DirtyRegistryError,
};
pub use import_flow::{
    EditorAssetImportAdmissionLimits, EditorAssetImportFlow, EditorAssetImportReason,
    EditorAssetImportRequest, EditorAssetImportResult, EditorAssetImportSubmitError,
    EditorAssetImportTicket,
};
pub use index::{EditorAssetImportState, EditorAssetIndex, EditorAssetIndexError, EditorAssetRow};
pub use source_authority::{
    AssetSourceAuthority, AssetSourceKind, AssetSourceWritePolicy, AssetWriteAccess,
};
pub use toolkit_route::AssetToolkitOpenRoute;
pub use type_registry::{
    builtin_asset_type_definition, AssetContextCommandAccess, AssetContextCommandDescriptor,
    AssetCreationTemplateDescriptor, AssetToolkitDescriptor, AssetTypeContribution,
    AssetTypeDefinition, AssetTypeId, AssetTypeIdError, AssetTypePresentation, AssetTypeRegistry,
    AssetTypeRegistryError, ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor,
};
