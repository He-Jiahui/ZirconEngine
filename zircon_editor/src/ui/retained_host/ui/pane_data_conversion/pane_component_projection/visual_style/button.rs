use std::collections::BTreeMap;

use zircon_runtime::ui::style::resolve_button_style_from_values;
use zircon_runtime_interface::ui::style::ResolvedButtonStyle;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::button_style::button_style_values_with_aliases;

pub(super) struct ProjectedButtonStyle {
    pub(super) variant: String,
    pub(super) resolved: ResolvedButtonStyle,
}

pub(super) fn projected_button_style(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedButtonStyle {
    let button_style_values = button_style_values_with_aliases(attributes);

    ProjectedButtonStyle {
        variant: attributes
            .get("button_variant")
            .and_then(value_as_string)
            .unwrap_or_default(),
        resolved: resolve_button_style_from_values(button_style_values.as_ref()),
    }
}
