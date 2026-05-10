mod alpha_mode;
mod color_material;
mod dependency_set;
mod fallback_policy;
mod readiness_report;
mod standard_material;
mod validation_error;

pub use alpha_mode::RenderMaterialAlphaMode;
pub use color_material::ColorMaterialDescriptor;
pub use dependency_set::RenderMaterialDependencySet;
pub use fallback_policy::RenderMaterialFallbackPolicy;
pub use readiness_report::{
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialReadinessReport,
};
pub use standard_material::StandardMaterialDescriptor;
pub use validation_error::RenderMaterialValidationError;
