mod array;
mod empty;
mod map;
mod roles;
mod type_tokens;
mod validation;

use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::RetainedUiHostBindingProjection;

pub(super) fn collection_fields_for_component(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
) -> Vec<host_contract::TemplatePaneCollectionFieldData> {
    match component {
        "ArrayField" => array::array_collection_fields(attributes, bindings),
        "MapField" => map::map_collection_fields(attributes, bindings),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
