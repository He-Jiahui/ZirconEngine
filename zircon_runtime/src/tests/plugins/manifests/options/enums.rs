use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::{
    shape::{assert_enum_token, assert_enum_values_contract},
    traversal::visit_option_rows,
};

#[test]
fn plugin_tomls_declare_enum_option_defaults_are_tokens() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_rows(
            table,
            relative_path,
            false,
            &mut |option, _key, option_context| {
                let value_type =
                    non_empty_string_value(option, relative_path, option_context, "value_type");
                if value_type == "enum" {
                    let default_value = non_empty_string_value(
                        option,
                        relative_path,
                        option_context,
                        "default_value",
                    );
                    assert_enum_token(
                        relative_path,
                        option_context,
                        "default_value",
                        default_value,
                    );
                }
            },
        );
    });
}

#[test]
fn plugin_tomls_declare_enum_options_define_values_and_default_membership() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_rows(
            table,
            relative_path,
            false,
            &mut |option, _key, option_context| {
                let value_type =
                    non_empty_string_value(option, relative_path, option_context, "value_type");
                if value_type == "enum" {
                    let default_value = non_empty_string_value(
                        option,
                        relative_path,
                        option_context,
                        "default_value",
                    );
                    assert_enum_values_contract(
                        relative_path,
                        option_context,
                        option,
                        value_type,
                        default_value,
                    );
                } else {
                    assert!(
                        option.get("enum_values").is_none(),
                        "plugin manifest {relative_path:?} {option_context} non-enum option should not declare `enum_values`"
                    );
                }
            },
        );
    });
}
