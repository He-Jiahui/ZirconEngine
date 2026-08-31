use super::*;
use crate::core::framework::render::{
    StandardPbrMaterialFeatures, STANDARD_PBR_TRANSMISSION_RENDER_QUEUE,
};

#[test]
fn render_advanced_material_asset_projects_owned_features_and_texture() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2
name = "Advanced Surface"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
clearcoat = 0.8
clearcoat_perceptual_roughness = 0.2
clearcoat_normal_scale = 0.35
anisotropy_strength = 0.6
anisotropy_rotation = 1.25
specular_transmission = 0.7
diffuse_transmission = 0.1
thickness = 0.4
ior = 1.52
attenuation_color = [0.8, 0.9, 1.0]
attenuation_distance = 12.0

[textures.clearcoat_normal]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/clearcoat-normal.png"
uv_channel = 1

[textures.clearcoat_normal.transform]
scale = [0.5, 0.75]
offset = [0.1, 0.2]
rotation = 0.4
"#,
    )
    .expect("advanced material document");

    let descriptor = material.standard_material_descriptor();
    assert_eq!(
        descriptor.render_queue_value,
        Some(STANDARD_PBR_TRANSMISSION_RENDER_QUEUE)
    );
    let features = descriptor.advanced_features;

    assert_eq!(features.clearcoat, 0.8);
    assert_eq!(features.clearcoat_perceptual_roughness, 0.2);
    assert_eq!(features.clearcoat_normal_scale, 0.35);
    assert_eq!(features.anisotropy_strength, 0.6);
    assert_eq!(features.anisotropy_rotation, 1.25);
    assert_eq!(features.specular_transmission, 0.7);
    assert_eq!(features.diffuse_transmission, 0.1);
    assert_eq!(features.thickness, 0.4);
    assert_eq!(features.ior, 1.52);
    assert_eq!(features.attenuation_color, [0.8, 0.9, 1.0]);
    assert_eq!(features.attenuation_distance, 12.0);
    assert_eq!(
        features
            .clearcoat_normal_texture
            .as_ref()
            .map(|reference| &reference.locator),
        Some(&AssetUri::parse("res://textures/clearcoat-normal.png").unwrap())
    );
    assert_eq!(descriptor.clearcoat_normal_texture_uv_channel, 1);
    assert_eq!(
        descriptor.clearcoat_normal_texture_transform,
        RenderMaterialTextureTransform {
            scale: [0.5, 0.75],
            offset: [0.1, 0.2],
            rotation: 0.4,
        }
    );
    assert!(features.requires_forward_path());
    assert!(features.requires_scene_color_copy());

    for property in [
        "clearcoat",
        "clearcoat_perceptual_roughness",
        "clearcoat_normal_scale",
        "anisotropy_strength",
        "anisotropy_rotation",
        "specular_transmission",
        "diffuse_transmission",
        "thickness",
        "ior",
        "attenuation_color",
        "attenuation_distance",
    ] {
        assert!(material.shader_property_override(property).is_none());
        assert!(material
            .shader_property_overrides()
            .all(|(name, _)| name != property));
    }
}

#[test]
fn render_advanced_material_asset_defaults_keep_legacy_contract() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"
"#,
    )
    .expect("default material document");

    assert_eq!(
        material.standard_material_descriptor().advanced_features,
        StandardPbrMaterialFeatures::default()
    );
    assert_eq!(
        material.standard_material_descriptor().render_queue_value,
        None
    );
}

#[test]
fn render_advanced_material_asset_reports_invalid_owned_values() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[overrides]
clearcoat = 2.0
ior = 0.5
attenuation_color = [1.0, -0.25, 1.0]
"#,
    )
    .expect("invalid advanced material still parses for typed validation");

    let errors = material.validation_errors();
    for property in ["clearcoat", "ior", "attenuation_color"] {
        assert!(errors.iter().any(|error| matches!(
            error,
            RenderMaterialValidationError::PropertyOverrideTypeMismatch { name, .. }
                if name == property
        )));
    }
}
