use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const AXIS_LABEL_FONT_SIZE:
    f32 = 11.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const AXIS_LABEL_COLOR: [u8;
    4] = [129, 136, 140, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const AXIS_LABEL_SCALE_COLOR: [u8; 4] = [126, 132, 136, 255];
const AXIS_LABEL_DISABLED: [u8; 4] = [82, 93, 100, 255];
const AXIS_LABEL_LINK_COLOR: [u8; 4] = [145, 157, 164, 255];
const AXIS_LABEL_LINK_DISABLED: [u8; 4] = [82, 93, 100, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        AXIS_LABEL_DISABLED
    } else if node.label_color.a > 0 {
        [
            node.label_color.r,
            node.label_color.g,
            node.label_color.b,
            node.label_color.a,
        ]
    } else if node
        .control_id
        .as_str()
        .starts_with("WorkbenchTransformScaleAxis")
    {
        AXIS_LABEL_SCALE_COLOR
    } else {
        AXIS_LABEL_COLOR
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scale_link_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        AXIS_LABEL_LINK_DISABLED
    } else {
        AXIS_LABEL_LINK_COLOR
    }
}
