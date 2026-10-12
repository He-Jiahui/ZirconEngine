use crate::asset::TextureUploadSupport;
use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceReadinessRowIdentity,
    ResourceRecord,
};

use super::material_readiness::prepared_material_cache_identity_is_current;
use super::{
    PreparedMaterialDependency, PreparedMaterialShaderDependency, PreparedMaterialTextureDependency,
};

pub(super) fn readiness_identity(id: ResourceId) -> ResourceReadinessRowIdentity {
    let resources = ResourceManager::new();
    let locator = ResourceLocator::parse(&format!("res://readiness/{id}.asset")).unwrap();
    resources
        .register_record(ResourceRecord::new(id, ResourceKind::Data, locator))
        .unwrap();
    resources.readiness_generation().row_identity(id).unwrap()
}

#[test]
fn prepared_material_dependency_cache_uses_registry_identity_and_revision() {
    let locator = ResourceLocator::parse("project://textures/albedo.png").unwrap();
    let id = ResourceId::from_stable_label("textures/albedo");
    let second_id = ResourceId::from_stable_label("textures/albedo-reimported");
    let shader_locator = ResourceLocator::parse("project://shaders/standard.zshader").unwrap();
    let shader_id = ResourceId::from_stable_label("shaders/standard");
    let material_id = ResourceId::from_stable_label("materials/painted-metal");
    let material_identity = readiness_identity(material_id);
    let shader_identity = readiness_identity(shader_id);
    let material_dependency = PreparedMaterialDependency {
        id: material_id,
        revision: 3,
        dependency_identity: material_identity.clone(),
    };
    let shader_dependency = PreparedMaterialShaderDependency {
        locator: shader_locator.clone(),
        id: Some(shader_id),
        revision: Some(4),
        dependency_identity: Some(shader_identity.clone()),
    };
    let dependencies = [PreparedMaterialTextureDependency {
        locator: locator.clone(),
        id: Some(id),
        revision: Some(7),
        upload_unsupported_reason: None,
    }];

    assert!(prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        &material_dependency,
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &shader_dependency,
        &dependencies,
        |id| (id == material_id).then_some((material_id, 3, material_identity.clone())),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, shader_identity.clone())),
        |candidate| (candidate == &locator).then_some((id, 7)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        &material_dependency,
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &shader_dependency,
        &dependencies,
        |id| (id == material_id).then_some((material_id, 3, material_identity.clone())),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, shader_identity.clone())),
        |_| Some((second_id, 7)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        &material_dependency,
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &shader_dependency,
        &dependencies,
        |id| (id == material_id).then_some((material_id, 3, material_identity.clone())),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, shader_identity.clone())),
        |_| Some((id, 8)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        &material_dependency,
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &shader_dependency,
        &dependencies,
        |id| (id == material_id).then_some((material_id, 3, material_identity.clone())),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, shader_identity.clone())),
        |_| None,
    ));
}

#[test]
fn prepared_material_cache_identity_rejects_revision_or_upload_support_change() {
    let uncompressed = TextureUploadSupport::uncompressed_only();
    let compressed = TextureUploadSupport::all_compressed();
    let shader_locator = ResourceLocator::parse("project://shaders/standard.zshader").unwrap();
    let shader_id = ResourceId::from_stable_label("shaders/standard");
    let material_id = ResourceId::from_stable_label("materials/standard");
    let material_identity = readiness_identity(material_id);
    let shader_identity = readiness_identity(shader_id);
    let material_dependency = PreparedMaterialDependency {
        id: material_id,
        revision: 11,
        dependency_identity: material_identity.clone(),
    };
    let shader_dependency = PreparedMaterialShaderDependency {
        locator: shader_locator,
        id: Some(shader_id),
        revision: Some(9),
        dependency_identity: Some(shader_identity.clone()),
    };

    assert!(prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        &material_dependency,
        uncompressed,
        uncompressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, material_identity.clone())),
        |_| Some((shader_id, 9, shader_identity.clone())),
        |_| None,
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(12),
        &material_dependency,
        uncompressed,
        uncompressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, material_identity.clone())),
        |_| Some((shader_id, 9, shader_identity.clone())),
        |_| None,
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        &material_dependency,
        uncompressed,
        compressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, material_identity.clone())),
        |_| Some((shader_id, 9, shader_identity.clone())),
        |_| None,
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        &material_dependency,
        uncompressed,
        uncompressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, material_identity.clone())),
        |_| Some((shader_id, 10, shader_identity.clone())),
        |_| None,
    ));

    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        &material_dependency,
        uncompressed,
        uncompressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, material_identity.clone())),
        |_| Some((shader_id, 9, readiness_identity(shader_id))),
        |_| None,
    ));
}

#[test]
fn prepared_material_cache_rejects_a_resolved_shader_without_its_publication() {
    let material_id = ResourceId::from_stable_label("materials/missing-publication");
    let material_dependency = PreparedMaterialDependency {
        id: material_id,
        revision: 1,
        dependency_identity: readiness_identity(material_id),
    };
    let shader_locator =
        ResourceLocator::parse("res://shaders/missing-publication.zshader").unwrap();
    let mut shader_dependency = PreparedMaterialShaderDependency {
        id: Some(ResourceId::from_locator(&shader_locator)),
        revision: Some(1),
        locator: shader_locator,
        dependency_identity: None,
    };
    let is_current = |shader_dependency: &PreparedMaterialShaderDependency| {
        prepared_material_cache_identity_is_current(
            Some(1),
            Some(1),
            &material_dependency,
            TextureUploadSupport::uncompressed_only(),
            TextureUploadSupport::uncompressed_only(),
            shader_dependency,
            &[],
            |_| {
                Some((
                    material_id,
                    1,
                    material_dependency.dependency_identity.clone(),
                ))
            },
            |_| None,
            |_| None,
        )
    };
    assert!(!is_current(&shader_dependency));
    shader_dependency.id = None;
    shader_dependency.revision = None;
    assert!(
        is_current(&shader_dependency),
        "an unresolved shader can retain its fallback until resolution changes"
    );
}
