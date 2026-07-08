use super::super::super::super::data::TemplatePaneNodeData;
use super::super::palette::axis_label_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = axis_label_palette();
    if node.disabled {
        palette.disabled_scale_link
    } else {
        palette.scale_link
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_link_color_uses_disabled_palette_when_disabled() {
        let mut node = TemplatePaneNodeData::default();
        let palette = axis_label_palette();

        assert_eq!(scale_link_color(&node), palette.scale_link);

        node.disabled = true;
        assert_eq!(scale_link_color(&node), palette.disabled_scale_link);
    }
}
