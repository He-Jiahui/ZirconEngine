use super::*;

#[test]
fn default_pbr_reference_targets_the_compound_asset_root() {
    let expected = AssetUri::parse(DEFAULT_PBR_SHADER_URI).unwrap();

    assert_eq!(default_pbr_shader_reference().locator, expected);
    assert!(!DEFAULT_PBR_SHADER_URI.ends_with(".zshader"));
}
