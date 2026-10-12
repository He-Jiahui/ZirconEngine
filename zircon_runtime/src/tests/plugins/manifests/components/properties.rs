use std::collections::BTreeMap;
use std::path::Path;

use super::super::{non_empty_string_value, optional_table_array};
use super::shape::assert_trimmed;
use super::uniqueness::assert_unique_row;

pub(super) fn validate_component_properties(
    relative_path: &Path,
    component: &toml::Table,
    component_context: &str,
) {
    let Some(properties) =
        optional_table_array(component, relative_path, component_context, "properties")
    else {
        return;
    };
    assert!(
        !properties.is_empty(),
        "plugin manifest {relative_path:?} {component_context} properties should not be empty when declared"
    );

    let mut property_names = BTreeMap::new();
    for property in properties {
        let property_name =
            non_empty_string_value(property, relative_path, component_context, "name");
        let property_context = format!("{component_context} property `{property_name}`");
        assert_trimmed(relative_path, &property_context, "name", property_name);
        assert_unique_row(
            relative_path,
            &mut property_names,
            property_name,
            property_context.clone(),
        );

        let value_type =
            non_empty_string_value(property, relative_path, &property_context, "value_type");
        assert_trimmed(relative_path, &property_context, "value_type", value_type);
        assert!(
            property
                .get("editable")
                .and_then(toml::Value::as_bool)
                .is_some(),
            "plugin manifest {relative_path:?} {property_context} should declare boolean `editable`"
        );
    }
}
