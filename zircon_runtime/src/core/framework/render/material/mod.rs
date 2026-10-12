#[path = "alpha_mode.rs"]
mod alpha_mode;
#[path = "color_material.rs"]
mod color_material;
#[path = "dependency_set.rs"]
mod dependency_set;
#[path = "diagnostic_source.rs"]
mod diagnostic_source;
#[path = "fallback_policy.rs"]
mod fallback_policy;
#[path = "lighting_model.rs"]
mod lighting_model;
#[path = "management.rs"]
mod management;
#[path = "property_override_block.rs"]
mod property_override_block;
#[path = "property_uniform.rs"]
mod property_uniform;
#[path = "property_value.rs"]
mod property_value;
#[path = "readiness_report.rs"]
mod readiness_report;
#[path = "shading_model.rs"]
mod shading_model;
#[path = "standard_material.rs"]
mod standard_material;
#[path = "texture_slot_summary.rs"]
mod texture_slot_summary;
#[path = "texture_transform.rs"]
mod texture_transform;
#[path = "validation_error.rs"]
mod validation_error;

pub use alpha_mode::RenderMaterialAlphaMode;
pub use color_material::ColorMaterialDescriptor;
pub use dependency_set::RenderMaterialDependencySet;
pub use diagnostic_source::RenderMaterialDiagnosticSource;
pub use fallback_policy::RenderMaterialFallbackPolicy;
pub use lighting_model::{RenderMaterialLightingModel, RenderMaterialLightingModelParseError};
pub use management::{
    RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind,
    RenderMaterialManagementIssueView, RenderMaterialManagementOverview,
    RenderMaterialManagementOverviewRecord, RenderMaterialManagementPageInfo,
    RenderMaterialManagementPageRequest, RenderMaterialManagementPageWindow,
    RenderMaterialManagementQuery, RenderMaterialManagementQueryControls,
    RenderMaterialManagementQueryFacet, RenderMaterialManagementQueryFacetKind,
    RenderMaterialManagementQueryFacets, RenderMaterialManagementQueryFilter,
    RenderMaterialManagementQueryFilterKind, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQueryResultActions, RenderMaterialManagementQueryResultState,
    RenderMaterialManagementQueryResultStateKind, RenderMaterialManagementQuerySelection,
    RenderMaterialManagementQueryState, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementSelection, RenderMaterialManagementSnapshot,
    RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
    RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
    RenderMaterialManagementStatusView,
};
pub use property_override_block::MaterialPropertyOverrideBlock;
pub use property_uniform::{
    RenderMaterialPropertyUniformField, RenderMaterialPropertyUniformPayload,
    RenderMaterialPropertyUniformSummary, RenderMaterialPropertyUniformUnsupported,
    RenderMaterialPropertyUniformUnsupportedReason,
};
pub use property_value::{
    RenderMaterialPropertyValue, RenderMaterialPropertyValueState,
    RenderMaterialPropertyValueSummary,
};
pub use readiness_report::{
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialIssueState,
    RenderMaterialPreparedState, RenderMaterialReadinessDiagnostic, RenderMaterialReadinessReport,
    RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
};
pub use shading_model::{
    GBufferChannelMask, ShadingModelDescriptor, ShadingModelId, ShadingModelRegistrationError,
    SHADING_MODEL_GBUFFER_ALPHA_SCALE, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
    SHADING_MODEL_ID_UNLIT, SHADING_MODEL_PLUGIN_ID_START,
};
pub use standard_material::{
    StandardMaterialDescriptor, STANDARD_MATERIAL_MIN_ROUGHNESS,
    STANDARD_MATERIAL_TEXTURE_UV_CHANNEL_COUNT,
};
pub use texture_slot_summary::{
    RenderMaterialTextureDimension, RenderMaterialTextureSlotFallback,
    RenderMaterialTextureSlotFallbackReason, RenderMaterialTextureSlotState,
    RenderMaterialTextureSlotSummary,
};
pub use texture_transform::RenderMaterialTextureTransform;
pub use validation_error::RenderMaterialValidationError;
