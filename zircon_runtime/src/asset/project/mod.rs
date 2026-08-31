mod catalog_input_generation;
mod generation_observation;
mod import_receipt;
mod manager;
mod manifest;
mod meta;
mod meta_preview_state;
mod meta_write_authority;
mod package_asset_registry;
mod paths;
mod reference_diagnostics;
mod script_manifest;
mod shader_resource_records;

pub(crate) use catalog_input_generation::ProjectCatalogInputSource;
pub use catalog_input_generation::{
    ProjectCatalogInputDelta, ProjectCatalogInputGeneration, ProjectCatalogInputRecord,
    ProjectCatalogInputRename,
};
pub(crate) use generation_observation::{ProjectGenerationObservation, ProjectGenerationPhase};
pub use import_receipt::ProjectImportReceipt;
pub use manager::ProjectManager;
pub(crate) use manager::{
    mint_meta_for_migration, ImportSourceWatchEcho, PreparedProjectSourceDeletion,
    PreparedProjectSourceRelocation,
};
pub use manifest::{ProjectManifest, ProjectManifestError};
pub use meta::{
    AssetMetaDocument, AssetMetaEntry, AssetMetaError, AssetMetaResult, AssetSourceUnit,
    PreviewState,
};
pub use meta_preview_state::{
    AssetMetaPreviewStateCasResult, AssetMetaPreviewStateExpectation, AssetMetaPreviewStateStale,
};
pub(crate) use meta_write_authority::{lock_meta_document_path, lock_meta_document_paths};
pub use package_asset_registry::PackageAssetRegistry;
pub use paths::{
    ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity, PROJECT_MANIFEST_FILE,
};
pub(crate) use reference_diagnostics::ProjectReferenceDiagnosticsStore;
pub use reference_diagnostics::{
    ProjectReferenceDiagnostic, ProjectReferenceDiagnosticKind, ProjectReferenceDiagnosticPhase,
    ProjectReferenceDiagnosticsEvent, ProjectReferenceDiagnosticsSnapshot,
};
pub use script_manifest::ProjectScriptManifest;
pub use shader_resource_records::{
    shader_resource_records_from_asset_root, shader_resource_records_from_asset_roots,
    shader_resource_records_from_loaded_meta_document_refs,
    shader_resource_records_from_loaded_meta_documents, shader_resource_records_from_meta_paths,
    ShaderResourceRecordExportError, ShaderResourceRecordExportResult,
};
