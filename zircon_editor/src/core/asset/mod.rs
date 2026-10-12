//! Canonical editor-side asset type contracts.

#[path = "activation.rs"]
mod activation;
#[path = "dirty/mod.rs"]
mod dirty;
#[path = "import_flow/mod.rs"]
mod import_flow;
#[path = "index.rs"]
mod index;
#[path = "refactor/mod.rs"]
mod refactor;
#[path = "source_authority.rs"]
mod source_authority;
#[path = "toolkit_route.rs"]
mod toolkit_route;
#[path = "type_registry/mod.rs"]
mod type_registry;

pub use activation::{
    AssetActivationIntent, AssetActivationReceipt, AssetActivationResult, AssetActivationSource,
};
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
    builtin_asset_type_definition, AssetContextCommandAccess, AssetContextCommandDescriptor,
    AssetCreationMenuEntry, AssetCreationMenuGeneration, AssetCreationTemplateDescriptor,
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeDefinition, AssetTypeId,
    AssetTypeIdError, AssetTypePresentation, AssetTypeRegistry, AssetTypeRegistryError,
    ThumbnailPlaceholderPalette, ThumbnailProviderDescriptor,
};
