use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;

use super::super::collection_fields::collection_fields_for_component;

pub(super) fn projected_collection_fields(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    collection_fields_for_component(component, attributes, bindings)
}
