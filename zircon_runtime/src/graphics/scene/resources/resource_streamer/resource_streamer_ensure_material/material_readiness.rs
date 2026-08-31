use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{ResourceId, ResourceLocator};
use crate::graphics::types::GraphicsError;

use super::super::super::prepared::{
    PreparedMaterialCandidateIdentity, PreparedMaterialDependency,
    PreparedMaterialShaderDependency, PreparedMaterialTextureDependency,
};

const FALLBACK_MATERIAL_URI: &str = "builtin://missing-material";

pub(super) fn prepared_material_cache_identity_is_current(
    prepared_revision: Option<u64>,
    requested_revision: Option<u64>,
    material_dependency: &PreparedMaterialDependency,
    prepared_texture_support: crate::asset::TextureUploadSupport,
    requested_texture_support: crate::asset::TextureUploadSupport,
    shader_dependency: &PreparedMaterialShaderDependency,
    dependencies: &[PreparedMaterialTextureDependency],
    mut material_identity_for_id: impl FnMut(ResourceId) -> Option<(ResourceId, u64, u64)>,
    mut shader_identity_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64, u64)>,
    mut texture_revision_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64)>,
) -> bool {
    prepared_revision == requested_revision
        && prepared_material_dependency_identity_is_current(
            material_dependency,
            material_identity_for_id(material_dependency.id),
        )
        && prepared_texture_support == requested_texture_support
        && shader_identity_for_locator(&shader_dependency.locator)
            == shader_dependency
                .id
                .zip(shader_dependency.revision)
                .zip(shader_dependency.dependency_revision)
                .map(|((id, revision), dependency_revision)| (id, revision, dependency_revision))
        && dependencies.iter().all(|dependency| {
            texture_revision_for_locator(&dependency.locator)
                == dependency.id.zip(dependency.revision)
        })
}

pub(super) fn prepared_material_candidate_identity_is_current(
    identity: &PreparedMaterialCandidateIdentity,
    requested_revision: Option<u64>,
    requested_texture_support: crate::asset::TextureUploadSupport,
    material_identity_for_id: impl FnMut(ResourceId) -> Option<(ResourceId, u64, u64)>,
    shader_identity_for_locator: impl FnMut(&ResourceLocator) -> Option<(ResourceId, u64, u64)>,
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
    current: Option<(ResourceId, u64, u64)>,
) -> bool {
    current == Some((prepared.id, prepared.revision, prepared.dependency_revision))
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
mod tests {
    use super::{
        has_blocking_material_validation, prepared_material_candidate_identity_is_current,
        prepared_material_dependency_identity_is_current,
    };
    use crate::asset::TextureUploadSupport;
    use crate::core::framework::render::RenderMaterialValidationError;
    use crate::core::resource::{ResourceId, ResourceLocator};
    use crate::graphics::scene::resources::prepared::{
        PreparedMaterialCandidateIdentity, PreparedMaterialDependency,
        PreparedMaterialShaderDependency, PreparedMaterialTextureDependency,
    };

    #[test]
    fn material_dependency_identity_requires_root_and_recursive_parent_generations() {
        let id = ResourceId::from_stable_label("res://materials/child.zmaterial");
        let dependency = PreparedMaterialDependency {
            id,
            revision: 7,
            dependency_revision: 11,
        };

        assert!(prepared_material_dependency_identity_is_current(
            &dependency,
            Some((id, 7, 11))
        ));
        assert!(!prepared_material_dependency_identity_is_current(
            &dependency,
            Some((id, 8, 11))
        ));
        assert!(!prepared_material_dependency_identity_is_current(
            &dependency,
            Some((id, 7, 12))
        ));
    }

    #[test]
    fn unsupported_texture_uv_channel_blocks_material_preparation() {
        assert!(has_blocking_material_validation(&[
            RenderMaterialValidationError::UnsupportedTextureUvChannel {
                slot: "base_color".to_string(),
                channel: 2,
                supported_channel_count: 2,
            },
        ]));
    }

    #[test]
    fn failed_candidate_cache_identity_covers_every_rebuild_input() {
        let material_id = ResourceId::from_stable_label("res://materials/child.zmaterial");
        let shader_id = ResourceId::from_stable_label("res://shaders/pbr.zshader");
        let texture_id = ResourceId::from_stable_label("res://textures/base.ztexture");
        let shader_locator = ResourceLocator::parse("res://shaders/pbr.zshader").unwrap();
        let texture_locator = ResourceLocator::parse("res://textures/base.ztexture").unwrap();
        let support = TextureUploadSupport::uncompressed_only();
        let identity = PreparedMaterialCandidateIdentity {
            revision: Some(7),
            material_dependency: PreparedMaterialDependency {
                id: material_id,
                revision: 7,
                dependency_revision: 11,
            },
            shader_dependency: PreparedMaterialShaderDependency {
                locator: shader_locator.clone(),
                id: Some(shader_id),
                revision: Some(13),
                dependency_revision: Some(17),
            },
            texture_dependencies: vec![PreparedMaterialTextureDependency {
                locator: texture_locator.clone(),
                id: Some(texture_id),
                revision: Some(19),
                upload_unsupported_reason: None,
            }],
            texture_support: support,
        };
        let is_current = |requested_revision,
                          material_dependency_revision,
                          shader_revision,
                          texture_revision,
                          requested_support| {
            prepared_material_candidate_identity_is_current(
                &identity,
                requested_revision,
                requested_support,
                |id| (id == material_id).then_some((material_id, 7, material_dependency_revision)),
                |locator| (locator == &shader_locator).then_some((shader_id, shader_revision, 17)),
                |locator| (locator == &texture_locator).then_some((texture_id, texture_revision)),
            )
        };

        assert!(is_current(Some(7), 11, 13, 19, support));
        assert!(!is_current(Some(8), 11, 13, 19, support));
        assert!(!is_current(Some(7), 12, 13, 19, support));
        assert!(!is_current(Some(7), 11, 14, 19, support));
        assert!(!is_current(Some(7), 11, 13, 20, support));
        assert!(!is_current(
            Some(7),
            11,
            13,
            19,
            TextureUploadSupport::all_compressed(),
        ));
    }
}
