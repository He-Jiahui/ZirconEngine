use super::super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_style_color(
    color: Option<&zircon_runtime_interface::ui::style::UiStyleColor>,
) -> Option<[u8; 4]> {
    // An authored transparent color is a value; only an absent/unresolved color uses fallback.
    resolved_style_color(color)
}

/// Keep authored state attributes distinct so transient pointer state can select its own value.
pub(super) fn authored_field_color(
    node: &crate::ui::retained_host::host_contract::data::TemplatePaneNodeData,
    keys: &[&str],
) -> Option<[u8; 4]> {
    keys.iter().find_map(|key| {
        node.authored_field_colors
            .get(*key)
            .and_then(|value| declared_style_color(Some(value)))
    })
}
