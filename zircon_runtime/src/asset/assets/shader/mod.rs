//! 着色器资产模块把导入模型、readiness 报告和 zshader 文档统一导出；消费者通过这些类型进入材质编译与运行时资源流。

#[path = "dependency.rs"]
mod dependency;
#[path = "entry_point.rs"]
mod entry_point;
#[path = "language.rs"]
mod language;
#[path = "property_layout.rs"]
mod property_layout;
#[path = "readiness.rs"]
mod readiness;
#[path = "shader_asset.rs"]
mod shader_asset;
#[path = "source_contract.rs"]
mod source_contract;
#[path = "zshader.rs"]
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
