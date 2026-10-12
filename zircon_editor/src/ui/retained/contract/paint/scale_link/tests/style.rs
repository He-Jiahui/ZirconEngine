use super::*;

#[test]
fn scale_link_asset_uses_the_projected_link_tone() {
    let node = TemplatePaneNodeData::default();

    assert_eq!(scale_link_asset_tint(&node), scale_link_color(&node));
}
