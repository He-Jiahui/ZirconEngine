use super::super::tests::readiness_identity;
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
fn material_dependency_identity_requires_root_and_retained_publication() {
    let id = ResourceId::from_stable_label("res://materials/child.zmaterial");
    let publication = readiness_identity(id);
    let dependency = PreparedMaterialDependency {
        id,
        revision: 7,
        dependency_identity: publication.clone(),
    };

    assert!(prepared_material_dependency_identity_is_current(
        &dependency,
        Some((id, 7, publication.clone()))
    ));
    assert!(!prepared_material_dependency_identity_is_current(
        &dependency,
        Some((id, 8, publication))
    ));
    assert!(!prepared_material_dependency_identity_is_current(
        &dependency,
        Some((id, 7, readiness_identity(id)))
    ));
    assert!(!prepared_material_dependency_identity_is_current(
        &dependency,
        None
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
    let material_identity = readiness_identity(material_id);
    let shader_identity = readiness_identity(shader_id);
    let identity = PreparedMaterialCandidateIdentity {
        revision: Some(7),
        material_dependency: PreparedMaterialDependency {
            id: material_id,
            revision: 7,
            dependency_identity: material_identity.clone(),
        },
        shader_dependency: PreparedMaterialShaderDependency {
            locator: shader_locator.clone(),
            id: Some(shader_id),
            revision: Some(13),
            dependency_identity: Some(shader_identity.clone()),
        },
        texture_dependencies: vec![PreparedMaterialTextureDependency {
            locator: texture_locator.clone(),
            id: Some(texture_id),
            revision: Some(19),
            upload_unsupported_reason: None,
        }],
        texture_support: support,
    };
    let is_current =
        |requested_revision,
         material_publication: &crate::core::resource::ResourceReadinessRowIdentity,
         shader_revision,
         texture_revision,
         requested_support| {
            prepared_material_candidate_identity_is_current(
                &identity,
                requested_revision,
                requested_support,
                |id| (id == material_id).then_some((material_id, 7, material_publication.clone())),
                |locator| {
                    (locator == &shader_locator).then_some((
                        shader_id,
                        shader_revision,
                        shader_identity.clone(),
                    ))
                },
                |locator| (locator == &texture_locator).then_some((texture_id, texture_revision)),
            )
        };

    assert!(is_current(Some(7), &material_identity, 13, 19, support));
    assert!(!is_current(Some(8), &material_identity, 13, 19, support));
    assert!(!is_current(
        Some(7),
        &readiness_identity(material_id),
        13,
        19,
        support
    ));
    assert!(!is_current(Some(7), &material_identity, 14, 19, support));
    assert!(!is_current(Some(7), &material_identity, 13, 20, support));
    assert!(!is_current(
        Some(7),
        &material_identity,
        13,
        19,
        TextureUploadSupport::all_compressed(),
    ));
}
