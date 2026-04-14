use crate::{AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset};

#[test]
fn material_asset_toml_roundtrip_preserves_pbr_fields() {
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
    };

    let document = material.to_toml_string().unwrap();
    let loaded = MaterialAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, material);
}

#[test]
fn material_asset_parses_legacy_locator_only_references() {
    let document = r#"
name = "Grid"
shader = "res://shaders/pbr.wgsl"
base_color = [0.9, 0.8, 0.7, 1.0]
base_color_texture = "res://textures/albedo.png"
metallic = 0.3
roughness = 0.6
emissive = [0.1, 0.2, 0.3]
alpha_mode = { mode = "opaque" }
double_sided = false
"#;

    let loaded = MaterialAsset::from_toml_str(document).unwrap();

    assert_eq!(loaded.shader.locator, AssetUri::parse("res://shaders/pbr.wgsl").unwrap());
    assert_eq!(
        loaded.base_color_texture.unwrap().locator,
        AssetUri::parse("res://textures/albedo.png").unwrap()
    );
}
