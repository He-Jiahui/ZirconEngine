use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{ResourceId, ResourceLocator};
use crate::graphics::types::GraphicsError;

use super::super::super::prepared::PreparedMaterialTextureDependency;

const FALLBACK_MATERIAL_URI: &str = "builtin://missing-material";

pub(super) fn prepared_material_cache_identity_is_current(
    prepared_revision: Option<u64>,
    requested_revision: Option<u64>,
    prepared_texture_support: crate::asset::TextureUploadSupport,
    requested_texture_support: crate::asset::TextureUploadSupport,
    dependencies: &[PreparedMaterialTextureDependency],
    mut revision_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64)>,
) -> bool {
    prepared_revision == requested_revision
        && prepared_texture_support == requested_texture_support
        && dependencies.iter().all(|dependency| {
            revision_for_locator(&dependency.locator) == dependency.id.zip(dependency.revision)
        })
}

pub(super) fn material_prepare_result(
    id: ResourceId,
    report: &RenderMaterialReadinessReport,
) -> Result<(), GraphicsError> {
    if has_blocking_material_validation(&report.validation_errors) {
        Err(GraphicsError::Asset(format!(
            "material {} is not render-ready: {:?}",
            id, report.validation_errors
        )))
    } else {
        Ok(())
    }
}

fn has_blocking_material_validation(validation_errors: &[RenderMaterialValidationError]) -> bool {
    validation_errors.iter().any(|error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { .. }
                | RenderMaterialValidationError::MissingRuntimeShaderSource
        )
    })
}

pub(super) fn material_uses_renderer_material_abi_fallback(
    validation_errors: &[RenderMaterialValidationError],
) -> bool {
    validation_errors.iter().any(|error| {
        matches!(
            error,
            RenderMaterialValidationError::ShaderReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::RendererMaterialAbi,
                ..
            }
        )
    })
}

pub(super) fn fallback_material_uri() -> ResourceLocator {
    ResourceLocator::parse(FALLBACK_MATERIAL_URI).expect("builtin fallback material uri")
}

pub(super) fn missing_material_fallback_usage(
    material: ResourceId,
) -> (RenderMaterialValidationError, RenderMaterialFallbackUsage) {
    (
        RenderMaterialValidationError::UnresolvedMaterialReference { material },
        RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Material { material },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        },
    )
}

pub(super) fn invalid_parent_diagnostic(diagnostic: String) -> RenderMaterialValidationError {
    RenderMaterialValidationError::InvalidMaterialParent {
        source: RenderMaterialDiagnosticSource::MaterialOverride,
        path: "parent".to_string(),
        diagnostic,
    }
}

pub(super) fn is_standard_texture_slot(slot: &str) -> bool {
    matches!(
        slot,
        "base_color"
            | "base_color_texture"
            | "albedo"
            | "diffuse"
            | "normal"
            | "normal_texture"
            | "metallic_roughness"
            | "metallic_roughness_texture"
            | "occlusion"
            | "occlusion_texture"
            | "emissive"
            | "emissive_texture"
    )
}
