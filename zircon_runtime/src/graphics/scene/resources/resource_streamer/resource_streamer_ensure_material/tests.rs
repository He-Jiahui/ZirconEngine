use crate::asset::TextureUploadSupport;
use crate::core::resource::{ResourceId, ResourceLocator};

use super::PreparedMaterialTextureDependency;
use super::material_readiness::prepared_material_cache_identity_is_current;

#[test]
fn prepared_material_dependency_cache_uses_registry_identity_and_revision() {
    let locator = ResourceLocator::parse("project://textures/albedo.png").unwrap();
    let id = ResourceId::from_stable_label("textures/albedo");
    let second_id = ResourceId::from_stable_label("textures/albedo-reimported");
    let dependencies = [PreparedMaterialTextureDependency {
        locator: locator.clone(),
        id: Some(id),
        revision: Some(7),
        upload_unsupported_reason: None,
    }];

    assert!(prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &dependencies,
        |_| Some((id, 7)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &dependencies,
        |_| Some((second_id, 7)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &dependencies,
        |_| Some((id, 8)),
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(3),
        Some(3),
        TextureUploadSupport::uncompressed_only(),
        TextureUploadSupport::uncompressed_only(),
        &dependencies,
        |_| None,
    ));
}

#[test]
fn prepared_material_cache_identity_rejects_revision_or_upload_support_change() {
    let uncompressed = TextureUploadSupport::uncompressed_only();
    let compressed = TextureUploadSupport::all_compressed();

    assert!(prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        uncompressed,
        uncompressed,
        &[],
        |_| None,
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(12),
        uncompressed,
        uncompressed,
        &[],
        |_| None,
    ));
    assert!(!prepared_material_cache_identity_is_current(
        Some(11),
        Some(11),
        uncompressed,
        compressed,
        &[],
        |_| None,
    ));
}
