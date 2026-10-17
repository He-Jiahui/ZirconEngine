#[path = "catalog_input_generation.rs"]
mod catalog_input_generation;
#[path = "generation_authority.rs"]
mod generation_authority;
#[path = "generation_observation.rs"]
mod generation_observation;
#[path = "import_receipt.rs"]
mod import_receipt;
#[path = "manager/mod.rs"]
mod manager;
#[path = "manifest.rs"]
mod manifest;
#[path = "meta.rs"]
mod meta;
#[path = "meta_preview_state.rs"]
mod meta_preview_state;
#[path = "meta_write_authority.rs"]
mod meta_write_authority;
#[path = "package_asset_registry.rs"]
mod package_asset_registry;
#[path = "paths.rs"]
mod paths;
#[path = "plugin_apply.rs"]
pub mod plugin_apply;
#[path = "reference_diagnostics.rs"]
mod reference_diagnostics;
#[path = "script_manifest.rs"]
mod script_manifest;
#[path = "shader_resource_records.rs"]
mod shader_resource_records;

pub(crate) use catalog_input_generation::ProjectCatalogInputSource;
pub use catalog_input_generation::{
    ProjectCatalogInputDelta, ProjectCatalogInputGeneration, ProjectCatalogInputRecord,
    ProjectCatalogInputRename,
};
pub(crate) use generation_authority::lock_project_generation;
#[cfg(test)]
pub(crate) use generation_authority::ProjectGenerationAdmissionObserver;
pub(crate) use generation_observation::{ProjectGenerationObservation, ProjectGenerationPhase};
pub use import_receipt::ProjectImportReceipt;
pub(crate) use manager::{
    mint_meta_for_migration, ImportSourceWatchEcho, PreparedProjectSourceDeletion,
    PreparedProjectSourceRelocation, PreparedWatchFileGeneration, WatchSourceCandidate,
};
pub use manager::{
    EditorDocumentCommitFault, EditorDocumentCommitOutcome, EditorDocumentScope,
    ProjectActivationIdentity, ProjectManager, ScopedRuntimeProjectManager,
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
pub(crate) use paths::LexicalProjectPathIdentity;
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
