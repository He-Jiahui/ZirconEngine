use super::super::super::super::data::TemplatePaneNodeData;
use super::super::identity::is_transform_scale_axis_control_id;
use super::super::palette::axis_label_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    let palette = axis_label_palette();
    if node.disabled {
        palette.disabled_axis
    } else if let Some(color) = declared_label_color(node) {
        color
    } else if is_scale_axis(node) {
        palette.scale_axis
    } else {
        palette.axis
    }
}

fn declared_label_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    (node.label_color.a > 0).then_some([
        node.label_color.r,
        node.label_color.g,
        node.label_color.b,
        node.label_color.a,
    ])
}

fn is_scale_axis(node: &TemplatePaneNodeData) -> bool {
    is_transform_scale_axis_control_id(node.control_id.as_str())
}

#[cfg(test)]
mod tests {
    use crate::ui::retained_host::primitives::Color;

    use super::*;

    fn node(control_id: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn declared_label_color_is_ignored_when_alpha_is_zero() {
        let mut axis = node("WorkbenchTransformPositionAxisX");
        axis.label_color = Color::from_argb_u8(0, 255, 255, 255);

        assert_eq!(declared_label_color(&axis), None);
    }

    #[test]
    fn scale_axis_matches_only_transform_scale_axis_prefix() {
        assert!(is_scale_axis(&node("WorkbenchTransformScaleAxisX")));
        assert!(!is_scale_axis(&node("WorkbenchTransformScaleLink")));
    }
}
