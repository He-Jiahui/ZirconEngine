use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{ResourceId, ResourceLocator, ResourceReadinessRowIdentity};
use crate::graphics::types::GraphicsError;

use super::super::super::prepared::{
    PreparedMaterialCandidateIdentity, PreparedMaterialDependency,
    PreparedMaterialShaderDependency, PreparedMaterialTextureDependency,
};

const FALLBACK_MATERIAL_URI: &str = "builtin://missing-material";

// 修订相同仍须比较发布身份；shader、texture 的未解析状态也必须与缓存一致，随后由 readiness 决定可绘制性。
pub(super) fn prepared_material_cache_identity_is_current(
    prepared_revision: Option<u64>,
    requested_revision: Option<u64>,
    material_dependency: &PreparedMaterialDependency,
    prepared_texture_support: crate::asset::TextureUploadSupport,
    requested_texture_support: crate::asset::TextureUploadSupport,
    shader_dependency: &PreparedMaterialShaderDependency,
    dependencies: &[PreparedMaterialTextureDependency],
    mut material_identity_for_id: impl FnMut(
        ResourceId,
    )
        -> Option<(ResourceId, u64, ResourceReadinessRowIdentity)>,
    mut shader_identity_for_locator: impl FnMut(
        &ResourceLocator,
    ) -> Option<(
        ResourceId,
        u64,
        ResourceReadinessRowIdentity,
    )>,
    mut texture_revision_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64)>,
) -> bool {
    prepared_revision == requested_revision
        && prepared_material_dependency_identity_is_current(
            material_dependency,
            material_identity_for_id(material_dependency.id),
        )
        && prepared_texture_support == requested_texture_support
        && match (
            shader_identity_for_locator(&shader_dependency.locator),
            shader_dependency.id,
            shader_dependency.revision,
            shader_dependency.dependency_identity.as_ref(),
        ) {
            (
                Some((id, revision, publication)),
                Some(prepared_id),
                Some(prepared_revision),
                Some(prepared_publication),
            ) => {
                id == prepared_id
                    && revision == prepared_revision
                    && &publication == prepared_publication
            }
            (None, None, None, None) => true,
            _ => false,
        }
        && dependencies.iter().all(|dependency| {
            texture_revision_for_locator(&dependency.locator)
                == dependency.id.zip(dependency.revision)
        })
}

pub(super) fn prepared_material_candidate_identity_is_current(
    identity: &PreparedMaterialCandidateIdentity,
    requested_revision: Option<u64>,
    requested_texture_support: crate::asset::TextureUploadSupport,
    material_identity_for_id: impl FnMut(
        ResourceId,
    ) -> Option<(ResourceId, u64, ResourceReadinessRowIdentity)>,
    shader_identity_for_locator: impl FnMut(
        &ResourceLocator,
    )
        -> Option<(ResourceId, u64, ResourceReadinessRowIdentity)>,
    texture_revision_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64)>,
) -> bool {
    prepared_material_cache_identity_is_current(
        identity.revision,
        requested_revision,
        &identity.material_dependency,
        identity.texture_support,
        requested_texture_support,
        &identity.shader_dependency,
        &identity.texture_dependencies,
        material_identity_for_id,
        shader_identity_for_locator,
        texture_revision_for_locator,
    )
}

pub(super) fn prepared_material_dependency_identity_is_current(
    prepared: &PreparedMaterialDependency,
    current: Option<(ResourceId, u64, ResourceReadinessRowIdentity)>,
) -> bool {
    current.is_some_and(|(id, revision, publication)| {
        id == prepared.id
            && revision == prepared.revision
            && publication == prepared.dependency_identity
    })
}

pub(super) fn material_prepare_result(
    id: ResourceId,
    report: &RenderMaterialReadinessReport,
) -> Result<(), GraphicsError> {
    if !material_readiness_allows_rendering(report) {
        Err(GraphicsError::Asset(format!(
            "material {} is not render-ready: {:?}",
            id, report.validation_errors
        )))
    } else {
        Ok(())
    }
}

pub(super) fn material_readiness_allows_rendering(report: &RenderMaterialReadinessReport) -> bool {
    !has_blocking_material_validation(&report.validation_errors)
}

fn has_blocking_material_validation(validation_errors: &[RenderMaterialValidationError]) -> bool {
    validation_errors.iter().any(|error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { .. }
                | RenderMaterialValidationError::MissingRuntimeShaderSource
                | RenderMaterialValidationError::UnsupportedTextureUvChannel { .. }
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

#[cfg(test)]
#[path = "tests/material_readiness.rs"]
mod tests;
