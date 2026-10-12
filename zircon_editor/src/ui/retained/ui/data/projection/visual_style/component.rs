use std::collections::BTreeMap;

use super::super::super::super::component_contract_metadata::tokens_for_component_role;
use super::super::surface_defaults::projected_component_variant;

pub(super) struct ProjectedComponentStyle {
    pub(super) category: &'static str,
    pub(super) layout_role: &'static str,
    pub(super) variant: String,
}

pub(super) fn projected_component_style(
    component: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedComponentStyle {
    let tokens = tokens_for_component_role(component, component_role);

    ProjectedComponentStyle {
        category: tokens.category,
        layout_role: tokens.layout_role,
        variant: projected_component_variant(attributes, component_role),
    }
}
