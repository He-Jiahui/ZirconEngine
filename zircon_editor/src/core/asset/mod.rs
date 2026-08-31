//! Canonical editor-side asset type contracts.

mod dirty;
mod import_flow;
mod index;
mod refactor;
mod source_authority;
mod toolkit_route;
mod type_registry;

pub use dirty::{
    DirtyDocumentSnapshot, DirtyExternalEffectId, DirtyExternalEffectIdError,
    DirtyExternalEffectRevision, DirtyRegistry, DirtyRegistryCursor, DirtyRegistryDelta,
    DirtyRegistryError, SaveDirtyViewCandidate, SaveDirtyViewCompletion, SaveDirtyViewExecutor,
    SaveDirtyViewFailure, SaveDirtyViewFailureKind, SaveDirtyViewIntent,
    SaveDirtyViewOutcomeStatus, SaveDirtyViewsAdmissionError, SaveDirtyViewsApplyError,
    SaveDirtyViewsJobAdapter, SaveDirtyViewsPreflightReport, SaveDirtyViewsRequest,
    SaveDirtyViewsResult,
};
pub use import_flow::{
    EditorAssetImportAdmissionLimits, EditorAssetImportExecutionError, EditorAssetImportFlow,
    EditorAssetImportReason, EditorAssetImportRequest, EditorAssetImportResult,
    EditorAssetImportSubmitError, EditorAssetImportTicket, EditorModelImportTicket,
};
pub use index::{EditorAssetImportState, EditorAssetIndex, EditorAssetIndexError, EditorAssetRow};
pub use refactor::{
    AssetDeleteDisposition, AssetDeletePreflight, EditorAssetDeletionResult,
    EditorAssetDeletionTicket, EditorAssetRelocationResult, EditorAssetRelocationTicket,
};
pub use source_authority::{
    AssetSourceAuthority, AssetSourceKind, AssetSourceWritePolicy, AssetWriteAccess,
};
pub use toolkit_route::AssetToolkitOpenRoute;
pub use type_registry::{
    AssetContextCommandAccess, AssetContextCommandDescriptor, AssetCreationMenuEntry,
    AssetCreationMenuGeneration, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeContribution, AssetTypeDefinition, AssetTypeId, AssetTypeIdError,
    AssetTypePresentation, AssetTypeRegistry, AssetTypeRegistryError, ThumbnailPlaceholderPalette,
    ThumbnailProviderDescriptor, builtin_asset_type_definition,
};
