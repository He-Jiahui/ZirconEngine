use crate::asset::{AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset};

#[test]
fn material_asset_zmaterial_roundtrip_maps_pbr_fields_to_shader_overrides() {
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: AssetReference::new(
            AssetUuid::from_stable_label("shader"),
            AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        ),
        base_color: [0.9, 0.8, 0.7, 1.0],
        base_color_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("albedo"),
            AssetUri::parse("res://textures/albedo.png").unwrap(),
        )),
        normal_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("normal"),
            AssetUri::parse("res://textures/normal.png").unwrap(),
        )),
        metallic: 0.3,
        roughness: 0.6,
        metallic_roughness_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("metal_rough"),
            AssetUri::parse("res://textures/metal_rough.png").unwrap(),
        )),
        occlusion_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("occlusion"),
            AssetUri::parse("res://textures/occlusion.png").unwrap(),
        )),
        emissive: [0.1, 0.2, 0.3],
        emissive_texture: Some(AssetReference::new(
            AssetUuid::from_stable_label("emissive"),
            AssetUri::parse("res://textures/emissive.png").unwrap(),
        )),
        alpha_mode: AlphaMode::Mask { cutoff: 0.5 },
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };

    let document = material.to_toml_string().unwrap();
    let loaded = MaterialAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded.name, material.name);
    assert_eq!(loaded.shader, material.shader);
    assert_eq!(loaded.base_color, material.base_color);
    assert_eq!(loaded.base_color_texture, material.base_color_texture);
    assert_eq!(loaded.normal_texture, material.normal_texture);
    assert_eq!(loaded.metallic, material.metallic);
    assert_eq!(loaded.roughness, material.roughness);
    assert_eq!(
        loaded.metallic_roughness_texture,
        material.metallic_roughness_texture
    );
    assert_eq!(loaded.occlusion_texture, material.occlusion_texture);
    assert_eq!(loaded.emissive, material.emissive);
    assert_eq!(loaded.emissive_texture, material.emissive_texture);
    assert_eq!(loaded.alpha_mode, material.alpha_mode);
    assert_eq!(loaded.double_sided, material.double_sided);
    assert!(loaded.property_overrides().contains_key("base_color"));
    assert!(loaded.property_overrides().contains_key("roughness"));
    assert!(loaded.texture_slots.contains_key("base_color"));
    assert!(loaded.texture_slots.contains_key("normal"));
}

#[test]
fn material_asset_parses_uuid_url_references() {
    let document = r#"
version = 1
name = "Grid"

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"

[overrides]
base_color = [0.9, 0.8, 0.7, 1.0]
metallic = 0.3
roughness = 0.6
emissive = [0.1, 0.2, 0.3]
double_sided = true

[overrides.alpha_mode]
mode = "opaque"

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://textures/albedo.png"

[textures.normal]
fallback = "normal"
"#;

    let loaded = MaterialAsset::from_toml_str(document).unwrap();

    assert_eq!(
        loaded.shader.locator,
        AssetUri::parse("res://shaders/pbr.wgsl").unwrap()
    );
    assert_eq!(
        loaded.base_color_texture.as_ref().unwrap().locator,
        AssetUri::parse("res://textures/albedo.png").unwrap()
    );
    assert_eq!(loaded.base_color, [0.9, 0.8, 0.7, 1.0]);
    assert!(loaded.double_sided);
    assert_eq!(
        loaded.texture_slots["normal"].fallback.as_deref(),
        Some("normal")
    );
    assert!(loaded.texture_slots["normal"].reference.is_none());
}

#[test]
fn material_asset_rejects_legacy_material_toml_shape() {
    let document = r#"
name = "Grid"
base_color = [0.9, 0.8, 0.7, 1.0]

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.wgsl"
"#;

    let error = MaterialAsset::from_toml_str(document).unwrap_err();

    assert!(
        error.to_string().contains("unknown field `base_color`"),
        "unexpected error: {error}"
    );
}
