mod dependency;
mod entry_point;
mod language;
mod property_layout;
mod readiness;
mod shader_asset;
mod source_contract;
mod zshader;

pub use dependency::ShaderDependencyAsset;
pub use entry_point::ShaderEntryPointAsset;
pub use language::ShaderSourceLanguage;
pub use property_layout::{generate_material_artifact, ShaderGeneratedMaterialArtifact};
pub use readiness::{
    ShaderAssetManagementRecord, ShaderAssetManagementRecordSet,
    ShaderAssetManagementRecordSetSummary, ShaderAssetReadinessSummary,
    ShaderBindGroupLayoutReadiness, ShaderBindingLayoutReadiness, ShaderDefinitionReadiness,
    ShaderEntryPointReadiness, ShaderImportReadiness, ShaderPipelineLayoutReadiness,
    ShaderReadinessReport, ShaderRuntimeSourceKind, ShaderRuntimeSourceReadiness,
};
pub use shader_asset::ShaderAsset;
use source_contract::classify_surface_source_contract;
pub use source_contract::{ShaderSurfaceSourceContract, ShaderSurfaceSourceContractError};
pub use zshader::{
    ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderOptionAsset,
    ShaderSourceFileAsset, ShaderTextureSlotAsset, ZShaderComputeDocumentV2, ZShaderDocumentV2,
    ZShaderEntryPointDocument, ZShaderFullscreenDocumentV2, ZShaderImportDocument,
    ZShaderIncludeDocumentV2, ZShaderOptionDocument, ZShaderSurfaceDocumentV2,
    ZShaderTextureSlotDocument, ZShaderV2Error, ZShaderV2Result,
};
