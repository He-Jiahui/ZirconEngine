use std::collections::BTreeMap;

use crate::ui::retained_host::primitives::Color;

use super::super::pane_value_conversion::value_as_color;

pub(super) fn projected_value_color(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Color {
    let color_fields: &[&str] = if component_role == "color-field" {
        &[
            "value",
            "value_color",
            "action_color",
            "arrow_color",
            "dot_color",
            "text_color",
            "foreground_color",
            "color",
        ]
    } else {
        &[
            "value_color",
            "action_color",
            "arrow_color",
            "dot_color",
            "text_color",
            "foreground_color",
            "color",
            "value",
        ]
    };

    color_fields
        .iter()
        .find_map(|field| attributes.get(*field).and_then(value_as_color))
        .unwrap_or_else(|| Color::from_argb_u8(0, 0, 0, 0))
}
