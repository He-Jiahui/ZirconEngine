mod alpha_mode;
mod color_material;
mod dependency_set;
mod diagnostic_source;
mod fallback_policy;
mod property_uniform;
mod property_value;
mod readiness_report;
mod standard_material;
mod validation_error;

pub use alpha_mode::RenderMaterialAlphaMode;
pub use color_material::ColorMaterialDescriptor;
pub use dependency_set::RenderMaterialDependencySet;
pub use diagnostic_source::RenderMaterialDiagnosticSource;
pub use fallback_policy::RenderMaterialFallbackPolicy;
pub use property_uniform::{
    RenderMaterialPropertyUniformField, RenderMaterialPropertyUniformPayload,
    RenderMaterialPropertyUniformSummary, RenderMaterialPropertyUniformUnsupported,
    RenderMaterialPropertyUniformUnsupportedReason,
};
pub use property_value::{RenderMaterialPropertyValue, RenderMaterialPropertyValueSummary};
pub use readiness_report::{
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialReadinessDiagnostic,
    RenderMaterialReadinessReport,
};
pub use standard_material::StandardMaterialDescriptor;
pub use validation_error::RenderMaterialValidationError;
