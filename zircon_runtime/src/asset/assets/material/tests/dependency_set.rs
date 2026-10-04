use super::MaterialAsset;

#[test]
fn direct_reference_projections_share_order_texture_dedup_and_parent() {
    let material = MaterialAsset::from_toml_str(
        r#"
version = 2

[shader]
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr.zshader"

[parent]
uuid = "00000000-0000-0000-0000-000000000002"
url = "res://materials/parent.zmaterial"

[textures.base_color]
uuid = "00000000-0000-0000-0000-000000000003"
url = "res://textures/shared.png"

[textures.normal]
uuid = "00000000-0000-0000-0000-000000000003"
url = "res://textures/shared.png"
"#,
    )
    .expect("material dependency fixture");

    let references = material.direct_references();
    let locators = material.direct_reference_locators();

    assert_eq!(references.len(), 3);
    assert_eq!(
        locators,
        references
            .iter()
            .map(|reference| reference.locator.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(locators[0].to_string(), "res://shaders/pbr.zshader");
    assert_eq!(locators[1].to_string(), "res://textures/shared.png");
    assert_eq!(locators[2].to_string(), "res://materials/parent.zmaterial");
}
