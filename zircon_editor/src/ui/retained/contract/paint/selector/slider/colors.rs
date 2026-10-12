use crate::ui::retained_host::primitives::Color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_color(
    color: Color,
) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
