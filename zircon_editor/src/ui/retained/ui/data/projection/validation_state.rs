use std::collections::BTreeMap;

use super::super::pane_value_conversion::{value_as_bool, value_as_string};
use super::surface_defaults::projected_validation_level;

pub(super) struct ProjectedValidationState {
    pub(super) disabled: bool,
    pub(super) level: String,
    pub(super) message: String,
}

pub(super) fn projected_validation_state(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    has_component_descriptor: bool,
) -> ProjectedValidationState {
    let disabled = attributes
        .get("disabled")
        .and_then(value_as_bool)
        .unwrap_or(false)
        || attributes.get("enabled").and_then(value_as_bool) == Some(false);

    ProjectedValidationState {
        disabled,
        level: projected_validation_level(
            attributes,
            component_role,
            disabled,
            has_component_descriptor,
        ),
        message: attributes
            .get("validation_message")
            .and_then(value_as_string)
            .unwrap_or_default(),
    }
}
