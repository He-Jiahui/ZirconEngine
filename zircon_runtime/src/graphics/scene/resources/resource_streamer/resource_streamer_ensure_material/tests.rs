use crate::asset::TextureUploadSupport;
use crate::core::resource::{ResourceId, ResourceLocator};

use super::material_readiness::prepared_material_cache_identity_is_current;
use super::{
    PreparedMaterialDependency, PreparedMaterialShaderDependency, PreparedMaterialTextureDependency,
};

#[test]
fn prepared_material_dependency_cache_uses_registry_identity_and_revision() {
    let locator = ResourceLocator::parse("project://textures/albedo.png").unwrap();
    let id = ResourceId::from_stable_label("textures/albedo");
    let second_id = ResourceId::from_stable_label("textures/albedo-reimported");
    let shader_locator = ResourceLocator::parse("project://shaders/standard.zshader").unwrap();
    let shader_id = ResourceId::from_stable_label("shaders/standard");
    let material_id = ResourceId::from_stable_label("materials/painted-metal");
    let material_dependency = PreparedMaterialDependency {
        id: material_id,
        revision: 3,
        dependency_revision: 6,
    };
    let shader_dependency = PreparedMaterialShaderDependency {
        locator: shader_locator.clone(),
        id: Some(shader_id),
        revision: Some(4),
        dependency_revision: Some(12),
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
        |id| (id == material_id).then_some((material_id, 3, 6)),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, 12)),
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
        |id| (id == material_id).then_some((material_id, 3, 6)),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, 12)),
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
        |id| (id == material_id).then_some((material_id, 3, 6)),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, 12)),
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
        |id| (id == material_id).then_some((material_id, 3, 6)),
        |locator| (locator == &shader_locator).then_some((shader_id, 4, 12)),
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
    let material_dependency = PreparedMaterialDependency {
        id: material_id,
        revision: 11,
        dependency_revision: 18,
    };
    let shader_dependency = PreparedMaterialShaderDependency {
        locator: shader_locator,
        id: Some(shader_id),
        revision: Some(9),
        dependency_revision: Some(15),
    };

    assert!(prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        &material_dependency,
        uncompressed,
        uncompressed,
        &shader_dependency,
        &[],
        |id| (id == material_id).then_some((material_id, 11, 18)),
        |_| Some((shader_id, 9, 15)),
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
        |id| (id == material_id).then_some((material_id, 11, 18)),
        |_| Some((shader_id, 9, 15)),
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
        |id| (id == material_id).then_some((material_id, 11, 18)),
        |_| Some((shader_id, 9, 15)),
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
        |id| (id == material_id).then_some((material_id, 11, 18)),
        |_| Some((shader_id, 10, 15)),
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
        |id| (id == material_id).then_some((material_id, 11, 18)),
        |_| Some((shader_id, 9, 16)),
        |_| None,
    ));
}
