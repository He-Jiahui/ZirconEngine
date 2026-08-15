mod catalog_input_generation;
mod generation_observation;
mod manager;
mod manifest;
mod meta;
mod meta_preview_state;
mod package_asset_registry;
mod paths;
mod script_manifest;
mod shader_resource_records;

pub(crate) use catalog_input_generation::ProjectCatalogInputSource;
pub use catalog_input_generation::{
    ProjectCatalogInputDelta, ProjectCatalogInputGeneration, ProjectCatalogInputRecord,
    ProjectCatalogInputRename,
};
pub(crate) use generation_observation::{ProjectGenerationObservation, ProjectGenerationPhase};
pub(crate) use manager::mint_meta_for_migration;
pub use manager::ProjectManager;
pub use manifest::{ProjectManifest, ProjectManifestError};
pub use meta::{
    AssetMetaDocument, AssetMetaEntry, AssetMetaError, AssetMetaResult, AssetSourceUnit,
    PreviewState,
};
pub(crate) use meta_preview_state::{lock_meta_document_path, lock_meta_document_paths};
pub use meta_preview_state::{
    AssetMetaPreviewStateCasResult, AssetMetaPreviewStateExpectation, AssetMetaPreviewStateStale,
};
pub use package_asset_registry::PackageAssetRegistry;
pub use paths::{ProjectPaths, ResolvedProjectPath, PROJECT_MANIFEST_FILE};
pub use script_manifest::ProjectScriptManifest;
pub use shader_resource_records::{
    shader_resource_records_from_asset_root, shader_resource_records_from_asset_roots,
    shader_resource_records_from_loaded_meta_document_refs,
    shader_resource_records_from_loaded_meta_documents, shader_resource_records_from_meta_paths,
    ShaderResourceRecordExportError, ShaderResourceRecordExportResult,
};
