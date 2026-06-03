use std::collections::BTreeMap;

use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::{
    shape::{
        assert_default_value_shape, assert_dot_namespaced_option_key, assert_enum_values_contract,
        assert_known_value_type, assert_trimmed,
    },
    traversal::visit_option_rows,
};

#[test]
fn plugin_tomls_declare_option_rows() {
    let mut option_keys = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_rows(
            table,
            relative_path,
            true,
            &mut |option, key, option_context| {
                assert_dot_namespaced_option_key(relative_path, option_context, key);
                if let Some(previous_context) =
                    option_keys.insert(key.to_string(), option_context.to_string())
                {
                    panic!(
                        "plugin option key `{key}` should be globally unique; first declared by {previous_context}, repeated by {option_context} in {}",
                        relative_path.display()
                    );
                }

                let display_name =
                    non_empty_string_value(option, relative_path, option_context, "display_name");
                assert_trimmed(relative_path, option_context, "display_name", display_name);
                let value_type =
                    non_empty_string_value(option, relative_path, option_context, "value_type");
                assert_known_value_type(relative_path, option_context, value_type);
                let default_value =
                    non_empty_string_value(option, relative_path, option_context, "default_value");
                assert_trimmed(
                    relative_path,
                    option_context,
                    "default_value",
                    default_value,
                );
                assert_default_value_shape(
                    relative_path,
                    option_context,
                    value_type,
                    default_value,
                );
                assert_enum_values_contract(
                    relative_path,
                    option_context,
                    option,
                    value_type,
                    default_value,
                );

                if option.get("required_capability").is_some() {
                    let capability = non_empty_string_value(
                        option,
                        relative_path,
                        option_context,
                        "required_capability",
                    );
                    assert_trimmed(
                        relative_path,
                        option_context,
                        "required_capability",
                        capability,
                    );
                }
            },
        );
    });
}
