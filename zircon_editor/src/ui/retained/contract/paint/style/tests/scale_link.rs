use super::*;

#[test]
fn scale_link_color_uses_disabled_palette_when_disabled() {
    let mut node = TemplatePaneNodeData::default();
    let palette = axis_label_palette();

    assert_eq!(scale_link_color(&node), palette.scale_link);

    node.disabled = true;
    assert_eq!(scale_link_color(&node), palette.disabled_scale_link);
}
