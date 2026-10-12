#[path = "array.rs"]
mod array;
#[path = "empty.rs"]
mod empty;
#[path = "map.rs"]
mod map;
#[path = "roles.rs"]
mod roles;
#[path = "../../collection_fields/type_tokens.rs"]
mod type_tokens;
#[path = "validation.rs"]
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
#[path = "../tests/cases.rs"]
mod tests;
